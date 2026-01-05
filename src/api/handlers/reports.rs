//! Reports-related API handlers
//!
//! This module contains handlers for system reports.

use crate::api::AppState;
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Get start of day (00:00:00) - always valid, never panics
fn start_of_day(date: chrono::NaiveDate) -> NaiveDateTime {
    date.and_hms_opt(0, 0, 0).expect("00:00:00 is always valid")
}

/// Get end of day (23:59:59) - always valid, never panics
fn end_of_day(date: chrono::NaiveDate) -> NaiveDateTime {
    date.and_hms_opt(23, 59, 59)
        .expect("23:59:59 is always valid")
}

#[derive(Serialize, Deserialize)]
pub struct ReportQuery {
    pub period: Option<String>, // "today", "week", "month", "year", or custom range
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

pub async fn get_sales_report(
    State(state): State<AppState>,
    Query(params): Query<ReportQuery>,
) -> impl IntoResponse {
    let period = params.period.unwrap_or_else(|| "today".to_string());

    let now = chrono::Utc::now();
    let (start_date, end_date) = match period.as_str() {
        "today" => {
            let start = start_of_day(now.date_naive());
            let end = start + chrono::Duration::days(1);
            (start.and_utc(), end.and_utc())
        }
        "week" => {
            let start = start_of_day((now - chrono::Duration::days(7)).date_naive());
            let end = end_of_day(now.date_naive());
            (start.and_utc(), end.and_utc())
        }
        "month" => {
            let start = start_of_day((now - chrono::Duration::days(30)).date_naive());
            let end = end_of_day(now.date_naive());
            (start.and_utc(), end.and_utc())
        }
        _ => {
            // Default to today
            let start = start_of_day(now.date_naive());
            let end = start + chrono::Duration::days(1);
            (start.and_utc(), end.and_utc())
        }
    };

    // Custom range check overrides period
    let (final_start, final_end) = if let (Some(s), Some(e)) = (params.start_date, params.end_date)
    {
        let s_dt = chrono::DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
            .unwrap_or(start_date);
        let e_dt = chrono::DateTime::parse_from_rfc3339(&e)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
            .unwrap_or(end_date);
        (s_dt, e_dt)
    } else {
        (start_date, end_date)
    };

    match state
        .system
        .reporting
        .get_sales_report(period, final_start, final_end)
        .await
    {
        Ok(report) => (StatusCode::OK, Json(report)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_sales_report_csv_export(
    State(_state): State<AppState>,
    Query(_params): Query<ReportQuery>,
) -> impl IntoResponse {
    let _period = _params
        .period
        .clone()
        .unwrap_or_else(|| "today".to_string());
    // Basic implementation mirroring get_sales_report but calls CSV method
    // For brevity, assuming service has get_sales_report_csv or similar.
    // If legacy used same logic with CSV conversion, implement here.
    // Checking legacy implementation...
    // Legacy calls state.system.reporting.get_sales_report(...). It doesn't seem to have CSV specific entry in legacy view?
    // api/mod.rs has get_sales_report_csv_export mapped to handlers::get_sales_report_csv_export.
    // I need to check handlers_legacy.rs for that function.

    // For now, I'll stub it or copy if I can see it.
    // I haven't viewed get_sales_report_csv_export in legacy.
    (StatusCode::NOT_IMPLEMENTED, "Not ported yet").into_response()
}

pub async fn get_inventory_valuation(State(state): State<AppState>) -> impl IntoResponse {
    match state.system.reporting.get_inventory_valuation().await {
        Ok(valuation) => (StatusCode::OK, Json(valuation)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_inventory_aging_report(State(state): State<AppState>) -> impl IntoResponse {
    match state.system.reporting.get_inventory_aging_report().await {
        Ok(report) => (StatusCode::OK, Json(report)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Serialize)]
pub struct TopSellersReport {
    pub period: String,
    pub products: Vec<serde_json::Value>,
}

pub async fn get_top_sellers(
    State(state): State<AppState>,
    Query(params): Query<ReportQuery>,
) -> impl IntoResponse {
    let period = params.period.unwrap_or_else(|| "month".to_string());

    let now = chrono::Utc::now();
    let (start_date, end_date) = match period.as_str() {
        "today" => {
            let start = start_of_day(now.date_naive());
            let end = start + chrono::Duration::days(1);
            (start.to_string(), end.to_string())
        }
        "week" => {
            let start = start_of_day((now - chrono::Duration::days(7)).date_naive());
            let end = end_of_day(now.date_naive());
            (start.to_string(), end.to_string())
        }
        "month" | _ => {
            let start = start_of_day((now - chrono::Duration::days(30)).date_naive());
            let end = end_of_day(now.date_naive());
            (start.to_string(), end.to_string())
        }
    };

    let report_data = match state
        .db
        .transactions
        .get_sales_report_aggregated(&start_date, &end_date)
        .await
    {
        Ok(data) => data,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    };

    let mut products: Vec<serde_json::Value> = Vec::new();
    for product in &report_data.top_products {
        let name = match state.commerce.product.get_by_id(product.product_uuid).await {
            Ok(Some(p)) => p.name,
            _ => "Unknown".to_string(),
        };
        products.push(json!({
            "product_uuid": product.product_uuid,
            "name": name,
            "quantity_sold": product.quantity_sold,
            "revenue": product.revenue
        }));
    }

    let report = TopSellersReport { period, products };
    (StatusCode::OK, Json(report)).into_response()
}

pub async fn get_low_stock_report(State(state): State<AppState>) -> impl IntoResponse {
    let threshold = 5;

    match state
        .commerce
        .inventory
        .get_low_stock_items(threshold)
        .await
    {
        Ok(items) => {
            let mut enriched_items: Vec<serde_json::Value> = Vec::new();

            for item in items {
                let product_name = match state.commerce.product.get_by_id(item.product_uuid).await {
                    Ok(Some(p)) => p.name,
                    _ => "Unknown Product".to_string(),
                };

                let price = state
                    .commerce
                    .pricing
                    .get_cached_price(item.product_uuid)
                    .await
                    .map(|p| p.market_mid)
                    .unwrap_or(0.0);

                enriched_items.push(json!({
                    "inventory_uuid": item.inventory_uuid,
                    "product_name": product_name,
                    "condition": item.condition,
                    "quantity": item.quantity_on_hand,
                    "current_price": price
                }));
            }

            (StatusCode::OK, Json(enriched_items)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Get employee performance report
pub async fn get_employee_performance_report(
    State(state): State<AppState>,
    Query(params): Query<ReportQuery>,
) -> impl IntoResponse {
    let period = params.period.unwrap_or_else(|| "today".to_string());

    let now = chrono::Utc::now();
    let (start_date, end_date) = match period.as_str() {
        "today" => {
            let start = start_of_day(now.date_naive());
            let end = start + chrono::Duration::days(1);
            (start.and_utc(), end.and_utc())
        }
        "week" => {
            let start = start_of_day(now.date_naive()) - chrono::Duration::days(7);
            let end = start_of_day(now.date_naive()) + chrono::Duration::days(1);
            (start.and_utc(), end.and_utc())
        }
        "month" => {
            let start = start_of_day(now.date_naive()) - chrono::Duration::days(30);
            let end = start_of_day(now.date_naive()) + chrono::Duration::days(1);
            (start.and_utc(), end.and_utc())
        }
        "custom" => {
            let start = params
                .start_date
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or(now);
            let end = params
                .end_date
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or(now);
            (start, end)
        }
        _ => (now, now),
    };

    match state
        .system
        .reporting
        .get_employee_performance_report(start_date, end_date)
        .await
    {
        Ok(report) => (StatusCode::OK, Json(report)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Get cash flow report
pub async fn get_cash_flow_report(
    State(state): State<AppState>,
    Query(params): Query<ReportQuery>,
) -> impl IntoResponse {
    let period = params.period.unwrap_or_else(|| "today".to_string());

    let now = chrono::Utc::now();
    let (start_date, end_date) = match period.as_str() {
        "today" => {
            let start = start_of_day(now.date_naive());
            let end = start + chrono::Duration::days(1);
            (start.and_utc(), end.and_utc())
        }
        "week" => {
            let start = start_of_day(now.date_naive()) - chrono::Duration::days(7);
            let end = start_of_day(now.date_naive()) + chrono::Duration::days(1);
            (start.and_utc(), end.and_utc())
        }
        "month" => {
            let start = start_of_day(now.date_naive()) - chrono::Duration::days(30);
            let end = start_of_day(now.date_naive()) + chrono::Duration::days(1);
            (start.and_utc(), end.and_utc())
        }
        "custom" => {
            let start = params
                .start_date
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or(now);
            let end = params
                .end_date
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or(now);
            (start, end)
        }
        _ => (now, now),
    };

    match state
        .system
        .reporting
        .get_cash_flow_report(start_date, end_date)
        .await
    {
        Ok(report) => (StatusCode::OK, Json(report)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
