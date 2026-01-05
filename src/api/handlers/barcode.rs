//! Barcode-related API handlers
//!
//! Handles barcode/QR code generation, bulk generation, and product lookup via barcode.

use crate::api::AppState;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

/// Generate a barcode SVG for the given data
pub async fn generate_barcode(
    State(state): State<AppState>,
    Path(data): Path<String>,
) -> impl IntoResponse {
    match state.system.barcode.generate_svg(&data) {
        Ok(svg) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "image/svg+xml")],
            svg,
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
pub struct BulkBarcodeRequest {
    pub items: Vec<String>,
}

/// Generate multiple barcodes in a single request
pub async fn bulk_generate_barcodes(
    State(state): State<AppState>,
    Json(req): Json<BulkBarcodeRequest>,
) -> impl IntoResponse {
    let mut results = Vec::new();
    for data in req.items {
        match state.system.barcode.generate_svg(&data) {
            Ok(svg) => results.push(json!({ "data": data, "svg": svg, "success": true })),
            Err(e) => {
                results.push(json!({ "data": data, "error": e.to_string(), "success": false }))
            }
        }
    }
    Json(results).into_response()
}

/// Generate a QR code SVG for the given data
pub async fn generate_qrcode(
    State(state): State<AppState>,
    Path(data): Path<String>,
) -> impl IntoResponse {
    match state.system.barcode.generate_qr_svg(&data) {
        Ok(svg) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "image/svg+xml")],
            svg,
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Look up a product or inventory item by barcode
pub async fn lookup_by_barcode(
    State(state): State<AppState>,
    Path(barcode): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    // 1. Try Inventory Item by UUID
    if let Ok(uuid) = Uuid::parse_str(&barcode) {
        if let Some(item) = state.commerce.inventory.get_item(uuid).await {
            // Log scan
            let _ = state
                .system
                .barcode
                .log_scan(
                    &barcode,
                    "InventoryLookup",
                    Some("InventoryItem"),
                    Some(item.inventory_uuid),
                    None,
                )
                .await;

            return (
                StatusCode::OK,
                Json(json!({
                    "type": "inventory_item",
                    "data": item
                })),
            )
                .into_response();
        }
    }

    // 2. Try Product by Barcode (UPC)
    let row = sqlx::query("SELECT product_uuid FROM Global_Catalog WHERE barcode = ?")
        .bind(&barcode)
        .fetch_optional(&state.db.pool)
        .await;

    match row {
        Ok(Some(r)) => {
            let product_uuid_str: String =
                sqlx::Row::try_get(&r, "product_uuid").unwrap_or_default();
            if let Ok(uuid) = Uuid::parse_str(&product_uuid_str) {
                // Log scan
                let _ = state
                    .system
                    .barcode
                    .log_scan(&barcode, "ProductLookup", Some("Product"), Some(uuid), None)
                    .await;

                return (
                    StatusCode::OK,
                    Json(json!({
                        "type": "product",
                        "data": {
                            "product_uuid": uuid,
                            "barcode": barcode
                        }
                    })),
                )
                    .into_response();
            }
        }
        Ok(None) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }

    // 3. Try External Lookup (if requested)
    let check_online = params.get("online").map(|s| s == "true").unwrap_or(false);
    if check_online {
        match state.system.catalog.lookup(&barcode).await {
            Ok(Some(item)) => {
                // Log scan (Success External)
                let _ = state
                    .system
                    .barcode
                    .log_scan(&barcode, "ExternalLookup", Some("CatalogItem"), None, None)
                    .await;

                return (
                    StatusCode::OK,
                    Json(json!({
                        "type": "external_catalog",
                        "data": item
                    })),
                )
                    .into_response();
            }
            Ok(None) => {}
            Err(e) => {
                tracing::error!("External lookup failed: {}", e);
            }
        }
    }

    // Log Failed Scan
    let _ = state
        .system
        .barcode
        .log_scan(&barcode, "LookupFailed", None, None, None)
        .await;

    (
        StatusCode::NOT_FOUND,
        Json(json!({"error": "Barcode not found"})),
    )
        .into_response()
}
