// Integration tests for Inventory Repository

use vaultsync::core::{Category, Condition};

mod common;

#[tokio::test]
async fn test_inventory_insert_and_retrieve() {
    let db = common::setup_test_db().await;

    // Create and insert product first
    let product = common::create_test_product("Test Card", Category::TCG);
    db.products
        .insert(&product)
        .await
        .expect("Failed to insert product");

    // Create and insert inventory
    let item = common::create_test_inventory_item(product.product_uuid, 10);
    db.inventory
        .insert(&item)
        .await
        .expect("Failed to insert inventory");

    // Retrieve and verify
    let retrieved = db
        .inventory
        .get_by_id(item.inventory_uuid)
        .await
        .expect("Failed to get inventory")
        .expect("Inventory not found");

    assert_eq!(retrieved.inventory_uuid, item.inventory_uuid);
    assert_eq!(retrieved.product_uuid, item.product_uuid);
    assert_eq!(retrieved.quantity_on_hand, 10);
}

#[tokio::test]
async fn test_inventory_update_quantity() {
    let db = common::setup_test_db().await;

    let product = common::create_test_product("Test Card", Category::TCG);
    db.products.insert(&product).await.unwrap();

    let item = common::create_test_inventory_item(product.product_uuid, 10);
    db.inventory.insert(&item).await.unwrap();

    // Update quantity (delta)
    db.inventory
        .update_quantity(item.inventory_uuid, -3)
        .await
        .expect("Failed to update quantity");

    // Verify
    let updated = db
        .inventory
        .get_by_id(item.inventory_uuid)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.quantity_on_hand, 7);
}

#[tokio::test]
async fn test_inventory_get_by_product() {
    let db = common::setup_test_db().await;

    let product = common::create_test_product("Multi-Variant Card", Category::TCG);
    db.products.insert(&product).await.unwrap();

    // Create multiple inventory items for same product (different conditions)
    let mut items = vec![];
    for condition in &[Condition::NM, Condition::LP, Condition::MP] {
        let mut item = common::create_test_inventory_item(product.product_uuid, 5);
        item.condition = condition.clone();
        db.inventory.insert(&item).await.unwrap();
        items.push(item);
    }

    // Get all items for this product
    let retrieved = db
        .inventory
        .get_by_product(product.product_uuid)
        .await
        .expect("Failed to get by product");

    assert_eq!(retrieved.len(), 3);
}

#[tokio::test]
async fn test_inventory_soft_delete() {
    let db = common::setup_test_db().await;

    let product = common::create_test_product("Delete Test", Category::TCG);
    db.products.insert(&product).await.unwrap();

    let item = common::create_test_inventory_item(product.product_uuid, 10);
    db.inventory.insert(&item).await.unwrap();

    // Soft delete
    db.inventory
        .soft_delete(item.inventory_uuid)
        .await
        .expect("Failed to soft delete");

    // Should not appear in normal queries
    let paginated = db.inventory.get_paginated(100, 0).await.unwrap();

    assert_eq!(
        paginated
            .iter()
            .filter(|i| i.inventory_uuid == item.inventory_uuid)
            .count(),
        0
    );

    // But can still retrieve by ID
    let deleted_item = db
        .inventory
        .get_by_id(item.inventory_uuid)
        .await
        .unwrap()
        .unwrap();

    assert!(deleted_item.deleted_at.is_some());
}

#[tokio::test]
async fn test_inventory_restore_after_soft_delete() {
    let db = common::setup_test_db().await;

    let product = common::create_test_product("Restore Test", Category::TCG);
    db.products.insert(&product).await.unwrap();

    let item = common::create_test_inventory_item(product.product_uuid, 10);
    db.inventory.insert(&item).await.unwrap();

    // Soft delete
    db.inventory.soft_delete(item.inventory_uuid).await.unwrap();

    // Restore
    db.inventory
        .restore(item.inventory_uuid)
        .await
        .expect("Failed to restore");

    // Should appear in queries again
    let restored = db
        .inventory
        .get_by_id(item.inventory_uuid)
        .await
        .unwrap()
        .unwrap();

    assert!(restored.deleted_at.is_none());
}

