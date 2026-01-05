//! Location and transfer API handlers
//!
//! Handles multi-location management and inventory transfers.

use crate::api::AppState;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

/// Get all locations
pub async fn get_locations(State(state): State<AppState>) -> impl IntoResponse {
    match state.system.locations.get_locations().await {
        Ok(locs) => (StatusCode::OK, Json(locs)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Create or update a location
pub async fn upsert_location(
    State(state): State<AppState>,
    Json(loc): Json<crate::services::Location>,
) -> impl IntoResponse {
    match state.system.locations.upsert_location(loc).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "success"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct CreateTransferRequest {
    pub source_location: String,
    pub target_location: String,
    pub requested_by: Uuid,
    pub items: Vec<crate::services::TransferItem>,
}

/// Create an inventory transfer request
pub async fn create_transfer(
    State(state): State<AppState>,
    Json(req): Json<CreateTransferRequest>,
) -> impl IntoResponse {
    match state
        .system
        .locations
        .create_transfer_request(
            req.source_location,
            req.target_location,
            req.requested_by,
            req.items,
        )
        .await
    {
        Ok(uuid) => (
            StatusCode::CREATED,
            Json(json!({"status": "created", "transfer_uuid": uuid})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct UpdateTransferStatusRequest {
    pub status: crate::services::TransferStatus,
    pub user_uuid: Uuid,
}

/// Update transfer status (approve, ship, receive, etc.)
pub async fn update_transfer_status(
    State(state): State<AppState>,
    Path(transfer_uuid): Path<Uuid>,
    Json(req): Json<UpdateTransferStatusRequest>,
) -> impl IntoResponse {
    match state
        .system
        .locations
        .update_transfer_status(transfer_uuid, req.status, req.user_uuid)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "updated"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
