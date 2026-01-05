//! Monitoring & Observability Module
//!
//! Phase 10: Provides metrics, health checks, and system monitoring
//!
//! Features:
//! - Prometheus-compatible metrics endpoint
//! - Comprehensive health checks (DB, disk, sync)
//! - Request latency tracking
//! - Request ID correlation
//! - Business metrics (sales, transactions)
//! - Alerting (error rate, disk space, sync failures)
//! - Audit logging for data modifications

pub mod alerting;
pub mod audit_log;
pub mod health;
pub mod metrics;
pub mod request_id;

pub use alerting::*;
pub use audit_log::*;
pub use health::*;
pub use metrics::*;
pub use request_id::*;
