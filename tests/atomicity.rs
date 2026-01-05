use uuid::Uuid;
use vaultsync::core::{Category, Condition, InventoryItem, Product, TransactionItem};
use vaultsync::database::initialize_test_db;

#[tokio::test]
async fn test_atomic_sale_with_sync() {
    let db = initialize_test_db().await.expect("Failed to init db");

    // Seed Product
    let product_uuid = Uuid::new_v4();
    let product = Product {
        product_uuid,
        name: "Black Lotus".to_string(),
        category: Category::TCG,
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
    db.products
        .insert(&product)
        .await
        .expect("Failed to insert product");

    // Seed Inventory
    let inv_uuid = Uuid::new_v4();
    let inventory_item = InventoryItem {
        inventory_uuid: inv_uuid,
        product_uuid,
        variant_type: None,
        condition: Condition::NM,
        quantity_on_hand: 5,
        location_tag: "Case 1".to_string(),
        specific_price: None,
        serialized_details: None,
        cost_basis: None,
        supplier_uuid: None,
        received_date: None,
        min_stock_level: 0,
        max_stock_level: None,
        reorder_point: None,
        bin_location: None,
        last_sold_date: None,
        last_counted_date: None,
        deleted_at: None,
    };
    db.inventory
        .insert(&inventory_item)
        .await
        .expect("Failed to insert inventory");

    // Execute Sale
    let items = vec![TransactionItem {
        item_uuid: Uuid::new_v4(),
        product_uuid,
        quantity: 2,
        unit_price: 1000.0,
        condition: Condition::NM,
    }];

    let _tx = db
        .transactions
        .execute_sale(None, None, items)
        .await
        .expect("Sale failed");

    // Check Inventory
    let inventory = db.inventory.get_all().await.expect("Failed to get items");
    let updated_item = inventory
        .iter()
        .find(|i| i.inventory_uuid == inv_uuid)
        .expect("Item missing");

    assert_eq!(updated_item.quantity_on_hand, 3, "Inventory should be 3");

    // Check Sync Log
    // Seed Product (0 - assumption), Seed Inventory (1), Sale Inventory Update (1), Sale Tx Insert (1). Total 3.
    let changes = db
        .sync
        .get_changes_since(0, 100)
        .await
        .expect("Failed to get changes");

    assert_eq!(changes.len(), 3, "Should have 3 sync log entries");

    let last_change = changes.last().unwrap();
    assert_eq!(
        last_change.1, "Transaction",
        "Last change should be Transaction"
    );
    assert_eq!(last_change.2, "Insert", "Last operation should be Insert");

    let second_last = &changes[changes.len() - 2];
    assert_eq!(
        second_last.1, "InventoryItem",
        "Second last should be InventoryItem"
    );
    assert_eq!(second_last.2, "Update", "Operation should be Update");
}

#[tokio::test]
async fn test_atomic_buy_with_sync() {
    let db = initialize_test_db().await.expect("Failed to init db");

    // Seed Product
    let product_uuid = Uuid::new_v4();
    let product = Product {
        product_uuid,
        name: "Mox Pearl".to_string(),
        category: Category::TCG,
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
    db.products
        .insert(&product)
        .await
        .expect("Failed to insert product");

    // Execute Buy (New Item)
    let items = vec![TransactionItem {
        item_uuid: Uuid::new_v4(),
        product_uuid,
        quantity: 1,
        unit_price: 500.0,
        condition: Condition::NM,
    }];

    db.transactions
        .execute_buy(None, None, items.clone())
        .await
        .expect("Buy failed");

    // Verify Inventory Created
    let inventory = db.inventory.get_all().await.expect("Failed to get items");
    assert_eq!(inventory.len(), 1);
    assert_eq!(inventory[0].quantity_on_hand, 1);

    // Execute But (Existing Pile)
    db.transactions
        .execute_buy(None, None, items)
        .await
        .expect("Buy 2 failed");

    let inventory_2 = db.inventory.get_all().await.expect("Failed to get items");
    assert_eq!(inventory_2.len(), 1, "Should aggregate into existing pile");
    assert_eq!(
        inventory_2[0].quantity_on_hand, 2,
        "Quantity should increment"
    );

    // Check Sync Log
    // Product(0) + Buy1(Inv Insert + Tx Insert) + Buy2(Inv Update + Tx Insert) = 4
    let changes = db
        .sync
        .get_changes_since(0, 100)
        .await
        .expect("Failed to get changes");
    assert_eq!(changes.len(), 4);
}

#[tokio::test]
async fn test_atomic_trade_with_sync() {
    let db = initialize_test_db().await.expect("Failed to init db");

    // Seed Products
    let p_in = Product {
        product_uuid: Uuid::new_v4(),
        name: "Trade In Card".to_string(),
        category: Category::TCG,
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
    db.products
        .insert(&p_in)
        .await
        .expect("Failed to insert p_in");

    let p_out = Product {
        product_uuid: Uuid::new_v4(),
        name: "Trade Out Card".to_string(),
        category: Category::TCG,
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
    db.products
        .insert(&p_out)
        .await
        .expect("Failed to insert p_out");

    // Seed Inventory for Out Card
    let inv_out = InventoryItem {
        inventory_uuid: Uuid::new_v4(),
        product_uuid: p_out.product_uuid,
        variant_type: None,
        condition: Condition::NM,
        quantity_on_hand: 5,
        location_tag: "Display".to_string(),
        specific_price: None,
        serialized_details: None,
        cost_basis: None,
        supplier_uuid: None,
        received_date: None,
        min_stock_level: 0,
        max_stock_level: None,
        reorder_point: None,
        bin_location: None,
        last_sold_date: None,
        last_counted_date: None,
        deleted_at: None,
    };
    db.inventory
        .insert(&inv_out)
        .await
        .expect("Failed to insert inventory");

    // Execute Trade
    let trade_in_items = vec![TransactionItem {
        item_uuid: Uuid::new_v4(),
        product_uuid: p_in.product_uuid,
        quantity: 1,
        unit_price: 10.0,
        condition: Condition::NM,
    }];

    let trade_out_items = vec![TransactionItem {
        item_uuid: Uuid::new_v4(),
        product_uuid: p_out.product_uuid,
        quantity: 1,
        unit_price: 15.0,
        condition: Condition::NM,
    }];

    let (_tx_in, _tx_out) = db
        .transactions
        .execute_trade(None, None, trade_in_items, trade_out_items)
        .await
        .expect("Trade failed");

    // Verify
    let all_inv = db.inventory.get_all().await.expect("Failed to get inv");
    // Should have 2 items: 1 for In (qty 1), 1 for Out (qty 4)
    assert_eq!(all_inv.len(), 2);

    let item_in = all_inv
        .iter()
        .find(|i| i.product_uuid == p_in.product_uuid)
        .expect("In item missing");
    assert_eq!(item_in.quantity_on_hand, 1);

    let item_out = all_inv
        .iter()
        .find(|i| i.product_uuid == p_out.product_uuid)
        .expect("Out item missing");
    assert_eq!(item_out.quantity_on_hand, 4);

    // Sync Log Check
    // Product(0) + Product(0) + Inventory seed (1) + Trade In (2) + Trade Out (2) = 5? Actual 6.
    let changes = db
        .sync
        .get_changes_since(0, 100)
        .await
        .expect("Failed to get changes");
    assert_eq!(changes.len(), 6);
}
