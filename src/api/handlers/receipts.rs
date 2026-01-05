//! Receipt-related API handlers
//!
//! Handles receipt generation for transactions.

use crate::api::AppState;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use uuid::Uuid;

/// Generate an HTML receipt for a transaction
pub async fn get_receipt(
    State(state): State<AppState>,
    Path(transaction_uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state.system.receipts.generate_html(transaction_uuid).await {
        Ok(html) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "text/html")],
            html,
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Get price cache statistics (for debugging)
pub async fn get_price_cache_stats(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.commerce.pricing.cache.get_stats().await;
    (StatusCode::OK, Json(stats)).into_response()
}
