use crate::database::Database;
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Returns processing service with fraud protection
pub struct ReturnsService {
    db: Arc<Database>,
    /// Default return window in days
    return_window_days: i32,
    /// Default restocking fee percentage
    restocking_fee_percent: f64,
    /// Threshold for manager approval
    manager_approval_threshold: f64,
    /// Maximum returns per customer per month
    monthly_return_limit: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnRequest {
    pub transaction_uuid: Uuid,
    pub items: Vec<ReturnItemRequest>,
    pub reason_code: ReturnReasonCode,
    pub reason_notes: Option<String>,
    pub customer_uuid: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnItemRequest {
    pub inventory_uuid: Uuid,
    pub quantity: i32,
    pub condition: ReturnCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReturnReasonCode {
    ChangedMind,
    Defective,
    WrongItem,
    NotAsDescribed,
    Damaged,
    Other,
}

impl std::fmt::Display for ReturnReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReturnReasonCode::ChangedMind => write!(f, "changed_mind"),
            ReturnReasonCode::Defective => write!(f, "defective"),
            ReturnReasonCode::WrongItem => write!(f, "wrong_item"),
            ReturnReasonCode::NotAsDescribed => write!(f, "not_as_described"),
            ReturnReasonCode::Damaged => write!(f, "damaged"),
            ReturnReasonCode::Other => write!(f, "other"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReturnCondition {
    Original,  // Unopened, as sold
    Good,      // Opened but like new
    Fair,      // Minor wear
    Poor,      // Significant wear/damage
    Defective, // Non-functional
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnResult {
    pub return_uuid: Uuid,
    pub transaction_uuid: Uuid,
    pub items: Vec<ReturnedItem>,
    pub subtotal: f64,
    pub restocking_fee: f64,
    pub refund_amount: f64,
    pub refund_method: String,
    pub requires_approval: bool,
    pub approval_reason: Option<String>,
    pub processed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnedItem {
    pub inventory_uuid: Uuid,
    pub product_name: String,
    pub quantity: i32,
    pub original_price: f64,
    pub refund_amount: f64,
    pub restocking_fee: f64,
    pub returned_to_inventory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnPolicy {
    pub return_window_days: i32,
    pub restocking_fee_percent: f64,
    pub damaged_restocking_fee_percent: f64,
    pub manager_approval_threshold: f64,
    pub allow_no_receipt: bool,
    pub max_no_receipt_value: f64,
}

impl Default for ReturnPolicy {
    fn default() -> Self {
        Self {
            return_window_days: 30,
            restocking_fee_percent: 0.0, // No restocking fee by default
            damaged_restocking_fee_percent: 15.0, // 15% for damaged
            manager_approval_threshold: 100.0,
            allow_no_receipt: true,
            max_no_receipt_value: 25.0,
        }
    }
}

impl ReturnsService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            return_window_days: 30,
            restocking_fee_percent: 0.0,
            manager_approval_threshold: 100.0,
            monthly_return_limit: 5,
        }
    }

    /// Configure with policy
    pub fn with_policy(mut self, policy: &ReturnPolicy) -> Self {
        self.return_window_days = policy.return_window_days;
        self.restocking_fee_percent = policy.restocking_fee_percent;
        self.manager_approval_threshold = policy.manager_approval_threshold;
        self
    }

    /// TASK-169: Process a return (potentially partial)
    pub async fn process_return(&self, request: ReturnRequest) -> Result<ReturnResult> {
        // Validate the original transaction
        let transaction = self.validate_transaction(request.transaction_uuid).await?;

        let mut returned_items = Vec::new();
        let mut subtotal = 0.0;
        let mut total_restocking = 0.0;

        for item_req in &request.items {
            // Get original item details
            let original_item = self
                .get_original_sale_item(request.transaction_uuid, item_req.inventory_uuid)
                .await?;

            // Validate return quantity against original sale
            if item_req.quantity > original_item.quantity {
                bail!(
                    "Return quantity {} exceeds original purchased quantity {}",
                    item_req.quantity,
                    original_item.quantity
                );
            }

            // TASK-168: Calculate restocking fee based on condition
            let restocking_percent = match item_req.condition {
                ReturnCondition::Original => 0.0,
                ReturnCondition::Good => self.restocking_fee_percent,
                ReturnCondition::Fair => self.restocking_fee_percent * 1.5,
                ReturnCondition::Poor => 15.0, // Fixed 15% for poor condition
                ReturnCondition::Defective => 0.0, // No fee for defective
            };

            let item_value = original_item.price * item_req.quantity as f64;
            let restocking_fee = item_value * (restocking_percent / 100.0);
            let refund_amount = item_value - restocking_fee;

            subtotal += item_value;
            total_restocking += restocking_fee;

            // TASK-171: Determine if item returns to inventory
            let returned_to_inventory = match item_req.condition {
                ReturnCondition::Original | ReturnCondition::Good => true,
                ReturnCondition::Fair => true,
                ReturnCondition::Poor | ReturnCondition::Defective => false,
            };

            // Return item to inventory if applicable
            if returned_to_inventory {
                self.restore_inventory(item_req.inventory_uuid, item_req.quantity)
                    .await?;
            }

            returned_items.push(ReturnedItem {
                inventory_uuid: item_req.inventory_uuid,
                product_name: original_item.product_name,
                quantity: item_req.quantity,
                original_price: original_item.price,
                refund_amount,
                restocking_fee,
                returned_to_inventory,
            });
        }

        let refund_amount = subtotal - total_restocking;

        // TASK-173: Check if requires approval
        let (requires_approval, approval_reason) = self
            .check_approval_required(request.customer_uuid, refund_amount, &request.reason_code)
            .await?;

        let return_uuid = Uuid::new_v4();

        // Record the return
        self.record_return(
            return_uuid,
            &request,
            &returned_items,
            subtotal,
            total_restocking,
            refund_amount,
        )
        .await?;

        Ok(ReturnResult {
            return_uuid,
            transaction_uuid: request.transaction_uuid,
            items: returned_items,
            subtotal,
            restocking_fee: total_restocking,
            refund_amount,
            refund_method: transaction.payment_method,
            requires_approval,
            approval_reason,
            processed_at: Utc::now(),
        })
    }

    /// Validate that return is allowed for this transaction
    async fn validate_transaction(&self, transaction_uuid: Uuid) -> Result<TransactionInfo> {
        let row = sqlx::query(
            "SELECT t.transaction_uuid, t.created_at, t.transaction_type, 
                    COALESCE(pm.payment_type, 'Unknown') as payment_method
             FROM Transactions t
             LEFT JOIN Payment_Methods pm ON t.transaction_uuid = pm.transaction_uuid
             WHERE t.transaction_uuid = ?",
        )
        .bind(transaction_uuid.to_string())
        .fetch_optional(&self.db.pool)
        .await
        .context("Database error")?;

        let row = row.context("Transaction not found")?;

        use sqlx::Row;
        let created_at_str: String = row.try_get("created_at").unwrap_or_default();
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(Utc::now());

        // Check return window
        let days_since = (Utc::now() - created_at).num_days();
        if days_since > self.return_window_days as i64 {
            bail!("Return window expired ({} days)", self.return_window_days);
        }

        let transaction_type: String = row.try_get("transaction_type").unwrap_or_default();
        if transaction_type != "Sale" {
            bail!("Only sales can be returned");
        }

        Ok(TransactionInfo {
            payment_method: row.try_get("payment_method").unwrap_or_default(),
        })
    }

    /// Get original sale item details
    async fn get_original_sale_item(
        &self,
        transaction_uuid: Uuid,
        inventory_uuid: Uuid,
    ) -> Result<OriginalItem> {
        let row = sqlx::query(
            "SELECT ti.quantity, ti.unit_price, p.name
             FROM Transaction_Items ti
             JOIN Products p ON ti.product_uuid = p.product_uuid
             WHERE ti.transaction_uuid = ? AND ti.inventory_uuid = ?",
        )
        .bind(transaction_uuid.to_string())
        .bind(inventory_uuid.to_string())
        .fetch_optional(&self.db.pool)
        .await
        .context("Database error")?;

        let row = row.context("Item not found in transaction")?;

        use sqlx::Row;
        Ok(OriginalItem {
            quantity: row.try_get("quantity").unwrap_or(1),
            price: row.try_get("unit_price").unwrap_or(0.0),
            product_name: row.try_get("name").unwrap_or_default(),
        })
    }

    /// Restore item to inventory
    async fn restore_inventory(&self, inventory_uuid: Uuid, quantity: i32) -> Result<()> {
        sqlx::query(
            "UPDATE Inventory SET quantity_on_hand = quantity_on_hand + ? WHERE inventory_uuid = ?",
        )
        .bind(quantity)
        .bind(inventory_uuid.to_string())
        .execute(&self.db.pool)
        .await
        .context("Database error")?;

        tracing::info!(
            "Restored {} units to inventory {}",
            quantity,
            inventory_uuid
        );
        Ok(())
    }

    /// TASK-172/173: Check if return requires manager approval
    async fn check_approval_required(
        &self,
        customer_uuid: Option<Uuid>,
        refund_amount: f64,
        reason: &ReturnReasonCode,
    ) -> Result<(bool, Option<String>)> {
        // High value requires approval
        if refund_amount > self.manager_approval_threshold {
            return Ok((
                true,
                Some(format!(
                    "Refund amount ${:.2} exceeds ${:.2} threshold",
                    refund_amount, self.manager_approval_threshold
                )),
            ));
        }

        // TASK-172: Check customer return frequency
        if let Some(uuid) = customer_uuid {
            let month_ago = Utc::now() - Duration::days(30);
            let return_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM Returns WHERE customer_uuid = ? AND processed_at > ?",
            )
            .bind(uuid.to_string())
            .bind(month_ago.to_rfc3339())
            .fetch_one(&self.db.pool)
            .await
            .unwrap_or(0);

            if return_count >= self.monthly_return_limit as i64 {
                return Ok((
                    true,
                    Some(format!(
                        "Customer has {} returns this month (limit: {})",
                        return_count, self.monthly_return_limit
                    )),
                ));
            }
        }

        // No receipt requires approval for certain reasons
        if *reason == ReturnReasonCode::ChangedMind && customer_uuid.is_none() {
            return Ok((
                true,
                Some("No receipt return requires approval".to_string()),
            ));
        }

        Ok((false, None))
    }

    /// Record the return in database
    async fn record_return(
        &self,
        return_uuid: Uuid,
        request: &ReturnRequest,
        items: &[ReturnedItem],
        subtotal: f64,
        restocking_fee: f64,
        refund_amount: f64,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO Returns (return_uuid, transaction_uuid, customer_uuid, reason_code, reason_notes, subtotal, restocking_fee, refund_amount, processed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(return_uuid.to_string())
        .bind(request.transaction_uuid.to_string())
        .bind(request.customer_uuid.map(|u| u.to_string()))
        .bind(request.reason_code.to_string())
        .bind(&request.reason_notes)
        .bind(subtotal)
        .bind(restocking_fee)
        .bind(refund_amount)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.db.pool)
        .await
        .context("Database error")?;

        // Record return items
        for item in items {
            sqlx::query(
                "INSERT INTO Return_Items (return_uuid, inventory_uuid, quantity, original_price, refund_amount, restocking_fee, returned_to_inventory)
                 VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(return_uuid.to_string())
            .bind(item.inventory_uuid.to_string())
            .bind(item.quantity)
            .bind(item.original_price)
            .bind(item.refund_amount)
            .bind(item.restocking_fee)
            .bind(item.returned_to_inventory)
            .execute(&self.db.pool)
            .await
            .context("Database error")?;
        }

        Ok(())
    }

    /// TASK-170: Get all return reason codes
    pub fn get_reason_codes() -> Vec<(String, String)> {
        vec![
            ("changed_mind".to_string(), "Changed Mind".to_string()),
            ("defective".to_string(), "Defective/Not Working".to_string()),
            ("wrong_item".to_string(), "Wrong Item Purchased".to_string()),
            (
                "not_as_described".to_string(),
                "Not As Described".to_string(),
            ),
            ("damaged".to_string(), "Damaged".to_string()),
            ("other".to_string(), "Other".to_string()),
        ]
    }

    /// Get return policy
    pub fn get_policy(&self) -> ReturnPolicy {
        ReturnPolicy {
            return_window_days: self.return_window_days,
            restocking_fee_percent: self.restocking_fee_percent,
            damaged_restocking_fee_percent: 15.0,
            manager_approval_threshold: self.manager_approval_threshold,
            allow_no_receipt: true,
            max_no_receipt_value: 25.0,
        }
    }
}

// Helper structs
struct TransactionInfo {
    payment_method: String,
}

struct OriginalItem {
    quantity: i32,
    price: f64,
    product_name: String,
}
