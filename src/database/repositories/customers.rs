use crate::core::{Customer, WantsItem, WantsList};
use crate::errors::Result;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use super::sync::SyncRepository;

#[derive(Clone)]
pub struct CustomerRepository {
    pool: SqlitePool,
    sync: SyncRepository,
}

impl CustomerRepository {
    pub fn new(pool: SqlitePool, sync: SyncRepository) -> Self {
        Self { pool, sync }
    }

    pub async fn insert(&self, customer: &Customer) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO Customers 
            (customer_uuid, name, email, phone, store_credit, tier, created_at) 
            VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(customer.customer_uuid.to_string())
        .bind(&customer.name)
        .bind(&customer.email)
        .bind(&customer.phone)
        .bind(customer.store_credit)
        .bind(&customer.tier)
        .bind(customer.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn get_all(&self) -> Result<Vec<Customer>> {
        let rows = sqlx::query("SELECT customer_uuid, name, email, phone, store_credit, tier, created_at FROM Customers")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut customers = Vec::new();

        for row in rows {
            let customer_uuid_str: String = row.try_get("customer_uuid").unwrap_or_default();
            let customer_uuid = Uuid::parse_str(&customer_uuid_str).unwrap_or_default();
            let name: String = row.try_get("name").unwrap_or_default();
            let email: Option<String> = row.try_get("email").ok();
            let phone: Option<String> = row.try_get("phone").ok();
            let store_credit: f64 = row.try_get("store_credit").unwrap_or_default();
            let tier: Option<String> = row.try_get("tier").ok();
            let created_at_str: String = row.try_get("created_at").unwrap_or_default();
            let created_at =
                chrono::DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&chrono::Utc);

            customers.push(Customer {
                customer_uuid,
                name,
                email,
                phone,
                store_credit,
                tier,
                created_at,
            });
        }
        Ok(customers)
    }

