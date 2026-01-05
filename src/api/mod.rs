use crate::database::Database;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod auth;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod validation;

#[derive(OpenApi)]
#[openapi(
    paths(),
    components(
        schemas(
            crate::core::Product,
            crate::core::Category,
            crate::core::InventoryItem,
            crate::core::VariantType,
            crate::core::Condition,
            crate::core::PriceInfo,
            crate::core::Transaction,
            crate::core::TransactionItem,
            crate::core::TransactionType,
            crate::core::Customer,
            crate::core::VectorTimestamp,
            crate::sync::ChangeRecord,
            crate::core::RecordType,
            crate::core::SyncOperation,
            crate::core::WantsList,
            crate::core::WantsItem,
            crate::core::Event,
            crate::core::EventParticipant
        )
    ),
    tags(
        (name = "VaultSync", description = "VaultSync POS API")
    )
)]
pub struct ApiDoc;

pub mod state_groups;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub commerce: state_groups::CommerceServices,
    pub system: state_groups::SystemServices,
    pub sync_actor: crate::sync::SyncActorHandle,
    pub config: Arc<crate::config::Config>,
}

pub fn create_router(state: AppState, config: &crate::config::Config) -> Router {
    // Rate limiting configuration from config
    let api_rate_limit = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(config.rate_limit_per_second)
            .burst_size(config.rate_limit_burst)
            .finish()
            .expect("Invalid rate limit configuration - check rate_limit_per_second and rate_limit_burst"),
    );

    // Stricter rate limiting for auth routes
    let auth_rate_limit = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(config.auth_rate_limit_per_second)
            .burst_size(config.auth_rate_limit_burst)
            .finish()
            .expect("Invalid auth rate limit configuration - check auth_rate_limit_* settings"),
    );

    // Manager-only routes (price overrides, pricing sync, some reports, backups)
    let manager_routes = Router::new()
        .route("/api/pricing/override", post(handlers::log_price_override))
        .route("/api/pricing/sync", post(handlers::trigger_price_sync))
        .route(
            "/api/pricing/cache/invalidate",
            post(handlers::invalidate_price_cache),
        )
        .route("/api/sync/trigger", post(handlers::trigger_peer_sync))
        // Backup routes (Phase 11)
        .route("/api/admin/backup", post(handlers::create_backup))
        .route("/api/admin/backups", get(handlers::list_backups))
        .route("/api/admin/backup/verify", post(handlers::verify_backup))
        .route(
            "/api/admin/backup/retention",
            post(handlers::apply_backup_retention),
        )
        // Audit Log routes (Phase 10 - TASK-203)
        .route("/api/admin/audit-log", get(handlers::get_audit_log))
        .route(
            "/api/admin/audit-log/:table_name/:record_uuid",
            get(handlers::get_record_audit_history),
        )
        .route_layer(axum::middleware::from_fn(middleware::require_manager))
        .route_layer(axum::middleware::from_fn(middleware::auth_middleware));

    // Protected API routes (require authentication, any role)
    let api_routes = Router::new()
        // Products
        .route(
            "/api/products",
            get(handlers::get_products).post(handlers::create_product),
        )
        .route("/api/products/search", get(handlers::search_products))
        .route(
            "/api/products/barcode/:barcode",
            get(handlers::lookup_by_barcode),
        )
        .route(
            "/api/products/:product_uuid",
            get(handlers::get_product_by_id),
        )
        // Inventory
        .route(
            "/api/inventory",
            get(handlers::get_inventory).post(handlers::add_inventory),
        )
        .route(
            "/api/inventory/barcode/bulk",
            post(handlers::bulk_generate_barcodes),
        )
        .route(
            "/api/inventory/barcode/:data",
            get(handlers::generate_barcode),
        )
        .route(
            "/api/inventory/qrcode/:data",
            get(handlers::generate_qrcode),
        )
        .route("/api/inventory/low-stock", get(handlers::get_low_stock))
        .route("/api/inventory/bulk", post(handlers::bulk_inventory_update))
        .route("/api/inventory/matrix", get(handlers::get_inventory_matrix))
        .route(
            "/api/inventory/:inventory_uuid",
            get(handlers::get_inventory_item)
                .delete(handlers::delete_inventory_item)
                .put(handlers::update_inventory_item),
        )
        .route(
            "/api/inventory/label/:inventory_uuid",
            get(handlers::get_inventory_label),
        )
        .route(
            "/api/products/label/:product_uuid",
            get(handlers::get_product_label),
        )
        // Pricing (read-only for regular users)
        .route(
            "/api/pricing/dashboard",
            get(handlers::get_pricing_dashboard),
        )
        .route(
            "/api/pricing/cache/stats",
            get(handlers::get_price_cache_stats),
        )
        .route("/api/pricing/:product_uuid", get(handlers::get_price_info))
        .route(
            "/api/pricing/:product_uuid/history",
            get(handlers::get_price_history),
        )
        // Transactions
        .route(
            "/api/transactions",
            get(handlers::get_transactions).post(handlers::create_transaction),
        )
        .route(
            "/api/transactions/:transaction_uuid",
            get(handlers::get_transaction_by_id),
        )
        .route(
            "/api/transactions/:transaction_uuid/receipt",
            get(handlers::get_receipt),
        )
        .route(
            "/api/transactions/:transaction_uuid/invoice",
            get(handlers::generate_invoice),
        )
        // Customers
        .route(
            "/api/customers",
            get(handlers::get_customers).post(handlers::create_customer),
        )
        .route(
            "/api/customers/history",
            get(handlers::get_customer_history),
        )
        .route("/api/customers/credit", post(handlers::update_store_credit))
        .route(
            "/api/customers/:customer_uuid",
            get(handlers::get_customer_by_id),
        )
        // Buylist
        .route("/api/buylist/quote", post(handlers::get_buylist_quote))
        .route("/api/buylist/process", post(handlers::process_buylist))
        .route("/api/buylist/trade-in", post(handlers::process_trade_in))
        // Sync
        .route("/api/sync/status", get(handlers::get_sync_status))
        .route("/api/sync/push", post(handlers::push_sync_changes))
        .route("/api/sync/pull", get(handlers::pull_sync_changes))
        // Network Discovery (TASK-117, TASK-118)
        .route(
            "/api/network/devices",
            get(handlers::get_discovered_devices),
        )
        .route("/api/network/pair", post(handlers::manual_pair_device))
        // Sync Conflicts (TASK-121)
        .route("/api/sync/conflicts", get(handlers::get_sync_conflicts))
        .route(
            "/api/sync/conflicts/resolve",
            post(handlers::resolve_sync_conflict),
        )
        // Sync Progress (TASK-125)
        .route("/api/sync/progress", get(handlers::get_sync_progress))
        // Reports
        .route("/api/reports/sales", get(handlers::get_sales_report))
        .route(
            "/api/reports/inventory-valuation",
            get(handlers::get_inventory_valuation),
        )
        .route("/api/reports/top-sellers", get(handlers::get_top_sellers))
        .route(
            "/api/reports/low-stock",
            get(handlers::get_low_stock_report),
        )
        // Dashboard
        .route("/api/dashboard/stats", get(handlers::get_dashboard_stats))
        // Events
        .route(
            "/api/events",
            get(handlers::get_events).post(handlers::create_event),
        )
        .route(
            "/api/events/:event_uuid/register",
            post(handlers::register_participant),
        )
        // Wants
        .route("/api/wants", post(handlers::create_wants_list))
        .route(
            "/api/customers/:customer_uuid/wants",
            get(handlers::get_wants_lists),
        )
        // Audit
        .route("/api/audit/conflicts", get(handlers::get_conflicts))
        .route(
            "/api/audit/submit-blind-count",
            post(handlers::submit_blind_count),
        )
        .route(
            "/api/audit/conflicts/:conflict_uuid/resolve",
            post(handlers::resolve_conflict),
        )
        // Tax Rates
        .route(
            "/api/tax/rates",
            get(handlers::get_tax_rates).post(handlers::create_tax_rate),
        )
        .route("/api/tax/default-rate", get(handlers::get_default_tax_rate))
        // Holds/Layaway
        .route("/api/holds", post(handlers::create_hold))
        .route("/api/holds/:hold_uuid", get(handlers::get_hold))
        .route(
            "/api/holds/:hold_uuid/payment",
            post(handlers::make_hold_payment),
        )
        .route("/api/holds/:hold_uuid/cancel", post(handlers::cancel_hold))
        .route(
            "/api/holds/:hold_uuid/complete",
            post(handlers::complete_hold),
        )
        .route(
            "/api/holds/expire-overdue",
            post(handlers::expire_overdue_holds),
        )
        .route(
            "/api/customers/:customer_uuid/holds",
            get(handlers::get_customer_holds),
        )
        // Cash Drawer (Phase 6: TASK-137 to TASK-144)
        .route("/api/cash-drawer/open", post(handlers::open_cash_drawer))
        .route("/api/cash-drawer/count", post(handlers::record_cash_count))
        .route("/api/shifts", post(handlers::open_shift))
        .route("/api/shifts/:shift_uuid/close", post(handlers::close_shift))
        .route(
            "/api/shifts/terminal/:terminal_id",
            get(handlers::get_current_shift),
        )
        .route(
            "/api/reports/cash-variance",
            get(handlers::get_cash_variance_report),
        )
        .route(
            "/api/shifts/:shift_uuid/z-report",
            get(handlers::get_shift_z_report),
        )
        .route(
            "/api/reports/cash-flow",
            get(handlers::get_cash_flow_report),
        )
        // Printers (Phase 6: TASK-145 to TASK-149)
        .route("/api/printers", get(handlers::get_printers))
        .route("/api/printers/queue", get(handlers::get_print_queue))
        // Returns (Phase 7: TASK-168 to TASK-173)
        .route("/api/returns/policy", get(handlers::get_return_policy))
        .route("/api/returns/reasons", get(handlers::get_return_reasons))
        .route("/api/returns", post(handlers::process_return))
        // Serialized Inventory (Phase 7: TASK-156 to TASK-161)
        .route(
            "/api/inventory/serialized/:inventory_uuid",
            get(handlers::get_serialized_details).put(handlers::update_serialized_details),
        )
        .route(
            "/api/inventory/serialized/:inventory_uuid/grading",
            post(handlers::add_grading),
        )
        .route(
            "/api/inventory/serialized/:inventory_uuid/certificate",
            post(handlers::add_certificate),
        )
        // Trade-In Protection (Phase 7: TASK-162 to TASK-167)
        .route(
            "/api/trade-in/check",
            post(handlers::check_trade_in_eligibility),
        )
        .route(
            "/api/trade-in/history/:customer_uuid",
            get(handlers::get_trade_in_history),
        )
        .route(
            "/api/trade-in/suspicious",
            post(handlers::log_suspicious_activity),
        )
        // Locations & Transfers (Phase 7.5: TASK-174 to TASK-179)
        .route(
            "/api/locations",
            get(handlers::get_locations).post(handlers::upsert_location),
        )
        .route("/api/transfers", post(handlers::create_transfer))
        .route(
            "/api/transfers/:transfer_uuid/status",
            axum::routing::put(handlers::update_transfer_status),
        )
        // Reporting (Phase 8)
        .route(
            "/api/reports/sales/csv",
            get(handlers::get_sales_report_csv_export),
        )
        .route(
            "/api/reports/employee-performance",
            get(handlers::get_employee_performance_report),
        )
        .route(
            "/api/reports/inventory-aging",
            get(handlers::get_inventory_aging_report),
        )
        // Notifications (Phase 9)
        .route(
            "/api/transactions/:transaction_uuid/email-receipt",
            post(handlers::email_receipt),
        )
        .route(
            "/api/customers/:customer_uuid/notify",
            post(handlers::notify_customer),
        )
        // Quote Email (Task 196)
        .route(
            "/api/buylist/quote/email",
            post(handlers::email_trade_in_quote),
        )
        // Notification Scheduler (Phase 9)
        .route(
            "/api/admin/notifications/run",
            post(handlers::run_scheduled_notifications),
        )
        .route(
            "/api/products/:product_uuid/check-wants-list",
            post(handlers::check_wants_list_matches),
        )
        // User
        .route("/api/user/me", get(handlers::get_current_user))
        .route_layer(axum::middleware::from_fn(middleware::auth_middleware));

    // CRIT-004 FIX: Auth routes with rate limiting applied
    let auth_routes = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh_token)) // MED-005 FIX
        .layer(GovernorLayer {
            config: auth_rate_limit,
        });

    // Apply rate limiting to protected API routes
    let api_routes = api_routes.layer(GovernorLayer {
        config: api_rate_limit,
    });

    // Build CORS layer based on configuration
    let cors_layer = match &config.cors_origins {
        Some(origins) if !origins.is_empty() => {
            use tower_http::cors::AllowOrigin;
            let origins: Vec<_> = origins.iter().filter_map(|s| s.parse().ok()).collect();
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(origins))
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PUT,
                    axum::http::Method::DELETE,
                    axum::http::Method::OPTIONS,
                ])
                .allow_headers(tower_http::cors::Any)
                .allow_credentials(true)
        }
        _ => {
            tracing::warn!("CORS_ALLOWED_ORIGINS not configured - using permissive mode");
            CorsLayer::permissive()
        }
    };

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Health & Monitoring (Phase 10)
        .route("/health", get(handlers::health_check))
        .route("/health/detailed", get(handlers::health_check_detailed))
        .route("/health/alerts", get(handlers::get_alerts))
        .merge(auth_routes)
        .merge(manager_routes)
        .merge(api_routes)
        .layer(cors_layer)
        // Request ID middleware for tracing (TASK-202)
        .layer(axum::middleware::from_fn(
            crate::monitoring::request_id_middleware,
        ))
        .with_state(state)
}
