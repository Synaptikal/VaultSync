// Integration tests for Product Repository

use uuid::Uuid;
use vaultsync::core::{Category, Product};

mod common;

#[tokio::test]
async fn test_product_insert_and_retrieve() {
    let db = common::setup_test_db().await;

    let product_uuid = Uuid::new_v4();
    let product = Product {
        product_uuid,
        name: "Test Booster Box".to_string(),
        category: Category::TCG,
        set_code: Some("SET1".to_string()),
        collector_number: None,
        barcode: Some("1234567890123".to_string()),
        release_year: Some(2024),
        metadata: serde_json::json!({"set_name": "New Set"}),
        weight_oz: Some(12.0),
        length_in: Some(5.0),
        width_in: Some(3.0),
        height_in: Some(2.0),
        upc: None,
        isbn: None,
        manufacturer: Some("Wizards".to_string()),
        msrp: Some(19.99),
        deleted_at: None,
    };

    db.products
        .insert(&product)
        .await
        .expect("Failed to insert product");

    // Retrieve by ID
    let retrieved = db
        .products
        .get_by_id(product_uuid)
        .await
        .expect("Failed to get product")
        .expect("Product not found");

    assert_eq!(retrieved.product_uuid, product_uuid);
    assert_eq!(retrieved.name, "Test Booster Box");
    assert_eq!(retrieved.category, Category::TCG);

    // Check metadata matches
    assert_eq!(retrieved.metadata["set_name"], "New Set");
}

#[tokio::test]
async fn test_product_search() {
    let db = common::setup_test_db().await;

    let p1 = common::create_test_product("Charizard Base Set", Category::TCG);
    let p2 = common::create_test_product("Blastoise Base Set", Category::TCG);
    let p3 = common::create_test_product("Magic the Gathering Deck", Category::TCG);

    db.products.insert(&p1).await.unwrap();
    db.products.insert(&p2).await.unwrap();
    db.products.insert(&p3).await.unwrap();

    // Search for "Base Set"
    let results = db
        .products
        .search("Base Set", 10, 0)
        .await
        .expect("Failed to search products");

    assert_eq!(results.len(), 2);
    let names: Vec<_> = results.iter().map(|p| p.name.clone()).collect();
    assert!(names.contains(&"Charizard Base Set".to_string()));
    assert!(names.contains(&"Blastoise Base Set".to_string()));
}

#[tokio::test]
async fn test_product_update() {
    let db = common::setup_test_db().await;

    let mut product = common::create_test_product("Original Name", Category::TCG);
    db.products.insert(&product).await.unwrap();

    // Update fields
    product.name = "Updated Name".to_string();
    product.msrp = Some(29.99);

    // Use insert to update (UPSERT)
    db.products
        .insert(&product)
        .await
        .expect("Failed to update product");

    // Verify update
    let updated = db
        .products
        .get_by_id(product.product_uuid)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.msrp, Some(29.99));
}

#[tokio::test]
async fn test_product_barcode_lookup() {
    let db = common::setup_test_db().await;

    let mut product = common::create_test_product("Barcode Test", Category::TCG);
    product.barcode = Some("123456789".to_string());

    db.products.insert(&product).await.unwrap();

    // Use search to find by barcode
    let results = db
        .products
        .search("123456789", 1, 0)
        .await
        .expect("Failed to search by barcode");

    assert!(!results.is_empty(), "Product not found by barcode search");
    let found = &results[0];

    assert_eq!(found.product_uuid, product.product_uuid);
}

#[tokio::test]
async fn test_product_pagination() {
    let db = common::setup_test_db().await;

    // Seed 15 products
    common::seed_test_products(&db, 15).await;

    // First page
    // Using search with empty query for all items
    let page1 = db.products.search("", 10, 0).await.unwrap();
    assert_eq!(page1.len(), 10);

    // Second page
    let page2 = db.products.search("", 10, 10).await.unwrap();
    assert_eq!(page2.len(), 5);
}

#[tokio::test]
async fn test_product_category_filtering() {
    let db = common::setup_test_db().await;

    let p1 = common::create_test_product("TCG Product", Category::TCG);
    let p2 = common::create_test_product("Sports Product", Category::SportsCard);

    db.products.insert(&p1).await.unwrap();
    db.products.insert(&p2).await.unwrap();

    let tcg_products = db
        .products
        .get_by_category(Category::TCG)
        .await
        .expect("Failed to get by category");

    assert_eq!(tcg_products.len(), 1);
    assert_eq!(tcg_products[0].name, "TCG Product");

    let sports_products = db
        .products
        .get_by_category(Category::SportsCard)
        .await
        .unwrap();
    assert_eq!(sports_products.len(), 1);
}
