//! Holds (Layaway) service
//!
//! Manages customer holds/layaways including:
//! - Creating holds with deposits
//! - Processing payments toward holds
//! - Hold expiration tracking
//! - Converting holds to completed sales

use crate::database::Database;
use crate::errors::Result;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Hold status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HoldStatus {
    Active,
    Completed,
    Cancelled,
    Expired,
}

impl std::fmt::Display for HoldStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HoldStatus::Active => write!(f, "Active"),
            HoldStatus::Completed => write!(f, "Completed"),
            HoldStatus::Cancelled => write!(f, "Cancelled"),
            HoldStatus::Expired => write!(f, "Expired"),
        }
    }
}

/// A hold/layaway record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hold {
    pub hold_uuid: Uuid,
    pub customer_uuid: Uuid,
    pub status: HoldStatus,
    pub total_amount: f64,
    pub deposit_amount: f64,
    pub balance_due: f64,
    pub expiration_date: chrono::DateTime<Utc>,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// An item in a hold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldItem {
    pub item_uuid: Uuid,
    pub hold_uuid: Uuid,
    pub inventory_uuid: Uuid,
    pub quantity: i32,
    pub unit_price: f64,
}

/// A payment toward a hold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldPayment {
    pub payment_uuid: Uuid,
    pub hold_uuid: Uuid,
    pub amount: f64,
    pub payment_method: String,
    pub created_at: chrono::DateTime<Utc>,
}

/// Request to create a new hold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateHoldRequest {
    pub customer_uuid: Uuid,
    pub items: Vec<HoldItemRequest>,
    pub deposit_amount: f64,
    pub deposit_method: String,
    pub notes: Option<String>,
    pub hold_days: Option<i64>,
}

/// Request for a single hold item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldItemRequest {
    pub inventory_uuid: Uuid,
    pub quantity: i32,
    pub unit_price: f64,
}

/// Hold summary with items and payments
#[derive(Debug, Clone, Serialize)]
pub struct HoldSummary {
    pub hold: Hold,
    pub items: Vec<HoldItem>,
    pub payments: Vec<HoldPayment>,
    pub total_paid: f64,
}

/// Holds service
pub struct HoldsService {
    db: Arc<Database>,
    default_hold_days: i64,
    minimum_deposit_percent: f64,
}

