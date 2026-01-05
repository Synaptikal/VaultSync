//! Transaction validation and processing service
//!
//! Enforces business rules for transactions including:
//! - Stock availability validation
//! - Customer credit limits
//! - Trade-in limits
//! - Split payment handling
//! - Transaction totals calculation

use crate::database::Database;
use crate::errors::Result;
use crate::services::{PaymentMethodType, PaymentRequest, PaymentService, TaxService};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Transaction creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub customer_uuid: Option<Uuid>,
    pub items: Vec<TransactionItemRequest>,
    pub payments: Vec<PaymentRequest>,
    pub trade_in_items: Option<Vec<TradeInItemRequest>>,
    pub notes: Option<String>,
    pub location_uuid: Option<Uuid>,
}

/// Request for a single item in a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionItemRequest {
    pub inventory_uuid: Uuid,
    pub quantity: i32,
    pub unit_price: f64,
    pub override_price: Option<f64>,
    pub override_reason: Option<String>,
}

/// Request for a trade-in item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeInItemRequest {
    pub product_uuid: Uuid,
    pub condition: String,
    pub quantity: i32,
    pub offered_price: f64,
}

/// Full transaction result with breakdown
#[derive(Debug, Clone, Serialize)]
pub struct TransactionResult {
    pub transaction_uuid: Uuid,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub trade_in_credit: f64,
    pub total: f64,
    pub amount_paid: f64,
    pub change_given: f64,
    pub success: bool,
    pub errors: Vec<String>,
}

/// Validation result for a transaction
#[derive(Debug, Clone, Serialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub trade_in_credit: f64,
    pub grand_total: f64,
}

/// Enhanced transaction processing service
pub struct TransactionValidationService {
    db: Arc<Database>,
    tax_service: Arc<TaxService>,
    payment_service: Arc<PaymentService>,
}

impl TransactionValidationService {
    pub fn new(
        db: Arc<Database>,
        tax_service: Arc<TaxService>,
        payment_service: Arc<PaymentService>,
    ) -> Self {
        Self {
            db,
            tax_service,
            payment_service,
        }
    }

    /// Validate a transaction request without processing it
    pub async fn validate_transaction(
        &self,
        request: &TransactionRequest,
    ) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut subtotal = 0.0;
        let mut trade_in_credit = 0.0;

        // Check customer if specified
        let customer_tax_exempt = if let Some(customer_uuid) = request.customer_uuid {
            match self.get_customer_info(customer_uuid).await? {
                Some(info) => {
                    if info.is_banned {
                        errors.push(format!(
                            "Customer is banned: {}",
                            info.ban_reason.unwrap_or_default()
                        ));
                    }

                    // Check trade-in limits if applicable
                    if let Some(ref trade_ins) = request.trade_in_items {
                        let trade_in_total: f64 = trade_ins
                            .iter()
                            .map(|t| t.offered_price * t.quantity as f64)
                            .sum();
                        trade_in_credit = trade_in_total;

                        if trade_in_total > info.trade_in_limit {
                            errors.push(format!(
                                "Trade-in total ${:.2} exceeds customer limit ${:.2}",
                                trade_in_total, info.trade_in_limit
                            ));
                        }
                    }

                    info.tax_exempt
                }
                None => {
                    errors.push(format!("Customer {} not found", customer_uuid));
                    false
                }
            }
        } else {
            false
        };

        // Validate each item
        for item in &request.items {
            match self.validate_item(item).await {
                Ok(item_total) => {
                    subtotal += item_total;
                }
                Err(e) => {
                    errors.push(e.to_string());
                }
            }

            // Warn on price overrides
            if item.override_price.is_some() {
                if let Some(price) = item.override_price {
                    warnings.push(format!(
                        "Item {} has price override to ${:.2}",
                        item.inventory_uuid, price
                    ));
                }
            }
        }

        // Calculate tax
        let tax_amount = if customer_tax_exempt {
            0.0
        } else {
            self.tax_service.get_default_rate().await? * subtotal
        };

        // Calculate grand total
        let grand_total = subtotal + tax_amount - trade_in_credit;

        // Validate payment amounts
        let payment_total: f64 = request.payments.iter().map(|p| p.amount).sum();

        if request.payments.is_empty() && grand_total > 0.0 {
            errors.push("No payment methods specified".to_string());
        } else if payment_total < grand_total - 0.01 {
            errors.push(format!(
                "Payment total ${:.2} is less than amount due ${:.2}",
                payment_total, grand_total
            ));
        }

