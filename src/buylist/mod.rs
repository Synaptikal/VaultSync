use crate::core::{Condition, InventoryItem, Transaction, TransactionItem, TransactionType};
use crate::database::Database;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;
use uuid::Uuid;

#[cfg(test)]
mod tests;

// PriceStatus moved to core
pub use crate::core::PriceStatus;

pub mod matcher;
use matcher::WantsMatchingService;

use crate::core::Product;
use crate::inventory::InventoryService;
use crate::pricing::{RuleContext, RuleEngine};

pub struct BuylistService {
    db: Arc<Database>,
    pricing_service: Arc<dyn PricingServiceTrait>,
    rule_engine: Arc<RuleEngine>,
    matcher: Arc<WantsMatchingService>,
    inventory_service: Arc<InventoryService>,
}

// Trait to allow mocking of pricing service in tests
#[async_trait::async_trait]
pub trait PricingServiceTrait: Send + Sync {
    async fn get_price_for_card(&self, product_uuid: Uuid) -> Option<crate::core::PriceInfo>;
    async fn get_cached_price(&self, product_uuid: Uuid) -> Option<crate::core::PriceInfo>;
    fn calculate_safety_status(
        &self,
        cached_price: f64,
        market_update: f64,
    ) -> crate::core::PriceStatus;
}

impl BuylistService {
    pub fn new(
        db: Arc<Database>,
        pricing_service: Arc<dyn PricingServiceTrait>,
        rule_engine: Arc<RuleEngine>,
        inventory_service: Arc<InventoryService>,
    ) -> Self {
        let matcher = Arc::new(WantsMatchingService::new(db.clone()));
        Self {
            db,
            pricing_service,
            rule_engine,
            matcher,
            inventory_service,
        }
    }

    pub async fn calculate_instant_quote(
        &self,
        product_uuid: Uuid,
        condition: Condition,
    ) -> Result<QuoteResult> {
        let price_info = self.pricing_service.get_price_for_card(product_uuid).await;

        if let Some(price) = price_info {
            // Fetch product for rule engine
            let product = self
                .db
                .products
                .get_by_id(product_uuid)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Product not found"))?;

            let cash_price = self.calculate_item_price(
                &price,
                &product,
                &condition,
                &PaymentMethod::Cash,
                1,
                None,
            );
            let credit_price = self.calculate_item_price(
                &price,
                &product,
                &condition,
                &PaymentMethod::StoreCredit,
                1,
                None,
            );

            Ok(QuoteResult {
                cash_price,
                credit_price,
                market_price: price.market_mid,
            })
        } else {
            Err(anyhow::anyhow!(
                "Price not found for product {}",
                product_uuid
            ))
        }
    }

