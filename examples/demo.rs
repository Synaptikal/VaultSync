use uuid::Uuid;
use vaultsync::core::{Category, Condition, InventoryItem, Product, VariantType};
use vaultsync::database;
use vaultsync::inventory::InventoryService;
use vaultsync::pricing::PricingService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("VaultSync Demo Application");
    println!("==========================\n");

    // Initialize database with demo node id
    let db = database::initialize_db("demo_node".to_string()).await?;
    println!("✓ Database initialized");

    // Create services
    let inventory_service = InventoryService::new(db.inventory.clone());
    let pricing_service = PricingService::new(db.clone());
    println!("✓ Services created");

    // Create a sample product (e.g., a Pokemon card)
    let pikachu_card = Product {
        product_uuid: Uuid::new_v4(),
        name: "Pikachu".to_string(),
        category: Category::TCG,
        set_code: Some("Base Set".to_string()),
        collector_number: Some("008/102".to_string()),
        barcode: None,
        release_year: Some(1999),
        metadata: serde_json::json!({
            "rarity": "Common",
            "type": "Lightning",
            "hp": 40
        }),
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
    db.products.insert(&pikachu_card).await?;
    println!(
        "✓ Product added to catalog: {} ({:?})",
        pikachu_card.name, pikachu_card.category
    );

    // Add inventory item
    let inventory_item = InventoryItem {
        inventory_uuid: Uuid::new_v4(),
        product_uuid: pikachu_card.product_uuid,
        variant_type: Some(VariantType::Normal),
        condition: Condition::NM,
        quantity_on_hand: 3,
        location_tag: "Display Case 1".to_string(),
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

    inventory_service.add_item(inventory_item.clone()).await?;
    println!(
        "✓ Inventory item added: {} units in stock",
        inventory_item.quantity_on_hand
    );

    // Sync pricing data
    pricing_service.sync_prices().await?;
    println!("✓ Prices synced with market data");

    // Display inventory status
    let all_items = inventory_service.get_all_items().await?;
    println!("\nCurrent Inventory:");
    for item in all_items {
        println!("  - Product UUID: {}", item.product_uuid);
        println!("    Condition: {:?}", item.condition);
        println!("    Quantity: {}", item.quantity_on_hand);
        println!("    Location: {}", item.location_tag);
    }

    // Demonstrate condition update
    inventory_service
        .update_condition(inventory_item.inventory_uuid, Condition::LP)
        .await?;
    println!("\n✓ Updated item condition to {:?}", Condition::LP);

    // Show updated inventory
    let updated_item = inventory_service
        .get_item(inventory_item.inventory_uuid)
        .await
        .unwrap();
    println!("Updated condition: {:?}", updated_item.condition);

    println!("\nVaultSync demo completed successfully!");
    println!("The system is ready for real-world use with TCG shops and other collectibles.");

    Ok(())
}
