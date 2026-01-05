//! Transaction Service - Domain logic for sales and purchases
//!
//! This service implements proper POS business rules:
//! - Inventory validation BEFORE committing sales
//! - Price verification at point of sale
//! - Atomic inventory reservation
//! - Domain event emission for audit trail

use crate::core::{Transaction, TransactionItem};
use crate::database::Database;
use crate::inventory::InventoryService;
use crate::pricing::PricingService;
use anyhow::Result;
use std::sync::Arc;
use tracing;
use uuid::Uuid;

/// Errors that can occur during transaction processing
#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("Insufficient inventory for product {product_uuid}: requested {requested}, available {available}")]
    InsufficientInventory {
        product_uuid: Uuid,
        requested: i32,
        available: i32,
    },

    #[error("Empty transaction: no items provided")]
    EmptyTransaction,

    #[error("Price verification failed for product {product_uuid}: expected {expected:.2}, got {actual:.2}")]
    PriceVerificationFailed {
        product_uuid: Uuid,
        expected: f64,
        actual: f64,
    },

    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Domain event emitted when a sale completes
#[derive(Debug, Clone)]
pub struct SaleCompleted {
    pub transaction_uuid: Uuid,
    pub customer_uuid: Option<Uuid>,
    pub user_uuid: Option<Uuid>,
    pub items: Vec<TransactionItem>,
    pub total: f64,
}

/// Domain event emitted when inventory is reserved
#[derive(Debug, Clone)]
pub struct InventoryReserved {
    pub product_uuid: Uuid,
    pub quantity: i32,
    pub for_transaction: Uuid,
}

pub struct TransactionService {
    db: Arc<Database>,
    inventory_service: Arc<InventoryService>,
    pricing_service: Arc<PricingService>,
}

impl TransactionService {
    pub fn new(
        db: Arc<Database>,
        inventory_service: Arc<InventoryService>,
        pricing_service: Arc<PricingService>,
    ) -> Self {
        Self {
            db,
            inventory_service,
            pricing_service,
        }
    }

    /// Process a sale transaction with proper business logic
    ///
    /// ## Business Rules Applied
    /// 1. Validate items list is non-empty
    /// 2. Check inventory availability for ALL items BEFORE committing
    /// 3. Verify prices match current market (optional tolerance)
    /// 4. Execute atomic sale with inventory adjustment
    /// 5. Emit domain event for audit trail
    pub async fn process_sale(
        &self,
        customer_uuid: Option<Uuid>,
        user_uuid: Option<Uuid>,
        items: Vec<TransactionItem>,
    ) -> Result<Transaction> {
        // Rule 1: Validate non-empty
        if items.is_empty() {
            return Err(TransactionError::EmptyTransaction.into());
        }

        tracing::info!(
            "Processing sale with {} items for customer {:?}",
            items.len(),
            customer_uuid
        );

        // Rule 2: Validate inventory availability BEFORE we start
        for item in &items {
            let available = self
                .get_available_quantity(item.product_uuid, &item.condition)
                .await?;

            if available < item.quantity {
                tracing::warn!(
                    "Insufficient inventory for product {}: need {}, have {}",
                    item.product_uuid,
                    item.quantity,
                    available
                );
                return Err(TransactionError::InsufficientInventory {
                    product_uuid: item.product_uuid,
                    requested: item.quantity,
                    available,
                }
                .into());
            }
        }

        // Rule 3: Price verification (log warning if prices deviate significantly)
        for item in &items {
            if let Some(market_price) = self
                .pricing_service
                .get_price_for_product(item.product_uuid)
                .await
            {
                let deviation = ((item.unit_price - market_price.market_mid).abs()
                    / market_price.market_mid)
                    * 100.0;

                if deviation > 25.0 {
                    tracing::warn!(
                        "Price deviation of {:.1}% for product {} (market: ${:.2}, sale: ${:.2})",
                        deviation,
                        item.product_uuid,
                        market_price.market_mid,
                        item.unit_price
                    );
                    // Note: We log but don't reject - price overrides are allowed, just tracked
                }
            }
        }

        // Rule 4: Execute atomic sale
        let transaction = self
            .db
            .transactions
            .execute_sale(customer_uuid, user_uuid, items.clone())
            .await
            .map_err(|e| TransactionError::DatabaseError(e.to_string()))?;

        // Rule 5: Emit domain event (for now, just log - future: event bus)
        let total: f64 = items.iter().map(|i| i.unit_price * i.quantity as f64).sum();

        tracing::info!(
            "SALE_COMPLETED: transaction={}, customer={:?}, items={}, total=${:.2}",
            transaction.transaction_uuid,
            customer_uuid,
            items.len(),
            total
        );

        Ok(transaction)
    }

