use crate::core::{Category, Condition, InventoryItem, InventoryItemWithProduct, VariantType};
use crate::errors::Result;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use super::sync::SyncRepository;

#[derive(Clone)]
pub struct InventoryRepository {
    pool: SqlitePool,
    sync: SyncRepository,
}

impl InventoryRepository {
    pub fn new(pool: SqlitePool, sync: SyncRepository) -> Self {
        Self { pool, sync }
    }

    fn map_row(row: &sqlx::sqlite::SqliteRow) -> Result<InventoryItem> {
        // CRIT-02 FIX: Proper error handling instead of silent corruption with nil UUIDs
        let inventory_uuid_str: String = row.try_get("inventory_uuid").map_err(|e| {
            crate::errors::VaultSyncError::DatabaseError(format!("Missing inventory_uuid: {}", e))
        })?;
        let inventory_uuid = Uuid::parse_str(&inventory_uuid_str).map_err(|e| {
            crate::errors::VaultSyncError::ValidationError(format!(
                "Invalid inventory_uuid '{}': {}",
                inventory_uuid_str, e
            ))
        })?;

        let product_uuid_str: String = row.try_get("product_uuid").map_err(|e| {
            crate::errors::VaultSyncError::DatabaseError(format!("Missing product_uuid: {}", e))
        })?;
        let product_uuid = Uuid::parse_str(&product_uuid_str).map_err(|e| {
            crate::errors::VaultSyncError::ValidationError(format!(
                "Invalid product_uuid '{}': {}",
                product_uuid_str, e
            ))
        })?;

        let variant_type_str: Option<String> = row.try_get("variant_type").ok();

        let variant_type = match variant_type_str.as_deref() {
            Some("Normal") => Some(VariantType::Normal),
            Some("Foil") => Some(VariantType::Foil),
            Some("ReverseHolo") => Some(VariantType::ReverseHolo),
            Some("FirstEdition") => Some(VariantType::FirstEdition),
            Some("Stamped") => Some(VariantType::Stamped),
            Some("Signed") => Some(VariantType::Signed),
            Some("Graded") => Some(VariantType::Graded),
            Some("Refractor") => Some(VariantType::Refractor),
            Some("Patch") => Some(VariantType::Patch),
            Some("Auto") => Some(VariantType::Auto),
            _ => None,
        };

        let condition_str: String = row.try_get("condition").unwrap_or_default();
        let condition = match condition_str.as_str() {
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
        };
        let quantity_on_hand: i64 = row.try_get("quantity_on_hand").unwrap_or_default();
        let quantity_on_hand = quantity_on_hand as i32;
        let location_tag: String = row.try_get("location_tag").unwrap_or_default();
        let specific_price: Option<f64> = row.try_get("specific_price").ok();
        let serialized_details_str: Option<String> = row.try_get("serialized_details").ok();
        let serialized_details = serialized_details_str.and_then(|s| serde_json::from_str(&s).ok());

        // Phase 14: New fields mapping
        let cost_basis: Option<f64> = row.try_get("cost_basis").ok();
        let supplier_uuid_str: Option<String> = row.try_get("supplier_uuid").ok();
        let supplier_uuid = supplier_uuid_str.and_then(|s| Uuid::parse_str(&s).ok());
        let received_date_str: Option<String> = row.try_get("received_date").ok();
        let received_date = received_date_str.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        });
        let min_stock_level: i32 = row.try_get("min_stock_level").unwrap_or(0);
        let max_stock_level: Option<i32> = row.try_get("max_stock_level").ok();
        let reorder_point: Option<i32> = row.try_get("reorder_point").ok();
        let bin_location: Option<String> = row.try_get("bin_location").ok();
        let last_sold_date_str: Option<String> = row.try_get("last_sold_date").ok();
        let last_sold_date = last_sold_date_str.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        });
        let last_counted_date_str: Option<String> = row.try_get("last_counted_date").ok();
        let last_counted_date = last_counted_date_str.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        });
        let deleted_at_str: Option<String> = row.try_get("deleted_at").ok();
        let deleted_at = deleted_at_str.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        });

        Ok(InventoryItem {
            inventory_uuid,
            product_uuid,
            variant_type,
            condition,
            quantity_on_hand,
            location_tag,
            specific_price,
            serialized_details,
            cost_basis,
            supplier_uuid,
            received_date,
            min_stock_level,
            max_stock_level,
            reorder_point,
            bin_location,
            last_sold_date,
            last_counted_date,
            deleted_at,
        })
    }

    pub async fn insert(&self, item: &InventoryItem) -> Result<()> {
        let variant_str = item.variant_type.as_ref().map(|v| format!("{:?}", v));

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        sqlx::query(
            "INSERT OR REPLACE INTO Local_Inventory 
            (inventory_uuid, product_uuid, variant_type, condition, quantity_on_hand, location_tag, specific_price, serialized_details) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(item.inventory_uuid.to_string())
        .bind(item.product_uuid.to_string())
        .bind(variant_str)
        .bind(format!("{:?}", item.condition))
        .bind(item.quantity_on_hand as i64)
        .bind(&item.location_tag)
        .bind(item.specific_price)
        .bind(item.serialized_details.as_ref().map(|v| v.to_string()))
        .execute(&mut *tx)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        self.sync
            .log_change_with_tx(
                &mut tx,
                &item.inventory_uuid.to_string(),
                "InventoryItem",
                "Update",
                &serde_json::to_value(item)
                    .map_err(|e| crate::errors::VaultSyncError::SerializationError(e))?,
            )
            .await?;

        tx.commit()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn insert_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        item: &InventoryItem,
    ) -> Result<()> {
        let variant_str = item.variant_type.as_ref().map(|v| format!("{:?}", v));

        sqlx::query(
            "INSERT OR REPLACE INTO Local_Inventory 
            (inventory_uuid, product_uuid, variant_type, condition, quantity_on_hand, location_tag, specific_price, serialized_details) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(item.inventory_uuid.to_string())
        .bind(item.product_uuid.to_string())
        .bind(variant_str)
        .bind(format!("{:?}", item.condition))
        .bind(item.quantity_on_hand as i64)
        .bind(&item.location_tag)
        .bind(item.specific_price)
        .bind(item.serialized_details.as_ref().map(|v| v.to_string()))
        .execute(&mut **tx)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        self.sync
            .log_change_with_tx(
                tx,
                &item.inventory_uuid.to_string(),
                "InventoryItem",
                "Update",
                &serde_json::to_value(item)
                    .map_err(|e| crate::errors::VaultSyncError::SerializationError(e))?,
            )
            .await?;

        Ok(())
    }

    pub async fn get_by_id(&self, inventory_uuid: Uuid) -> Result<Option<InventoryItem>> {
        let row = sqlx::query("SELECT inventory_uuid, product_uuid, variant_type, condition, quantity_on_hand, location_tag, specific_price, serialized_details, cost_basis, supplier_uuid, received_date, min_stock_level, max_stock_level, reorder_point, bin_location, last_sold_date, last_counted_date, deleted_at FROM Local_Inventory WHERE inventory_uuid = ?")
            .bind(inventory_uuid.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            Ok(Some(Self::map_row(&row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_paginated(&self, limit: i32, offset: i32) -> Result<Vec<InventoryItem>> {
        // MED-010: Exclude soft-deleted items
        let rows = sqlx::query("SELECT inventory_uuid, product_uuid, variant_type, condition, quantity_on_hand, location_tag, specific_price, serialized_details FROM Local_Inventory WHERE deleted_at IS NULL LIMIT ? OFFSET ?")
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(Self::map_row(&row)?);
        }
        Ok(items)
    }

    pub async fn get_by_location(&self, location_tag: &str) -> Result<Vec<InventoryItem>> {
        // MED-010: Exclude soft-deleted items
        let rows = sqlx::query("SELECT inventory_uuid, product_uuid, variant_type, condition, quantity_on_hand, location_tag, specific_price, serialized_details FROM Local_Inventory WHERE location_tag = ? AND deleted_at IS NULL")
            .bind(location_tag)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(Self::map_row(&row)?);
        }
        Ok(items)
    }

    pub async fn get_all(&self) -> Result<Vec<InventoryItem>> {
        // Keeping this but adding warning if used for large datasets?
        // For 'select all' critique, we should try to avoid using this where possible.
        self.get_paginated(10000, 0).await
    }

    pub async fn get_by_product(&self, product_uuid: Uuid) -> Result<Vec<InventoryItem>> {
        // MED-010: Exclude soft-deleted items
        let rows = sqlx::query("SELECT inventory_uuid, product_uuid, variant_type, condition, quantity_on_hand, location_tag, specific_price, serialized_details FROM Local_Inventory WHERE product_uuid = ? AND deleted_at IS NULL")
            .bind(product_uuid.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(Self::map_row(&row)?);
        }
        Ok(items)
    }

    pub async fn update_quantity(&self, inventory_uuid: Uuid, delta: i32) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        sqlx::query("UPDATE Local_Inventory SET quantity_on_hand = quantity_on_hand + ? WHERE inventory_uuid = ?")
            .bind(delta)
            .bind(inventory_uuid.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        // Fetch updated item for logging
        let row = sqlx::query("SELECT inventory_uuid, product_uuid, variant_type, condition, quantity_on_hand, location_tag, specific_price, serialized_details FROM Local_Inventory WHERE inventory_uuid = ?")
            .bind(inventory_uuid.to_string())
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let item = Self::map_row(&row)?;
            self.sync
                .log_change_with_tx(
                    &mut tx,
                    &inventory_uuid.to_string(),
                    "InventoryItem",
                    "Update",
                    &serde_json::to_value(item)
                        .map_err(|e| crate::errors::VaultSyncError::SerializationError(e))?,
                )
                .await?;
        }

        tx.commit()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn delete(&self, inventory_uuid: Uuid) -> Result<()> {
        // Use soft delete by default
        self.soft_delete(inventory_uuid).await
    }

    /// MED-010 FIX: Soft delete - sets deleted_at timestamp instead of hard delete
    pub async fn soft_delete(&self, inventory_uuid: Uuid) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        sqlx::query("UPDATE Local_Inventory SET deleted_at = ? WHERE inventory_uuid = ?")
            .bind(&now)
            .bind(inventory_uuid.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        self.sync
            .log_change_with_tx(
                &mut tx,
                &inventory_uuid.to_string(),
                "InventoryItem",
                "SoftDelete",
                &serde_json::json!({
                    "inventory_uuid": inventory_uuid,
                    "deleted_at": now
                }),
            )
            .await?;

        tx.commit()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Restore a soft-deleted item
    pub async fn restore(&self, inventory_uuid: Uuid) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        sqlx::query("UPDATE Local_Inventory SET deleted_at = NULL WHERE inventory_uuid = ?")
            .bind(inventory_uuid.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        self.sync
            .log_change_with_tx(
                &mut tx,
                &inventory_uuid.to_string(),
                "InventoryItem",
                "Restore",
                &serde_json::json!({
                    "inventory_uuid": inventory_uuid
                }),
            )
            .await?;

        tx.commit()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Hard delete - use sparingly, only for cleanup operations
    pub async fn hard_delete(&self, inventory_uuid: Uuid) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        sqlx::query("DELETE FROM Local_Inventory WHERE inventory_uuid = ?")
            .bind(inventory_uuid.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        self.sync
            .log_change_with_tx(
                &mut tx,
                &inventory_uuid.to_string(),
                "InventoryItem",
                "Delete",
                &serde_json::json!({ "inventory_uuid": inventory_uuid }),
            )
            .await?;

        tx.commit()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_low_stock(&self, threshold: i32) -> Result<Vec<InventoryItem>> {
        let rows = sqlx::query("SELECT inventory_uuid, product_uuid, variant_type, condition, quantity_on_hand, location_tag, specific_price, serialized_details FROM Local_Inventory WHERE quantity_on_hand <= ?")
            .bind(threshold as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(Self::map_row(&row)?);
        }
        Ok(items)
    }

    /// HIGH-005 FIX: Search inventory by product name using proper JOIN
    pub async fn search_by_name(&self, query: &str, limit: i32) -> Result<Vec<InventoryItem>> {
        let search_pattern = format!("%{}%", query);

        let rows = sqlx::query(
            "SELECT i.inventory_uuid, i.product_uuid, i.variant_type, i.condition, 
                    i.quantity_on_hand, i.location_tag, i.specific_price, i.serialized_details 
             FROM Local_Inventory i
             JOIN Global_Catalog p ON i.product_uuid = p.product_uuid
             WHERE p.name LIKE ? OR p.set_code LIKE ? OR p.barcode LIKE ?
             LIMIT ?",
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(Self::map_row(&row)?);
        }
        Ok(items)
    }

    /// Get total inventory count (for dashboard)
    pub async fn get_total_count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as cnt FROM Local_Inventory")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let count: i64 = row.try_get("cnt").unwrap_or(0);
        Ok(count)
    }

    /// Get total quantity across all inventory (for dashboard)
    pub async fn get_total_quantity(&self) -> Result<i64> {
        let row =
            sqlx::query("SELECT COALESCE(SUM(quantity_on_hand), 0) as total FROM Local_Inventory")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let total: i64 = row.try_get("total").unwrap_or(0);
        Ok(total)
    }

    /// Get low stock count (for dashboard)
    pub async fn get_low_stock_count(&self, threshold: i32) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as cnt FROM Local_Inventory WHERE quantity_on_hand <= ? AND quantity_on_hand > 0")
            .bind(threshold as i64)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let count: i64 = row.try_get("cnt").unwrap_or(0);
        Ok(count)
    }

    /// MED-002 FIX: Clean up zero-quantity bulk inventory items
    /// Serialized items (with serialized_details) are kept for audit trail
    /// Returns the number of items deleted
    pub async fn cleanup_zero_quantity(&self) -> Result<i64> {
        // First, get the items we're about to delete for sync logging
        let rows = sqlx::query(
            "SELECT inventory_uuid FROM Local_Inventory 
             WHERE quantity_on_hand <= 0 AND serialized_details IS NULL",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if rows.is_empty() {
            return Ok(0);
        }

        let count = rows.len() as i64;

        // Start transaction for atomic delete + sync logging
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        // Delete zero-quantity bulk items (preserve serialized items for audit)
        sqlx::query(
            "DELETE FROM Local_Inventory 
             WHERE quantity_on_hand <= 0 AND serialized_details IS NULL",
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        // Log deletions for sync
        for row in &rows {
            let uuid_str: String = row.try_get("inventory_uuid").unwrap_or_default();
            self.sync
                .log_change_with_tx(
                    &mut tx,
                    &uuid_str,
                    "InventoryItem",
                    "Delete",
                    &serde_json::json!({
                        "inventory_uuid": uuid_str,
                        "reason": "zero_quantity_cleanup"
                    }),
                )
                .await?;
        }

        tx.commit()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        tracing::info!("Cleaned up {} zero-quantity inventory items", count);
        Ok(count)
    }

    /// Get count of zero-quantity items that would be cleaned up
    pub async fn get_zero_quantity_count(&self) -> Result<i64> {
        let row = sqlx::query(
            "SELECT COUNT(*) as cnt FROM Local_Inventory 
             WHERE quantity_on_hand <= 0 AND serialized_details IS NULL",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let count: i64 = row.try_get("cnt").unwrap_or(0);
        Ok(count)
    }

    pub async fn get_inventory_aging(&self) -> Result<Vec<(i32, f64)>> {
        let rows = sqlx::query(
            "SELECT 
                i.received_date,
                i.quantity_on_hand,
                COALESCE(i.specific_price, pm.market_mid, 0.0) as unit_price
             FROM Local_Inventory i
             LEFT JOIN Pricing_Matrix pm ON i.product_uuid = pm.product_uuid
             WHERE i.quantity_on_hand > 0 AND i.deleted_at IS NULL",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let now = chrono::Utc::now();
        let mut result = Vec::new();

        for row in rows {
            let received_date_str: Option<String> = row.try_get("received_date").ok();
            let qty: i32 = row.try_get("quantity_on_hand").unwrap_or(0);
            let price: f64 = row.try_get("unit_price").unwrap_or(0.0);
            let value = qty as f64 * price;

            let days_old = if let Some(s) = received_date_str {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&s) {
                    let dt_utc = dt.with_timezone(&chrono::Utc);
                    (now - dt_utc).num_days() as i32
                } else {
                    -1 // Unknown
                }
            } else {
                -1 // Unknown
            };

            result.push((days_old, value));
        }

        Ok(result)
    }

    pub async fn get_paginated_with_products(
        &self,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<InventoryItemWithProduct>> {
        let rows = sqlx::query(
            "SELECT 
                i.*,
                p.name as product_name,
                p.category,
                p.metadata as product_metadata
             FROM Local_Inventory i
             JOIN Global_Catalog p ON i.product_uuid = p.product_uuid
             WHERE i.deleted_at IS NULL 
             LIMIT ? OFFSET ?",
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let item = Self::map_row(&row)?;

            let product_name: String = row.try_get("product_name").unwrap_or_default();
            let category_str: String = row.try_get("category").unwrap_or_default();
            let category = match category_str.as_str() {
                "TCG" => Category::TCG,
                "SportsCard" => Category::SportsCard,
                "Comic" => Category::Comic,
                "Bobblehead" => Category::Bobblehead,
                "Apparel" => Category::Apparel,
                "Figure" => Category::Figure,
                "Accessory" => Category::Accessory,
                "Other" => Category::Other,
                _ => Category::Other,
            };

            let metadata_str: Option<String> = row.try_get("product_metadata").ok();
            let product_metadata = metadata_str
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or(serde_json::json!({}));

            results.push(InventoryItemWithProduct {
                item,
                product_name,
                category,
                product_metadata,
            });
        }
        Ok(results)
    }
}
