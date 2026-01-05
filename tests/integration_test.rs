use uuid::Uuid;
use vaultsync::core::{
    Category, Condition, InventoryItem, PriceInfo, Product, TransactionItem, TransactionType,
    VariantType,
};
use vaultsync::database;
use vaultsync::inventory::InventoryService;
use vaultsync::pricing::PricingService;
use vaultsync::transactions::TransactionService;

#[tokio::test]
async fn test_inventory_management() {
    // Initialize database
    let db = database::initialize_test_db().await.unwrap();

    // Create inventory service
    let inventory_service = InventoryService::new(db.inventory.clone());

    // Create a sample product (TCG Card)
    let product = Product {
        product_uuid: Uuid::new_v4(),
        name: "Bulbasaur".to_string(),
        category: Category::TCG,
        set_code: Some("151".to_string()),
        collector_number: Some("001/151".to_string()),
        barcode: None,
        release_year: Some(2023),
        metadata: serde_json::json!({"game": "Pokemon", "rarity": "Common"}),
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

    // Insert product into database
    db.products.insert(&product).await.unwrap();

    // Create an inventory item
    let inventory_item = InventoryItem {
        inventory_uuid: Uuid::new_v4(),
        product_uuid: product.product_uuid,
        variant_type: Some(VariantType::Normal),
        condition: Condition::NM,
        quantity_on_hand: 5,
        location_tag: "Display Case".to_string(),
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

    // Add item to inventory
    inventory_service
        .add_item(inventory_item.clone())
        .await
        .unwrap();

    // Verify the item was added
    let items = inventory_service.get_all_items().await.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].quantity_on_hand, 5);

    // Test removing an item
    inventory_service
        .remove_item(inventory_item.inventory_uuid, 2)
        .await
        .unwrap();

    let updated_items = inventory_service.get_all_items().await.unwrap();
    assert_eq!(updated_items[0].quantity_on_hand, 3);
}

