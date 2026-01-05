use super::*;
use crate::core::{Condition, PriceInfo};
use crate::pricing::RuleEngine;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

struct MockPricingService {
    cached_price: Option<PriceInfo>,
    fresh_price: Option<PriceInfo>,
}

#[async_trait::async_trait]
impl PricingServiceTrait for MockPricingService {
    async fn get_price_for_card(&self, _product_uuid: Uuid) -> Option<PriceInfo> {
        self.fresh_price.clone()
    }

    async fn get_cached_price(&self, _product_uuid: Uuid) -> Option<PriceInfo> {
        self.cached_price.clone()
    }

    fn calculate_safety_status(
        &self,
        cached_price: f64,
        market_update: f64,
    ) -> crate::buylist::PriceStatus {
        let variance = (market_update - cached_price).abs() / cached_price;
        if variance > 0.15 {
            crate::buylist::PriceStatus::Flagged
        } else {
            crate::buylist::PriceStatus::Safe
        }
    }
}

#[tokio::test]
async fn test_volatility_guard_triggers() {
    // Setup In-Memory DB
    let db = Database::new("sqlite::memory:", "test_node_1".to_string())
        .await
        .expect("Failed to create in-memory DB");
    db.initialize_tables().await.expect("Failed to init tables");
    let db_arc = Arc::new(db);

    let product_uuid = Uuid::new_v4();

    // Setup Mock Pricing Service with Volatility > 15%
    // Cached: $100
    // Fresh: $120 (20% increase) -> Should Flag
    let cached_price = PriceInfo {
        price_uuid: Uuid::new_v4(),
        product_uuid,
        market_mid: 100.0,
        market_low: 90.0,
        last_sync_timestamp: Utc::now(),
    };

    let fresh_price = PriceInfo {
        price_uuid: Uuid::new_v4(),
        product_uuid,
        market_mid: 120.0, // 20% jump
        market_low: 100.0,
        last_sync_timestamp: Utc::now(),
    };

    let pricing_service = Arc::new(MockPricingService {
        cached_price: Some(cached_price),
        fresh_price: Some(fresh_price),
    });

    // Insert product so foreign key and rule engine lookup passes
    let product_obj = crate::core::Product {
        product_uuid,
        name: "Test Card".to_string(),
        category: crate::core::Category::TCG,
        set_code: None,
        collector_number: None,
        barcode: None,
        release_year: None,
        metadata: serde_json::json!({}),
        weight_oz: None,
        length_in: None,
        width_in: None,
        height_in: None,
        upc: None,
        isbn: None,
        manufacturer: None,
        msrp: None,
        deleted_at: None,
    };
    db_arc
        .products
        .insert(&product_obj)
        .await
        .expect("Failed to insert product");

    let rule_engine = Arc::new(RuleEngine::new());
    let inventory_service = Arc::new(crate::inventory::InventoryService::new(
        db_arc.inventory.clone(),
    ));
    let service = BuylistService::new(db_arc, pricing_service, rule_engine, inventory_service);

    let items = vec![BuylistItem {
        product_uuid,
        condition: Condition::NM,
        quantity: 1,
    }];

    let result = service
        .process_buylist_transaction(None, items, PaymentMethod::Cash)
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Volatility Alert"));
}