    pub async fn get_by_id(&self, customer_uuid: Uuid) -> Result<Option<Customer>> {
        let row = sqlx::query("SELECT customer_uuid, name, email, phone, store_credit, tier, created_at FROM Customers WHERE customer_uuid = ?")
            .bind(customer_uuid.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let customer_uuid_str: String = row.try_get("customer_uuid").unwrap_or_default();
            let customer_uuid = Uuid::parse_str(&customer_uuid_str).unwrap_or_default();
            let name: String = row.try_get("name").unwrap_or_default();
            let email: Option<String> = row.try_get("email").ok();
            let phone: Option<String> = row.try_get("phone").ok();
            let store_credit: f64 = row.try_get("store_credit").unwrap_or_default();
            let tier: Option<String> = row.try_get("tier").ok();
            let created_at_str: String = row.try_get("created_at").unwrap_or_default();
            let created_at =
                chrono::DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&chrono::Utc);

            Ok(Some(Customer {
                customer_uuid,
                name,
                email,
                phone,
                store_credit,
                tier,
                created_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_store_credit(&self, customer_uuid: Uuid, amount: f64) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        sqlx::query("UPDATE Customers SET store_credit = store_credit + ? WHERE customer_uuid = ?")
            .bind(amount)
            .bind(customer_uuid.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        // Fetch for logging
        let row = sqlx::query("SELECT customer_uuid, name, email, phone, store_credit, tier, created_at FROM Customers WHERE customer_uuid = ?")
            .bind(customer_uuid.to_string())
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let customer_uuid_str: String = row.try_get("customer_uuid").unwrap_or_default();
            let customer_uuid = Uuid::parse_str(&customer_uuid_str).unwrap_or_default();
            let name: String = row.try_get("name").unwrap_or_default();
            let email: Option<String> = row.try_get("email").ok();
            let phone: Option<String> = row.try_get("phone").ok();
            let store_credit: f64 = row.try_get("store_credit").unwrap_or_default();
            let tier: Option<String> = row.try_get("tier").ok();
            let created_at_str: String = row.try_get("created_at").unwrap_or_default();
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .unwrap_or_default()
                .with_timezone(&chrono::Utc);

            let customer = Customer {
                customer_uuid,
                name,
                email,
                phone,
                store_credit,
                tier,
                created_at,
            };

            self.sync
                .log_change_with_tx(
                    &mut tx,
                    &customer_uuid.to_string(),
                    "Customer",
                    "Update",
                    &serde_json::to_value(customer).unwrap_or_default(),
                )
                .await?;
        }

        tx.commit()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // --- Wants Lists ---
    pub async fn save_wants_list(&self, list: &WantsList) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        sqlx::query(
            "INSERT OR REPLACE INTO Wants_Lists (wants_list_uuid, customer_uuid, created_at) VALUES (?, ?, ?)",
        )
        .bind(list.wants_list_uuid.to_string())
        .bind(list.customer_uuid.to_string())
        .bind(list.created_at.to_rfc3339())
        .execute(&mut *tx)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        // Start fresh with items
        sqlx::query("DELETE FROM Wants_Items WHERE wants_list_uuid = ?")
            .bind(list.wants_list_uuid.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        for item in &list.items {
            sqlx::query(
                "INSERT INTO Wants_Items (item_uuid, wants_list_uuid, product_uuid, min_condition, max_price, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(item.item_uuid.to_string())
            .bind(list.wants_list_uuid.to_string())
            .bind(item.product_uuid.to_string())
            .bind(format!("{:?}", item.min_condition))
            .bind(item.max_price)
            .bind(item.created_at.to_rfc3339())
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        }

        self.sync
            .log_change_with_tx(
                &mut tx,
                &list.wants_list_uuid.to_string(),
                "WantsList",
                "Update",
                &serde_json::to_value(list).unwrap_or_default(),
            )
            .await?;

        tx.commit()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_wants_lists(&self, customer_uuid: Uuid) -> Result<Vec<WantsList>> {
        let rows = sqlx::query("SELECT wants_list_uuid, customer_uuid, created_at FROM Wants_Lists WHERE customer_uuid = ?")
            .bind(customer_uuid.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut lists = Vec::new();
        for row in rows {
            let wants_list_uuid_str: String = row.try_get("wants_list_uuid").unwrap_or_default();
            let wants_list_uuid = Uuid::parse_str(&wants_list_uuid_str).unwrap_or_default();
            let created_at_str: String = row.try_get("created_at").unwrap_or_default();
            let created_at =
                chrono::DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&chrono::Utc);

            let items = self.get_wants_items(wants_list_uuid).await?;

            lists.push(WantsList {
                wants_list_uuid,
                customer_uuid,
                items,
                created_at,
            });
        }

        Ok(lists)
    }

    async fn get_wants_items(&self, wants_list_uuid: Uuid) -> Result<Vec<WantsItem>> {
        let rows = sqlx::query("SELECT item_uuid, product_uuid, min_condition, max_price, created_at FROM Wants_Items WHERE wants_list_uuid = ?")
            .bind(wants_list_uuid.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            let item_uuid_str: String = row.try_get("item_uuid").unwrap_or_default();
            let item_uuid = Uuid::parse_str(&item_uuid_str).unwrap_or_default();
            let product_uuid_str: String = row.try_get("product_uuid").unwrap_or_default();
            let product_uuid = Uuid::parse_str(&product_uuid_str).unwrap_or_default();
            let min_condition_str: String = row.try_get("min_condition").unwrap_or_default();

            // Simple parsing fallbacks as seen in original code
            let min_condition = match min_condition_str.as_str() {
                "NM" => crate::core::Condition::NM,
                _ => crate::core::Condition::NM, // fallback
            };

            let max_price: Option<f64> = row.try_get("max_price").ok();
            let created_at_str: String = row.try_get("created_at").unwrap_or_default();
            let created_at =
                chrono::DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&chrono::Utc);

            items.push(WantsItem {
                item_uuid,
                product_uuid,
                min_condition,
                max_price,
                created_at,
            });
        }
        Ok(items)
    }

    /// HIGH-007 FIX: Get wants items by product_uuid directly using index
    /// Returns list of (Customer, WantsItem) tuples for items matching the product
    pub async fn get_wants_items_by_product(
        &self,
        product_uuid: Uuid,
    ) -> Result<Vec<(Customer, WantsItem)>> {
        // Use JOIN to fetch wants items with their owning customer in one query
        let rows = sqlx::query(
            "SELECT wi.item_uuid, wi.product_uuid, wi.min_condition, wi.max_price, wi.created_at,
                    c.customer_uuid, c.name, c.email, c.phone, c.store_credit, c.tier, c.created_at as customer_created_at
             FROM Wants_Items wi
             JOIN Wants_Lists wl ON wi.wants_list_uuid = wl.wants_list_uuid
             JOIN Customers c ON wl.customer_uuid = c.customer_uuid
             WHERE wi.product_uuid = ?"
        )
        .bind(product_uuid.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut results = Vec::new();

        for row in rows {
            // Parse WantsItem
            let item_uuid_str: String = row.try_get("item_uuid").unwrap_or_default();
            let item_uuid = Uuid::parse_str(&item_uuid_str).unwrap_or_default();
            let product_uuid_str: String = row.try_get("product_uuid").unwrap_or_default();
            let product_uuid = Uuid::parse_str(&product_uuid_str).unwrap_or_default();
            let min_condition_str: String = row.try_get("min_condition").unwrap_or_default();
            let min_condition = match min_condition_str.as_str() {
                "NM" => crate::core::Condition::NM,
                "LP" => crate::core::Condition::LP,
                "MP" => crate::core::Condition::MP,
                "HP" => crate::core::Condition::HP,
                "DMG" => crate::core::Condition::DMG,
                _ => crate::core::Condition::NM,
            };
            let max_price: Option<f64> = row.try_get("max_price").ok();
            let created_at_str: String = row.try_get("created_at").unwrap_or_default();
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .unwrap_or_default()
                .with_timezone(&chrono::Utc);

            let wants_item = WantsItem {
                item_uuid,
                product_uuid,
                min_condition,
                max_price,
                created_at,
            };

            // Parse Customer
            let customer_uuid_str: String = row.try_get("customer_uuid").unwrap_or_default();
            let customer_uuid = Uuid::parse_str(&customer_uuid_str).unwrap_or_default();
            let name: String = row.try_get("name").unwrap_or_default();
            let email: Option<String> = row.try_get("email").ok();
            let phone: Option<String> = row.try_get("phone").ok();
            let store_credit: f64 = row.try_get("store_credit").unwrap_or_default();
            let tier: Option<String> = row.try_get("tier").ok();
            let customer_created_at_str: String =
                row.try_get("customer_created_at").unwrap_or_default();
            let customer_created_at =
                chrono::DateTime::parse_from_rfc3339(&customer_created_at_str)
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc);

            let customer = Customer {
                customer_uuid,
                name,
                email,
                phone,
                store_credit,
                tier,
                created_at: customer_created_at,
            };

            results.push((customer, wants_item));
        }

        Ok(results)
    }
}
