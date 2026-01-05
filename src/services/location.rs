use crate::database::Database;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub location_uuid: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub location_type: LocationType,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LocationType {
    Retail,
    Warehouse,
    RepairCenter,
    Transit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub transfer_uuid: Uuid,
    pub source_location: String,
    pub target_location: String,
    pub status: TransferStatus,
    pub requested_by: Uuid,
    pub approved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub items: Vec<TransferItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferItem {
    pub product_uuid: Uuid,
    pub inventory_uuid: Option<Uuid>, // If specific serialized item
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TransferStatus {
    Pending,
    Approved,
    InTransit,
    Received,
    Cancelled,
}

pub struct LocationService {
    db: Arc<Database>,
}

impl LocationService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// TASK-174: Get all locations
    pub async fn get_locations(&self) -> Result<Vec<Location>> {
        let rows = sqlx::query(
            "SELECT location_uuid, name, address, location_type, is_active FROM Locations",
        )
        .fetch_all(&self.db.pool)
        .await
        .context("Failed to fetch locations")?;

        let mut locations = Vec::new();
        for row in rows {
            use sqlx::Row;
            let type_str: String = row.try_get("location_type").unwrap_or_default();

            locations.push(Location {
                location_uuid: Uuid::parse_str(
                    &row.try_get::<String, _>("location_uuid")
                        .unwrap_or_default(),
                )
                .unwrap_or_default(),
                name: row.try_get("name").unwrap_or_default(),
                address: row.try_get("address").ok(),
                location_type: match type_str.as_str() {
                    "warehouse" => LocationType::Warehouse,
                    "repair_center" => LocationType::RepairCenter,
                    "transit" => LocationType::Transit,
                    _ => LocationType::Retail,
                },
                is_active: row.try_get("is_active").unwrap_or(true),
            });
        }
        Ok(locations)
    }

    /// TASK-174: Create/Update location
    pub async fn upsert_location(&self, location: Location) -> Result<()> {
        let type_str = match location.location_type {
            LocationType::Retail => "retail",
            LocationType::Warehouse => "warehouse",
            LocationType::RepairCenter => "repair_center",
            LocationType::Transit => "transit",
        };

        sqlx::query(
            "INSERT INTO Locations (location_uuid, name, address, location_type, is_active)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(location_uuid) DO UPDATE SET
             name = excluded.name,
             address = excluded.address,
             location_type = excluded.location_type,
             is_active = excluded.is_active",
        )
        .bind(location.location_uuid.to_string())
        .bind(location.name)
        .bind(location.address)
        .bind(type_str)
        .bind(location.is_active)
        .execute(&self.db.pool)
        .await
        .context("Failed to upsert location")?;

        Ok(())
    }

    /// TASK-176: Create transfer request
    pub async fn create_transfer_request(
        &self,
        source: String,
        target: String,
        requester: Uuid,
        items: Vec<TransferItem>,
    ) -> Result<Uuid> {
        let transfer_uuid = Uuid::new_v4();
        let now = Utc::now();

        // Start transaction
        let mut tx = self
            .db
            .pool
            .begin()
            .await
            .context("Failed to begin transaction")?;

        // Insert Transfer
        sqlx::query(
            "INSERT INTO Inventory_Transfers (transfer_uuid, source_location, target_location, status, requested_by, created_at, updated_at)
             VALUES (?, ?, ?, 'pending', ?, ?, ?)"
        )
        .bind(transfer_uuid.to_string())
        .bind(source)
        .bind(target)
        .bind(requester.to_string())
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&mut *tx)
        .await
        .context("Failed to insert transfer")?;

        // Insert Items
        for item in items {
            sqlx::query(
                "INSERT INTO Transfer_Items (transfer_item_uuid, transfer_uuid, product_uuid, inventory_uuid, quantity)
                 VALUES (?, ?, ?, ?, ?)"
            )
            .bind(Uuid::new_v4().to_string())
            .bind(transfer_uuid.to_string())
            .bind(item.product_uuid.to_string())
            .bind(item.inventory_uuid.map(|u| u.to_string()))
            .bind(item.quantity)
            .execute(&mut *tx)
            .await
            .context("Failed to insert transfer items")?;
        }

        tx.commit().await.context("Failed to commit transaction")?;

        Ok(transfer_uuid)
    }

    /// TASK-176/177: Update transfer status (Approve, Start Transit, Complete)
    pub async fn update_transfer_status(
        &self,
        transfer_uuid: Uuid,
        status: TransferStatus,
        user_uuid: Uuid,
    ) -> Result<()> {
        let now = Utc::now();
        let status_str = match status {
            TransferStatus::Pending => "pending",
            TransferStatus::Approved => "approved",
            TransferStatus::InTransit => "in_transit",
            TransferStatus::Received => "received",
            TransferStatus::Cancelled => "cancelled",
        };

        // If approving, set approved_by
        if status == TransferStatus::Approved {
            sqlx::query(
                "UPDATE Inventory_Transfers SET status = ?, approved_by = ?, updated_at = ? WHERE transfer_uuid = ?"
            )
            .bind(status_str)
            .bind(user_uuid.to_string())
            .bind(now.to_rfc3339())
            .bind(transfer_uuid.to_string())
            .execute(&self.db.pool)
            .await
            .context("Failed to approve transfer")?;
        } else if status == TransferStatus::Received {
            // TASK-177: Finalize transfer (Inventory Movement)
            self.finalize_transfer(transfer_uuid).await?;
        } else {
            sqlx::query(
                "UPDATE Inventory_Transfers SET status = ?, updated_at = ? WHERE transfer_uuid = ?",
            )
            .bind(status_str)
            .bind(now.to_rfc3339())
            .bind(transfer_uuid.to_string())
            .execute(&self.db.pool)
            .await
            .context("Failed to update transfer status")?;
        }

        Ok(())
    }

    /// Internal: Execute the inventory movement upon receipt
    async fn finalize_transfer(&self, transfer_uuid: Uuid) -> Result<()> {
        let mut tx = self
            .db
            .pool
            .begin()
            .await
            .context("Failed to begin transaction")?;

        // Get transfer details
        let transfer = sqlx::query(
            "SELECT source_location, target_location FROM Inventory_Transfers WHERE transfer_uuid = ?",
        )
        .bind(transfer_uuid.to_string())
        .fetch_optional(&mut *tx)
        .await
        .context("Database error")?
        .context("Transfer not found")?;

        use sqlx::Row;
        let _source: String = transfer.try_get("source_location")?;
        let target: String = transfer.try_get("target_location")?;

        // Get items
        let items = sqlx::query(
            "SELECT product_uuid, inventory_uuid, quantity FROM Transfer_Items WHERE transfer_uuid = ?",
        )
        .bind(transfer_uuid.to_string())
        .fetch_all(&mut *tx)
        .await
        .context("Failed to fetch items")?;

        for row in items {
            let product_uuid: String = row.try_get("product_uuid")?;
            let inv_uuid_str: Option<String> = row.try_get("inventory_uuid").ok();
            let quantity: i32 = row.try_get("quantity")?;

            // We require inventory_uuid to identify source pile and condition
            let inv_id = inv_uuid_str.ok_or_else(|| {
                anyhow::anyhow!(
                    "Transfer item missing inventory_uuid (required for source identification)"
                )
            })?;

            // Fetch Source Inventory Item to get condition and serialized status
            let source_item = sqlx::query(
                "SELECT condition, serialized_details, quantity_on_hand FROM Local_Inventory WHERE inventory_uuid = ?",
            )
            .bind(&inv_id)
            .fetch_optional(&mut *tx)
            .await
            .context("Failed to fetch source inventory item")?
            .ok_or_else(|| anyhow::anyhow!("Source inventory item not found"))?;

            let condition: String = source_item.try_get("condition")?;
            let serialized_details: Option<serde_json::Value> =
                source_item.try_get("serialized_details").ok();

            if serialized_details.is_some() {
                // Serialized Item Move: Update location tag directly
                // Verify it is currently at source? Not strictly necessary if we trust the workflow, but good practice.
                sqlx::query("UPDATE Local_Inventory SET location_tag = ? WHERE inventory_uuid = ?")
                    .bind(&target)
                    .bind(&inv_id)
                    .execute(&mut *tx)
                    .await
                    .context("Failed to move serialized item")?;
            } else {
                // Bulk Item Move
                // Decrement from SPECIFIC source pile
                sqlx::query(
                    "UPDATE Local_Inventory SET quantity_on_hand = quantity_on_hand - ? WHERE inventory_uuid = ?",
                )
                .bind(quantity)
                .bind(&inv_id)
                .execute(&mut *tx)
                .await
                .context("Failed to decrement source inventory")?;

                // Upsert into Target (match product + condition + location)
                let target_exists = sqlx::query(
                    "SELECT inventory_uuid FROM Local_Inventory WHERE product_uuid = ? AND location_tag = ? AND condition = ?",
                )
                .bind(&product_uuid)
                .bind(&target)
                .bind(&condition)
                .fetch_optional(&mut *tx)
                .await?;

                if let Some(_) = target_exists {
                    sqlx::query(
                        "UPDATE Local_Inventory SET quantity_on_hand = quantity_on_hand + ? 
                         WHERE product_uuid = ? AND location_tag = ? AND condition = ?",
                    )
                    .bind(quantity)
                    .bind(&product_uuid)
                    .bind(&target)
                    .bind(&condition)
                    .execute(&mut *tx)
                    .await?;
                } else {
                    // Create new pile with same condition
                    sqlx::query(
                        "INSERT INTO Local_Inventory (inventory_uuid, product_uuid, location_tag, quantity_on_hand, condition)
                         VALUES (?, ?, ?, ?, ?)",
                    )
                    .bind(Uuid::new_v4().to_string())
                    .bind(&product_uuid)
                    .bind(&target)
                    .bind(quantity)
                    .bind(&condition)
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }

        // Update status to received
        sqlx::query(
            "UPDATE Inventory_Transfers SET status = 'received', updated_at = ? WHERE transfer_uuid = ?",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(transfer_uuid.to_string())
        .execute(&mut *tx)
        .await?;

        tx.commit().await.context("Failed to commit transfer")?;
        Ok(())
    }
}
