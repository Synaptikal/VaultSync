use crate::config::Config;
use crate::database::Database;
use anyhow::Result;
use serde::Serialize;
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct InvoiceData {
    pub invoice_number: String,
    pub issue_date: String,
    pub due_date: String,
    pub store_info: StoreInfo,
    pub bill_to: CustomerInfo,
    pub items: Vec<InvoiceItem>,
    pub subtotal: f64,
    pub tax_rate: f64,
    pub tax_amount: f64,
    pub total: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StoreInfo {
    pub name: String,
    pub address: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CustomerInfo {
    pub name: String,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InvoiceItem {
    pub description: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub amount: f64,
}

pub struct InvoiceService {
    db: Arc<Database>,
    config: Config,
}

impl InvoiceService {
    pub fn new(db: Arc<Database>, config: Config) -> Self {
        Self { db, config }
    }

    pub async fn generate_invoice_data(&self, transaction_uuid: Uuid) -> Result<InvoiceData> {
        // Query Transaction
        let tx_row = sqlx::query(
             "SELECT transaction_uuid, timestamp, subtotal, tax_amount, total, customer_uuid, notes FROM Transactions WHERE transaction_uuid = ?"
         )
         .bind(transaction_uuid)
         .fetch_optional(&self.db.pool)
         .await?
         .ok_or_else(|| anyhow::anyhow!("Transaction not found"))?;

        let timestamp: chrono::DateTime<chrono::Utc> = tx_row.try_get("timestamp")?;
        let subtotal: f64 = tx_row.try_get("subtotal").unwrap_or(0.0);
        let tax_amount: f64 = tx_row.try_get("tax_amount").unwrap_or(0.0);
        let total: f64 = tx_row.try_get("total").unwrap_or(0.0);
        let customer_uuid_opt: Option<String> = tx_row.try_get("customer_uuid").ok();
        let notes: Option<String> = tx_row.try_get("notes").ok();

        // Calculate tax rate approximate (backwards) or fetch default?
        // simple calc: if subtotal > 0, rate = tax / subtotal
        let tax_rate = if subtotal > 0.0 {
            (tax_amount / subtotal) * 100.0
        } else {
            0.0
        };

        // Customer Info
        let mut bill_to = CustomerInfo {
            name: "Guest Customer".to_string(),
            address: None,
            email: None,
            phone: None,
        };

        if let Some(uid_str) = customer_uuid_opt {
            if let Ok(uid) = Uuid::parse_str(&uid_str) {
                if let Ok(Some(cust)) = self.db.customers.get_by_id(uid).await {
                    bill_to.name = cust.name;
                    bill_to.email = cust.email;
                    bill_to.phone = cust.phone;
                    // Address? We updated Customer table but I don't recall 'address' column specifically in PHASE 1 list.
                    // Let's check schema. Customers usually need address for Invoices.
                    // If not present, we leave None.
                }
            }
        }

        // Items
        let items_rows = sqlx::query(
            r#"
             SELECT ti.quantity, ti.unit_price, p.name 
             FROM Transaction_Items ti
             JOIN Global_Catalog p ON ti.product_uuid = p.product_uuid
             WHERE ti.transaction_uuid = ?
             "#,
        )
        .bind(transaction_uuid)
        .fetch_all(&self.db.pool)
        .await?;

        let items: Vec<InvoiceItem> = items_rows
            .into_iter()
            .map(|r| {
                let qty: i32 = r.try_get("quantity").unwrap_or(0);
                let price: f64 = r.try_get("unit_price").unwrap_or(0.0);
                let name: String = r.try_get("name").unwrap_or("Unknown Item".to_string());
                InvoiceItem {
                    description: name,
                    quantity: qty,
                    unit_price: price,
                    amount: (qty as f64) * price,
                }
            })
            .collect();

        Ok(InvoiceData {
            invoice_number: format!("INV-{}", &transaction_uuid.to_string()[..8]), // Simple ID
            issue_date: timestamp.format("%Y-%m-%d").to_string(),
            due_date: timestamp.format("%Y-%m-%d").to_string(), // Due on receipt
            store_info: StoreInfo {
                name: self.config.store_name.clone(),
                address: self.config.store_address.clone(),
                phone: self.config.store_phone.clone(),
                email: None, // Config doesn't have email yet
                website: self.config.store_website.clone(),
            },
            bill_to,
            items,
            subtotal,
            tax_rate,
            tax_amount,
            total,
            notes,
        })
    }

    pub async fn generate_html(&self, transaction_uuid: Uuid) -> Result<String> {
        let data = self.generate_invoice_data(transaction_uuid).await?;

        let mut items_html = String::new();
        for item in &data.items {
            items_html.push_str(&format!(
                "<tr>
                    <td style='padding: 8px; border-bottom: 1px solid #ddd;'>{}</td>
                    <td style='padding: 8px; border-bottom: 1px solid #ddd; text-align: right;'>{}</td>
                    <td style='padding: 8px; border-bottom: 1px solid #ddd; text-align: right;'>${:.2}</td>
                    <td style='padding: 8px; border-bottom: 1px solid #ddd; text-align: right;'>${:.2}</td>
                </tr>",
                item.description, item.quantity, item.unit_price, item.amount
            ));
        }

        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Invoice {}</title>
    <style>
        body {{ font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif; color: #555; max-width: 800px; margin: auto; padding: 30px; }}
        .invoice-box {{ border: 1px solid #eee; box-shadow: 0 0 10px rgba(0, 0, 0, 0.15); padding: 30px; font-size: 16px; line-height: 24px; color: #555; }}
        .header {{ display: flex; justify-content: space-between; margin-bottom: 50px; }}
        .header-left h1 {{ margin: 0; color: #333; }}
        .header-right {{ text-align: right; }}
        .billing-info {{ display: flex; justify-content: space-between; margin-bottom: 40px; }}
        .items-table {{ width: 100%; border-collapse: collapse; }}
        .items-table th {{ padding: 8px; background: #eee; border-bottom: 1px solid #ddd; text-align: left; }}
        .totals {{ text-align: right; margin-top: 30px; }}
        .total-row {{ font-weight: bold; font-size: 1.1em; }}
        .footer {{ margin-top: 50px; font-size: 12px; text-align: center; color: #999; }}
    </style>
</head>
<body>
    <div class="invoice-box">
        <div class="header">
            <div class="header-left">
                <h1>INVOICE</h1>
                <p>#{}</p>
                <p>Date: {}</p>
            </div>
            <div class="header-right">
                <strong>{}</strong><br>
                {}<br>
                {}<br>
                {}
            </div>
        </div>

        <div class="billing-info">
            <div class="bill-to">
                <strong>Bill To:</strong><br>
                {}<br>
                {}
                {}
            </div>
        </div>

        <table class="items-table">
            <thead>
                <tr>
                    <th>Item</th>
                    <th style="text-align: right;">Quantity</th>
                    <th style="text-align: right;">Unit Price</th>
                    <th style="text-align: right;">Amount</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>

        <div class="totals">
            <p>Subtotal: ${:.2}</p>
            <p>Tax ({:.1}%): ${:.2}</p>
            <p class="total-row">Total: ${:.2}</p>
        </div>

        <div class="footer">
            <p>Thank you for your business!</p>
        </div>
    </div>
</body>
</html>"#,
            data.invoice_number,
            data.invoice_number,
            data.issue_date,
            data.store_info.name,
            data.store_info.address,
            data.store_info.phone.as_deref().unwrap_or(""),
            data.store_info.website.as_deref().unwrap_or(""),
            data.bill_to.name,
            data.bill_to.email.as_deref().unwrap_or(""),
            data.bill_to.phone.as_deref().unwrap_or(""),
            items_html,
            data.subtotal,
            data.tax_rate,
            data.tax_amount,
            data.total
        );
        Ok(html)
    }
}