#[tokio::test]
async fn test_inventory_get_low_stock() {
    let db = common::setup_test_db().await;

    let products = common::seed_test_products(&db, 5).await;

    // Create inventory with varying quantities
    for (i, product) in products.iter().enumerate() {
        let item = common::create_test_inventory_item(product.product_uuid, (i + 1) as i32);
        db.inventory.insert(&item).await.unwrap();
    }

    // Get items with quantity <= 3
    let low_stock = db
        .inventory
        .get_low_stock(3)
        .await
        .expect("Failed to get low stock");

    assert_eq!(low_stock.len(), 3); // Items with qty 1, 2, 3
}

#[tokio::test]
async fn test_inventory_pagination() {
    let db = common::setup_test_db().await;

    let products = common::seed_test_products(&db, 10).await;
    common::seed_test_inventory(&db, &products, 5).await;

    // Test pagination
    let page1 = db.inventory.get_paginated(5, 0).await.unwrap();
    assert_eq!(page1.len(), 5);

    let page2 = db.inventory.get_paginated(5, 5).await.unwrap();
    assert_eq!(page2.len(), 5);

    // Verify no overlap
    let page1_uuids: Vec<_> = page1.iter().map(|i| i.inventory_uuid).collect();
    let page2_uuids: Vec<_> = page2.iter().map(|i| i.inventory_uuid).collect();

    for uuid in &page1_uuids {
        assert!(!page2_uuids.contains(uuid));
    }
}

#[tokio::test]
async fn test_inventory_cleanup_zero_quantity() {
    let db = common::setup_test_db().await;

    let products = common::seed_test_products(&db, 3).await;

    // Create items with zero quantity
    for product in &products[0..2] {
        let item = common::create_test_inventory_item(product.product_uuid, 0);
        db.inventory.insert(&item).await.unwrap();
    }

    // Create item with positive quantity
    let item = common::create_test_inventory_item(products[2].product_uuid, 5);
    db.inventory.insert(&item).await.unwrap();

    // Cleanup zero quantity items
    let deleted_count = db
        .inventory
        .cleanup_zero_quantity()
        .await
        .expect("Failed to cleanup");

    assert_eq!(deleted_count, 2);

    // Verify only non-zero item remains
    let remaining = db.inventory.get_paginated(100, 0).await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].quantity_on_hand, 5);
}

/// CRITICAL TEST: Ensure invalid UUIDs return errors instead of nil UUIDs
#[tokio::test]
async fn test_inventory_rejects_invalid_uuid() {
    let db = common::setup_test_db().await;

    // Manually insert a row with invalid UUID (this would corrupt data in old version)
    let result = sqlx::query(
        "INSERT INTO Local_Inventory 
         (inventory_uuid, product_uuid, condition, quantity_on_hand, location_tag) 
         VALUES ('invalid-uuid', 'also-invalid', 'NM', 10, 'TEST')",
    )
    .execute(&db.pool)
    .await;

    // SQLite might accept it, but our map_row should reject it
    if result.is_ok() {
        let items_result = db.inventory.get_paginated(10, 0).await;

        // Should get error when trying to parse invalid UUID
        assert!(
            items_result.is_err(),
            "Should reject invalid UUID instead of silently corrupting to nil UUID"
        );
    }
}

#[tokio::test]
async fn test_inventory_search_by_name() {
    let db = common::setup_test_db().await;

    let product1 = common::create_test_product("Charizard VMAX", Category::TCG);
    let product2 = common::create_test_product("Pikachu V", Category::TCG);
    let product3 = common::create_test_product("Mewtwo GX", Category::TCG);

    db.products.insert(&product1).await.unwrap();
    db.products.insert(&product2).await.unwrap();
    db.products.insert(&product3).await.unwrap();

    db.inventory
        .insert(&common::create_test_inventory_item(
            product1.product_uuid,
            5,
        ))
        .await
        .unwrap();
    db.inventory
        .insert(&common::create_test_inventory_item(
            product2.product_uuid,
            3,
        ))
        .await
        .unwrap();
    db.inventory
        .insert(&common::create_test_inventory_item(
            product3.product_uuid,
            7,
        ))
        .await
        .unwrap();

    // Search by partial name
    let results = db
        .inventory
        .search_by_name("Char", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].product_uuid, product1.product_uuid);
}

#[tokio::test]
async fn test_inventory_counts() {
    let db = common::setup_test_db().await;

    let products = common::seed_test_products(&db, 5).await;
    common::seed_test_inventory(&db, &products, 10).await;

    let total_count = db.inventory.get_total_count().await.unwrap();
    assert_eq!(total_count, 5);

    let total_quantity = db.inventory.get_total_quantity().await.unwrap();
    assert_eq!(total_quantity, 50); // 5 items × 10 each
}
