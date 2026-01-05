//! Audit-related API handlers
//!
//! Handles inventory reconciliation, conflict resolution, and blind count audit.

use crate::api::AppState;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

/// Get pending inventory conflicts
pub async fn get_conflicts(State(state): State<AppState>) -> impl IntoResponse {
    match state.system.audit.get_pending_conflicts().await {
        Ok(conflicts) => Json(conflicts).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct SubmitBlindCountRequest {
    pub location_tag: String,
    pub items: Vec<(Uuid, i32)>,
}

/// Submit a blind count for inventory audit
pub async fn submit_blind_count(
    State(state): State<AppState>,
    Json(payload): Json<SubmitBlindCountRequest>,
) -> impl IntoResponse {
    match state
        .system
        .audit
        .submit_blind_count(payload.location_tag, payload.items)
        .await
    {
        Ok(conflicts) => Json(conflicts).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct ResolveConflictRequest {
    pub resolution: String,
}

/// Resolve a specific inventory conflict
pub async fn resolve_conflict(
    State(state): State<AppState>,
    Path(conflict_uuid): Path<Uuid>,
    Json(payload): Json<ResolveConflictRequest>,
) -> impl IntoResponse {
    use std::str::FromStr;
    let resolution = match crate::audit::ResolutionStatus::from_str(&payload.resolution) {
        Ok(s) => s,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid resolution status"})),
            )
                .into_response()
        }
    };

    match state
        .system
        .audit
        .resolve_conflict(conflict_uuid, resolution)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "resolved"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
