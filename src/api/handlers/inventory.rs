//! Inventory-related API handlers
//!
//! This module contains handlers for inventory management, stock levels, and bulk operations.

use crate::api::error::AppError;
use crate::api::AppState;
use crate::core::InventoryItem;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

/// Get inventory items with pagination, returns joined product data
pub async fn get_inventory(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    let limit = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    let offset = params
        .get("offset")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let items = state
        .commerce
        .inventory
        .get_items_with_products_paginated(limit, offset)
        .await?;
    Ok((StatusCode::OK, Json(items)))
}

/// Add a new inventory item
pub async fn add_inventory(
    State(state): State<AppState>,
    Json(item): Json<InventoryItem>,
) -> Result<impl IntoResponse, AppError> {
    state.commerce.inventory.add_item(item).await?;
    Ok((StatusCode::CREATED, Json(json!({"status": "success"}))))
}

/// Get low stock items below threshold
pub async fn get_low_stock(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let threshold = params
        .get("threshold")
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);
    match state
        .commerce
        .inventory
        .get_low_stock_items(threshold)
        .await
    {
        Ok(items) => (StatusCode::OK, Json(items)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct BulkInventoryRequest {
    pub items: Vec<InventoryItem>,
}

/// Bulk update inventory items
pub async fn bulk_inventory_update(
    State(state): State<AppState>,
    Json(payload): Json<BulkInventoryRequest>,
) -> impl IntoResponse {
    for item in payload.items {
        if let Err(e) = state.commerce.inventory.add_item(item).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response();
        }
    }
    (StatusCode::OK, Json(json!({"status": "success"}))).into_response()
}

/// Get inventory matrix view
pub async fn get_inventory_matrix(State(state): State<AppState>) -> impl IntoResponse {
    match state.commerce.inventory.get_all_items().await {
        Ok(items) => (StatusCode::OK, Json(items)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Get a single inventory item by UUID
pub async fn get_inventory_item(
    State(state): State<AppState>,
    Path(inventory_uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state.commerce.inventory.get_item(inventory_uuid).await {
        Some(item) => (StatusCode::OK, Json(item)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Inventory item not found"})),
        )
            .into_response(),
    }
}

/// Update an inventory item
pub async fn update_inventory_item(
    State(state): State<AppState>,
    Path(inventory_uuid): Path<Uuid>,
    Json(mut item): Json<InventoryItem>,
) -> impl IntoResponse {
    item.inventory_uuid = inventory_uuid;
    match state.commerce.inventory.update_item(item).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "updated"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Delete an inventory item (soft delete)
pub async fn delete_inventory_item(
    State(state): State<AppState>,
    Path(inventory_uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state.db.inventory.delete(inventory_uuid).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "deleted"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
