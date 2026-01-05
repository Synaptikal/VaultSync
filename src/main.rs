use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber;
use vaultsync::{
    api, audit, buylist, config::Config, database, inventory, network, pricing, services, sync,
    transactions,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables FIRST
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    // Validate configuration - fail fast if missing/invalid
    let config = match Config::from_env() {
        Ok(c) => {
            tracing::info!("Configuration loaded successfully");
            tracing::info!("  Node ID: {}", c.node_id);
            tracing::info!("  API Port: {}", c.api_port);
            tracing::info!("  Production mode: {}", c.is_production());
            c
        }
        Err(e) => {
            eprintln!("╔══════════════════════════════════════════════════════════════╗");
            eprintln!("║                    CONFIGURATION ERROR                        ║");
            eprintln!("╠══════════════════════════════════════════════════════════════╣");
            eprintln!("║ {}", e);
            eprintln!("╠══════════════════════════════════════════════════════════════╣");
            eprintln!("║ Please check your .env file or environment variables.        ║");
            eprintln!("║ See .env.example for required configuration.                  ║");
            eprintln!("╚══════════════════════════════════════════════════════════════╝");
            std::process::exit(1);
        }
    };

    tracing::info!("VaultSync POS System starting up...");

    // Initialize the database with config
    let db = database::Database::new(&config.database_url, config.node_id.clone()).await?;
    db.initialize_tables().await?;
    let db = Arc::new(db);

    // Initialize network discovery
    let mut network = network::NetworkService::new()?;
    network
        .start_discovery(&config.node_id, config.api_port)
        .await?;

    // Initialize services
    let pricing_service_concrete = pricing::PricingService::new(db.clone());

    // Warm up cache
    if let Err(e) = pricing_service_concrete.warm_cache().await {
        tracing::warn!("Failed to warm price cache (non-fatal): {}", e);
    }

    let pricing_service = Arc::new(pricing_service_concrete.clone());

    let inventory_service = inventory::InventoryService::new(db.inventory.clone());
    let inventory_service_arc = Arc::new(inventory_service.clone());

    let rule_engine = Arc::new(pricing::RuleEngine::new());

    let buylist_service = buylist::BuylistService::new(
        db.clone(),
        Arc::new(pricing_service_concrete),
        rule_engine,
        inventory_service_arc.clone(),
    );
    let buylist_service_arc = Arc::new(buylist_service);

    let transaction_service = transactions::TransactionService::new(
        db.clone(),
        inventory_service_arc.clone(),
        pricing_service.clone(),
    );
    let transaction_service_arc = Arc::new(transaction_service);

    // P0-3 Fix: Initialize sync actor (no global lock)
    // Create a new network service for the actor (can't share mutable network)
    let actor_network = network::NetworkService::new().ok();
    let (sync_actor_handle, sync_actor) = sync::SyncActor::new(
        db.clone(),
        actor_network,
        config.node_id.clone(),
        100, // buffer size
    );

    // Spawn the sync actor task
    tokio::spawn(sync_actor.run());

    // Initialize new Phase 2 services
    let tax_service = Arc::new(vaultsync::services::TaxService::new(db.clone()));
    let payment_service = Arc::new(vaultsync::services::PaymentService::new(db.clone()));
    let holds_service = Arc::new(vaultsync::services::HoldsService::new(db.clone()));
    let barcode_service = Arc::new(vaultsync::services::BarcodeService::new(db.clone()));
    let receipt_service = Arc::new(vaultsync::services::ReceiptService::new(
        db.clone(),
        config.clone(),
    ));
    let invoice_service = Arc::new(vaultsync::services::InvoiceService::new(
        db.clone(),
        config.clone(),
    ));
    let catalog_lookup_service = Arc::new(vaultsync::services::CatalogLookupService::new());
    let label_service = Arc::new(vaultsync::services::LabelService::new(
        db.clone(),
        barcode_service.clone(),
        pricing_service.clone(),
    ));

    // Phase 6: Cash Drawer and Printer Services
    let cash_drawer_service = Arc::new(vaultsync::services::CashDrawerService::new(db.clone()));
    let printer_service = Arc::new(vaultsync::services::PrinterService::new());

    // Phase 7: Advanced Features
    let returns_service = Arc::new(vaultsync::services::ReturnsService::new(db.clone()));
    let serialized_inventory_service = Arc::new(
        vaultsync::services::SerializedInventoryService::new(db.clone()),
    );
    let trade_in_protection_service = Arc::new(vaultsync::services::TradeInProtectionService::new(
        db.clone(),
    ));

    // Phase 7.5: Multi-Location Support
    let location_service = Arc::new(vaultsync::services::LocationService::new(db.clone()));

    // Phase 8: Reporting
    let reporting_service = Arc::new(vaultsync::services::ReportingService::new(db.clone()));

    // Phase 9: Notifications
    let email_service = Arc::new(vaultsync::services::notification::email::get_email_provider());
    let sms_service = Arc::new(vaultsync::services::notification::sms::get_sms_provider());
    let notification_scheduler = Arc::new(
        vaultsync::services::notification::scheduler::NotificationScheduler::new(
            db.clone(),
            email_service.clone(),
            sms_service.clone(),
        ),
    );

    // Create ProductService (ARCH-02: Services inject repos directly)
    let product_service = Arc::new(services::ProductService::new(db.products.clone()));

    // Setup App State
    // Setup App State
    let sync_actor_bg = sync_actor_handle.clone();
    let app_state = api::AppState {
        db: db.clone(),
        commerce: api::state_groups::CommerceServices {
            product: product_service,
            inventory: inventory_service_arc.clone(),
            pricing: pricing_service.clone(),
            transactions: transaction_service_arc.clone(),
            buylist: buylist_service_arc.clone(),
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
            notification_scheduler: notification_scheduler.clone(),
        },
        sync_actor: sync_actor_handle,
        config: Arc::new(config.clone()),
    };

    // Create Router with config for CORS
    let app = api::create_router(app_state, &config);

    // Initialize Service Supervisor
    let supervisor = services::supervisor::ServiceSupervisor::new();
    let supervisor_monitor = supervisor.clone();
    tokio::spawn(async move {
        supervisor_monitor.monitor().await;
    });

    // 1. Background Sync Service (Supervised)
    let sync_handle_for_task = sync_actor_bg.clone();
    supervisor
        .spawn("pending_sync_trigger", move || {
            let handle = sync_handle_for_task.clone();
            async move {
                tracing::info!("Background sync service started");
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                    tracing::debug!("Triggering background sync...");

                    // Use the actor handle (non-blocking)
                    if let Err(e) = handle.sync_with_peers().await {
                        tracing::error!("Background sync trigger failed: {}", e);
                    }
                }
            }
        })
        .await;

    // 2. Backup Service (Supervised)
    // CRIT-06 FIX: Backups enabled by default (opt-out instead of opt-in)
    if std::env::var("BACKUP_ENABLED")
        .map(|v| v.to_lowercase() != "false")
        .unwrap_or(true)
    {
        let interval_str =
            std::env::var("BACKUP_INTERVAL_HOURS").unwrap_or_else(|_| "24".to_string());
        supervisor
            .spawn("scheduled_backup", move || {
                let interval: u64 = interval_str.parse().unwrap_or(24);
                async move {
                    tracing::info!(
                        "Scheduled backup service started (interval: {} hours)",
                        interval
                    );
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(interval * 3600)).await;

                        let backup_service = vaultsync::services::backup::BackupService::from_env();
                        match backup_service.create_backup().await {
                            Ok(result) => {
                                if result.success {
                                    tracing::info!(
                                        "Scheduled backup completed: {}",
                                        result.message
                                    );
                                    if let Err(e) = backup_service.apply_retention_policy().await {
                                        tracing::warn!("Failed to apply retention policy: {}", e);
                                    }
                                } else {
                                    tracing::error!("Scheduled backup failed: {}", result.message);
                                }
                            }
                            Err(e) => tracing::error!("Scheduled backup error: {}", e),
                        }
                    }
                }
            })
            .await;
    }

    // 3. Notification Service (Supervised)
    let notify_scheduler_for_task = notification_scheduler.clone();
    supervisor
        .spawn("notification_scheduler", move || {
            let scheduler = notify_scheduler_for_task.clone();
            async move {
                tracing::info!("Notification scheduler started (interval: 1 hour)");
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                    tracing::debug!("Running scheduled notification tasks...");
                    if let Err(e) = scheduler.run_scheduled_tasks().await {
                        tracing::error!("Scheduled notification tasks failed: {}", e);
                    }
                }
            }
        })
        .await;

    // Start Server
    let bind_addr = format!("0.0.0.0:{}", config.api_port);
    let listener = TcpListener::bind(&bind_addr).await?;
    tracing::info!("VaultSync API listening on {}", listener.local_addr()?);

    if !config.is_production() {
        tracing::warn!("Running in DEVELOPMENT mode - CORS is permissive");
    }

    axum::serve(listener, app).await?;

    Ok(())
}
