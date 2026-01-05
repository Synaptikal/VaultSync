//! Label generation API handlers
//!
//! Handles inventory and product label generation.

use crate::api::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use uuid::Uuid;

/// Get label data for an inventory item
pub async fn get_inventory_label(
    State(state): State<AppState>,
    Path(inventory_uuid): Path<Uuid>,
) -> impl IntoResponse {
    // Get inventory item
    let item = match state.commerce.inventory.get_item(inventory_uuid).await {
        Some(i) => i,
        None => {
            return (
                StatusCode::NOT_FOUND,
                axum::extract::Json(json!({"error": "Inventory item not found"})),
            )
                .into_response()
        }
    };

    // Get product info
    let product = state
        .commerce
        .product
        .get_by_id(item.product_uuid)
        .await
        .ok()
        .flatten();

    // Generate label data
    let label_data = json!({
        "inventory_uuid": inventory_uuid,
        "product_name": product.as_ref().map(|p| p.name.clone()).unwrap_or_default(),
        "barcode": product.as_ref().and_then(|p| p.barcode.clone()).unwrap_or_else(|| inventory_uuid.to_string()),
        "price": item.specific_price,
        "condition": format!("{:?}", item.condition),
        "location": item.location_tag,
    });

    (StatusCode::OK, axum::extract::Json(label_data)).into_response()
}

/// Get label data for a product
pub async fn get_product_label(
    State(state): State<AppState>,
    Path(product_uuid): Path<Uuid>,
) -> impl IntoResponse {
    let product = match state.commerce.product.get_by_id(product_uuid).await {
        Ok(Some(p)) => p,
        _ => {
            return (
                StatusCode::NOT_FOUND,
                axum::extract::Json(json!({"error": "Product not found"})),
            )
                .into_response()
        }
    };

    // Generate label data
    let label_data = json!({
        "product_uuid": product_uuid,
        "product_name": product.name,
        "barcode": product.barcode,
        "category": format!("{:?}", product.category),
    });

    (StatusCode::OK, axum::extract::Json(label_data)).into_response()
}
