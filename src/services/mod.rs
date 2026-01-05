//! Services module
//!
//! Contains business logic services for the VaultSync POS system.

pub mod backup;
pub mod barcode;
pub mod cash_drawer;
pub mod catalog_lookup;
pub mod holds;
pub mod invoice;
pub mod label;
pub mod location;
pub mod notification;
pub mod offline_queue;
pub mod payment;
pub mod printer;
pub mod product;
pub mod receipt;
pub mod reporting;
pub mod returns;
pub mod serialized_inventory;
pub mod supervisor;
pub mod tax;
pub mod trade_in_protection;
pub mod transaction;

pub use product::ProductService;

pub use barcode::BarcodeService;
pub use cash_drawer::{
    CashCount, CashCountType, CashDrawerService, CashVarianceReport, Shift, ShiftStatus,
    ShiftVariance,
};
pub use catalog_lookup::CatalogLookupService;
pub use holds::{
    CreateHoldRequest, Hold, HoldItem, HoldPayment, HoldStatus, HoldSummary, HoldsService,
};
pub use invoice::InvoiceService;
pub use label::LabelService;
pub use location::{
    Location, LocationService, LocationType, TransferItem, TransferRequest, TransferStatus,
};
pub use offline_queue::{OfflineQueueService, QueueStatus, QueuedOperation};
pub use payment::{
    CashPaymentResult, PaymentMethodType, PaymentRecord, PaymentRequest, PaymentResult,
    PaymentService,
};
pub use printer::{
    EscPosBuilder, PrintJob, PrintJobType, PrinterInfo, PrinterService, PrinterType,
};
pub use receipt::ReceiptService;
pub use reporting::{InventoryValuationReport, ReportingService, SalesReport};
pub use returns::{ReturnPolicy, ReturnReasonCode, ReturnRequest, ReturnResult, ReturnsService};
pub use serialized_inventory::{
    CertificateInfo, GradingInfo, SerializedInventoryService, SerializedItem,
    SerializedSearchResult,
};
pub use tax::{ItemTax, TaxBreakdown, TaxRate, TaxService};
pub use trade_in_protection::{
    AlertSeverity, CustomerTradeInHistory, SuspiciousActivity, SuspiciousActivityType,
    TradeInBlacklistEntry, TradeInCheck, TradeInProtectionService,
};
pub use transaction::{
    TradeInItemRequest, TransactionItemRequest, TransactionRequest, TransactionResult,
    TransactionValidationService, ValidationResult,
};
