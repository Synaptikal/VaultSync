//! Buylist-related API handlers
//!
//! Handles trade-in quotes, buylist processing, and trade-in transactions.

use crate::api::AppState;
use crate::buylist::{BuylistItem, PaymentMethod};
use crate::core::TransactionItem;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct BuylistProcessRequest {
    pub items: Vec<BuylistItem>,
    pub customer_uuid: Option<Uuid>,
    pub payment_method: PaymentMethod,
}

#[derive(Deserialize)]
pub struct TradeInProcessRequest {
    pub trade_in_items: Vec<BuylistItem>,
    pub purchase_items: Vec<TransactionItem>,
    pub customer_uuid: Option<Uuid>,
}

/// Get an instant quote for a buylist item
pub async fn get_buylist_quote(
    State(state): State<AppState>,
    Json(item): Json<BuylistItem>,
) -> impl IntoResponse {
    match state
        .commerce
        .buylist
        .calculate_instant_quote(item.product_uuid, item.condition)
        .await
    {
        Ok(quote) => (StatusCode::OK, Json(quote)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Process a buylist transaction (buying items from customer)
pub async fn process_buylist(
    State(state): State<AppState>,
    Json(payload): Json<BuylistProcessRequest>,
) -> impl IntoResponse {
    match state
        .commerce
        .buylist
        .process_buylist_transaction(payload.customer_uuid, payload.items, payload.payment_method)
        .await
    {
        Ok(transaction) => (StatusCode::OK, Json(transaction)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Process a trade-in transaction (customer trades items for credit toward purchase)
pub async fn process_trade_in(
    State(state): State<AppState>,
    Json(payload): Json<TradeInProcessRequest>,
) -> impl IntoResponse {
    match state
        .commerce
        .buylist
        .process_trade_in_transaction(
            payload.customer_uuid,
            payload.trade_in_items,
            payload.purchase_items,
        )
        .await
    {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
