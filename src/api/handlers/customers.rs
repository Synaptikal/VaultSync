//! Customer-related API handlers

use crate::api::AppState;
use crate::core::Customer;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

pub async fn get_customers(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    match state.db.customers.get_all().await {
        Ok(customers) => {
            let limit = params
                .get("limit")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(100);
            let offset = params
                .get("offset")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            let paged: Vec<_> = customers.into_iter().skip(offset).take(limit).collect();
            (StatusCode::OK, Json(paged)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_customer(
    State(state): State<AppState>,
    Json(customer): Json<Customer>,
) -> impl IntoResponse {
    match state.db.customers.insert(&customer).await {
        Ok(_) => (StatusCode::CREATED, Json(customer)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_customer_by_id(
    State(state): State<AppState>,
    Path(customer_uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state.db.customers.get_by_id(customer_uuid).await {
        Ok(Some(customer)) => (StatusCode::OK, Json(customer)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Customer not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_customer_history(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(uuid_str) = params.get("customer_uuid") {
        if let Ok(uuid) = Uuid::parse_str(uuid_str) {
            match state.db.transactions.get_by_customer(uuid).await {
                Ok(transactions) => (StatusCode::OK, Json(transactions)).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
                )
                    .into_response(),
            }
        } else {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid UUID"})),
            )
                .into_response()
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Missing customer_uuid"})),
        )
            .into_response()
    }
}

#[derive(Deserialize)]
pub struct UpdateCreditRequest {
    pub customer_uuid: Uuid,
    pub amount: f64,
}

pub async fn update_store_credit(
    State(state): State<AppState>,
    Json(req): Json<UpdateCreditRequest>,
) -> impl IntoResponse {
    match state
        .db
        .customers
        .update_store_credit(req.customer_uuid, req.amount)
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