    /// Process a buylist transaction where the shop buys cards from a customer
    ///
    /// HIGH-001 FIX: Now uses atomic database transaction for inventory + transaction
    pub async fn process_buylist_transaction(
        &self,
        customer_uuid: Option<Uuid>,
        items: Vec<BuylistItem>,
        payment_method: PaymentMethod,
    ) -> Result<Transaction> {
        let mut transaction_items = Vec::new();
        let mut total_value = 0.0;
        let mut matches = Vec::new();

        // Fetch customer tier if customer exists
        let customer_tier_data = if let Some(uuid) = customer_uuid {
            self.db
                .customers
                .get_by_id(uuid)
                .await
                .ok()
                .flatten()
                .and_then(|c| c.tier)
        } else {
            None
        };
        let customer_tier = customer_tier_data.as_deref();

        // Phase 1: Validate all items and calculate prices BEFORE any database changes
        for item in &items {
            // Get cached price first to check for volatility
            let cached_price_info = self
                .pricing_service
                .get_cached_price(item.product_uuid)
                .await;

            // Get current price (this may trigger an API fetch if cache is stale)
            let current_price_info = self
                .pricing_service
                .get_price_for_card(item.product_uuid)
                .await;

            if let Some(current_price) = current_price_info {
                // If we had a cached price, check for volatility
                if let Some(cached) = cached_price_info {
                    let safety_status = self
                        .pricing_service
                        .calculate_safety_status(cached.market_mid, current_price.market_mid);

                    if matches!(safety_status, crate::buylist::PriceStatus::Flagged) {
                        return Err(anyhow::anyhow!(
                            "Price for product {} is flagged for review (Volatility Alert)",
                            item.product_uuid
                        ));
                    }
                }

                // Validate product exists
                let product = self
                    .db
                    .products
                    .get_by_id(item.product_uuid)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Product {} not found", item.product_uuid))?;

                // Calculate price based on condition and payment method
                let item_price = self.calculate_item_price(
                    &current_price,
                    &product,
                    &item.condition,
                    &payment_method,
                    item.quantity,
                    customer_tier,
                );

                total_value += item_price * item.quantity as f64;

                // Create transaction item for atomic execution
                let transaction_item = TransactionItem {
                    item_uuid: Uuid::new_v4(),
                    product_uuid: item.product_uuid,
                    quantity: item.quantity,
                    unit_price: item_price,
                    condition: item.condition.clone(),
                };

                transaction_items.push(transaction_item);

                // Check for wants matches (read-only operation)
                let item_matches = self
                    .matcher
                    .find_matches(item.product_uuid, &item.condition, item_price)
                    .await?;
                matches.extend(item_matches);
            } else {
                return Err(anyhow::anyhow!(
                    "No price available for product {}",
                    item.product_uuid
                ));
            }
        }
        // Phase 2: Execute atomic buy transaction (adds to inventory + creates transaction in one TX)
        // This uses the TransactionRepository's atomic execute_buy method
        let transaction = self
            .db
            .transactions
            .execute_buy(customer_uuid, None, transaction_items)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute buy transaction: {}", e))?;

        // Phase 3: Post-transaction operations (logging, notifications)
        for match_item in &matches {
            let (customer, wants_item) = match_item;
            tracing::info!(
                "WANTS MATCH: Customer {} ({}) matched item {}",
                customer.name,
                customer.email.as_deref().unwrap_or("no-email"),
                wants_item.product_uuid
            );
        }

        tracing::info!(
            "Processed Buy Transaction: {}, Total Value: ${:.2}",
            transaction.transaction_uuid,
            total_value
        );

        Ok(transaction)
    }

