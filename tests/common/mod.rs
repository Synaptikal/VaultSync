// Common test utilities for integration tests
#![allow(dead_code)]

use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
use vaultsync::core::*;
use vaultsync::database::Database;

/// Create an in-memory test database with all tables initialized
pub async fn setup_test_db() -> Arc<Database> {
    let db = Database::new("sqlite::memory:", format!("test-node-{}", Uuid::new_v4()))
        .await
        .expect("Failed to create test DB");

    db.initialize_tables()
        .await
        .expect("Failed to initialize tables");

    Arc::new(db)
}

/// Create a test product with default values
pub fn create_test_product(name: &str, category: Category) -> Product {
    Product {
        product_uuid: Uuid::new_v4(),
        name: name.to_string(),
        category,
        set_code: Some("TEST-001".to_string()),
        collector_number: Some("1".to_string()),
        barcode: Some("123456789".to_string()),
        release_year: Some(2024),
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
    }
}

/// Create a test inventory item
pub fn create_test_inventory_item(product_uuid: Uuid, quantity: i32) -> InventoryItem {
    InventoryItem {
        inventory_uuid: Uuid::new_v4(),
        product_uuid,
        variant_type: None,
        condition: Condition::NM,
        quantity_on_hand: quantity,
        location_tag: "MAIN".to_string(),
        specific_price: None,
        serialized_details: None,
        cost_basis: Some(10.0),
        supplier_uuid: None,
        received_date: Some(Utc::now()),
        min_stock_level: 0,
        max_stock_level: None,
        reorder_point: None,
        bin_location: Some("A1".to_string()),
        last_sold_date: None,
        last_counted_date: None,
        deleted_at: None,
    }
}

/// Create a test customer
pub fn create_test_customer(name: &str) -> Customer {
    Customer {
        customer_uuid: Uuid::new_v4(),
        name: name.to_string(),
        email: Some(format!("{}@test.com", name.to_lowercase())),
        phone: Some("555-0100".to_string()),
        store_credit: 0.0,
        tier: Some("Standard".to_string()),
        created_at: Utc::now(),
    }
}

/// Create a test transaction item
pub fn create_test_transaction_item(
    product_uuid: Uuid,
    quantity: i32,
    price: f64,
) -> TransactionItem {
    TransactionItem {
        item_uuid: Uuid::new_v4(),
        product_uuid,
        quantity,
        unit_price: price,
        condition: Condition::NM,
    }
}

/// Create a test user
pub fn create_test_user(username: &str, role: vaultsync::auth::UserRole) -> vaultsync::auth::User {
    vaultsync::auth::User {
        user_uuid: Uuid::new_v4(),
        username: username.to_string(),
        role,
    }
}

/// Insert test products into database
pub async fn seed_test_products(db: &Database, count: usize) -> Vec<Product> {
    let mut products = Vec::new();

    for i in 0..count {
        let product = create_test_product(&format!("Test Card {}", i), Category::TCG);

        db.products
            .insert(&product)
            .await
            .expect("Failed to insert product");
        products.push(product);
    }

    products
}

/// Insert test inventory into database
pub async fn seed_test_inventory(
    db: &Database,
    products: &[Product],
    quantity_each: i32,
) -> Vec<InventoryItem> {
    let mut items = Vec::new();

    for product in products {
        let item = create_test_inventory_item(product.product_uuid, quantity_each);
        db.inventory
            .insert(&item)
            .await
            .expect("Failed to insert inventory");
        items.push(item);
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_test_db() {
        let db = setup_test_db().await;

        // Verify tables exist by querying
        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
            .fetch_all(&db.pool)
            .await;

        assert!(result.is_ok());
        let tables = result.unwrap();
        assert!(tables.len() > 10); // Should have many tables
    }

    #[test]
    fn test_create_test_product() {
        let product = create_test_product("Test", Category::TCG);
        assert_eq!(product.name, "Test");
        assert_eq!(product.category, Category::TCG);
    }

    #[test]
    fn test_create_test_customer() {
        let customer = create_test_customer("John Doe");
        assert_eq!(customer.name, "John Doe");
        assert_eq!(customer.email, Some("john doe@test.com".to_string()));
    }
}
