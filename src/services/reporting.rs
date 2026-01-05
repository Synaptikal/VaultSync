use crate::database::Database;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SalesReport {
    pub period: String,
    pub total_sales: f64,
    pub total_transactions: i64,
    pub average_transaction: f64,
    pub top_selling_products: Vec<serde_json::Value>,
    pub sales_by_category: HashMap<String, f64>,
    pub sales_by_payment_method: HashMap<String, f64>,
    pub sales_by_day: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmployeePerformanceEntry {
    pub user_uuid: Option<String>,
    pub username: Option<String>, // Resolved from Auth
    pub transaction_count: i64,
    pub total_sales: f64,
}

#[derive(Serialize, Deserialize)]
pub struct InventoryValuationReport {
    pub total_items: i64,
    pub total_quantity: i64,
    pub total_retail_value: f64,
    pub total_cost_value: f64,
    pub valuation_by_category: HashMap<String, f64>,
    pub valuation_by_condition: HashMap<String, f64>,
    pub low_stock_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryAgingReport {
    pub buckets: Vec<AgingBucket>,
    pub total_value: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgingBucket {
    pub range: String,
    pub value: f64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZReport {
    pub shift_details: crate::services::cash_drawer::Shift,
    pub sales_report: SalesReport,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CashFlowReport {
    pub period: String,
    pub cash_sales: f64,
    pub cash_refunds: f64,
    pub safe_drops: f64,
    pub net_cash_flow: f64,
}

pub struct ReportingService {
    db: Arc<Database>,
}

impl ReportingService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn get_sales_report(
        &self,
        period: String,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<SalesReport> {
        let start_str = start.to_rfc3339();
        let end_str = end.to_rfc3339();

        // Use Repo Aggregation
        let report_data = self
            .db
            .transactions
            .get_sales_report_aggregated(&start_str, &end_str)
            .await?;

        // Category Breakdown
        let category_data = self
            .db
            .transactions
            .get_sales_by_category(&start_str, &end_str)
            .await?;

        // Payment Method Breakdown
        let payment_data = self
            .db
            .transactions
            .get_sales_by_payment_method(&start_str, &end_str)
            .await?;

        // Enrich with product names
        let mut top_selling: Vec<serde_json::Value> = Vec::new();
        for product in &report_data.top_products {
            let name = match self.db.products.get_by_id(product.product_uuid).await {
                Ok(Some(p)) => p.name,
                _ => "Unknown".to_string(),
            };
            top_selling.push(json!({
                "product_uuid": product.product_uuid,
                "name": name,
                "revenue": product.revenue,
                "quantity_sold": product.quantity_sold
            }));
        }

        Ok(SalesReport {
            period,
            total_sales: report_data.total_sales,
            total_transactions: report_data.transaction_count,
            average_transaction: report_data.average_transaction,
            top_selling_products: top_selling,
            sales_by_category: category_data,
            sales_by_payment_method: payment_data,
            sales_by_day: Vec::new(), // Placeholder for future implementation
        })
    }

    pub async fn get_employee_performance_report(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<EmployeePerformanceEntry>> {
        let start_str = start.to_rfc3339();
        let end_str = end.to_rfc3339();

        let raw_data = self
            .db
            .transactions
            .get_sales_by_employee(&start_str, &end_str)
            .await?;

        let mut report = Vec::new();
        for (user_uuid, count, revenue) in raw_data {
            // Resolve username if possible
            let user_uuid_str = user_uuid.map(|u| u.to_string());
            let mut username = None;
            if let Some(uuid) = user_uuid {
                if let Ok(Some((_, name, _, _))) = self.db.auth.get_user_by_uuid(uuid).await {
                    username = Some(name);
                }
            }

            report.push(EmployeePerformanceEntry {
                user_uuid: user_uuid_str,
                username,
                transaction_count: count,
                total_sales: revenue,
            });
        }
        Ok(report)
    }

    pub async fn get_inventory_valuation(&self) -> Result<InventoryValuationReport> {
        // Aggregated query for valuation
        // Logic: Use specific_price if set, otherwise use Pricing Matrix market_mid
        let row = sqlx::query(
             "SELECT 
                 COUNT(*) as total_rows,
                 COALESCE(SUM(i.quantity_on_hand), 0) as total_qty,
                 COALESCE(SUM(i.quantity_on_hand * COALESCE(i.specific_price, pm.market_mid, 0)), 0) as total_value
              FROM Local_Inventory i
              LEFT JOIN Pricing_Matrix pm ON i.product_uuid = pm.product_uuid
              WHERE i.deleted_at IS NULL AND i.quantity_on_hand > 0"
         )
         .fetch_one(&self.db.pool)
         .await
         .context("Failed to fetch inventory valuation")?;

        let total_items: i64 = row.try_get("total_rows")?;
        let total_quantity: i64 = row.try_get("total_qty")?; // Using i64 for SUM result
        let total_value: f64 = row.try_get("total_value")?;

        // Get low stock count from existing repo method
        let low_stock_count = self.db.inventory.get_low_stock_count(5).await?;

        Ok(InventoryValuationReport {
            total_items,
            total_quantity,
            total_retail_value: total_value,
            total_cost_value: 0.0, // Cost tracking to be implemented
            valuation_by_category: HashMap::new(),
            valuation_by_condition: HashMap::new(),
            low_stock_count,
        })
    }
    pub async fn get_inventory_aging_report(&self) -> Result<InventoryAgingReport> {
        let raw_data = self.db.inventory.get_inventory_aging().await?;

        // Buckets
        let mut b_0_30 = 0.0;
        let mut b_31_60 = 0.0;
        let mut b_61_90 = 0.0;
        let mut b_91_180 = 0.0;
        let mut b_181_plus = 0.0;
        let mut b_unknown = 0.0;
        let mut total_value = 0.0;

        for (days, value) in raw_data {
            total_value += value;
            match days {
                d if d < 0 => b_unknown += value,
                0..=30 => b_0_30 += value,
                31..=60 => b_31_60 += value,
                61..=90 => b_61_90 += value,
                91..=180 => b_91_180 += value,
                _ => b_181_plus += value,
            }
        }

        let calculate_percent = |val: f64| -> f64 {
            if total_value > 0.0 {
                (val / total_value) * 100.0
            } else {
                0.0
            }
        };

        let buckets = vec![
            AgingBucket {
                range: "0-30 Days".to_string(),
                value: b_0_30,
                percentage: calculate_percent(b_0_30),
            },
            AgingBucket {
                range: "31-60 Days".to_string(),
                value: b_31_60,
                percentage: calculate_percent(b_31_60),
            },
            AgingBucket {
                range: "61-90 Days".to_string(),
                value: b_61_90,
                percentage: calculate_percent(b_61_90),
            },
            AgingBucket {
                range: "91-180 Days".to_string(),
                value: b_91_180,
                percentage: calculate_percent(b_91_180),
            },
            AgingBucket {
                range: "180+ Days".to_string(),
                value: b_181_plus,
                percentage: calculate_percent(b_181_plus),
            },
            AgingBucket {
                range: "Unknown".to_string(),
                value: b_unknown,
                percentage: calculate_percent(b_unknown),
            },
        ];

        Ok(InventoryAgingReport {
            buckets,
            total_value,
        })
    }

    pub async fn get_shift_z_report(&self, shift_uuid: Uuid) -> Result<ZReport> {
        let row = sqlx::query(
            "SELECT shift_uuid, user_uuid, terminal_id, opened_at, closed_at, opening_count_uuid, closing_count_uuid, expected_cash, actual_cash, variance, status 
             FROM Shifts WHERE shift_uuid = ?"
        )
        .bind(shift_uuid.to_string())
        .fetch_optional(&self.db.pool)
        .await
        .context("Database error")?;

        let row = row.context("Shift not found")?;

        let user_uuid_str: String = row.try_get("user_uuid").unwrap_or_default();
        let terminal_id: String = row.try_get("terminal_id").unwrap_or_default();
        let opened_at_str: String = row.try_get("opened_at").unwrap_or_default();
        let opened_at = DateTime::parse_from_rfc3339(&opened_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(Utc::now());

        let closed_at_str: Option<String> = row.try_get("closed_at").ok();
        let closed_at = closed_at_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        });

        let status_str: String = row.try_get("status").unwrap_or_else(|_| "open".to_string());
        let status = match status_str.as_str() {
            "closed" => crate::services::cash_drawer::ShiftStatus::Closed,
            "reconciled" => crate::services::cash_drawer::ShiftStatus::Reconciled,
            _ => crate::services::cash_drawer::ShiftStatus::Open,
        };

        let shift = crate::services::cash_drawer::Shift {
            shift_uuid,
            user_uuid: Uuid::parse_str(&user_uuid_str).unwrap_or_default(),
            terminal_id,
            opened_at,
            closed_at,
            opening_count_uuid: row
                .try_get::<String, _>("opening_count_uuid")
                .ok()
                .and_then(|s| Uuid::parse_str(&s).ok()),
            closing_count_uuid: row
                .try_get::<String, _>("closing_count_uuid")
                .ok()
                .and_then(|s| Uuid::parse_str(&s).ok()),
            expected_cash: row.try_get("expected_cash").unwrap_or(0.0),
            actual_cash: row.try_get("actual_cash").ok(),
            variance: row.try_get("variance").ok(),
            status,
        };

        // If shift is open, use now as end time.
        let report_end = closed_at.unwrap_or(Utc::now());

        let sales_report = self
            .get_sales_report("shift".to_string(), opened_at, report_end)
            .await?;

        Ok(ZReport {
            shift_details: shift,
            sales_report,
        })
    }

    pub async fn get_cash_flow_report(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<CashFlowReport> {
        let start_str = start.to_rfc3339();
        let end_str = end.to_rfc3339();

        // Cash Sales
        let sales: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(pm.amount), 0.0) FROM Payment_Methods pm
              JOIN Transactions t ON pm.transaction_uuid = t.transaction_uuid
              WHERE pm.method_type = 'Cash' AND t.timestamp >= ? AND t.timestamp <= ?",
        )
        .bind(&start_str)
        .bind(&end_str)
        .fetch_one(&self.db.pool)
        .await
        .unwrap_or(0.0);

        // Cash Refunds
        let refunds: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(r.refund_amount), 0.0) 
              FROM Returns r
              JOIN Payment_Methods pm ON r.transaction_uuid = pm.transaction_uuid
              WHERE pm.method_type = 'Cash' AND r.processed_at >= ? AND r.processed_at <= ?",
        )
        .bind(&start_str)
        .bind(&end_str)
        .fetch_one(&self.db.pool)
        .await
        .unwrap_or(0.0);

        // Safe Drops
        let drops: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_amount), 0.0) FROM Cash_Counts 
              WHERE count_type = 'drop_safe' AND counted_at >= ? AND counted_at <= ?",
        )
        .bind(&start_str)
        .bind(&end_str)
        .fetch_one(&self.db.pool)
        .await
        .unwrap_or(0.0);

        Ok(CashFlowReport {
            period: format!("{} to {}", start_str, end_str),
            cash_sales: sales,
            cash_refunds: refunds,
            safe_drops: drops,
            net_cash_flow: sales - refunds - drops,
        })
    }

    pub async fn export_transactions_csv(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<String> {
        let start_str = start.to_rfc3339();
        let end_str = end.to_rfc3339();

        let rows = sqlx::query(
            "SELECT t.timestamp, t.transaction_uuid, t.total, t.transaction_type, pm.payment_type 
             FROM Transactions t
             LEFT JOIN Payment_Methods pm ON t.transaction_uuid = pm.transaction_uuid
             WHERE t.timestamp >= ? AND t.timestamp <= ?",
        )
        .bind(start_str)
        .bind(end_str)
        .fetch_all(&self.db.pool)
        .await
        .context("Database error")?;

        let mut csv = String::from("Date,Transaction ID,Type,Total,Payment Method\n");
        for row in rows {
            let ts: String = row.try_get("timestamp").unwrap_or_default();
            let tid: String = row.try_get("transaction_uuid").unwrap_or_default();
            let total: f64 = row.try_get("total").unwrap_or(0.0);
            let ttype: String = row.try_get("transaction_type").unwrap_or_default();
            let method: String = row
                .try_get("payment_type")
                .unwrap_or_else(|_| "Unknown".to_string());

            csv.push_str(&format!(
                "{},{},{},{:.2},{}\n",
                ts, tid, ttype, total, method
            ));
        }

        Ok(csv)
    }
}
