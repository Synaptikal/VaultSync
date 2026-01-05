//! Invoice-related API handlers
//!
//! Handles invoice generation for transactions.

use crate::api::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use uuid::Uuid;

/// Generate an HTML invoice for a transaction
pub async fn generate_invoice(
    State(state): State<AppState>,
    Path(transaction_uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state.system.invoices.generate_html(transaction_uuid).await {
        Ok(html) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "text/html")],
            html,
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::extract::Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
