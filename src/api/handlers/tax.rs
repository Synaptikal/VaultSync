//! Tax-related API handlers
//!
//! Handles tax rate CRUD and default tax rate lookup.

use crate::api::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;

/// Get all configured tax rates
pub async fn get_tax_rates(State(state): State<AppState>) -> impl IntoResponse {
    match state.commerce.taxes.get_all_rates().await {
        Ok(rates) => Json(rates).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct CreateTaxRateRequest {
    pub name: String,
    pub rate: f64,
    pub applies_to_category: Option<String>,
    pub is_default: Option<bool>,
}

/// Create a new tax rate
pub async fn create_tax_rate(
    State(state): State<AppState>,
    Json(req): Json<CreateTaxRateRequest>,
) -> impl IntoResponse {
    let mut tax_rate = crate::services::TaxRate::new(req.name, req.rate);
    tax_rate.applies_to_category = req.applies_to_category;
    if let Some(is_default) = req.is_default {
        tax_rate.is_default = is_default;
    }

    match state.commerce.taxes.create_tax_rate(&tax_rate).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({"rate_id": tax_rate.rate_id})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Get the default tax rate
pub async fn get_default_tax_rate(State(state): State<AppState>) -> impl IntoResponse {
    match state.commerce.taxes.get_default_rate().await {
        Ok(rate) => Json(json!({"rate": rate})).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
