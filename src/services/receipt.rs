use crate::config::Config;
use crate::database::Database;
use anyhow::Result;
use serde::Serialize;
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ReceiptData {
    pub store_name: String,
    pub store_address: String,
    pub store_phone: Option<String>,
    pub store_website: Option<String>,
    pub transaction_id: String,
    pub date: String,
    pub items: Vec<ReceiptItem>,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub total: f64,
    pub customer_name: Option<String>,
    pub payments: Vec<ReceiptPayment>, // New field
}

#[derive(Debug, Serialize)]
pub struct ReceiptItem {
    pub name: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub total: f64,
}

#[derive(Debug, Serialize)]
pub struct ReceiptPayment {
    pub method: String,
    pub amount: f64,
    pub reference: Option<String>,
}

pub struct ReceiptService {
    db: Arc<Database>,
    config: Config,
}

impl ReceiptService {
    pub fn new(db: Arc<Database>, config: Config) -> Self {
        Self { db, config }
    }

    pub async fn generate_receipt_data(&self, transaction_uuid: Uuid) -> Result<ReceiptData> {
        // Query Transaction
        let tx_row = sqlx::query(
             "SELECT transaction_uuid, timestamp, subtotal, tax_amount, total, customer_uuid FROM Transactions WHERE transaction_uuid = ?"
         )
         .bind(transaction_uuid)
         .fetch_optional(&self.db.pool)
         .await?
         .ok_or_else(|| anyhow::anyhow!("Transaction not found"))?;

        let subtotal: f64 = tx_row.try_get("subtotal").unwrap_or(0.0);
        let tax_amount: f64 = tx_row.try_get("tax_amount").unwrap_or(0.0);
        let total: f64 = tx_row.try_get("total").unwrap_or(0.0);
        let timestamp: chrono::DateTime<chrono::Utc> = tx_row.try_get("timestamp")?;
        let customer_uuid_opt: Option<String> = tx_row.try_get("customer_uuid").ok();

        let mut customer_name = None;
        if let Some(uid_str) = customer_uuid_opt {
            if let Ok(uid) = Uuid::parse_str(&uid_str) {
                if let Ok(Some(cust)) = self.db.customers.get_by_id(uid).await {
                    customer_name = Some(cust.name);
                }
            }
        }

        // Query Items
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

        let items: Vec<ReceiptItem> = items_rows
            .into_iter()
            .map(|r| {
                let qty: i32 = r.try_get("quantity").unwrap_or(0);
                let price: f64 = r.try_get("unit_price").unwrap_or(0.0);
                let name: String = r.try_get("name").unwrap_or("Unknown Item".to_string());
                ReceiptItem {
                    name,
                    quantity: qty,
                    unit_price: price,
                    total: (qty as f64) * price,
                }
            })
            .collect();

        // Query Payments (Task 104)
        let payment_rows = sqlx::query(
            "SELECT method_type, amount, reference FROM Payment_Methods WHERE transaction_uuid = ?",
        )
        .bind(transaction_uuid)
        .fetch_all(&self.db.pool)
        .await?;

        let payments: Vec<ReceiptPayment> = payment_rows
            .into_iter()
            .map(|r| {
                let method: String = r.try_get("method_type").unwrap_or("Unknown".to_string());
                let amount: f64 = r.try_get("amount").unwrap_or(0.0);
                let reference: Option<String> = r.try_get("reference").ok();
                ReceiptPayment {
                    method,
                    amount,
                    reference,
                }
            })
            .collect();

        Ok(ReceiptData {
            store_name: self.config.store_name.clone(),
            store_address: self.config.store_address.clone(),
            store_phone: self.config.store_phone.clone(),
            store_website: self.config.store_website.clone(),
            transaction_id: transaction_uuid.to_string(),
            date: timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
            items,
            subtotal,
            tax_amount,
            total,
            customer_name,
            payments,
        })
    }

    pub async fn generate_html(&self, transaction_uuid: Uuid) -> Result<String> {
        let data = self.generate_receipt_data(transaction_uuid).await?;

        // Format Items
        let mut items_html = String::new();
        for item in &data.items {
            items_html.push_str(&format!(
                "<tr><td>{}</td><td align='right'>{}</td><td align='right'>${:.2}</td></tr>",
                item.name, item.quantity, item.total
            ));
        }

        // Format Payments
        let mut payments_html = String::new();
        if !data.payments.is_empty() {
            payments_html.push_str("<tr><td colspan='2' style='padding-top: 5px;'><strong>Payment Method:</strong></td></tr>");
            for payment in &data.payments {
                let method_display = if let Some(ref r) = payment.reference {
                    format!("{} ({})", payment.method, r)
                } else {
                    payment.method.clone()
                };
                payments_html.push_str(&format!(
                    "<tr><td>{}</td><td align='right'>${:.2}</td></tr>",
                    method_display, payment.amount
                ));
            }
        }

        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Receipt {}</title>
    <style>
        body {{ font-family: 'Courier New', monospace; width: 300px; font-size: 12px; margin: 0 auto; background: white; }}
        .header {{ text-align: center; margin-bottom: 20px; }}
        .header h3 {{ margin: 0 0 5px 0; }}
        .header p {{ margin: 0; }}
        .total-section {{ font-weight: bold; border-top: 1px dashed black; margin-top: 10px; padding-top: 5px; }}
        .payment-section {{ margin-top: 10px; border-top: 1px dashed black; padding-top: 5px; }}
        table {{ width: 100%; border-collapse: collapse; }}
        td {{ vertical-align: top; padding: 2px 0; }}
        .footer {{ text-align: center; margin-top: 20px; font-size: 10px; }}
    </style>
</head>
<body>
    <div class="header">
        <h3>{}</h3>
        <p>{}</p>
        <p>{}</p>
        <p>{}</p>
        <br>
        <p>Tx: {}</p>
        <p>Date: {}</p>
    </div>
    
    <table>
        <thead>
            <tr style="border-bottom: 1px solid black;">
                <th align='left'>Item</th>
                <th align='right'>Qty</th>
                <th align='right'>Amt</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
    </table>
    
    <div class="total-section">
        <table>
            <tr><td>Subtotal:</td><td align='right'>${:.2}</td></tr>
            <tr><td>Tax:</td><td align='right'>${:.2}</td></tr>
            <tr><td><strong>Total:</strong></td><td align='right'><strong>${:.2}</strong></td></tr>
        </table>
    </div>

    <div class="payment-section">
        <table>
            {}
        </table>
    </div>
    
    <div class="footer">
        <p>Thank you for shopping!</p>
        <p>Returns accepted within 14 days<br>with original receipt.</p>
    </div>
</body>
</html>"#,
            data.transaction_id,
            data.store_name,
            data.store_address,
            data.store_phone.as_deref().unwrap_or(""),
            data.store_website.as_deref().unwrap_or(""),
            data.transaction_id,
            data.date,
            items_html,
            data.subtotal,
            data.tax_amount,
            data.total,
            payments_html
        );
        Ok(html)
    }
}
