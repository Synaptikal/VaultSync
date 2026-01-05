//! Returns-related API handlers
//!
//! Handles return policy, reasons, and return processing.

use crate::api::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;

/// Get the store's return policy
pub async fn get_return_policy(State(state): State<AppState>) -> impl IntoResponse {
    let policy = state.commerce.returns.get_policy();
    (StatusCode::OK, Json(policy))
}

/// Get available return reason codes
pub async fn get_return_reasons() -> impl IntoResponse {
    let reasons = crate::services::ReturnsService::get_reason_codes();
    (StatusCode::OK, Json(reasons))
}

/// Process a return
pub async fn process_return(
    State(state): State<AppState>,
    Json(request): Json<crate::services::ReturnRequest>,
) -> impl IntoResponse {
    match state.commerce.returns.process_return(request).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