#[tokio::test]
async fn test_inventory_management_multi_category() {
    // Initialize database
    let db = database::initialize_test_db().await.unwrap();
    let inventory_service = InventoryService::new(db.inventory.clone());

    // 1. Create a Sports Card
    let sports_card = Product {
        product_uuid: Uuid::new_v4(),
        name: "Michael Jordan Rookie".to_string(),
        category: Category::SportsCard,
        set_code: Some("Fleer".to_string()),
        collector_number: Some("57".to_string()),
        barcode: None,
        release_year: Some(1986),
        metadata: serde_json::json!({"sport": "Basketball", "team": "Chicago Bulls"}),
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
    db.products.insert(&sports_card).await.unwrap();

    let sports_item = InventoryItem {
        inventory_uuid: Uuid::new_v4(),
        product_uuid: sports_card.product_uuid,
        variant_type: None,
        condition: Condition::NearMintMint, // PSA 8 equivalent maybe?
        quantity_on_hand: 1,
        location_tag: "High Value Case".to_string(),
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
    inventory_service.add_item(sports_item).await.unwrap();

    // 2. Create a Comic Book
    let comic = Product {
        product_uuid: Uuid::new_v4(),
        name: "Amazing Fantasy #15".to_string(),
        category: Category::Comic,
        set_code: None,
        collector_number: None,
        barcode: None,
        release_year: Some(1962),
        metadata: serde_json::json!({"publisher": "Marvel", "hero": "Spider-Man"}),
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
    db.products.insert(&comic).await.unwrap();

    let comic_item = InventoryItem {
        inventory_uuid: Uuid::new_v4(),
        product_uuid: comic.product_uuid,
        variant_type: None,
        condition: Condition::Good, // Vintage condition
        quantity_on_hand: 1,
        location_tag: "Comic Bin 1".to_string(),
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
    inventory_service.add_item(comic_item).await.unwrap();

    // 3. Create Merch
    let merch = Product {
        product_uuid: Uuid::new_v4(),
        name: "VaultSync T-Shirt".to_string(),
        category: Category::Apparel,
        set_code: None,
        collector_number: None,
        barcode: Some("123456789012".to_string()),
        release_year: Some(2024),
        metadata: serde_json::json!({"size": "L", "color": "Black"}),
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
    db.products.insert(&merch).await.unwrap();

    let merch_item = InventoryItem {
        inventory_uuid: Uuid::new_v4(),
        product_uuid: merch.product_uuid,
        variant_type: None,
        condition: Condition::New,
        quantity_on_hand: 50,
        location_tag: "Apparel Rack".to_string(),
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
    inventory_service.add_item(merch_item).await.unwrap();

    // Verify all items
    let all_items = inventory_service.get_all_items().await.unwrap();
    assert_eq!(all_items.len(), 3);
}

#[tokio::test]
async fn test_pricing_service() {
    // Initialize database
    let db = database::initialize_test_db().await.unwrap();

    // Create pricing service
    let pricing_service = PricingService::new(db.clone());

    // Sync prices
    pricing_service.sync_prices().await.unwrap();

    // Verify that the sync time is set
    assert!(pricing_service.get_last_sync_time().await.is_some());

    // Verify that the price cache is fresh
    assert!(pricing_service.is_price_cache_fresh().await);

    // Test price safety status
    let status = pricing_service.calculate_safety_status(10.0, 11.0); // 10% increase
    assert!(matches!(status, vaultsync::core::PriceStatus::Safe));

    let status = pricing_service.calculate_safety_status(10.0, 12.0); // 20% increase
    assert!(matches!(status, vaultsync::core::PriceStatus::Flagged));
}

#[tokio::test]
async fn test_transaction_processing() {
    // Initialize database
    let db = database::initialize_test_db().await.unwrap();

    // Create services
    let inventory_service = std::sync::Arc::new(InventoryService::new(db.inventory.clone()));
    let pricing_service = std::sync::Arc::new(PricingService::new(db.clone()));
    let transaction_service = TransactionService::new(
        db.clone(),
        inventory_service.clone(),
        pricing_service.clone(),
    );

    // Create a sample product and add to inventory
    let product = Product {
        product_uuid: Uuid::new_v4(),
        name: "Bulbasaur".to_string(),
        category: Category::TCG,
        set_code: Some("Base Set".to_string()),
        collector_number: Some("001/102".to_string()),
        barcode: None,
        release_year: Some(1999),
        metadata: serde_json::json!({"rarity": "Common"}),
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

    db.products.insert(&product).await.unwrap();

    // Add inventory item
    let inventory_item = InventoryItem {
        inventory_uuid: Uuid::new_v4(),
        product_uuid: product.product_uuid,
        variant_type: Some(VariantType::Normal),
        condition: Condition::NM,
        quantity_on_hand: 10,
        location_tag: "Display Case".to_string(),
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

    inventory_service.add_item(inventory_item).await.unwrap();

    // Process a sale
    let transaction_item = TransactionItem {
        item_uuid: Uuid::new_v4(),
        product_uuid: product.product_uuid,
        quantity: 2,
        unit_price: 5.0,
        condition: Condition::NM,
    };

    let transaction = transaction_service
        .process_sale(None, None, vec![transaction_item])
        .await
        .unwrap();

    // Verify transaction properties
    assert_eq!(transaction.transaction_type, TransactionType::Sale);
    assert_eq!(transaction.items.len(), 1);

    // Check that inventory was reduced
    let items = inventory_service
        .get_items_by_product(product.product_uuid)
        .await
        .unwrap();
    let remaining_item = items
        .iter()
        .find(|item| item.condition == Condition::NM)
        .unwrap();
    assert_eq!(remaining_item.quantity_on_hand, 8); // Started with 10, sold 2
}

#[tokio::test]
async fn test_buylist_functionality() {
    use vaultsync::buylist::{BuylistItem, BuylistService, PaymentMethod};

    // Initialize database
    let db = database::initialize_test_db().await.unwrap();

    // Create services
    let pricing_service_raw = PricingService::new(db.clone());
    let pricing_service = std::sync::Arc::new(pricing_service_raw);
    let rule_engine = std::sync::Arc::new(vaultsync::pricing::RuleEngine::new());

    let inventory_service = std::sync::Arc::new(InventoryService::new(db.inventory.clone()));
    let buylist_service = BuylistService::new(
        db.clone(),
        pricing_service.clone(),
        rule_engine,
        inventory_service,
    );

    // Create a sample product
    let product = Product {
        product_uuid: Uuid::new_v4(),
        name: "Charizard".to_string(),
        category: Category::TCG,
        set_code: Some("Base Set".to_string()),
        collector_number: Some("4/102".to_string()),
        barcode: None,
        release_year: Some(1999),
        metadata: serde_json::json!({"rarity": "Holo Rare"}),
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

    db.products.insert(&product).await.unwrap();

    let price_info = PriceInfo {
        price_uuid: Uuid::new_v4(),
        product_uuid: product.product_uuid,
        market_mid: 300.0,
        market_low: 250.0,
        last_sync_timestamp: chrono::Utc::now(),
    };
    db.pricing.insert_matrix(&price_info).await.unwrap();

    // Create a buylist item
    let buylist_item = BuylistItem {
        product_uuid: product.product_uuid,
        condition: Condition::NM,
        quantity: 3,
    };

    // Process a buylist transaction
    let transaction = buylist_service
        .process_buylist_transaction(None, vec![buylist_item], PaymentMethod::Cash)
        .await
        .unwrap();

    // Verify transaction properties
    assert_eq!(transaction.transaction_type, TransactionType::Buy);
    assert_eq!(transaction.items.len(), 1);
}

#[tokio::test]
async fn test_error_handling() {
    // Initialize database
    let db = database::initialize_test_db().await.unwrap();

    // Create inventory service
    let inventory_service = InventoryService::new(db.inventory.clone());

    // Try to remove more items than available to trigger an error
    let fake_uuid = Uuid::new_v4();
    let result = inventory_service.remove_item(fake_uuid, 1).await;

    // Should not panic, should return an error
    assert!(result.is_err());
}

#[test]
fn test_price_info_creation() {
    let price_info = PriceInfo {
        price_uuid: Uuid::new_v4(),
        product_uuid: Uuid::new_v4(),
        market_mid: 10.50,
        market_low: 8.25,
        last_sync_timestamp: chrono::Utc::now(),
    };

    assert_eq!(price_info.market_mid, 10.50);
    assert!(price_info.market_low < price_info.market_mid);
}

#[tokio::test]
async fn test_auth_flow() {
    use vaultsync::auth::{create_jwt, hash_password, verify_jwt, verify_password, UserRole};

    // Set JWT_SECRET for test
    std::env::set_var("JWT_SECRET", "test_secret_key_for_integration_tests_12345");

    // Initialize database
    let db = database::initialize_test_db().await.unwrap();

    // 1. Test Password Hashing
    let password = "my_secure_password";
    let hash = hash_password(password).unwrap();
    assert_ne!(password, hash);
    assert!(verify_password(password, &hash).unwrap());
    assert!(!verify_password("wrong_password", &hash).unwrap());

    // 2. Test User Creation in DB
    let user_uuid = Uuid::new_v4();
    let username = "admin";
    let role = UserRole::Admin;

    db.auth
        .insert_user(user_uuid, username, &hash, role.clone())
        .await
        .unwrap();

    let retrieved_user = db.auth.get_user_by_username(username).await.unwrap();
    assert!(retrieved_user.is_some());
    let (uuid, u_name, p_hash, u_role) = retrieved_user.unwrap();
    assert_eq!(uuid, user_uuid);
    assert_eq!(u_name, username);
    assert_eq!(p_hash, hash);
    assert_eq!(u_role, role);

    // 3. Test JWT Generation and Verification
    let token = create_jwt(user_uuid, username, role.clone()).unwrap();
    let claims = verify_jwt(&token).unwrap();
    assert_eq!(claims.sub, user_uuid.to_string());
    assert_eq!(claims.username, username);
    assert_eq!(claims.role, role);
}
