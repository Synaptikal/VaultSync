//! Notification scheduler API handlers
//!
//! Handles scheduled notification tasks and wants list matching.

use crate::api::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use uuid::Uuid;

/// Run all scheduled notification tasks (event reminders, hold expirations)
/// Admin endpoint to manually trigger scheduled notifications
pub async fn run_scheduled_notifications(State(state): State<AppState>) -> impl IntoResponse {
    match state
        .system
        .notification_scheduler
        .run_scheduled_tasks()
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            axum::extract::Json(json!({"message": "Scheduled notification tasks completed"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::extract::Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Check wants list matches for a specific product
/// Call this when new inventory is added
pub async fn check_wants_list_matches(
    State(state): State<AppState>,
    Path(product_uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state
        .system
        .notification_scheduler
        .check_wants_list_matches(product_uuid)
        .await
    {
        Ok(notified) => (
            StatusCode::OK,
            axum::extract::Json(json!({
                "message": "Wants list check completed",
                "notified_customers": notified.len(),
                "customer_uuids": notified.iter().map(|u| u.to_string()).collect::<Vec<_>>()
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::extract::Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
