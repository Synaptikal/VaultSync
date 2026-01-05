//! Printer API handlers
//!
//! Handles printer discovery and print queue management.

use crate::api::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
};

/// Get discovered printers
pub async fn get_printers(
    State(state): State<AppState>,
) -> (StatusCode, Json<Vec<crate::services::PrinterInfo>>) {
    let printers = state.system.printers.discover_printers().await;
    (StatusCode::OK, Json(printers))
}

/// Get pending print jobs
pub async fn get_print_queue(
    State(state): State<AppState>,
) -> (StatusCode, Json<Vec<crate::services::PrintJob>>) {
    let jobs = state.system.printers.get_pending_jobs().await;
    (StatusCode::OK, Json(jobs))
}
