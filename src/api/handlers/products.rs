//! Product-related API handlers
//!
//! This module contains all handlers for product CRUD operations and search.

use crate::api::error::AppError;
use crate::api::AppState;
use crate::core::Product;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use uuid::Uuid;

/// Get products with optional search and pagination
/// ARCH-02: Now uses ProductService instead of Database proxy methods
pub async fn get_products(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    let limit = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    // Hard cap limit to prevent abuse
    let limit = std::cmp::min(limit, 1000);
    let offset = params
        .get("offset")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let search = params.get("search").map(|s| s.as_str()).unwrap_or("");

    let products = state.commerce.product.search(search, limit, offset).await?;
    Ok((StatusCode::OK, Json(products)))
}

/// Create a new product
/// ARCH-02: Now uses ProductService instead of Database proxy methods
pub async fn create_product(
    State(state): State<AppState>,
    Json(product): Json<Product>,
) -> Result<impl IntoResponse, AppError> {
    state.commerce.product.upsert(&product).await?;
    Ok((StatusCode::CREATED, Json(product)))
}

/// Search products by name, barcode, or set code
pub async fn search_products(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let query = params.get("q").map(|s| s.as_str()).unwrap_or("");
    let limit = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);
    let offset = params
        .get("offset")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    match state.commerce.product.search(query, limit, offset).await {
        Ok(products) => (StatusCode::OK, Json(products)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Get a single product by UUID
pub async fn get_product_by_id(
    State(state): State<AppState>,
    Path(product_uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state.commerce.product.get_by_id(product_uuid).await {
        Ok(Some(product)) => (StatusCode::OK, Json(product)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Product not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Lookup product by barcode
pub async fn lookup_by_barcode(
    State(state): State<AppState>,
    Path(barcode): Path<String>,
) -> impl IntoResponse {
    match state.system.catalog.lookup(&barcode).await {
        Ok(Some(item)) => (StatusCode::OK, Json(item)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Barcode not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

