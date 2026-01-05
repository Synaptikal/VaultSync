//! Transaction-related API handlers

use crate::api::AppState;
use crate::core::{TransactionItem, TransactionType};
use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateTransactionRequest {
    pub customer_uuid: Option<Uuid>,
    pub items: Vec<TransactionItem>,
    pub transaction_type: TransactionType,
}

pub async fn create_transaction(
    State(state): State<AppState>,
    Extension(user): Extension<crate::api::middleware::AuthenticatedUser>,
    Json(req): Json<CreateTransactionRequest>,
) -> impl IntoResponse {
    let user_uuid = Uuid::parse_str(&user.user_uuid).unwrap_or_default();

    let result = match req.transaction_type {
        TransactionType::Sale => {
            state
                .commerce.transactions
                .process_sale(req.customer_uuid, Some(user_uuid), req.items)
                .await
        }
        TransactionType::Buy => {
            state
                .commerce.transactions
                .process_buy(req.customer_uuid, Some(user_uuid), req.items)
                .await
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Unsupported transaction type for this endpoint"})),
            )
                .into_response()
        }
    };

    match result {
        Ok(transaction) => (StatusCode::CREATED, Json(transaction)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct TransactionListQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub customer_uuid: Option<Uuid>,
    pub transaction_type: Option<String>,
}

pub async fn get_transactions(
    State(state): State<AppState>,
    Query(params): Query<TransactionListQuery>,
) -> impl IntoResponse {
    let transactions = if let Some(customer_uuid) = params.customer_uuid {
        state.db.transactions.get_by_customer(customer_uuid).await
    } else {
        state.db.transactions.get_recent(100).await
    };

    match transactions {
        Ok(mut txns) => {
            if let Some(limit) = params.limit {
                txns.truncate(limit as usize);
            }
            (StatusCode::OK, Json(txns)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_transaction_by_id(
    State(state): State<AppState>,
    Path(transaction_uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state.db.transactions.get_by_id(transaction_uuid).await {
        Ok(Some(txn)) => (StatusCode::OK, Json(txn)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Transaction not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

