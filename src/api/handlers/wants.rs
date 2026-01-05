//! Wants list API handlers
//!
//! Handles customer wants list creation, listing, and inventory updates.

use crate::api::AppState;
use crate::core::InventoryItem;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateWantsListRequest {
    pub customer_uuid: Uuid,
    pub items: Vec<crate::core::WantsItem>,
}

/// Create a new wants list for a customer
pub async fn create_wants_list(
    State(state): State<AppState>,
    Json(req): Json<CreateWantsListRequest>,
) -> impl IntoResponse {
    let wants_list = crate::core::WantsList {
        wants_list_uuid: Uuid::new_v4(),
        customer_uuid: req.customer_uuid,
        items: req.items,
        created_at: Utc::now(),
    };

    match state.db.customers.save_wants_list(&wants_list).await {
        Ok(_) => (StatusCode::CREATED, Json(wants_list)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Get all wants lists for a customer
pub async fn get_wants_lists(
    State(state): State<AppState>,
    Path(customer_uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state.db.customers.get_wants_lists(customer_uuid).await {
        Ok(lists) => (StatusCode::OK, Json(lists)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Update an inventory item
pub async fn update_inventory_item(
    State(state): State<AppState>,
    Path(inventory_uuid): Path<Uuid>,
    Json(item): Json<InventoryItem>,
) -> impl IntoResponse {
    if item.inventory_uuid != inventory_uuid {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "UUID mismatch"})),
        )
            .into_response();
    }

    match state.commerce.inventory.update_item(item).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "updated"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