    /// Process a trade-in transaction where the customer trades cards for store credit
    ///
    /// # Returns
    /// - `net_value > 0`: Customer owes this amount (pay cash/card)
    /// - `net_value < 0`: Customer has excess credit (added to store credit)
    /// - `net_value == 0`: Even trade
    pub async fn process_trade_in_transaction(
        &self,
        customer_uuid: Option<Uuid>,
        trade_in_items: Vec<BuylistItem>,
        purchase_items: Vec<TransactionItem>,
    ) -> Result<TradeInResult> {
        // Fetch customer tier
        let customer_tier_data = if let Some(uuid) = customer_uuid {
            self.db
                .customers
                .get_by_id(uuid)
                .await
                .ok()
                .flatten()
                .and_then(|c| c.tier)
        } else {
            None
        };
        let customer_tier = customer_tier_data.as_deref();

        // Calculate value of trade-in items (what shop pays customer)
        let trade_in_value = self
            .calculate_trade_in_value(&trade_in_items, customer_tier)
            .await?;

        // Calculate value of purchase items (what customer pays shop)
        let purchase_value = self.calculate_purchase_value(&purchase_items).await?;

        // net_value = what customer owes
        // Positive = customer pays difference
        // Negative = customer gets credit for excess
        let net_value = purchase_value - trade_in_value;

        // Convert buylist items to transaction items with proper pricing
        let mut trade_in_tx_items = Vec::new();
        for item in &trade_in_items {
            let price_info = self
                .pricing_service
                .get_price_for_card(item.product_uuid)
                .await;
            if let Some(price) = price_info {
                let product = self
                    .db
                    .products
                    .get_by_id(item.product_uuid)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Product not found"))?;
                let item_price = self.calculate_item_price(
                    &price,
                    &product,
                    &item.condition,
                    &PaymentMethod::StoreCredit,
                    item.quantity,
                    customer_tier,
                );
                trade_in_tx_items.push(TransactionItem {
                    item_uuid: Uuid::new_v4(),
                    product_uuid: item.product_uuid,
                    quantity: item.quantity,
                    unit_price: item_price,
                    condition: item.condition.clone(),
                });
            }
        }

        // Create trade-in transaction (shop buying from customer)
        let trade_in_transaction = Transaction {
            transaction_uuid: Uuid::new_v4(),
            items: trade_in_tx_items,
            customer_uuid,
            timestamp: chrono::Utc::now(),
            transaction_type: TransactionType::Trade,
            // Tasks 182/187: Track user. Buylist automated flows might need user context passed in future.
            // For now, None or system user.
            user_uuid: None,
        };

        // Create purchase transaction (customer buying from shop)
        let purchase_transaction = Transaction {
            transaction_uuid: Uuid::new_v4(),
            items: purchase_items,
            customer_uuid,
            timestamp: chrono::Utc::now(),
            transaction_type: TransactionType::Sale,
            user_uuid: None, // Buylist automated creation
        };

        // CRIT-001 FIX: Actually persist both transactions to database
        self.db.transactions.insert(&trade_in_transaction).await?;
        self.db
            .sync
            .log_change(
                &trade_in_transaction.transaction_uuid.to_string(),
                "Transaction",
                "Insert",
                &serde_json::to_value(&trade_in_transaction).unwrap_or_default(),
            )
            .await?;

        self.db.transactions.insert(&purchase_transaction).await?;
        self.db
            .sync
            .log_change(
                &purchase_transaction.transaction_uuid.to_string(),
                "Transaction",
                "Insert",
                &serde_json::to_value(&purchase_transaction).unwrap_or_default(),
            )
            .await?;

        // Add trade-in items to inventory
        for item in &trade_in_items {
            // Calculate what we paid for this item (cost basis = unit_price from trade-in)
            let price_info = self
                .pricing_service
                .get_price_for_card(item.product_uuid)
                .await;
            let cost_basis = if let Some(price) = price_info {
                let product = self
                    .db
                    .products
                    .get_by_id(item.product_uuid)
                    .await
                    .ok()
                    .flatten();
                if let Some(prod) = product {
                    let customer_tier_data = if let Some(uuid) = customer_uuid {
                        self.db
                            .customers
                            .get_by_id(uuid)
                            .await
                            .ok()
                            .flatten()
                            .and_then(|c| c.tier)
                    } else {
                        None
                    };
                    Some(self.calculate_item_price(
                        &price,
                        &prod,
                        &item.condition,
                        &PaymentMethod::StoreCredit, // Trade-ins typically get credit rate
                        item.quantity,
                        customer_tier_data.as_deref(),
                    ))
                } else {
                    None
                }
            } else {
                None
            };

            let inventory_item = InventoryItem {
                inventory_uuid: Uuid::new_v4(),
                product_uuid: item.product_uuid,
                variant_type: None,
                condition: item.condition.clone(),
                quantity_on_hand: item.quantity,
                location_tag: "Buylist".to_string(),
                specific_price: None, // Mixed stock usually doesn't have specific price unless serialized
                serialized_details: None,
                cost_basis, // Now calculated from trade value
                supplier_uuid: customer_uuid,
                received_date: Some(chrono::Utc::now()),
                min_stock_level: 0,
                max_stock_level: None,
                reorder_point: None,
                bin_location: None,
                last_sold_date: None,
                last_counted_date: None,
                deleted_at: None,
            };
            self.inventory_service.add_item(inventory_item).await?;
        }

        // CRIT-002 FIX: Correct store credit logic
        // When net_value < 0, customer has EXCESS trade-in value -> ADD to store credit
        // When net_value > 0, customer OWES money -> they pay cash, no automatic credit change
        // When net_value == 0, even trade -> no credit change
        if net_value < 0.0 {
            // Customer has excess trade-in value, add the absolute value to their store credit
            if let Some(uuid) = customer_uuid {
                let credit_to_add = net_value.abs(); // Convert negative to positive
                self.update_customer_store_credit(uuid, credit_to_add)
                    .await?;
                tracing::info!(
                    "Trade-in: Added ${:.2} store credit to customer {}",
                    credit_to_add,
                    uuid
                );
            }
        } else if net_value > 0.0 {
            // Customer owes money - they pay cash/card at register
            // Do NOT automatically deduct from store credit here
            // The front-end should handle payment method selection
            tracing::info!(
                "Trade-in: Customer owes ${:.2} (to be collected at register)",
                net_value
            );
        }

        tracing::info!(
            "Processed Trade-In: trade_in_tx={}, purchase_tx={}, net_value=${:.2}",
            trade_in_transaction.transaction_uuid,
            purchase_transaction.transaction_uuid,
            net_value
        );

        Ok(TradeInResult {
            trade_in_transaction,
            purchase_transaction,
            net_value,
        })
    }

