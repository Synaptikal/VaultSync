//! Handler modules - each domain has its own file
//!
//! This modular structure provides clean, scalable, and maintainable code.
//! Each domain has its own handler file with related endpoints.
//!
//! **MIGRATION STATUS: COMPLETE** - All handlers have been migrated from handlers_legacy.rs

// Domain-focused handler modules
pub mod audit;
pub mod backups;
pub mod barcode;
pub mod buylist;
pub mod cash_drawer;
pub mod customers;
pub mod dashboard;
pub mod events;
pub mod health;
pub mod holds;
pub mod inventory;
pub mod invoices;
pub mod labels;
pub mod locations;
pub mod notifications;
pub mod pricing;
pub mod printers;
pub mod products;
pub mod receipts;
pub mod reports;
pub mod returns;
pub mod scheduler;
pub mod serialized_inventory;
pub mod sync;
pub mod tax;
pub mod trade_in;
pub mod transactions;
pub mod users;
pub mod wants;

// ===========================================================================
// Re-exports - organized by domain
// ===========================================================================

// Audit handlers
pub use audit::get_conflicts;
pub use audit::resolve_conflict;
pub use audit::submit_blind_count;

// Backup handlers
pub use backups::apply_backup_retention;
pub use backups::create_backup;
pub use backups::list_backups;
pub use backups::verify_backup;

// Barcode handlers
pub use barcode::bulk_generate_barcodes;
pub use barcode::generate_barcode;
pub use barcode::generate_qrcode;
pub use barcode::lookup_by_barcode;

// Buylist handlers
pub use buylist::get_buylist_quote;
pub use buylist::process_buylist;
pub use buylist::process_trade_in;

// Cash drawer handlers
pub use cash_drawer::close_shift;
pub use cash_drawer::get_cash_variance_report;
pub use cash_drawer::get_current_shift;
pub use cash_drawer::get_shift_z_report;
pub use cash_drawer::open_cash_drawer;
pub use cash_drawer::open_shift;
pub use cash_drawer::record_cash_count;

// Customer handlers
pub use customers::create_customer;
pub use customers::get_customer_by_id;
pub use customers::get_customer_history;
pub use customers::get_customers;
pub use customers::update_store_credit;

// Dashboard handlers
pub use dashboard::get_alerts;
pub use dashboard::get_dashboard_stats;

// Event handlers
pub use events::create_event;
pub use events::get_events;
pub use events::register_participant;

// Health handlers
pub use health::get_audit_log;
pub use health::get_record_audit_history;
pub use health::health_check;
pub use health::health_check_detailed;

// Holds/Layaway handlers
pub use holds::cancel_hold;
pub use holds::complete_hold;
pub use holds::create_hold;
pub use holds::expire_overdue_holds;
pub use holds::get_customer_holds;
pub use holds::get_hold;
pub use holds::make_hold_payment;

// Inventory handlers
pub use inventory::add_inventory;
pub use inventory::bulk_inventory_update;
pub use inventory::delete_inventory_item;
pub use inventory::get_inventory;
pub use inventory::get_inventory_item;
pub use inventory::get_inventory_matrix;
pub use inventory::get_low_stock;

// Invoice handlers
pub use invoices::generate_invoice;

// Label handlers
pub use labels::get_inventory_label;
pub use labels::get_product_label;

// Location/Transfer handlers
pub use locations::create_transfer;
pub use locations::get_locations;
pub use locations::update_transfer_status;
pub use locations::upsert_location;

// Notification handlers
pub use notifications::email_receipt;
pub use notifications::email_trade_in_quote;
pub use notifications::notify_customer;

// Pricing handlers
pub use pricing::get_price_cache_stats;
pub use pricing::get_price_history;
pub use pricing::get_price_info;
pub use pricing::get_pricing_dashboard;
pub use pricing::invalidate_price_cache;
pub use pricing::log_price_override;
pub use pricing::trigger_price_sync;

// Printer handlers
pub use printers::get_print_queue;
pub use printers::get_printers;

// Product handlers
pub use products::create_product;
pub use products::get_product_by_id;
pub use products::get_products;
pub use products::search_products;

// Receipt handlers
pub use receipts::get_receipt;

// Report handlers
pub use reports::get_cash_flow_report;
pub use reports::get_employee_performance_report;
pub use reports::get_inventory_aging_report;
pub use reports::get_inventory_valuation;
pub use reports::get_low_stock_report;
pub use reports::get_sales_report;
pub use reports::get_sales_report_csv_export;
pub use reports::get_top_sellers;

// Returns handlers
pub use returns::get_return_policy;
pub use returns::get_return_reasons;
pub use returns::process_return;

// Scheduler handlers
pub use scheduler::check_wants_list_matches;
pub use scheduler::run_scheduled_notifications;

// Serialized inventory handlers
pub use serialized_inventory::add_certificate;
pub use serialized_inventory::add_grading;
pub use serialized_inventory::get_serialized_details;
pub use serialized_inventory::update_serialized_details;

// Sync handlers
pub use sync::get_discovered_devices;
pub use sync::get_sync_conflicts;
pub use sync::get_sync_progress;
pub use sync::get_sync_status;
pub use sync::manual_pair_device;
pub use sync::pull_sync_changes;
pub use sync::push_sync_changes;
pub use sync::resolve_sync_conflict;
pub use sync::trigger_peer_sync;

// Tax handlers
pub use tax::create_tax_rate;
pub use tax::get_default_tax_rate;
pub use tax::get_tax_rates;

// Trade-in handlers
pub use trade_in::check_trade_in_eligibility;
pub use trade_in::get_trade_in_history;
pub use trade_in::log_suspicious_activity;

// Transaction handlers
pub use transactions::create_transaction;
pub use transactions::get_transaction_by_id;
pub use transactions::get_transactions;

// User handlers
pub use users::get_current_user;

// Wants list handlers
pub use wants::create_wants_list;
pub use wants::get_wants_lists;
pub use wants::update_inventory_item;
