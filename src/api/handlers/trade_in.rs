//! Trade-in protection API handlers
//!
//! Handles trade-in eligibility checks, history, and suspicious activity logging.

use crate::api::AppState;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct TradeInCheckRequest {
    pub customer_uuid: Uuid,
    pub proposed_value: f64,
}

/// Check if a customer is eligible for trade-in
pub async fn check_trade_in_eligibility(
    State(state): State<AppState>,
    Json(req): Json<TradeInCheckRequest>,
) -> impl IntoResponse {
    match state
        .commerce
        .trade_in
        .check_trade_in(req.customer_uuid, req.proposed_value)
        .await
    {
        Ok(check) => (StatusCode::OK, Json(check)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Get trade-in history for a customer
pub async fn get_trade_in_history(
    State(state): State<AppState>,
    Path(customer_uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state
        .commerce
        .trade_in
        .get_customer_history(customer_uuid)
        .await
    {
        Ok(history) => (StatusCode::OK, Json(history)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct SuspiciousActivityLogRequest {
    pub customer_uuid: Uuid,
    pub activity_type: crate::services::SuspiciousActivityType,
    pub description: String,
    pub severity: crate::services::AlertSeverity,
}

/// Log suspicious trade-in activity
pub async fn log_suspicious_activity(
    State(state): State<AppState>,
    Json(req): Json<SuspiciousActivityLogRequest>,
) -> impl IntoResponse {
    match state
        .commerce
        .trade_in
        .log_suspicious_activity(
            req.customer_uuid,
            req.activity_type,
            &req.description,
            req.severity,
        )
        .await
    {
        Ok(_) => (StatusCode::CREATED, Json(json!({"status": "logged"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