    /// Calculate the total value of trade-in items
    async fn calculate_trade_in_value(
        &self,
        items: &[BuylistItem],
        customer_tier: Option<&str>,
    ) -> Result<f64> {
        let mut total_value = 0.0;

        for item in items {
            let price_info = self
                .pricing_service
                .get_price_for_card(item.product_uuid)
                .await;

            if let Some(price_info) = price_info {
                let product = self
                    .db
                    .products
                    .get_by_id(item.product_uuid)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Product not found"))?;
                let item_price = self.calculate_item_price(
                    &price_info,
                    &product,
                    &item.condition,
                    &PaymentMethod::Cash,
                    item.quantity,
                    customer_tier,
                ); // Trade-in value usually treats as Cash/Base or specific TradeIn rate? Using Cash base for now as TradeInValue
                total_value += item_price * item.quantity as f64;
            } else {
                return Err(anyhow::anyhow!(
                    "No price available for product {}",
                    item.product_uuid
                ));
            }
        }

        Ok(total_value)
    }

    /// Calculate the total value of purchase items
    async fn calculate_purchase_value(&self, items: &[TransactionItem]) -> Result<f64> {
        let mut total_value = 0.0;

        for item in items {
            let price_info = self
                .pricing_service
                .get_price_for_card(item.product_uuid)
                .await;

            if let Some(_price_info) = price_info {
                total_value += item.unit_price * item.quantity as f64;
            } else {
                return Err(anyhow::anyhow!(
                    "No price available for product {}",
                    item.product_uuid
                ));
            }
        }

        Ok(total_value)
    }

    /// Calculate item price based on condition and payment method
    /// Calculate item price based on condition, rules, and payment method
    fn calculate_item_price(
        &self,
        price_info: &crate::core::PriceInfo,
        product: &Product,
        condition: &Condition,
        payment_method: &PaymentMethod,
        quantity: i32,
        customer_tier: Option<&str>,
    ) -> f64 {
        // 1. Apply Condition Scaling (Standard Industry Scale)
        let condition_scale = match condition {
            Condition::NM
            | Condition::NearMintMint
            | Condition::Mint
            | Condition::GemMint
            | Condition::New => 1.0,
            Condition::LP | Condition::VeryFine | Condition::OpenBox => 0.8,
            Condition::MP | Condition::Fine | Condition::Used => 0.6,
            Condition::HP | Condition::Good => 0.4,
            Condition::DMG | Condition::Poor => 0.2,
        };

        // LOW-001 FIX: Use market_low as safer baseline for buying if available
        let base_price = if price_info.market_low > 0.0 {
            price_info.market_low
        } else {
            price_info.market_mid
        };

        let scaled_market_price = base_price * condition_scale;

        // 2. Get Buy Rate from Rule Engine
        let context = RuleContext {
            product: Some(product),
            condition,
            market_price: scaled_market_price,
            quantity,
            customer_tier,
        };
        let (cash_rate, credit_rate) = self.rule_engine.calculate_multipliers(context);

        // Apply payment method multiplier
        match payment_method {
            PaymentMethod::Cash => scaled_market_price * cash_rate,
            PaymentMethod::StoreCredit => scaled_market_price * credit_rate,
        }
    }

    /// Update customer's store credit
    async fn update_customer_store_credit(&self, customer_uuid: Uuid, amount: f64) -> Result<()> {
        self.db
            .customers
            .update_store_credit(customer_uuid, amount)
            .await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuylistItem {
    pub product_uuid: Uuid,
    pub condition: Condition,
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentMethod {
    Cash,
    StoreCredit,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuoteResult {
    pub cash_price: f64,
    pub credit_price: f64,
    pub market_price: f64,
}

#[derive(Serialize, Deserialize)]
pub struct TradeInResult {
    pub trade_in_transaction: Transaction,
    pub purchase_transaction: Transaction,
    pub net_value: f64,
}
