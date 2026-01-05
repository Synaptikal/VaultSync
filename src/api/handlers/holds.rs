//! Holds/Layaway-related API handlers
//!
//! Handles creating, managing, and completing holds/layaway orders.

use crate::api::AppState;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

/// Get all holds for a customer
pub async fn get_customer_holds(
    State(state): State<AppState>,
    Path(customer_uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state.commerce.holds.get_customer_holds(customer_uuid).await {
        Ok(holds) => Json(holds).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Get a specific hold by ID
pub async fn get_hold(
    State(state): State<AppState>,
    Path(hold_uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state.commerce.holds.get_hold(hold_uuid).await {
        Ok(Some(hold)) => Json(hold).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Hold not found"})),
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
pub struct CreateHoldRequest {
    pub customer_uuid: Uuid,
    pub items: Vec<HoldItemRequest>,
    pub deposit_amount: f64,
    pub deposit_method: String,
    pub notes: Option<String>,
    pub hold_days: Option<i64>,
}

#[derive(Deserialize)]
pub struct HoldItemRequest {
    pub inventory_uuid: Uuid,
    pub quantity: i32,
    pub unit_price: f64,
}

/// Create a new hold/layaway
pub async fn create_hold(
    State(state): State<AppState>,
    Json(req): Json<CreateHoldRequest>,
) -> impl IntoResponse {
    let request = crate::services::holds::CreateHoldRequest {
        customer_uuid: req.customer_uuid,
        items: req
            .items
            .into_iter()
            .map(|i| crate::services::holds::HoldItemRequest {
                inventory_uuid: i.inventory_uuid,
                quantity: i.quantity,
                unit_price: i.unit_price,
            })
            .collect(),
        deposit_amount: req.deposit_amount,
        deposit_method: req.deposit_method,
        notes: req.notes,
        hold_days: req.hold_days,
    };

    match state.commerce.holds.create_hold(request).await {
        Ok(summary) => (StatusCode::CREATED, Json(summary)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct MakeHoldPaymentRequest {
    pub amount: f64,
    pub payment_method: String,
}

/// Make a payment toward a hold
pub async fn make_hold_payment(
    State(state): State<AppState>,
    Path(hold_uuid): Path<Uuid>,
    Json(req): Json<MakeHoldPaymentRequest>,
) -> impl IntoResponse {
    match state
        .commerce
        .holds
        .make_payment(hold_uuid, req.amount, &req.payment_method)
        .await
    {
        Ok(summary) => Json(summary).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct CancelHoldRequest {
    pub reason: String,
}

/// Cancel a hold
pub async fn cancel_hold(
    State(state): State<AppState>,
    Path(hold_uuid): Path<Uuid>,
    Json(req): Json<CancelHoldRequest>,
) -> impl IntoResponse {
    match state
        .commerce
        .holds
        .cancel_hold(hold_uuid, &req.reason)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "cancelled"}))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Complete a hold (mark as picked up)
pub async fn complete_hold(
    State(state): State<AppState>,
    Path(hold_uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state.commerce.holds.complete_hold(hold_uuid).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "completed"}))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Expire all overdue holds
pub async fn expire_overdue_holds(State(state): State<AppState>) -> impl IntoResponse {
    match state.commerce.holds.expire_overdue_holds().await {
        Ok(expired) => Json(json!({
            "expired_count": expired.len(),
            "expired_ids": expired.iter().map(|u| u.to_string()).collect::<Vec<_>>()
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
