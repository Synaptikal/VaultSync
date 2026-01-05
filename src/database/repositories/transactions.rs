use crate::core::{
    Condition, InventoryItem, Transaction, TransactionItem, TransactionType, VectorTimestamp,
};
use crate::errors::Result;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

#[derive(Clone)]
pub struct TransactionRepository {
    pool: SqlitePool,
    node_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DashboardMetrics {
    pub total_sales_today: f64,
    pub transaction_count_today: i64,
    pub average_transaction_value: f64,
}

/// HIGH-004: Aggregated sales report data structure
#[derive(Serialize, Deserialize, Debug)]
pub struct SalesReportData {
    pub total_sales: f64,
    pub transaction_count: i64,
    pub average_transaction: f64,
    pub top_products: Vec<TopProductData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopProductData {
    pub product_uuid: Uuid,
    pub quantity_sold: i64,
    pub revenue: f64,
}

impl TransactionRepository {
    pub fn new(pool: SqlitePool, node_id: String) -> Self {
        Self { pool, node_id }
    }

    pub async fn insert(&self, transaction: &Transaction) -> Result<()> {
        // Legacy insert - considers non-atomic usage (or used by sync apply)
        // If used individually, it should log sync, but we are moving to atomic execute methods.
        // Keeping as is for sync application where we replay remote changes.

        // Insert Transaction Header
        sqlx::query(
            "INSERT INTO Transactions (transaction_uuid, customer_uuid, user_uuid, timestamp, transaction_type) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(transaction.transaction_uuid.to_string())
        .bind(transaction.customer_uuid.map(|id| id.to_string()))
        .bind(transaction.user_uuid.map(|id| id.to_string()))
        .bind(transaction.timestamp.to_rfc3339())
        .bind(format!("{:?}", transaction.transaction_type))
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        // Insert Transaction Items
        for item in &transaction.items {
            sqlx::query(
                "INSERT INTO Transaction_Items 
                (item_uuid, transaction_uuid, product_uuid, quantity, unit_price, condition) 
                VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(item.item_uuid.to_string())
            .bind(transaction.transaction_uuid.to_string())
            .bind(item.product_uuid.to_string())
            .bind(item.quantity as i64)
            .bind(item.unit_price)
            .bind(format!("{:?}", item.condition))
            .execute(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn get_by_id(&self, transaction_uuid: Uuid) -> Result<Option<Transaction>> {
        let row = sqlx::query("SELECT transaction_uuid, customer_uuid, user_uuid, timestamp, transaction_type FROM Transactions WHERE transaction_uuid = ?")
            .bind(transaction_uuid.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let transaction_uuid_str: String = row.try_get("transaction_uuid").unwrap_or_default();
            let transaction_uuid = Uuid::parse_str(&transaction_uuid_str).unwrap_or_default();
            let customer_uuid_str: Option<String> = row.try_get("customer_uuid").ok();
            let customer_uuid = customer_uuid_str.and_then(|s| Uuid::parse_str(&s).ok());
            let user_uuid_str: Option<String> = row.try_get("user_uuid").ok();
            let user_uuid = user_uuid_str.and_then(|s| Uuid::parse_str(&s).ok());
            let timestamp_str: String = row.try_get("timestamp").unwrap_or_default();
            let timestamp =
                chrono::DateTime::parse_from_rfc3339(&timestamp_str)?.with_timezone(&chrono::Utc);
            let transaction_type_str: String = row.try_get("transaction_type").unwrap_or_default();
            let transaction_type = match transaction_type_str.as_str() {
                "Sale" => TransactionType::Sale,
                "Buy" => TransactionType::Buy,
                "Trade" => TransactionType::Trade,
                "Return" => TransactionType::Return,
                _ => TransactionType::Sale,
            };

            let items = self.get_items(transaction_uuid).await?;

            Ok(Some(Transaction {
                transaction_uuid,
                items,
                customer_uuid,
                user_uuid,
                timestamp,
                transaction_type,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_items(&self, transaction_uuid: Uuid) -> Result<Vec<TransactionItem>> {
        let rows = sqlx::query("SELECT item_uuid, product_uuid, quantity, unit_price, condition FROM Transaction_Items WHERE transaction_uuid = ?")
            .bind(transaction_uuid.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut items = Vec::new();

        for row in rows {
            let item_uuid_str: String = row.try_get("item_uuid").unwrap_or_default();
            let item_uuid = Uuid::parse_str(&item_uuid_str).unwrap_or_default();
            let product_uuid_str: String = row.try_get("product_uuid").unwrap_or_default();
            let product_uuid = Uuid::parse_str(&product_uuid_str).unwrap_or_default();
            let quantity: i64 = row.try_get("quantity").unwrap_or_default();
            let unit_price: f64 = row.try_get("unit_price").unwrap_or_default();
            let condition_str: String = row.try_get("condition").unwrap_or_default();
            let condition = Self::parse_condition(&condition_str);

            items.push(TransactionItem {
                item_uuid,
                product_uuid,
                quantity: quantity as i32,
                unit_price,
                condition,
            });
        }

        Ok(items)
    }

    pub async fn get_by_customer(&self, customer_uuid: Uuid) -> Result<Vec<Transaction>> {
        let rows = sqlx::query("SELECT transaction_uuid, customer_uuid, user_uuid, timestamp, transaction_type FROM Transactions WHERE customer_uuid = ? ORDER BY timestamp DESC")
            .bind(customer_uuid.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        // Collect UUIDs and build transaction map
        let mut tx_uuids: Vec<String> = Vec::new();
        let mut tx_map: std::collections::HashMap<Uuid, Transaction> =
            std::collections::HashMap::new();

        for row in &rows {
            let transaction_uuid_str: String = row.try_get("transaction_uuid").unwrap_or_default();
            let transaction_uuid = Uuid::parse_str(&transaction_uuid_str).unwrap_or_default();
            let customer_uuid_str: Option<String> = row.try_get("customer_uuid").ok();
            let customer_uuid = customer_uuid_str.and_then(|s| Uuid::parse_str(&s).ok());
            let user_uuid_str: Option<String> = row.try_get("user_uuid").ok();
            let user_uuid = user_uuid_str.and_then(|s| Uuid::parse_str(&s).ok());
            let timestamp_str: String = row.try_get("timestamp").unwrap_or_default();
            let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_str)
                .unwrap_or_default()
                .with_timezone(&chrono::Utc);
            let tx_type_str: String = row.try_get("transaction_type").unwrap_or_default();
            let transaction_type = match tx_type_str.as_str() {
                "Sale" => TransactionType::Sale,
                "Buy" => TransactionType::Buy,
                "Trade" => TransactionType::Trade,
                "Return" => TransactionType::Return,
                _ => TransactionType::Sale,
            };

            tx_uuids.push(transaction_uuid_str);
            tx_map.insert(
                transaction_uuid,
                Transaction {
                    transaction_uuid,
                    items: Vec::new(),
                    customer_uuid,
                    user_uuid,
                    timestamp,
                    transaction_type,
                },
            );
        }

        // Batch fetch items
        let placeholders: String = tx_uuids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT transaction_uuid, item_uuid, product_uuid, quantity, unit_price, condition 
             FROM Transaction_Items 
             WHERE transaction_uuid IN ({})",
            placeholders
        );

        let mut query_builder = sqlx::query(&query);
        for uuid in &tx_uuids {
            query_builder = query_builder.bind(uuid);
        }

        let item_rows = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        for row in item_rows {
            let tx_uuid_str: String = row.try_get("transaction_uuid").unwrap_or_default();
            let tx_uuid = Uuid::parse_str(&tx_uuid_str).unwrap_or_default();

            let item_uuid_str: String = row.try_get("item_uuid").unwrap_or_default();
            let item_uuid = Uuid::parse_str(&item_uuid_str).unwrap_or_default();
            let product_uuid_str: String = row.try_get("product_uuid").unwrap_or_default();
            let product_uuid = Uuid::parse_str(&product_uuid_str).unwrap_or_default();
            let quantity: i64 = row.try_get("quantity").unwrap_or_default();
            let unit_price: f64 = row.try_get("unit_price").unwrap_or_default();
            let condition_str: String = row.try_get("condition").unwrap_or_default();
            let condition = Self::parse_condition(&condition_str);

            if let Some(tx) = tx_map.get_mut(&tx_uuid) {
                tx.items.push(TransactionItem {
                    item_uuid,
                    product_uuid,
                    quantity: quantity as i32,
                    unit_price,
                    condition,
                });
            }
        }

        // Return in order
        let mut result = Vec::new();
        for row in &rows {
            let tx_uuid_str: String = row.try_get("transaction_uuid").unwrap_or_default();
            let tx_uuid = Uuid::parse_str(&tx_uuid_str).unwrap_or_default();
            if let Some(tx) = tx_map.remove(&tx_uuid) {
                result.push(tx);
            }
        }
        Ok(result)
    }

    pub async fn get_recent(&self, limit: i64) -> Result<Vec<Transaction>> {
        let rows = sqlx::query(
            "SELECT transaction_uuid FROM Transactions ORDER BY timestamp DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut transactions = Vec::new();
        for row in rows {
            let transaction_uuid_str: String = row.try_get("transaction_uuid").unwrap_or_default();
            let transaction_uuid = Uuid::parse_str(&transaction_uuid_str).unwrap_or_default();
            if let Some(transaction) = self.get_by_id(transaction_uuid).await? {
                transactions.push(transaction);
            }
        }
        Ok(transactions)
    }

    // --- AGGREGATION METHODS ---
    pub async fn get_dashboard_metrics(&self) -> Result<DashboardMetrics> {
        let today = chrono::Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            // SECURITY FIX: and_hms_opt only returns None for invalid h/m/s values; 0,0,0 is always valid
            .expect("0,0,0 is always a valid time")
            .to_string();

        let row = sqlx::query(
            "SELECT 
                COUNT(*) as tx_count, 
                COALESCE(SUM(ti.quantity * ti.unit_price), 0.0) as total_sales
             FROM Transactions t
             JOIN Transaction_Items ti ON t.transaction_uuid = ti.transaction_uuid
             WHERE t.timestamp >= ? AND t.transaction_type = 'Sale'",
        )
        .bind(&today)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let tx_count: i64 = row.try_get("tx_count").unwrap_or(0);
        let total_sales: f64 = row.try_get("total_sales").unwrap_or(0.0);
        let avg_val = if tx_count > 0 {
            total_sales / tx_count as f64
        } else {
            0.0
        };

        Ok(DashboardMetrics {
            total_sales_today: total_sales,
            transaction_count_today: tx_count,
            average_transaction_value: avg_val,
        })
    }

    /// HIGH-004 FIX: Get sales transactions filtered by date range at SQL level
    pub async fn get_sales_by_date_range(
        &self,
        start_date: &str,
        end_date: &str,
        limit: i64,
    ) -> Result<Vec<Transaction>> {
        // Query transaction IDs first with SQL date filtering
        let rows = sqlx::query(
            "SELECT transaction_uuid, customer_uuid, user_uuid, timestamp, transaction_type 
             FROM Transactions 
             WHERE timestamp >= ? AND timestamp < ? AND transaction_type = 'Sale'
             ORDER BY timestamp DESC
             LIMIT ?",
        )
        .bind(start_date)
        .bind(end_date)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        // Collect transaction UUIDs for batch item fetching
        let mut tx_uuids: Vec<String> = Vec::new();
        let mut tx_map: std::collections::HashMap<Uuid, Transaction> =
            std::collections::HashMap::new();

        for row in &rows {
            let transaction_uuid_str: String = row.try_get("transaction_uuid").unwrap_or_default();
            let transaction_uuid = Uuid::parse_str(&transaction_uuid_str).unwrap_or_default();
            let customer_uuid_str: Option<String> = row.try_get("customer_uuid").ok();
            let customer_uuid = customer_uuid_str.and_then(|s| Uuid::parse_str(&s).ok());
            let user_uuid_str: Option<String> = row.try_get("user_uuid").ok();
            let user_uuid = user_uuid_str.and_then(|s| Uuid::parse_str(&s).ok());
            let timestamp_str: String = row.try_get("timestamp").unwrap_or_default();
            let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_str)
                .unwrap_or_default()
                .with_timezone(&chrono::Utc);

            tx_uuids.push(transaction_uuid_str);
            tx_map.insert(
                transaction_uuid,
                Transaction {
                    transaction_uuid,
                    items: Vec::new(), // Will be filled by batch query
                    customer_uuid,
                    user_uuid,
                    timestamp,
                    transaction_type: TransactionType::Sale,
                },
            );
        }

        // HIGH-008 FIX: Batch fetch all items for all transactions in ONE query
        let placeholders: String = tx_uuids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT transaction_uuid, item_uuid, product_uuid, quantity, unit_price, condition 
             FROM Transaction_Items 
             WHERE transaction_uuid IN ({})",
            placeholders
        );

        let mut query_builder = sqlx::query(&query);
        for uuid in &tx_uuids {
            query_builder = query_builder.bind(uuid);
        }

        let item_rows = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        // Assign items to their transactions
        for row in item_rows {
            let tx_uuid_str: String = row.try_get("transaction_uuid").unwrap_or_default();
            let tx_uuid = Uuid::parse_str(&tx_uuid_str).unwrap_or_default();

            let item_uuid_str: String = row.try_get("item_uuid").unwrap_or_default();
            let item_uuid = Uuid::parse_str(&item_uuid_str).unwrap_or_default();
            let product_uuid_str: String = row.try_get("product_uuid").unwrap_or_default();
            let product_uuid = Uuid::parse_str(&product_uuid_str).unwrap_or_default();
            let quantity: i64 = row.try_get("quantity").unwrap_or_default();
            let unit_price: f64 = row.try_get("unit_price").unwrap_or_default();
            let condition_str: String = row.try_get("condition").unwrap_or_default();
            let condition = Self::parse_condition(&condition_str);

            if let Some(tx) = tx_map.get_mut(&tx_uuid) {
                tx.items.push(TransactionItem {
                    item_uuid,
                    product_uuid,
                    quantity: quantity as i32,
                    unit_price,
                    condition,
                });
            }
        }

        // Return in order
        let mut result = Vec::new();
        for row in &rows {
            let tx_uuid_str: String = row.try_get("transaction_uuid").unwrap_or_default();
            let tx_uuid = Uuid::parse_str(&tx_uuid_str).unwrap_or_default();
            if let Some(tx) = tx_map.remove(&tx_uuid) {
                result.push(tx);
            }
        }

        Ok(result)
    }

    /// HIGH-008 FIX: Get recent transactions with batch item loading (2 queries instead of N+1)
    pub async fn get_recent_optimized(&self, limit: i64) -> Result<Vec<Transaction>> {
        let rows = sqlx::query(
            "SELECT transaction_uuid, customer_uuid, user_uuid, timestamp, transaction_type 
             FROM Transactions 
             ORDER BY timestamp DESC 
             LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        // Collect UUIDs and build transaction map
        let mut tx_uuids: Vec<String> = Vec::new();
        let mut tx_map: std::collections::HashMap<Uuid, Transaction> =
            std::collections::HashMap::new();

        for row in &rows {
            let transaction_uuid_str: String = row.try_get("transaction_uuid").unwrap_or_default();
            let transaction_uuid = Uuid::parse_str(&transaction_uuid_str).unwrap_or_default();
            let customer_uuid_str: Option<String> = row.try_get("customer_uuid").ok();
            let customer_uuid = customer_uuid_str.and_then(|s| Uuid::parse_str(&s).ok());
            let user_uuid_str: Option<String> = row.try_get("user_uuid").ok();
            let user_uuid = user_uuid_str.and_then(|s| Uuid::parse_str(&s).ok());
            let timestamp_str: String = row.try_get("timestamp").unwrap_or_default();
            let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_str)
                .unwrap_or_default()
                .with_timezone(&chrono::Utc);
            let tx_type_str: String = row.try_get("transaction_type").unwrap_or_default();
            let transaction_type = match tx_type_str.as_str() {
                "Sale" => TransactionType::Sale,
                "Buy" => TransactionType::Buy,
                "Trade" => TransactionType::Trade,
                "Return" => TransactionType::Return,
                _ => TransactionType::Sale,
            };

            tx_uuids.push(transaction_uuid_str);
            tx_map.insert(
                transaction_uuid,
                Transaction {
                    transaction_uuid,
                    items: Vec::new(),
                    customer_uuid,
                    user_uuid,
                    timestamp,
                    transaction_type,
                },
            );
        }

        // Batch fetch items
        let placeholders: String = tx_uuids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT transaction_uuid, item_uuid, product_uuid, quantity, unit_price, condition 
             FROM Transaction_Items 
             WHERE transaction_uuid IN ({})",
            placeholders
        );

        let mut query_builder = sqlx::query(&query);
        for uuid in &tx_uuids {
            query_builder = query_builder.bind(uuid);
        }

        let item_rows = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        for row in item_rows {
            let tx_uuid_str: String = row.try_get("transaction_uuid").unwrap_or_default();
            let tx_uuid = Uuid::parse_str(&tx_uuid_str).unwrap_or_default();

            let item_uuid_str: String = row.try_get("item_uuid").unwrap_or_default();
            let item_uuid = Uuid::parse_str(&item_uuid_str).unwrap_or_default();
            let product_uuid_str: String = row.try_get("product_uuid").unwrap_or_default();
            let product_uuid = Uuid::parse_str(&product_uuid_str).unwrap_or_default();
            let quantity: i64 = row.try_get("quantity").unwrap_or_default();
            let unit_price: f64 = row.try_get("unit_price").unwrap_or_default();
            let condition_str: String = row.try_get("condition").unwrap_or_default();
            let condition = Self::parse_condition(&condition_str);

            if let Some(tx) = tx_map.get_mut(&tx_uuid) {
                tx.items.push(TransactionItem {
                    item_uuid,
                    product_uuid,
                    quantity: quantity as i32,
                    unit_price,
                    condition,
                });
            }
        }

        // Return in order
        let mut result = Vec::new();
        for row in &rows {
            let tx_uuid_str: String = row.try_get("transaction_uuid").unwrap_or_default();
            let tx_uuid = Uuid::parse_str(&tx_uuid_str).unwrap_or_default();
            if let Some(tx) = tx_map.remove(&tx_uuid) {
                result.push(tx);
            }
        }

        Ok(result)
    }

    /// Get aggregated sales report data at SQL level (for HIGH-004)
    pub async fn get_sales_report_aggregated(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<SalesReportData> {
        // Get totals
        let totals_row = sqlx::query(
            "SELECT 
                COUNT(DISTINCT t.transaction_uuid) as tx_count,
                COALESCE(SUM(ti.quantity * ti.unit_price), 0.0) as total_sales
             FROM Transactions t
             JOIN Transaction_Items ti ON t.transaction_uuid = ti.transaction_uuid
             WHERE t.timestamp >= ? AND t.timestamp < ? AND t.transaction_type = 'Sale'",
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let transaction_count: i64 = totals_row.try_get("tx_count").unwrap_or(0);
        let total_sales: f64 = totals_row.try_get("total_sales").unwrap_or(0.0);

        // Get top selling products with aggregation at SQL level
        let top_rows = sqlx::query(
            "SELECT 
                ti.product_uuid,
                SUM(ti.quantity) as total_quantity,
                SUM(ti.quantity * ti.unit_price) as total_revenue
             FROM Transactions t
             JOIN Transaction_Items ti ON t.transaction_uuid = ti.transaction_uuid
             WHERE t.timestamp >= ? AND t.timestamp < ? AND t.transaction_type = 'Sale'
             GROUP BY ti.product_uuid
             ORDER BY total_revenue DESC
             LIMIT 10",
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut top_products = Vec::new();
        for row in top_rows {
            let product_uuid_str: String = row.try_get("product_uuid").unwrap_or_default();
            let product_uuid = Uuid::parse_str(&product_uuid_str).unwrap_or_default();
            let total_quantity: i64 = row.try_get("total_quantity").unwrap_or(0);
            let total_revenue: f64 = row.try_get("total_revenue").unwrap_or(0.0);

            top_products.push(TopProductData {
                product_uuid,
                quantity_sold: total_quantity,
                revenue: total_revenue,
            });
        }

        Ok(SalesReportData {
            total_sales,
            transaction_count,
            average_transaction: if transaction_count > 0 {
                total_sales / transaction_count as f64
            } else {
                0.0
            },
            top_products,
        })
    }

    pub async fn get_sales_by_category(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<std::collections::HashMap<String, f64>> {
        let rows = sqlx::query(
            "SELECT 
                 p.category,
                 COALESCE(SUM(ti.quantity * ti.unit_price), 0.0) as revenue
              FROM Transactions t
              JOIN Transaction_Items ti ON t.transaction_uuid = ti.transaction_uuid
              JOIN Global_Catalog p ON ti.product_uuid = p.product_uuid
              WHERE t.timestamp >= ? AND t.timestamp < ? AND t.transaction_type = 'Sale'
              GROUP BY p.category",
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut result = std::collections::HashMap::new();
        for row in rows {
            let category: String = row
                .try_get("category")
                .unwrap_or_else(|_| "Unknown".to_string());
            let revenue: f64 = row.try_get("revenue").unwrap_or(0.0);
            result.insert(category, revenue);
        }
        Ok(result)
    }

    pub async fn get_sales_by_employee(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<(Option<Uuid>, i64, f64)>> {
        // Returns (user_uuid, count, total_sales)
        let rows = sqlx::query(
            "SELECT 
                 t.user_uuid,
                 COUNT(DISTINCT t.transaction_uuid) as tx_count,
                 COALESCE(SUM(ti.quantity * ti.unit_price), 0.0) as total_sales
              FROM Transactions t
              JOIN Transaction_Items ti ON t.transaction_uuid = ti.transaction_uuid
              WHERE t.timestamp >= ? AND t.timestamp < ? AND t.transaction_type = 'Sale'
              GROUP BY t.user_uuid",
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            let user_uuid_str: Option<String> = row.try_get("user_uuid").ok();
            let user_uuid = user_uuid_str.and_then(|s| Uuid::parse_str(&s).ok());
            let count: i64 = row.try_get("tx_count").unwrap_or(0);
            let revenue: f64 = row.try_get("total_sales").unwrap_or(0.0);
            result.push((user_uuid, count, revenue));
        }
        Ok(result)
    }

    pub async fn get_sales_by_payment_method(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<std::collections::HashMap<String, f64>> {
        // Query Payment_Methods table directly
        // Note: We should filter by transaction date, so we join Transactions
        let rows = sqlx::query(
            "SELECT 
                 pm.method_type,
                 COALESCE(SUM(pm.amount), 0.0) as total
              FROM Payment_Methods pm
              JOIN Transactions t ON pm.transaction_uuid = t.transaction_uuid
              WHERE t.timestamp >= ? AND t.timestamp < ? AND t.transaction_type = 'Sale'
              GROUP BY pm.method_type",
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut result = std::collections::HashMap::new();
        for row in rows {
            let method: String = row
                .try_get("method_type")
                .unwrap_or_else(|_| "Unknown".to_string());
            let total: f64 = row.try_get("total").unwrap_or(0.0);
            result.insert(method, total);
        }
        Ok(result)
    }

    // --- ATOMIC TRANSACTION METHODS ---

    // Helper to log changes within a transaction
    async fn log_change_internal<'a>(
        &self,
        tx: &mut sqlx::Transaction<'a, sqlx::Sqlite>,
        record_id: &str,
        record_type: &str,
        operation: &str,
        data: &serde_json::Value,
    ) -> Result<()> {
        // 1. Get Vector Timestamp
        let rows =
            sqlx::query("SELECT node_id, counter FROM Version_Vectors WHERE entity_uuid = ?")
                .bind(record_id)
                .fetch_all(&mut **tx)
                .await
                .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut entries = std::collections::HashMap::new();
        for row in rows {
            let node_id: String = row.try_get("node_id").unwrap_or_default();
            let counter: i64 = row.try_get("counter").unwrap_or_default();
            entries.insert(node_id, counter as u64);
        }
        let mut vector = VectorTimestamp::from_entries(entries);

        // 2. Increment
        vector.increment(self.node_id.clone());

        // 3. Update Vector
        for (node_id, counter) in &vector.entries {
            sqlx::query("INSERT OR REPLACE INTO Version_Vectors (entity_uuid, node_id, counter) VALUES (?, ?, ?)")
                .bind(record_id)
                .bind(node_id)
                .bind(*counter as i64)
                .execute(&mut **tx)
                .await
                .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        }

        // 4. Log Change
        sqlx::query(
            "INSERT OR REPLACE INTO Sync_Log 
            (record_id, record_type, operation, data, node_id, local_clock, version_vector, timestamp) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(record_id)
        .bind(record_type)
        .bind(operation)
        .bind(data.to_string())
        .bind(&self.node_id)
        .bind(vector.get_clock(&self.node_id) as i64)
        // SECURITY FIX: Use ? operator instead of unwrap to propagate serialization errors
        .bind(serde_json::to_string(&vector)?)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // Internal helper for logging changes
    // (Already defined above, not replacing it, but assuming it's available in context if I was inside impl)

    // Internal Helper for Sale Logic
    async fn execute_sale_internal<'a>(
        &self,
        tx: &mut sqlx::Transaction<'a, sqlx::Sqlite>,
        customer_uuid: Option<Uuid>,
        user_uuid: Option<Uuid>,
        items: Vec<TransactionItem>,
        transaction_type: TransactionType,
    ) -> Result<Transaction> {
        // 1. Validate and Deduct Inventory
        for item in &items {
            let rows = sqlx::query(
                "SELECT * FROM Local_Inventory 
                 WHERE product_uuid = ? AND condition = ? 
                 ORDER BY inventory_uuid ASC",
            )
            .bind(item.product_uuid.to_string())
            .bind(format!("{:?}", item.condition))
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

            let mut remaining_needed = item.quantity;
            let mut available_total = 0;
            for row in &rows {
                let q: i64 = row.try_get("quantity_on_hand").unwrap_or(0);
                available_total += q as i32;
            }

            if available_total < remaining_needed {
                return Err(crate::errors::VaultSyncError::InventoryError(format!(
                    "Insufficient inventory for product {}. Required: {}, Available: {}",
                    item.product_uuid, item.quantity, available_total
                ))
                .into());
            }

            // Deduct
            for row in rows {
                if remaining_needed <= 0 {
                    break;
                }

                // SECURITY FIX: Replaced unwrap() with map_err for production safety
                let inv_uuid_str: String = row.try_get("inventory_uuid").map_err(|e| {
                    crate::errors::VaultSyncError::DatabaseError(format!(
                        "Missing inventory_uuid: {}",
                        e
                    ))
                })?;
                let product_uuid_str: String = row.try_get("product_uuid").map_err(|e| {
                    crate::errors::VaultSyncError::DatabaseError(format!(
                        "Missing product_uuid: {}",
                        e
                    ))
                })?;
                let quantity_on_hand: i64 = row.try_get("quantity_on_hand").map_err(|e| {
                    crate::errors::VaultSyncError::DatabaseError(format!(
                        "Missing quantity_on_hand: {}",
                        e
                    ))
                })?;
                let location_tag: String = row.try_get("location_tag").unwrap_or_default();
                let condition_str: String = row.try_get("condition").unwrap_or_default();
                let _variant_str: Option<String> = row.try_get("variant_type").ok();
                let specific_price: Option<f64> = row.try_get("specific_price").ok();
                let serialized_str: Option<String> = row.try_get("serialized_details").ok();

                let mut inv_item = InventoryItem {
                    inventory_uuid: Uuid::parse_str(&inv_uuid_str).map_err(|e| {
                        crate::errors::VaultSyncError::DatabaseError(format!(
                            "Invalid inventory UUID '{}': {}",
                            inv_uuid_str, e
                        ))
                    })?,
                    product_uuid: Uuid::parse_str(&product_uuid_str).map_err(|e| {
                        crate::errors::VaultSyncError::DatabaseError(format!(
                            "Invalid product UUID '{}': {}",
                            product_uuid_str, e
                        ))
                    })?,
                    quantity_on_hand: quantity_on_hand as i32,
                    location_tag,
                    condition: Self::parse_condition(&condition_str),
                    variant_type: None,
                    specific_price,
                    serialized_details: serialized_str.and_then(|s| serde_json::from_str(&s).ok()),
                    cost_basis: None,
                    supplier_uuid: None,
                    received_date: None,
                    min_stock_level: 0,
                    max_stock_level: None,
                    reorder_point: None,
                    bin_location: None,
                    last_sold_date: None,
                    last_counted_date: None,
                    deleted_at: None,
                };

                let current_qty = quantity_on_hand as i32;
                let deduct = std::cmp::min(remaining_needed, current_qty);
                let new_qty = current_qty - deduct;
                remaining_needed -= deduct;

                inv_item.quantity_on_hand = new_qty;

                if new_qty == 0 {
                    sqlx::query(
                        "UPDATE Local_Inventory SET quantity_on_hand = 0 WHERE inventory_uuid = ?",
                    )
                    .bind(&inv_uuid_str)
                    .execute(&mut **tx)
                    .await
                    .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
                } else {
                    sqlx::query(
                        "UPDATE Local_Inventory SET quantity_on_hand = ? WHERE inventory_uuid = ?",
                    )
                    .bind(new_qty as i64)
                    .bind(&inv_uuid_str)
                    .execute(&mut **tx)
                    .await
                    .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
                }

                // SECURITY FIX: Use ? operator for serialization
                self.log_change_internal(
                    tx,
                    &inv_uuid_str,
                    "InventoryItem",
                    "Update",
                    &serde_json::to_value(&inv_item)?,
                )
                .await?;
            }
        }

        // 2. Create Transaction Record
        let transaction_uuid = Uuid::new_v4();
        let timestamp = chrono::Utc::now();

        let transaction = Transaction {
            transaction_uuid,
            items: items.clone(),
            customer_uuid,
            user_uuid,
            timestamp,
            transaction_type: transaction_type.clone(),
        };

        sqlx::query(
            "INSERT INTO Transactions (transaction_uuid, customer_uuid, user_uuid, timestamp, transaction_type) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(transaction_uuid.to_string())
        .bind(customer_uuid.map(|id| id.to_string()))
        .bind(user_uuid.map(|id| id.to_string()))
        .bind(timestamp.to_rfc3339())
        .bind(format!("{:?}", transaction_type))
        .execute(&mut **tx)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        // 3. Create Transaction Items
        for item in &items {
            sqlx::query(
                "INSERT INTO Transaction_Items 
                (item_uuid, transaction_uuid, product_uuid, quantity, unit_price, condition) 
                VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(transaction_uuid.to_string())
            .bind(item.product_uuid.to_string())
            .bind(item.quantity as i64)
            .bind(item.unit_price)
            .bind(format!("{:?}", item.condition))
            .execute(&mut **tx)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        }

        // SECURITY FIX: Use ? operator for serialization
        self.log_change_internal(
            tx,
            &transaction_uuid.to_string(),
            "Transaction",
            "Insert",
            &serde_json::to_value(&transaction)?,
        )
        .await?;

        Ok(transaction)
    }

    // Internal Helper for Buy Logic
    async fn execute_buy_internal<'a>(
        &self,
        tx: &mut sqlx::Transaction<'a, sqlx::Sqlite>,
        customer_uuid: Option<Uuid>,
        user_uuid: Option<Uuid>,
        items: Vec<TransactionItem>,
        transaction_type: TransactionType,
    ) -> Result<Transaction> {
        // 1. Add Inventory
        for item in &items {
            // Check for existing bulk pile (Same product, condition, no special fields)
            let existing_row = sqlx::query(
                "SELECT * FROM Local_Inventory 
                 WHERE product_uuid = ? AND condition = ? AND serialized_details IS NULL AND specific_price IS NULL
                 LIMIT 1"
            )
            .bind(item.product_uuid.to_string())
            .bind(format!("{:?}", item.condition))
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

            if let Some(row) = existing_row {
                // Update existing
                // SECURITY FIX: Use map_err for proper error handling
                let inv_uuid_str: String = row.try_get("inventory_uuid").map_err(|e| {
                    crate::errors::VaultSyncError::DatabaseError(format!(
                        "Missing inventory_uuid: {}",
                        e
                    ))
                })?;
                let current_qty: i64 = row.try_get("quantity_on_hand").map_err(|e| {
                    crate::errors::VaultSyncError::DatabaseError(format!(
                        "Missing quantity_on_hand: {}",
                        e
                    ))
                })?;
                let new_qty = current_qty + item.quantity as i64;

                sqlx::query(
                    "UPDATE Local_Inventory SET quantity_on_hand = ? WHERE inventory_uuid = ?",
                )
                .bind(new_qty)
                .bind(&inv_uuid_str)
                .execute(&mut **tx)
                .await
                .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

                let product_uuid_str: String = row.try_get("product_uuid").map_err(|e| {
                    crate::errors::VaultSyncError::DatabaseError(format!(
                        "Missing product_uuid: {}",
                        e
                    ))
                })?;
                let location_tag: String = row.try_get("location_tag").unwrap_or_default();
                let condition_str: String = row.try_get("condition").unwrap_or_default();

                let inv_item = InventoryItem {
                    inventory_uuid: Uuid::parse_str(&inv_uuid_str)?,
                    product_uuid: Uuid::parse_str(&product_uuid_str)?,
                    quantity_on_hand: new_qty as i32,
                    location_tag,
                    condition: Self::parse_condition(&condition_str),
                    variant_type: None,
                    specific_price: None,
                    serialized_details: None,
                    cost_basis: None,
                    supplier_uuid: None,
                    received_date: None,
                    min_stock_level: 0,
                    max_stock_level: None,
                    reorder_point: None,
                    bin_location: None,
                    last_sold_date: None,
                    last_counted_date: None,
                    deleted_at: None,
                };
                self.log_change_internal(
                    tx,
                    &inv_uuid_str,
                    "InventoryItem",
                    "Update",
                    &serde_json::to_value(&inv_item)?,
                )
                .await?;
            } else {
                // Insert new
                let new_inv_uuid = Uuid::new_v4();
                let inventory_item = InventoryItem {
                    inventory_uuid: new_inv_uuid,
                    product_uuid: item.product_uuid,
                    variant_type: None,
                    condition: item.condition.clone(),
                    quantity_on_hand: item.quantity,
                    location_tag: "Purchased".to_string(),
                    specific_price: None,
                    serialized_details: None,
                    cost_basis: None,
                    supplier_uuid: None,
                    received_date: None,
                    min_stock_level: 0,
                    max_stock_level: None,
                    reorder_point: None,
                    bin_location: None,
                    last_sold_date: None,
                    last_counted_date: None,
                    deleted_at: None,
                };

                sqlx::query(
                    "INSERT INTO Local_Inventory (inventory_uuid, product_uuid, quantity_on_hand, condition, location_tag)
                     VALUES (?, ?, ?, ?, ?)"
                )
                .bind(new_inv_uuid.to_string())
                .bind(item.product_uuid.to_string())
                .bind(item.quantity)
                .bind(format!("{:?}", item.condition))
                .bind("Purchased")
                .execute(&mut **tx)
                .await
                .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

                self.log_change_internal(
                    tx,
                    &new_inv_uuid.to_string(),
                    "InventoryItem",
                    "Insert",
                    &serde_json::to_value(&inventory_item)?,
                )
                .await?;
            }
        }

        // 2. Create Transaction Record
        let transaction_uuid = Uuid::new_v4();
        let timestamp = chrono::Utc::now();

        let transaction = Transaction {
            transaction_uuid,
            items: items.clone(),
            customer_uuid,
            user_uuid,
            timestamp,
            transaction_type: transaction_type.clone(),
        };

        sqlx::query(
            "INSERT INTO Transactions (transaction_uuid, customer_uuid, user_uuid, timestamp, transaction_type) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(transaction_uuid.to_string())
        .bind(customer_uuid.map(|id| id.to_string()))
        .bind(user_uuid.map(|id| id.to_string()))
        .bind(timestamp.to_rfc3339())
        .bind(format!("{:?}", transaction_type))
        .execute(&mut **tx)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        // 3. Create Transaction Items
        for item in &items {
            sqlx::query(
                "INSERT INTO Transaction_Items 
                (item_uuid, transaction_uuid, product_uuid, quantity, unit_price, condition) 
                VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(transaction_uuid.to_string())
            .bind(item.product_uuid.to_string())
            .bind(item.quantity as i64)
            .bind(item.unit_price)
            .bind(format!("{:?}", item.condition))
            .execute(&mut **tx)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        }

        self.log_change_internal(
            tx,
            &transaction_uuid.to_string(),
            "Transaction",
            "Insert",
            &serde_json::to_value(&transaction)?,
        )
        .await?;

        Ok(transaction)
    }

    pub async fn execute_sale(
        &self,
        customer_uuid: Option<Uuid>,
        user_uuid: Option<Uuid>,
        items: Vec<TransactionItem>,
    ) -> Result<Transaction> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        let transaction = self
            .execute_sale_internal(
                &mut tx,
                customer_uuid,
                user_uuid,
                items,
                TransactionType::Sale,
            )
            .await?;
        tx.commit()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        Ok(transaction)
    }

    pub async fn execute_buy(
        &self,
        customer_uuid: Option<Uuid>,
        user_uuid: Option<Uuid>,
        items: Vec<TransactionItem>,
    ) -> Result<Transaction> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        let transaction = self
            .execute_buy_internal(
                &mut tx,
                customer_uuid,
                user_uuid,
                items,
                TransactionType::Buy,
            )
            .await?;
        tx.commit()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        Ok(transaction)
    }

    pub async fn execute_trade(
        &self,
        customer_uuid: Option<Uuid>,
        user_uuid: Option<Uuid>,
        trade_in_items: Vec<TransactionItem>,
        trade_out_items: Vec<TransactionItem>,
    ) -> Result<(Transaction, Transaction)> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        // Process trade-in as Buy (TransactionType::Trade)
        let buy_transaction = self
            .execute_buy_internal(
                &mut tx,
                customer_uuid,
                user_uuid,
                trade_in_items,
                TransactionType::Trade,
            )
            .await?;

        // Process trade-out as Sale (TransactionType::Trade)
        let sale_transaction = self
            .execute_sale_internal(
                &mut tx,
                customer_uuid,
                user_uuid,
                trade_out_items,
                TransactionType::Trade,
            )
            .await?;

        tx.commit()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok((buy_transaction, sale_transaction))
    }

    fn parse_condition(s: &str) -> Condition {
        match s {
            "NM" => Condition::NM,
            "LP" => Condition::LP,
            "MP" => Condition::MP,
            "HP" => Condition::HP,
            "DMG" => Condition::DMG,
            "New" => Condition::New,
            "OpenBox" => Condition::OpenBox,
            "Used" => Condition::Used,
            "GemMint" => Condition::GemMint,
            "Mint" => Condition::Mint,
            "NearMintMint" => Condition::NearMintMint,
            "VeryFine" => Condition::VeryFine,
            "Fine" => Condition::Fine,
            "Good" => Condition::Good,
            "Poor" => Condition::Poor,
            _ => Condition::Used,
        }
    }
}