        // Validate store credit payments have a customer
        for payment in &request.payments {
            if payment.method == PaymentMethodType::StoreCredit && request.customer_uuid.is_none() {
                errors.push("Store credit payment requires a customer".to_string());
            }
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            subtotal,
            tax_amount,
            trade_in_credit,
            grand_total,
        })
    }

    /// Validate a single transaction item
    async fn validate_item(&self, item: &TransactionItemRequest) -> Result<f64> {
        // Check inventory availability
        let row = sqlx::query(
            "SELECT quantity_on_hand, deleted_at FROM Local_Inventory WHERE inventory_uuid = ?",
        )
        .bind(item.inventory_uuid.to_string())
        .fetch_optional(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        match row {
            Some(r) => {
                let deleted_at: Option<String> =
                    sqlx::Row::try_get(&r, "deleted_at").ok().flatten();
                if deleted_at.is_some() {
                    return Err(anyhow::anyhow!(
                        "Item {} has been deleted",
                        item.inventory_uuid
                    ));
                }

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

        // Calculate item total
        let price = item.override_price.unwrap_or(item.unit_price);
        Ok(price * item.quantity as f64)
    }

    /// Get customer info for validation
    async fn get_customer_info(&self, customer_uuid: Uuid) -> Result<Option<CustomerInfo>> {
        let row = sqlx::query(
            "SELECT customer_uuid, store_credit, trade_in_limit, is_banned, ban_reason, tax_exempt
             FROM Customers WHERE customer_uuid = ? AND deleted_at IS NULL",
        )
        .bind(customer_uuid.to_string())
        .fetch_optional(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(row.map(|r| CustomerInfo {
            customer_uuid,
            store_credit: sqlx::Row::try_get(&r, "store_credit").unwrap_or(0.0),
            trade_in_limit: sqlx::Row::try_get(&r, "trade_in_limit").unwrap_or(500.0),
            is_banned: sqlx::Row::try_get::<i32, _>(&r, "is_banned").unwrap_or(0) == 1,
            ban_reason: sqlx::Row::try_get(&r, "ban_reason").ok(),
            tax_exempt: sqlx::Row::try_get::<i32, _>(&r, "tax_exempt").unwrap_or(0) == 1,
        }))
    }

    /// Process a validated transaction (creates the transaction atomically)
    pub async fn process_transaction(
        &self,
        request: &TransactionRequest,
        user_uuid: Option<Uuid>,
    ) -> Result<TransactionResult> {
        // First validate (read-only checks)
        let validation = self.validate_transaction(request).await?;

        if !validation.is_valid {
            return Ok(TransactionResult {
                transaction_uuid: Uuid::nil(),
                subtotal: validation.subtotal,
                tax_amount: validation.tax_amount,
                trade_in_credit: validation.trade_in_credit,
                total: validation.grand_total,
                amount_paid: 0.0,
                change_given: 0.0,
                success: false,
                errors: validation.errors,
            });
        }

        // Begin Atomic Transaction
        let mut tx = self
            .db
            .pool
            .begin()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start transaction: {}", e))?;

        let transaction_uuid = Uuid::new_v4();
        let now = Utc::now();

        // Create transaction record
        sqlx::query(
            "INSERT INTO Transactions 
             (transaction_uuid, customer_uuid, user_uuid, timestamp, transaction_type, subtotal, tax_amount, total, notes, location_uuid)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(transaction_uuid.to_string())
        .bind(request.customer_uuid.map(|u| u.to_string()))
        .bind(user_uuid.map(|u| u.to_string()))
        .bind(now.to_rfc3339())
        .bind("Sale")
        .bind(validation.subtotal)
        .bind(validation.tax_amount)
        .bind(validation.grand_total)
        .bind(&request.notes)
        .bind(request.location_uuid.map(|u| u.to_string()))
        .execute(&mut *tx)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create transaction: {}", e))?;

        // Create transaction items and update inventory
        for item in &request.items {
            let item_uuid = Uuid::new_v4();
            let price = item.override_price.unwrap_or(item.unit_price);

            // Get product_uuid from inventory (Read within TX for consistency, though low risk of race if UUIDs are stable)
            let inv_row =
                sqlx::query("SELECT product_uuid FROM Local_Inventory WHERE inventory_uuid = ?")
                    .bind(item.inventory_uuid.to_string())
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get inventory item: {}", e))?;

            let product_uuid: String = sqlx::Row::try_get(&inv_row, "product_uuid")
                .map_err(|e| anyhow::anyhow!("Missing product_uuid in inventory: {}", e))?;

            // Insert transaction item
            sqlx::query(
                "INSERT INTO Transaction_Items 
                 (item_uuid, transaction_uuid, product_uuid, quantity, unit_price, condition)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(item_uuid.to_string())
            .bind(transaction_uuid.to_string())
            .bind(&product_uuid)
            .bind(item.quantity)
            .bind(price)
            .bind("NM") // Default, should get from inventory or request
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create transaction item: {}", e))?;

            // Deduct inventory
            sqlx::query(
                "UPDATE Local_Inventory 
                 SET quantity_on_hand = quantity_on_hand - ?, last_sold_date = ?
                 WHERE inventory_uuid = ?",
            )
            .bind(item.quantity)
            .bind(now.to_rfc3339())
            .bind(item.inventory_uuid.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update inventory: {}", e))?;
        }

        // Process payments
        let payment_result = self
            .payment_service
            .process_split_payment_with_tx(
                &mut tx,
                transaction_uuid,
                request.customer_uuid,
                request.payments.clone(),
                validation.grand_total,
            )
            .await?;

        // Add trade-in credit to customer if applicable
        if validation.trade_in_credit > 0.0 {
            if let Some(customer_uuid) = request.customer_uuid {
                sqlx::query(
                    "UPDATE Customers SET store_credit = store_credit + ? WHERE customer_uuid = ?",
                )
                .bind(validation.trade_in_credit)
                .bind(customer_uuid.to_string())
                .execute(&mut *tx)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to add trade-in credit: {}", e))?;
            }
        }

        // Calculate change for cash payments
        let mut change_given = 0.0;
        if payment_result.total_paid > validation.grand_total {
            change_given = payment_result.total_paid - validation.grand_total;
            let change_rounded = ((change_given * 100.0).round()) / 100.0;
            change_given = change_rounded;

            // Update transaction with change given
            sqlx::query("UPDATE Transactions SET change_given = ? WHERE transaction_uuid = ?")
                .bind(change_given)
                .bind(transaction_uuid.to_string())
                .execute(&mut *tx)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to record change: {}", e))?;
        }

        // COMMIT TRANSACTION
        tx.commit()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to commit transaction: {}", e))?;

        tracing::info!(
            "Transaction {} completed: ${:.2} subtotal, ${:.2} tax, ${:.2} total",
            transaction_uuid,
            validation.subtotal,
            validation.tax_amount,
            validation.grand_total
        );

        Ok(TransactionResult {
            transaction_uuid,
            subtotal: validation.subtotal,
            tax_amount: validation.tax_amount,
            trade_in_credit: validation.trade_in_credit,
            total: validation.grand_total,
            amount_paid: payment_result.total_paid,
            change_given,
            success: true,
            errors: Vec::new(),
        })
    }

    /// Void an existing transaction
    pub async fn void_transaction(
        &self,
        transaction_uuid: Uuid,
        reason: &str,
        voided_by: &str,
    ) -> Result<()> {
        let now = Utc::now();

        // Get transaction items to restore inventory
        let items = sqlx::query(
            "SELECT ti.product_uuid, ti.quantity, li.inventory_uuid
             FROM Transaction_Items ti
             JOIN Local_Inventory li ON ti.product_uuid = li.product_uuid
             WHERE ti.transaction_uuid = ?",
        )
        .bind(transaction_uuid.to_string())
        .fetch_all(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get transaction items: {}", e))?;

        // Restore inventory
        for item in items {
            let quantity: i32 = sqlx::Row::try_get(&item, "quantity")
                .map_err(|e| anyhow::anyhow!("Missing quantity in transaction item: {}", e))?;
            let inventory_uuid: String =
                sqlx::Row::try_get(&item, "inventory_uuid").map_err(|e| {
                    anyhow::anyhow!("Missing inventory_uuid in transaction item: {}", e)
                })?;

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

        // Mark transaction as voided
        sqlx::query(
            "UPDATE Transactions SET void_reason = ?, voided_at = ?, notes = COALESCE(notes, '') || ? WHERE transaction_uuid = ?",
        )
        .bind(reason)
        .bind(now.to_rfc3339())
        .bind(format!(" [Voided by: {}]", voided_by))
        .bind(transaction_uuid.to_string())
        .execute(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to void transaction: {}", e))?;

        tracing::info!("Transaction {} voided: {}", transaction_uuid, reason);

        Ok(())
    }
}

/// Internal customer info for validation
#[allow(dead_code)]
struct CustomerInfo {
    customer_uuid: Uuid,
    store_credit: f64,
    trade_in_limit: f64,
    is_banned: bool,
    ban_reason: Option<String>,
    tax_exempt: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_defaults() {
        let result = ValidationResult {
            is_valid: true,
            errors: vec![],
            warnings: vec![],
            subtotal: 100.0,
            tax_amount: 8.0,
            trade_in_credit: 0.0,
            grand_total: 108.0,
        };

        assert!(result.is_valid);
        assert_eq!(result.grand_total, 108.0);
    }
}
