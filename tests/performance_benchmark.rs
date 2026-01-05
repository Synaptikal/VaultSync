use std::time::Instant;
use uuid::Uuid;
use vaultsync::core::{Category, Condition, InventoryItem, Product};
use vaultsync::database::initialize_test_db;

#[tokio::test]
async fn benchmark_inventory_retrieval() {
    // 1. Setup
    let db = initialize_test_db().await.expect("Failed to init DB");

    // 2. Seed Data (1000 items)
    let product_uuid = Uuid::new_v4();
    db.products
        .insert(&Product {
            product_uuid,
            name: "Benchmark Product".to_string(),
            category: Category::TCG,
            set_code: None,
            collector_number: None,
            barcode: None,
            release_year: None,
            metadata: serde_json::Value::Null,
            weight_oz: None,
            length_in: None,
            width_in: None,
            height_in: None,
            upc: None,
            isbn: None,
            manufacturer: None,
            msrp: None,
            deleted_at: None,
        })
        .await
        .expect("Failed to insert product");

    for i in 0..1000 {
        db.inventory
            .insert(&InventoryItem {
                inventory_uuid: Uuid::new_v4(),
                product_uuid,
                variant_type: None,
                condition: Condition::NM,
                quantity_on_hand: 1,
                location_tag: format!("LOC-{}", i),
                specific_price: None,
                serialized_details: None,
                deleted_at: None,
                cost_basis: None,
                supplier_uuid: None,
                received_date: None,
                min_stock_level: 0,
                max_stock_level: None,
                reorder_point: None,
                bin_location: None,
                last_sold_date: None,
                last_counted_date: None,
            })
            .await
            .expect("Failed to insert item");
    }

    // 3. Measure
    let start = Instant::now();
    let items = db
        .inventory
        .get_by_product(product_uuid)
        .await
        .expect("Failed to fetch");
    let duration = start.elapsed();

    // 4. Assert
    assert_eq!(items.len(), 1000);
    println!("Fetched 1000 items in {:?}", duration);

    // Requirement: < 200ms
    assert!(
        duration.as_millis() < 200,
        "Performance Check Failed: took {}ms",
        duration.as_millis()
    );
}
