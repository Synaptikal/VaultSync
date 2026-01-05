//! Serialized inventory API handlers
//!
//! Handles unique/serialized item details, grading, and certificates.

use crate::api::AppState;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use uuid::Uuid;

/// Get serialized item details
pub async fn get_serialized_details(
    State(state): State<AppState>,
    Path(inventory_uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state
        .system
        .serialized
        .get_serialized_item(inventory_uuid)
        .await
    {
        Ok(Some(d)) => (StatusCode::OK, Json(d)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Serialized details not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Update serialized item details
pub async fn update_serialized_details(
    State(state): State<AppState>,
    Path(inventory_uuid): Path<Uuid>,
    Json(item): Json<crate::services::SerializedItem>,
) -> impl IntoResponse {
    match state
        .system
        .serialized
        .update_serialized_item(inventory_uuid, &item)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "success"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Add grading info to a serialized item
pub async fn add_grading(
    State(state): State<AppState>,
    Path(inventory_uuid): Path<Uuid>,
    Json(grading): Json<crate::services::GradingInfo>,
) -> impl IntoResponse {
    match state
        .system
        .serialized
        .add_grading(inventory_uuid, grading)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "success"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Add certificate info to a serialized item
pub async fn add_certificate(
    State(state): State<AppState>,
    Path(inventory_uuid): Path<Uuid>,
    Json(cert): Json<crate::services::CertificateInfo>,
) -> impl IntoResponse {
    match state
        .system
        .serialized
        .add_certificate(inventory_uuid, cert)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "success"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
