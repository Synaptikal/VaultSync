//! Core Types Tests
//!
//! Tests for core domain types, serialization, and business logic

use chrono::Utc;
use uuid::Uuid;
use vaultsync::core::{
    Category, Condition, Customer, InventoryItem, PriceInfo, Product, Transaction, TransactionItem,
    TransactionType,
};

mod product_tests {
    use super::*;
    use serde_json;

    fn create_test_product() -> Product {
        Product {
            product_uuid: Uuid::new_v4(),
            name: "Black Lotus".to_string(),
            category: Category::TCG,
            set_code: Some("LEA".to_string()),
            collector_number: Some("232".to_string()),
            barcode: None,
            release_year: Some(1993),
            metadata: serde_json::json!({
                "game": "magic",
                "rarity": "Rare"
            }),
            weight_oz: None,
            length_in: None,
            width_in: None,
            height_in: None,
            upc: None,
            isbn: None,
            manufacturer: Some("Wizards of the Coast".to_string()),
            msrp: None,
            deleted_at: None,
        }
    }

    #[test]
    fn test_product_creation() {
        let product = create_test_product();
        assert!(!product.name.is_empty());
        assert!(matches!(product.category, Category::TCG));
    }

    #[test]
    fn test_product_serialization_roundtrip() {
        let product = create_test_product();
        let serialized = serde_json::to_string(&product).expect("Should serialize");
        let deserialized: Product = serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(product.product_uuid, deserialized.product_uuid);
        assert_eq!(product.name, deserialized.name);
    }

    #[test]
    fn test_product_with_deletion_timestamp() {
        let product = Product {
            deleted_at: Some(Utc::now()),
            ..create_test_product()
        };
        assert!(product.deleted_at.is_some());
    }
}

mod category_tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_category_variants() {
        let categories = vec![
            Category::TCG,
            Category::SportsCard,
            Category::Comic,
            Category::Bobblehead,
            Category::Apparel,
            Category::Figure,
            Category::Accessory,
            Category::Other,
        ];

        for cat in categories {
            let serialized = serde_json::to_string(&cat).expect("Should serialize");
            assert!(!serialized.is_empty());
        }
    }

    #[test]
    fn test_category_serialization() {
        assert_eq!(serde_json::to_string(&Category::TCG).unwrap(), "\"TCG\"");
        assert_eq!(
            serde_json::to_string(&Category::SportsCard).unwrap(),
            "\"SportsCard\""
        );
    }
}

mod condition_tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_condition_variants_exist() {
        let conditions = vec![
            Condition::Mint,
            Condition::NearMintMint,
            Condition::NM,
            Condition::VeryFine,
            Condition::LP,
            Condition::Fine,
            Condition::MP,
            Condition::Good,
            Condition::HP,
            Condition::Poor,
            Condition::DMG,
        ];

        assert!(
            conditions.len() >= 10,
            "Should have multiple condition grades"
        );
    }

    #[test]
    fn test_condition_serialization() {
        let cond = Condition::NM;
        let serialized = serde_json::to_string(&cond).expect("Should serialize");
        let deserialized: Condition =
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert!(matches!(deserialized, Condition::NM));
    }
}

mod inventory_item_tests {
    use super::*;

    fn create_test_inventory_item() -> InventoryItem {
        InventoryItem {
            inventory_uuid: Uuid::new_v4(),
            product_uuid: Uuid::new_v4(),
            variant_type: None,
            condition: Condition::NM,
            quantity_on_hand: 10,
            location_tag: "Showcase A".to_string(),
            specific_price: Some(99.99),
            serialized_details: None,
            cost_basis: Some(50.0),
            supplier_uuid: None,
            received_date: Some(Utc::now()),
            min_stock_level: 2,
            max_stock_level: Some(20),
            reorder_point: Some(5),
            bin_location: Some("A1-B2".to_string()),
            last_sold_date: None,
            last_counted_date: Some(Utc::now()),
            deleted_at: None,
        }
    }

    #[test]
    fn test_inventory_item_creation() {
        let item = create_test_inventory_item();
        assert_eq!(item.quantity_on_hand, 10);
        assert!(item.specific_price.is_some());
    }