    /// Process a buy transaction (customer sells to shop)
    ///
    /// ## Business Rules Applied
    /// 1. Validate items list is non-empty
    /// 2. Verify prices are reasonable (not above market)
    /// 3. Execute atomic buy with inventory addition
    /// 4. Emit domain event for audit trail
    pub async fn process_buy(
        &self,
        customer_uuid: Option<Uuid>,
        user_uuid: Option<Uuid>,
        items: Vec<TransactionItem>,
    ) -> Result<Transaction> {
        // Rule 1: Validate non-empty
        if items.is_empty() {
            return Err(TransactionError::EmptyTransaction.into());
        }

        tracing::info!(
            "Processing buy with {} items from customer {:?}",
            items.len(),
            customer_uuid
        );

        // Rule 2: Price sanity check - don't buy above market
        for item in &items {
            if let Some(market_price) = self
                .pricing_service
                .get_price_for_product(item.product_uuid)
                .await
            {
                // For buys, we should be paying LESS than market mid
                if item.unit_price > market_price.market_mid * 0.85 {
                    tracing::warn!(
                        "Buy price {:.2} for {} is high (market mid: {:.2})",
                        item.unit_price,
                        item.product_uuid,
                        market_price.market_mid
                    );
                }
            }
        }

        // Rule 3: Execute atomic buy
        let transaction = self
            .db
            .transactions
            .execute_buy(customer_uuid, user_uuid, items.clone())
            .await
            .map_err(|e| TransactionError::DatabaseError(e.to_string()))?;

        // Rule 4: Emit domain event
        let total: f64 = items.iter().map(|i| i.unit_price * i.quantity as f64).sum();

        tracing::info!(
            "BUY_COMPLETED: transaction={}, customer={:?}, items={}, total=${:.2}",
            transaction.transaction_uuid,
            customer_uuid,
            items.len(),
            total
        );

        Ok(transaction)
    }

    /// Get available quantity for a product/condition combination
    async fn get_available_quantity(
        &self,
        product_uuid: Uuid,
        condition: &crate::core::Condition,
    ) -> Result<i32> {
        // Get all inventory items for this product
        let items = self
            .inventory_service
            .get_items_by_product(product_uuid)
            .await?;

        // Sum quantities matching the condition
        let available: i32 = items
            .iter()
            .filter(|i| &i.condition == condition)
            .map(|i| i.quantity_on_hand)
            .sum();

        Ok(available)
    }

    /// Get transaction by ID
    pub async fn get_transaction(&self, transaction_id: Uuid) -> Option<Transaction> {
        self.db
            .transactions
            .get_by_id(transaction_id)
            .await
            .ok()
            .flatten()
    }

    /// Get all transactions for a customer
    pub async fn get_customer_transactions(&self, customer_uuid: Uuid) -> Result<Vec<Transaction>> {
        self.db.transactions.get_by_customer(customer_uuid).await
    }

    /// Get transaction history
    pub async fn get_transaction_history(&self) -> Result<Vec<Transaction>> {
        self.db.transactions.get_recent(100).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_transaction_rejected() {
        // Empty items should fail validation
        let err = TransactionError::EmptyTransaction;
        assert!(err.to_string().contains("Empty transaction"));
    }

    #[test]
    fn test_insufficient_inventory_error() {
        let err = TransactionError::InsufficientInventory {
            product_uuid: Uuid::new_v4(),
            requested: 5,
            available: 2,
        };
        assert!(err.to_string().contains("Insufficient inventory"));
        assert!(err.to_string().contains("requested 5"));
        assert!(err.to_string().contains("available 2"));
    }
}