impl HoldsService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            default_hold_days: 14,         // 2 weeks default
            minimum_deposit_percent: 0.20, // 20% minimum deposit
        }
    }

    /// Create a new hold/layaway
    pub async fn create_hold(&self, request: CreateHoldRequest) -> Result<HoldSummary> {
        let now = Utc::now();
        let hold_uuid = Uuid::new_v4();

        // Calculate totals
        let total_amount: f64 = request
            .items
            .iter()
            .map(|i| i.unit_price * i.quantity as f64)
            .sum();

        // Validate minimum deposit
        let min_deposit = total_amount * self.minimum_deposit_percent;
        if request.deposit_amount < min_deposit {
            return Err(anyhow::anyhow!(
                "Minimum deposit is ${:.2} ({}% of ${:.2})",
                min_deposit,
                self.minimum_deposit_percent * 100.0,
                total_amount
            ));
        }

        let balance_due = total_amount - request.deposit_amount;
        let hold_days = request.hold_days.unwrap_or(self.default_hold_days);
        let expiration_date = now + Duration::days(hold_days);

        // Validate inventory availability and reserve items
        for item in &request.items {
            let row = sqlx::query(
                "SELECT quantity_on_hand FROM Local_Inventory 
                 WHERE inventory_uuid = ? AND deleted_at IS NULL",
            )
            .bind(item.inventory_uuid.to_string())
            .fetch_optional(&self.db.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

            match row {
                Some(r) => {
                    let on_hand: i32 = sqlx::Row::try_get(&r, "quantity_on_hand").unwrap_or(0);
                    if on_hand < item.quantity {
                        return Err(anyhow::anyhow!(
                            "Insufficient stock for item {}: {} available, {} requested",
                            item.inventory_uuid,
                            on_hand,
                            item.quantity
                        ));
                    }
                }
                None => {
                    return Err(anyhow::anyhow!(
                        "Inventory item {} not found",
                        item.inventory_uuid
                    ));
                }
            }
        }

        // Create hold record
        sqlx::query(
            "INSERT INTO Holds 
             (hold_uuid, customer_uuid, status, total_amount, deposit_amount, balance_due, expiration_date, notes, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(hold_uuid.to_string())
        .bind(request.customer_uuid.to_string())
        .bind(HoldStatus::Active.to_string())
        .bind(total_amount)
        .bind(request.deposit_amount)
        .bind(balance_due)
        .bind(expiration_date.to_rfc3339())
        .bind(&request.notes)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create hold: {}", e))?;

        // Create hold items and reserve inventory
        let mut hold_items = Vec::new();
        for item in &request.items {
            let item_uuid = Uuid::new_v4();

            sqlx::query(
                "INSERT INTO Hold_Items (item_uuid, hold_uuid, inventory_uuid, quantity, unit_price)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(item_uuid.to_string())
            .bind(hold_uuid.to_string())
            .bind(item.inventory_uuid.to_string())
            .bind(item.quantity)
            .bind(item.unit_price)
            .execute(&self.db.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create hold item: {}", e))?;

            // Reserve inventory (reduce available quantity)
            sqlx::query(
                "UPDATE Local_Inventory 
                 SET quantity_on_hand = quantity_on_hand - ?
                 WHERE inventory_uuid = ?",
            )
            .bind(item.quantity)
            .bind(item.inventory_uuid.to_string())
            .execute(&self.db.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to reserve inventory: {}", e))?;

            hold_items.push(HoldItem {
                item_uuid,
                hold_uuid,
                inventory_uuid: item.inventory_uuid,
                quantity: item.quantity,
                unit_price: item.unit_price,
            });
        }

        // Record deposit payment
        let payment_uuid = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO Hold_Payments (payment_uuid, hold_uuid, amount, payment_method, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(payment_uuid.to_string())
        .bind(hold_uuid.to_string())
        .bind(request.deposit_amount)
        .bind(&request.deposit_method)
        .bind(now.to_rfc3339())
        .execute(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to record deposit: {}", e))?;

        let payments = vec![HoldPayment {
            payment_uuid,
            hold_uuid,
            amount: request.deposit_amount,
            payment_method: request.deposit_method,
            created_at: now,
        }];

        let hold = Hold {
            hold_uuid,
            customer_uuid: request.customer_uuid,
            status: HoldStatus::Active,
            total_amount,
            deposit_amount: request.deposit_amount,
            balance_due,
            expiration_date,
            notes: request.notes,
            created_at: now,
            updated_at: now,
        };

        tracing::info!(
            "Hold {} created for customer {}: ${:.2} total, ${:.2} deposit",
            hold_uuid,
            request.customer_uuid,
            total_amount,
            request.deposit_amount
        );

        Ok(HoldSummary {
            hold,
            items: hold_items,
            payments,
            total_paid: request.deposit_amount,
        })
    }

    /// Make a payment toward a hold
    pub async fn make_payment(
        &self,
        hold_uuid: Uuid,
        amount: f64,
        payment_method: &str,
    ) -> Result<HoldSummary> {
        let now = Utc::now();

        // Get current hold
        let summary = self.get_hold(hold_uuid).await?;
        let summary = summary.ok_or_else(|| anyhow::anyhow!("Hold not found"))?;

        if summary.hold.status != HoldStatus::Active {
            return Err(anyhow::anyhow!(
                "Cannot make payment on {} hold",
                summary.hold.status
            ));
        }

        // Calculate new balance
        let new_balance = summary.hold.balance_due - amount;
        let payment_uuid = Uuid::new_v4();

        // Record payment
        sqlx::query(
            "INSERT INTO Hold_Payments (payment_uuid, hold_uuid, amount, payment_method, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(payment_uuid.to_string())
        .bind(hold_uuid.to_string())
        .bind(amount)
        .bind(payment_method)
        .bind(now.to_rfc3339())
        .execute(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to record payment: {}", e))?;

        // Update hold balance
        sqlx::query("UPDATE Holds SET balance_due = ?, updated_at = ? WHERE hold_uuid = ?")
            .bind(new_balance.max(0.0))
            .bind(now.to_rfc3339())
            .bind(hold_uuid.to_string())
            .execute(&self.db.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update hold: {}", e))?;

        // If fully paid, complete the hold
        if new_balance <= 0.0 {
            self.complete_hold(hold_uuid).await?;
        }

        self.get_hold(hold_uuid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Hold not found"))
    }

    /// Complete a hold (convert to sale)
    pub async fn complete_hold(&self, hold_uuid: Uuid) -> Result<()> {
        let now = Utc::now();

        sqlx::query("UPDATE Holds SET status = ?, updated_at = ? WHERE hold_uuid = ?")
            .bind(HoldStatus::Completed.to_string())
            .bind(now.to_rfc3339())
            .bind(hold_uuid.to_string())
            .execute(&self.db.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to complete hold: {}", e))?;

        tracing::info!("Hold {} completed", hold_uuid);
        Ok(())
    }

    /// Cancel a hold and restore inventory
    pub async fn cancel_hold(&self, hold_uuid: Uuid, reason: &str) -> Result<()> {
        let now = Utc::now();

        // Get hold items
        let items =
            sqlx::query("SELECT inventory_uuid, quantity FROM Hold_Items WHERE hold_uuid = ?")
                .bind(hold_uuid.to_string())
                .fetch_all(&self.db.pool)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get hold items: {}", e))?;

        // Restore inventory
        for item in items {
            let inventory_uuid: String =
                sqlx::Row::try_get(&item, "inventory_uuid").unwrap_or_default();
            let quantity: i32 = sqlx::Row::try_get(&item, "quantity").unwrap_or(0);

            sqlx::query(
                "UPDATE Local_Inventory 
                 SET quantity_on_hand = quantity_on_hand + ?
                 WHERE inventory_uuid = ?",
            )
            .bind(quantity)
            .bind(&inventory_uuid)
            .execute(&self.db.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to restore inventory: {}", e))?;
        }

        // Update hold status
        sqlx::query(
            "UPDATE Holds SET status = ?, notes = COALESCE(notes, '') || ?, updated_at = ? WHERE hold_uuid = ?",
        )
        .bind(HoldStatus::Cancelled.to_string())
        .bind(format!(" [Cancelled: {}]", reason))
        .bind(now.to_rfc3339())
        .bind(hold_uuid.to_string())
        .execute(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to cancel hold: {}", e))?;

        tracing::info!("Hold {} cancelled: {}", hold_uuid, reason);
        Ok(())
    }

    /// Get a hold with items and payments
    pub async fn get_hold(&self, hold_uuid: Uuid) -> Result<Option<HoldSummary>> {
        // Get hold record
        let row = sqlx::query("SELECT * FROM Holds WHERE hold_uuid = ?")
            .bind(hold_uuid.to_string())
            .fetch_optional(&self.db.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };

        let hold = Hold {
            hold_uuid,
            customer_uuid: Uuid::parse_str(
                &sqlx::Row::try_get::<String, _>(&row, "customer_uuid").unwrap_or_default(),
            )
            .unwrap_or_default(),
            status: match sqlx::Row::try_get::<String, _>(&row, "status")
                .unwrap_or_default()
                .as_str()
            {
                "Active" => HoldStatus::Active,
                "Completed" => HoldStatus::Completed,
                "Cancelled" => HoldStatus::Cancelled,
                "Expired" => HoldStatus::Expired,
                _ => HoldStatus::Active,
            },
            total_amount: sqlx::Row::try_get(&row, "total_amount").unwrap_or(0.0),
            deposit_amount: sqlx::Row::try_get(&row, "deposit_amount").unwrap_or(0.0),
            balance_due: sqlx::Row::try_get(&row, "balance_due").unwrap_or(0.0),
            expiration_date: sqlx::Row::try_get::<String, _>(&row, "expiration_date")
                .ok()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
            notes: sqlx::Row::try_get(&row, "notes").ok(),
            created_at: sqlx::Row::try_get::<String, _>(&row, "created_at")
                .ok()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
            updated_at: sqlx::Row::try_get::<String, _>(&row, "updated_at")
                .ok()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
        };

        // Get items
        let item_rows = sqlx::query("SELECT * FROM Hold_Items WHERE hold_uuid = ?")
            .bind(hold_uuid.to_string())
            .fetch_all(&self.db.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        let items: Vec<HoldItem> = item_rows
            .iter()
            .map(|r| HoldItem {
                item_uuid: Uuid::parse_str(
                    &sqlx::Row::try_get::<String, _>(r, "item_uuid").unwrap_or_default(),
                )
                .unwrap_or_default(),
                hold_uuid,
                inventory_uuid: Uuid::parse_str(
                    &sqlx::Row::try_get::<String, _>(r, "inventory_uuid").unwrap_or_default(),
                )
                .unwrap_or_default(),
                quantity: sqlx::Row::try_get(r, "quantity").unwrap_or(0),
                unit_price: sqlx::Row::try_get(r, "unit_price").unwrap_or(0.0),
            })
            .collect();

        // Get payments
        let payment_rows = sqlx::query("SELECT * FROM Hold_Payments WHERE hold_uuid = ?")
            .bind(hold_uuid.to_string())
            .fetch_all(&self.db.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        let payments: Vec<HoldPayment> = payment_rows
            .iter()
            .map(|r| HoldPayment {
                payment_uuid: Uuid::parse_str(
                    &sqlx::Row::try_get::<String, _>(r, "payment_uuid").unwrap_or_default(),
                )
                .unwrap_or_default(),
                hold_uuid,
                amount: sqlx::Row::try_get(r, "amount").unwrap_or(0.0),
                payment_method: sqlx::Row::try_get(r, "payment_method").unwrap_or_default(),
                created_at: sqlx::Row::try_get::<String, _>(r, "created_at")
                    .ok()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
            })
            .collect();

        let total_paid: f64 = payments.iter().map(|p| p.amount).sum();

        Ok(Some(HoldSummary {
            hold,
            items,
            payments,
            total_paid,
        }))
    }

    /// Get all active holds for a customer
    pub async fn get_customer_holds(&self, customer_uuid: Uuid) -> Result<Vec<HoldSummary>> {
        let rows =
            sqlx::query("SELECT hold_uuid FROM Holds WHERE customer_uuid = ? AND status = ?")
                .bind(customer_uuid.to_string())
                .bind(HoldStatus::Active.to_string())
                .fetch_all(&self.db.pool)
                .await
                .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        let mut holds = Vec::new();
        for row in rows {
            let hold_uuid_str: String = sqlx::Row::try_get(&row, "hold_uuid").unwrap_or_default();
            if let Ok(hold_uuid) = Uuid::parse_str(&hold_uuid_str) {
                if let Some(summary) = self.get_hold(hold_uuid).await? {
                    holds.push(summary);
                }
            }
        }

        Ok(holds)
    }

    /// Check and expire overdue holds
    pub async fn expire_overdue_holds(&self) -> Result<Vec<Uuid>> {
        let now = Utc::now();

        let rows =
            sqlx::query("SELECT hold_uuid FROM Holds WHERE status = ? AND expiration_date < ?")
                .bind(HoldStatus::Active.to_string())
                .bind(now.to_rfc3339())
                .fetch_all(&self.db.pool)
                .await
                .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        let mut expired = Vec::new();
        for row in rows {
            let hold_uuid_str: String = sqlx::Row::try_get(&row, "hold_uuid").unwrap_or_default();
            if let Ok(hold_uuid) = Uuid::parse_str(&hold_uuid_str) {
                self.cancel_hold(hold_uuid, "Expired").await?;
                expired.push(hold_uuid);
            }
        }

        if !expired.is_empty() {
            tracing::info!("Expired {} overdue holds", expired.len());
        }

        Ok(expired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hold_status_display() {
        assert_eq!(HoldStatus::Active.to_string(), "Active");
        assert_eq!(HoldStatus::Cancelled.to_string(), "Cancelled");
    }
}