    #[test]
    fn test_inventory_item_serialization() {
        let item = create_test_inventory_item();
        let json = serde_json::to_string(&item).expect("Should serialize");
        let parsed: InventoryItem = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(item.inventory_uuid, parsed.inventory_uuid);
        assert_eq!(item.quantity_on_hand, parsed.quantity_on_hand);
    }
}

mod transaction_tests {
    use super::*;

    fn create_test_transaction_item() -> TransactionItem {
        TransactionItem {
            item_uuid: Uuid::new_v4(),
            product_uuid: Uuid::new_v4(),
            quantity: 2,
            unit_price: 25.00,
            condition: Condition::NM,
        }
    }

    #[test]
    fn test_transaction_creation() {
        let item = create_test_transaction_item();
        let transaction = Transaction {
            transaction_uuid: Uuid::new_v4(),
            items: vec![item],
            customer_uuid: Some(Uuid::new_v4()),
            timestamp: Utc::now(),
            transaction_type: TransactionType::Sale,
            user_uuid: Some(Uuid::new_v4()),
        };

        assert_eq!(transaction.items.len(), 1);
        assert!(matches!(
            transaction.transaction_type,
            TransactionType::Sale
        ));
    }

    #[test]
    fn test_transaction_types() {
        let types = vec![
            TransactionType::Sale,
            TransactionType::Buy,
            TransactionType::Trade,
            TransactionType::Return,
        ];

        for t in types {
            let json = serde_json::to_string(&t).expect("Should serialize");
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_transaction_with_multiple_items() {
        let items: Vec<TransactionItem> = (0..5).map(|_| create_test_transaction_item()).collect();

        let transaction = Transaction {
            transaction_uuid: Uuid::new_v4(),
            items: items.clone(),
            customer_uuid: None,
            timestamp: Utc::now(),
            transaction_type: TransactionType::Sale,
            user_uuid: None,
        };

        assert_eq!(transaction.items.len(), 5);
    }

    #[test]
    fn test_transaction_item_total_calculation() {
        let item = TransactionItem {
            item_uuid: Uuid::new_v4(),
            product_uuid: Uuid::new_v4(),
            quantity: 3,
            unit_price: 10.00,
            condition: Condition::NM,
        };

        let total = item.quantity as f64 * item.unit_price;
        assert!((total - 30.00).abs() < 0.01);
    }
}

mod customer_tests {
    use super::*;

    fn create_test_customer() -> Customer {
        Customer {
            customer_uuid: Uuid::new_v4(),
            name: "John Doe".to_string(),
            email: Some("john@example.com".to_string()),
            phone: Some("555-1234".to_string()),
            store_credit: 0.0,
            tier: Some("Gold".to_string()),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_customer_creation() {
        let customer = create_test_customer();
        assert_eq!(customer.name, "John Doe");
        assert_eq!(customer.store_credit, 0.0);
    }

    #[test]
    fn test_customer_with_store_credit() {
        let customer = Customer {
            store_credit: 150.50,
            tier: Some("Platinum".to_string()),
            ..create_test_customer()
        };

        assert!((customer.store_credit - 150.50).abs() < 0.01);
    }

    #[test]
    fn test_customer_serialization() {
        let customer = create_test_customer();
        let json = serde_json::to_string(&customer).expect("Should serialize");
        let parsed: Customer = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(customer.customer_uuid, parsed.customer_uuid);
        assert_eq!(customer.name, parsed.name);
        assert_eq!(customer.store_credit, parsed.store_credit);
    }
}

mod price_info_tests {
    use super::*;

    fn create_test_price_info() -> PriceInfo {
        PriceInfo {
            price_uuid: Uuid::new_v4(),
            product_uuid: Uuid::new_v4(),
            market_mid: 15.00,
            market_low: 10.00,
            last_sync_timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_price_info_creation() {
        let price = create_test_price_info();
        assert!(price.market_low <= price.market_mid);
    }

    #[test]
    fn test_price_info_serialization() {
        let price = create_test_price_info();
        let json = serde_json::to_string(&price).expect("Should serialize");
        let parsed: PriceInfo = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(price.product_uuid, parsed.product_uuid);
        assert!((price.market_mid - parsed.market_mid).abs() < 0.01);
    }
}
