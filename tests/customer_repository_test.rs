// Integration tests for Customer Repository

mod common;

#[tokio::test]
async fn test_customer_insert_and_retrieve() {
    let db = common::setup_test_db().await;

    let customer = common::create_test_customer("Jane Doe");
    db.customers
        .insert(&customer)
        .await
        .expect("Failed to insert customer");

    let retrieved = db
        .customers
        .get_by_id(customer.customer_uuid)
        .await
        .expect("Failed to get customer")
        .expect("Customer not found");

    assert_eq!(retrieved.customer_uuid, customer.customer_uuid);
    assert_eq!(retrieved.name, "Jane Doe");
    assert_eq!(retrieved.email, Some("jane doe@test.com".to_string()));
    assert_eq!(retrieved.store_credit, 0.0);
}

#[tokio::test]
async fn test_customer_get_all() {
    let db = common::setup_test_db().await;

    let c1 = common::create_test_customer("Alice Smith");
    let c2 = common::create_test_customer("Bob Jones");

    db.customers.insert(&c1).await.unwrap();
    db.customers.insert(&c2).await.unwrap();

    let results = db.customers.get_all().await.expect("Failed to get all");

    assert_eq!(results.len(), 2);
    let names: Vec<_> = results.iter().map(|c| c.name.clone()).collect();
    assert!(names.contains(&"Alice Smith".to_string()));
    assert!(names.contains(&"Bob Jones".to_string()));
}

#[tokio::test]
async fn test_customer_update_credit() {
    let db = common::setup_test_db().await;

    let customer = common::create_test_customer("Credit Test");
    db.customers.insert(&customer).await.unwrap();

    // Add credit
    db.customers
        .update_store_credit(customer.customer_uuid, 50.0)
        .await
        .expect("Failed to add credit");

    let updated = db
        .customers
        .get_by_id(customer.customer_uuid)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.store_credit, 50.0);

    // Subtract credit
    db.customers
        .update_store_credit(customer.customer_uuid, -20.0)
        .await
        .expect("Failed to subtract credit");

    let final_cust = db
        .customers
        .get_by_id(customer.customer_uuid)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(final_cust.store_credit, 30.0);
}

/// CRITICAL TEST: Store credit negative handling
#[tokio::test]
async fn test_customer_negative_store_credit_handling() {
    let db = common::setup_test_db().await;

    let customer = common::create_test_customer("Negative Balance");
    db.customers.insert(&customer).await.unwrap();

    // Try to remove 100 from 0 balance
    let result = db
        .customers
        .update_store_credit(customer.customer_uuid, -100.0)
        .await;

    assert!(result.is_ok());

    let updated = db
        .customers
        .get_by_id(customer.customer_uuid)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.store_credit, -100.0);
}
