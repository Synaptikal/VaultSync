use crate::services;
use std::sync::Arc;

#[derive(Clone)]
pub struct CommerceServices {
    pub product: Arc<services::ProductService>,
    pub inventory: Arc<crate::inventory::InventoryService>,
    pub pricing: Arc<crate::pricing::PricingService>,
    pub transactions: Arc<crate::transactions::TransactionService>,
    pub buylist: Arc<crate::buylist::BuylistService>,
    pub holds: Arc<services::HoldsService>,
    pub payments: Arc<services::PaymentService>,
    pub taxes: Arc<services::TaxService>,
    pub returns: Arc<services::ReturnsService>,
    pub trade_in: Arc<services::TradeInProtectionService>,
}

#[derive(Clone)]
pub struct SystemServices {
    pub audit: Arc<crate::audit::AuditService>,
    pub events: Arc<crate::events::EventService>,
    pub barcode: Arc<services::BarcodeService>,
    pub receipts: Arc<services::ReceiptService>,
    pub invoices: Arc<services::InvoiceService>,
    pub labels: Arc<services::LabelService>,
    pub cash_drawer: Arc<services::CashDrawerService>,
    pub printers: Arc<services::PrinterService>,
    pub catalog: Arc<services::CatalogLookupService>,
    pub serialized: Arc<services::SerializedInventoryService>,
    pub locations: Arc<services::LocationService>,
    pub reporting: Arc<services::ReportingService>,
    pub email: Arc<Box<dyn services::notification::EmailProvider>>,
    pub sms: Arc<Box<dyn services::notification::sms::SmsProvider>>,
    pub notification_scheduler: Arc<services::notification::scheduler::NotificationScheduler>,
}
