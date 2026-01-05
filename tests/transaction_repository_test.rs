// Integration tests for Transaction Repository

use chrono::Utc;
use uuid::Uuid;
use vaultsync::core::TransactionType;

mod common;

#[tokio::test]
async fn test_transaction_insert_and_retrieve() {
    let db = common::setup_test_db().await;

    // Create product
    let product = common::create_test_product("Sold Item", vaultsync::core::Category::TCG);
    db.products.insert(&product).await.unwrap();

    // Create transaction items
    let item1 = common::create_test_transaction_item(product.product_uuid, 2, 10.0);

    let transaction_uuid = Uuid::new_v4();
    let items = vec![item1];

    let transaction = vaultsync::core::Transaction {
        transaction_uuid,
        items: items.clone(),
        customer_uuid: None,
        user_uuid: None,
        timestamp: Utc::now(),
        transaction_type: TransactionType::Sale,
    };

    db.transactions
        .insert(&transaction)
        .await
        .expect("Failed to insert transaction");

    // Retrieve
    let retrieved = db
        .transactions
        .get_by_id(transaction_uuid)
        .await
        .expect("Failed to get transaction")
        .expect("Transaction not found");

    assert_eq!(retrieved.transaction_uuid, transaction_uuid);
    assert_eq!(retrieved.items.len(), 1);
    assert_eq!(retrieved.items[0].product_uuid, product.product_uuid);
    assert_eq!(retrieved.items[0].quantity, 2);
    assert_eq!(retrieved.items[0].unit_price, 10.0);
}

#[tokio::test]
async fn test_get_recent_transactions() {
    let db = common::setup_test_db().await;

    let product = common::create_test_product("Item", vaultsync::core::Category::TCG);
    db.products.insert(&product).await.unwrap();

    // Insert 3 transactions
    for i in 0..3 {
        let item = common::create_test_transaction_item(product.product_uuid, 1, 5.0);
        let transaction = vaultsync::core::Transaction {
            transaction_uuid: Uuid::new_v4(),
            items: vec![item],
            customer_uuid: None,
            user_uuid: None,
            timestamp: Utc::now() + chrono::Duration::seconds(i),
            transaction_type: TransactionType::Sale,
        };
        db.transactions.insert(&transaction).await.unwrap();
    }

    let recent = db
        .transactions
        .get_recent(10)
        .await
        .expect("Failed to get recent");
    assert_eq!(recent.len(), 3);
}

#[tokio::test]
async fn test_dashboard_metrics() {
    let db = common::setup_test_db().await;

    let product = common::create_test_product("Metrics Item", vaultsync::core::Category::TCG);
    db.products.insert(&product).await.unwrap();

    // Transaction 1: $100 total (2 * 50)
    let item1 = common::create_test_transaction_item(product.product_uuid, 2, 50.0);
    let t1 = vaultsync::core::Transaction {
        transaction_uuid: Uuid::new_v4(),
        items: vec![item1],
        customer_uuid: None,
        user_uuid: None,
        timestamp: Utc::now(),
        transaction_type: TransactionType::Sale,
    };
    db.transactions.insert(&t1).await.unwrap();

    // Transaction 2: $30 total (1 * 30)
    let item2 = common::create_test_transaction_item(product.product_uuid, 1, 30.0);
    let t2 = vaultsync::core::Transaction {
        transaction_uuid: Uuid::new_v4(),
        items: vec![item2],
        customer_uuid: None,
        user_uuid: None,
        timestamp: Utc::now(),
        transaction_type: TransactionType::Sale,
    };
    db.transactions.insert(&t2).await.unwrap();

    let metrics = db
        .transactions
        .get_dashboard_metrics()
        .await
        .expect("Failed to get metrics");

    assert_eq!(metrics.transaction_count_today, 2);
    assert_eq!(metrics.total_sales_today, 130.0);
    assert_eq!(metrics.average_transaction_value, 65.0);
}

#[tokio::test]
async fn test_get_by_customer() {
    let db = common::setup_test_db().await;

    let customer = common::create_test_customer("Buyer");
    db.customers.insert(&customer).await.unwrap();

    let product = common::create_test_product("Product", vaultsync::core::Category::TCG);
    db.products.insert(&product).await.unwrap();

    let item = common::create_test_transaction_item(product.product_uuid, 1, 10.0);
    let t1 = vaultsync::core::Transaction {
        transaction_uuid: Uuid::new_v4(),
        items: vec![item],
        customer_uuid: Some(customer.customer_uuid),
        user_uuid: None,
        timestamp: Utc::now(),
        transaction_type: TransactionType::Sale,
    };
    db.transactions.insert(&t1).await.unwrap();

    let customer_txs = db
        .transactions
        .get_by_customer(customer.customer_uuid)
        .await
        .expect("Failed to get by customer");

    assert_eq!(customer_txs.len(), 1);
    assert_eq!(customer_txs[0].transaction_uuid, t1.transaction_uuid);
}

#[tokio::test]
async fn test_transaction_items_persistence() {
    let db = common::setup_test_db().await;

    let product = common::create_test_product("Persist", vaultsync::core::Category::TCG);
    db.products.insert(&product).await.unwrap();

    let item1 = common::create_test_transaction_item(product.product_uuid, 5, 20.0);
    let item2 = common::create_test_transaction_item(product.product_uuid, 3, 10.0);

    let t1 = vaultsync::core::Transaction {
        transaction_uuid: Uuid::new_v4(),
        // transaction_uuid inside items is ignored/overwritten by repository during insert usually?
        // Actually repository `insert` binds `item_uuid`, `transaction_uuid`, etc.
        // It uses `transaction.transaction_uuid` for the FK.
        items: vec![item1, item2],
        customer_uuid: None,
        user_uuid: None,
        timestamp: Utc::now(),
        transaction_type: TransactionType::Sale,
    };
    db.transactions.insert(&t1).await.unwrap();

    let retrieved = db
        .transactions
        .get_by_id(t1.transaction_uuid)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(retrieved.items.len(), 2);

    let total_qty: i32 = retrieved.items.iter().map(|i| i.quantity).sum();
    assert_eq!(total_qty, 8);
}
