use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::sync::Arc;
use tower::ServiceExt;
use vaultsync::{
    api, audit,
    auth::{create_jwt, UserRole},
    buylist::BuylistService,
    config::Config,
    core::{Category, Customer, Product},
    database,
    inventory::InventoryService,
    network, pricing, services,
    sync::SyncActor,
    transactions::TransactionService,
};

async fn setup_test_app() -> axum::Router {
    // 1. Initialize Test Config
    let mut config = Config::default();
    // Default config has cors_origins: None, which means is_production() returns false.
    config.node_id = format!("test_node_{}", uuid::Uuid::new_v4());

    // 2. Initialize Test DB (In-Memory)
    let db = database::initialize_test_db()
        .await
        .expect("Failed to init test db");

    // 3. Initialize Services (Mimicking main.rs)
    let pricing_service_concrete = pricing::PricingService::new(db.clone());
    let pricing_service = Arc::new(pricing_service_concrete.clone());

    let inventory_service = InventoryService::new(db.inventory.clone());
    let inventory_service_arc = Arc::new(inventory_service.clone());

    let rule_engine = Arc::new(pricing::RuleEngine::new());

    let buylist_service = BuylistService::new(
        db.clone(),
        Arc::new(pricing_service_concrete),
        rule_engine,
        inventory_service_arc.clone(),
    );
    let buylist_service_arc = Arc::new(buylist_service);

    let transaction_service = TransactionService::new(
        db.clone(),
        inventory_service_arc.clone(),
        pricing_service.clone(),
    );
    let transaction_service_arc = Arc::new(transaction_service);

    // Sync Actor
    let actor_network = network::NetworkService::new().ok();
    let (sync_actor_handle, sync_actor) =
        SyncActor::new(db.clone(), actor_network, config.node_id.clone(), 100);
    tokio::spawn(sync_actor.run());

    let product_service = Arc::new(services::ProductService::new(db.products.clone()));

    // Phase 2 Services
    let tax_service = Arc::new(services::TaxService::new(db.clone()));
    let payment_service = Arc::new(services::PaymentService::new(db.clone()));
    let holds_service = Arc::new(services::HoldsService::new(db.clone()));
    let barcode_service = Arc::new(services::BarcodeService::new(db.clone()));
    let receipt_service = Arc::new(services::ReceiptService::new(db.clone(), config.clone()));
    let invoice_service = Arc::new(services::InvoiceService::new(db.clone(), config.clone()));
    let catalog_lookup_service = Arc::new(services::CatalogLookupService::new());
    let label_service = Arc::new(services::LabelService::new(
        db.clone(),
        barcode_service.clone(),
        pricing_service.clone(),
    ));

    // Phase 6
    let cash_drawer_service = Arc::new(services::CashDrawerService::new(db.clone()));
    let printer_service = Arc::new(services::PrinterService::new());

    // Phase 7
    let returns_service = Arc::new(services::ReturnsService::new(db.clone()));
    let serialized_inventory_service =
        Arc::new(services::SerializedInventoryService::new(db.clone()));
    let trade_in_protection_service = Arc::new(services::TradeInProtectionService::new(db.clone()));
    let location_service = Arc::new(services::LocationService::new(db.clone()));

    // Phase 8 & 9
    let reporting_service = Arc::new(services::ReportingService::new(db.clone()));

    // Explicitly use types to match AppState definition
    let email_service: Arc<Box<dyn services::notification::EmailProvider>> =
        Arc::new(services::notification::email::get_email_provider());

    let sms_service: Arc<Box<dyn services::notification::sms::SmsProvider>> =
        Arc::new(services::notification::sms::get_sms_provider());

    let notification_scheduler = Arc::new(
        services::notification::scheduler::NotificationScheduler::new(
            db.clone(),
            email_service.clone(),
            sms_service.clone(),
        ),
    );

    let app_state = api::AppState {
        db: db.clone(),
        commerce: api::state_groups::CommerceServices {
            product: product_service,
            inventory: inventory_service_arc,
            pricing: pricing_service,
            transactions: transaction_service_arc,
            buylist: buylist_service_arc,
            holds: holds_service,
            payments: payment_service,
            taxes: tax_service,
            returns: returns_service,
            trade_in: trade_in_protection_service,
        },
        system: api::state_groups::SystemServices {
            audit: Arc::new(audit::AuditService::new(db.clone())),
            events: Arc::new(vaultsync::events::EventService::new(db.clone())),
            barcode: barcode_service,
            receipts: receipt_service,
            invoices: invoice_service,
            labels: label_service,
            cash_drawer: cash_drawer_service,
            printers: printer_service,
            catalog: catalog_lookup_service,
            serialized: serialized_inventory_service,
            locations: location_service,
            reporting: reporting_service,
            email: email_service,
            sms: sms_service,
            notification_scheduler,
        },
        sync_actor: sync_actor_handle,
        config: Arc::new(config.clone()),
    };

    api::create_router(app_state, &config)
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                // Add ConnectInfo for rate limiting middleware
                .extension(axum::extract::ConnectInfo(std::net::SocketAddr::from((
                    [127, 0, 0, 1],
                    8080,
                ))))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_products_unauthorized() {
    let app = setup_test_app().await;

    // Attempt to access protected route without auth header
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/products")
                // Add ConnectInfo for rate limiting middleware
                .extension(axum::extract::ConnectInfo(std::net::SocketAddr::from((
                    [127, 0, 0, 1],
                    8080,
                ))))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_products_authorized_empty() {
    let app = setup_test_app().await;

    // Generate token (match Config::default secret)
    std::env::set_var(
        "JWT_SECRET",
        "test_secret_key_must_be_at_least_32_chars_long_for_security",
    );

    let token = create_jwt(uuid::Uuid::new_v4(), "admin", UserRole::Admin).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/products")
                .header("Authorization", format!("Bearer {}", token))
                .extension(axum::extract::ConnectInfo(std::net::SocketAddr::from((
                    [127, 0, 0, 1],
                    8080,
                ))))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Read body
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body_json.is_array());
    assert_eq!(body_json.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_create_and_get_product() {
    let app = setup_test_app().await;

    // Generate token
    std::env::set_var(
        "JWT_SECRET",
        "test_secret_key_must_be_at_least_32_chars_long_for_security",
    );
    let token = create_jwt(uuid::Uuid::new_v4(), "admin", UserRole::Admin).unwrap();

    // 1. Create Product
    let product_uuid = uuid::Uuid::new_v4();
    let new_product = Product {
        product_uuid,
        name: "Test API Product".to_string(),
        category: Category::TCG,
        set_code: Some("API1".to_string()),
        collector_number: Some("123".to_string()),
        barcode: Some("987654321".to_string()),
        release_year: Some(2025),
        metadata: serde_json::json!({"rarity": "Ultra Rare"}),
        weight_oz: Some(5.0),
        length_in: None,
        width_in: None,
        height_in: None,
        upc: None,
        isbn: None,
        manufacturer: None,
        msrp: Some(10.0),
        deleted_at: None,
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/products")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .extension(axum::extract::ConnectInfo(std::net::SocketAddr::from((
                    [127, 0, 0, 1],
                    8080,
                ))))
                .body(Body::from(serde_json::to_string(&new_product).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // 2. Get Product by ID
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/products/{}", product_uuid))
                .header("Authorization", format!("Bearer {}", token))
                .extension(axum::extract::ConnectInfo(std::net::SocketAddr::from((
                    [127, 0, 0, 1],
                    8080,
                ))))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let fetched_product: Product = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(fetched_product.product_uuid, product_uuid);
    assert_eq!(fetched_product.name, "Test API Product");
    assert_eq!(fetched_product.category, Category::TCG);
}

#[tokio::test]
async fn test_customer_lifecycle_api() {
    let app = setup_test_app().await;

    // Generate token
    std::env::set_var(
        "JWT_SECRET",
        "test_secret_key_must_be_at_least_32_chars_long_for_security",
    );
    let token = create_jwt(uuid::Uuid::new_v4(), "manager", UserRole::Manager).unwrap();

    // 1. Create Customer
    let customer_uuid = uuid::Uuid::new_v4();
    let new_customer = Customer {
        customer_uuid,
        name: "API Customer".to_string(),
        email: Some("api@example.com".to_string()),
        phone: Some("555-9999".to_string()),
        store_credit: 0.0,
        tier: Some("Gold".to_string()),
        created_at: chrono::Utc::now(),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/customers")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .extension(axum::extract::ConnectInfo(std::net::SocketAddr::from((
                    [127, 0, 0, 1],
                    8080,
                ))))
                .body(Body::from(serde_json::to_string(&new_customer).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // 2. Get Customer by ID
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/customers/{}", customer_uuid))
                .header("Authorization", format!("Bearer {}", token))
                .extension(axum::extract::ConnectInfo(std::net::SocketAddr::from((
                    [127, 0, 0, 1],
                    8080,
                ))))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let fetched_customer: Customer = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(fetched_customer.name, "API Customer");
    assert_eq!(fetched_customer.email, Some("api@example.com".to_string()));
}

#[tokio::test]
async fn test_manager_route_protection() {
    let app = setup_test_app().await;

    // Generate token
    std::env::set_var(
        "JWT_SECRET",
        "test_secret_key_must_be_at_least_32_chars_long_for_security",
    );

    // 1. Test as Employee (Should fail)
    let employee_token = create_jwt(uuid::Uuid::new_v4(), "employee", UserRole::Employee).unwrap();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/audit-log")
                .header("Authorization", format!("Bearer {}", employee_token))
                .extension(axum::extract::ConnectInfo(std::net::SocketAddr::from((
                    [127, 0, 0, 1],
                    8080,
                ))))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // 2. Test as Manager (Should succeed)
    let manager_token = create_jwt(uuid::Uuid::new_v4(), "manager", UserRole::Manager).unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/audit-log")
                .header("Authorization", format!("Bearer {}", manager_token))
                .extension(axum::extract::ConnectInfo(std::net::SocketAddr::from((
                    [127, 0, 0, 1],
                    8080,
                ))))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