#[tokio::test]
async fn test_safe_price_passes() {
    // Setup In-Memory DB
    let db = Database::new("sqlite::memory:", "test_node_2".to_string())
        .await
        .expect("Failed to create in-memory DB");
    db.initialize_tables().await.expect("Failed to init tables");
    let db_arc = Arc::new(db);

    let product_uuid = Uuid::new_v4();

    // Insert product so foreign key constraint passes
    let product = crate::core::Product {
        product_uuid,
        name: "Test Card".to_string(),
        category: crate::core::Category::TCG,
        set_code: None,
        collector_number: None,
        barcode: None,
        release_year: None,
        metadata: serde_json::json!({}),
        weight_oz: None,
        length_in: None,
        width_in: None,
        height_in: None,
        upc: None,
        isbn: None,
        manufacturer: None,
        msrp: None,
        deleted_at: None,
    };
    db_arc
        .products
        .insert(&product)
        .await
        .expect("Failed to insert product");

    // Setup Mock Pricing Service with Volatility < 15%
    // Cached: $100
    // Fresh: $105 (5% increase) -> Safe
    let cached_price = PriceInfo {
        price_uuid: Uuid::new_v4(),
        product_uuid,
        market_mid: 100.0,
        market_low: 90.0,
        last_sync_timestamp: Utc::now(),
    };

    let fresh_price = PriceInfo {
        price_uuid: Uuid::new_v4(),
        product_uuid,
        market_mid: 105.0,
        market_low: 95.0,
        last_sync_timestamp: Utc::now(),
    };

    let pricing_service = Arc::new(MockPricingService {
        cached_price: Some(cached_price),
        fresh_price: Some(fresh_price),
    });

    let rule_engine = Arc::new(RuleEngine::new());
    let inventory_service = Arc::new(crate::inventory::InventoryService::new(
        db_arc.inventory.clone(),
    ));
    let service = BuylistService::new(db_arc, pricing_service, rule_engine, inventory_service);

    let items = vec![BuylistItem {
        product_uuid,
        condition: Condition::NM,
        quantity: 1,
    }];

    let result = service
        .process_buylist_transaction(None, items, PaymentMethod::Cash)
        .await;

    if let Err(e) = &result {
        println!("DEBUG_ERROR: {:?}", e);
    }
    assert!(result.is_ok(), "Test failed with error: {:?}", result.err());
}

#[tokio::test]
async fn test_instant_quote() {
    // Setup DB
    let db = Database::new("sqlite::memory:", "test_node_3".to_string())
        .await
        .expect("Failed to create in-memory DB");
    db.initialize_tables().await.expect("Failed to init tables");
    let db_arc = Arc::new(db);
    let product_uuid = Uuid::new_v4();

    // Insert Product
    let product = crate::core::Product {
        product_uuid,
        name: "Test Card".to_string(),
        category: crate::core::Category::TCG,
        set_code: None,
        collector_number: None,
        barcode: None,
        release_year: None,
        metadata: serde_json::json!({}),
        weight_oz: None,
        length_in: None,
        width_in: None,
        height_in: None,
        upc: None,
        isbn: None,
        manufacturer: None,
        msrp: None,
        deleted_at: None,
    };
    db_arc
        .products
        .insert(&product)
        .await
        .expect("Failed to insert product");

    // Mock Price: $100
    let fresh_price = PriceInfo {
        price_uuid: Uuid::new_v4(),
        product_uuid,
        market_mid: 100.0,
        market_low: 90.0,
        last_sync_timestamp: Utc::now(),
    };

    let pricing_service = Arc::new(MockPricingService {
        cached_price: None,
        fresh_price: Some(fresh_price),
    });

    let rule_engine = Arc::new(RuleEngine::new());
    let inventory_service = Arc::new(crate::inventory::InventoryService::new(
        db_arc.inventory.clone(),
    ));
    let service = BuylistService::new(db_arc, pricing_service, rule_engine, inventory_service);

    // Calculate quote for NM
    let quote = service
        .calculate_instant_quote(product_uuid, Condition::NM)
        .await
        .expect("Failed to get quote");

    // Cash: 60% of Market Low ($90 * 0.6 = $54) - based on high_end rule (> $50)
    // Credit: 75% of Market Low ($90 * 0.75 = $67.5)
    // Note: Code uses market_low (90.0) as baseline if available (LOW-001 FIX)

    assert_eq!(quote.cash_price, 54.0);
    assert_eq!(quote.credit_price, 67.5);
    assert_eq!(quote.market_price, 100.0);
}
