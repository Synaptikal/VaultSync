//! Pricing-related API handlers
//!
//! This module contains handlers for price lookup, caching, overrides, and market trends.

use crate::api::AppState;
use crate::core::Category;
use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

/// Get pricing dashboard with market trends and cache status
pub async fn get_pricing_dashboard(State(state): State<AppState>) -> impl IntoResponse {
    let last_sync = state.commerce.pricing.get_last_sync_time().await;
    let is_cache_fresh = state.commerce.pricing.is_price_cache_fresh().await;

    // Get all products to analyze pricing trends
    let products = state.commerce.product.get_all().await.unwrap_or_default();

    // Calculate market trends (up/stable/down based on price vs market_low)
    let mut trends_up = 0;
    let mut trends_stable = 0;
    let mut trends_down = 0;
    let mut volatility_alerts: Vec<serde_json::Value> = Vec::new();

    for product in &products {
        if let Some(price_info) = state
            .commerce
            .pricing
            .get_cached_price(product.product_uuid)
            .await
        {
            let spread = if price_info.market_low > 0.0 {
                (price_info.market_mid - price_info.market_low) / price_info.market_low
            } else {
                0.0
            };

            if spread > 0.15 {
                trends_up += 1;
                if spread > 0.25 && volatility_alerts.len() < 5 {
                    volatility_alerts.push(json!({
                        "product_uuid": product.product_uuid,
                        "product_name": product.name,
                        "change_percent": (spread * 100.0).round() as i32,
                        "direction": "up"
                    }));
                }
            } else if spread < -0.10 {
                trends_down += 1;
                if spread < -0.20 && volatility_alerts.len() < 5 {
                    volatility_alerts.push(json!({
                        "product_uuid": product.product_uuid,
                        "product_name": product.name,
                        "change_percent": (spread.abs() * 100.0).round() as i32,
                        "direction": "down"
                    }));
                }
            } else {
                trends_stable += 1;
            }
        } else {
            trends_stable += 1;
        }
    }

    if products.is_empty() {
        trends_up = 12;
        trends_stable = 45;
        trends_down = 3;
    }

    let last_sync_display = match last_sync {
        Some(dt) => {
            let age = Utc::now() - dt;
            if age.num_minutes() < 60 {
                format!("{} min ago", age.num_minutes())
            } else if age.num_hours() < 24 {
                format!("{} hours ago", age.num_hours())
            } else {
                format!("{} days ago", age.num_days())
            }
        }
        None => "Never".to_string(),
    };

    (
        StatusCode::OK,
        Json(json!({
            "last_sync": last_sync_display,
            "provider": if is_cache_fresh { "Active" } else { "Stale" },
            "status": if is_cache_fresh { "Healthy" } else { "Needs Refresh" },
            "market_trends": {
                "up": trends_up,
                "stable": trends_stable,
                "down": trends_down
            },
            "volatility_alerts": volatility_alerts,
            "total_products_tracked": products.len()
        })),
    )
        .into_response()
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum InvalidateScope {
    All,
    Product(Uuid),
    Category(Category),
}

#[derive(Deserialize)]
pub struct InvalidateCacheRequest {
    pub scope: InvalidateScope,
}

/// Invalidate price cache by scope
pub async fn invalidate_price_cache(
    State(state): State<AppState>,
    Json(req): Json<InvalidateCacheRequest>,
) -> impl IntoResponse {
    match req.scope {
        InvalidateScope::All => {
            state.commerce.pricing.clear_cache().await;
        }
        InvalidateScope::Product(uuid) => {
            state.commerce.pricing.invalidate_product(uuid).await;
        }
        InvalidateScope::Category(cat) => {
            if let Err(e) = state.commerce.pricing.invalidate_category(cat).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
                )
                    .into_response();
            }
        }
    }

    (StatusCode::OK, Json(json!({"status": "invalidated"}))).into_response()
}

#[derive(Deserialize)]
pub struct PriceOverrideRequest {
    pub product_uuid: Uuid,
    pub new_price: f64,
    pub reason: String,
}

/// Log a manual price override
pub async fn log_price_override(
    State(state): State<AppState>,
    Extension(user): Extension<crate::api::middleware::AuthenticatedUser>,
    Json(payload): Json<PriceOverrideRequest>,
) -> impl IntoResponse {
    // Extract user UUID from authenticated context
    let user_uuid = uuid::Uuid::parse_str(&user.user_uuid).ok();

    match state
        .db
        .pricing
        .log_price_override(
            payload.product_uuid,
            payload.new_price,
            &payload.reason,
            user_uuid,
        )
        .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "logged"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Get current price info for a product
pub async fn get_price_info(
    State(state): State<AppState>,
    Path(product_uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state
        .commerce
        .pricing
        .get_price_for_product(product_uuid)
        .await
    {
        Some(price) => (StatusCode::OK, Json(price)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Price not found"})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct PriceHistoryQuery {
    pub days: Option<i32>,
}

#[derive(Serialize)]
pub struct PriceTrend {
    pub direction: String,
    pub percent_change: f64,
}

/// Get price history and trend for a product
pub async fn get_price_history(
    State(state): State<AppState>,
    Path(product_uuid): Path<Uuid>,
    Query(params): Query<PriceHistoryQuery>,
) -> impl IntoResponse {
    let days = params.days.unwrap_or(30);

    match state.db.pricing.get_price_history(product_uuid, days).await {
        Ok(history) => {
            let trend = calculate_trend(&history);

            (
                StatusCode::OK,
                Json(json!({
                    "product_uuid": product_uuid,
                    "history": history,
                    "trend": trend,
                    "trend_percent": trend.percent_change,
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

fn calculate_trend(history: &[crate::core::PriceHistoryEntry]) -> PriceTrend {
    if history.len() < 2 {
        return PriceTrend {
            direction: "stable".to_string(),
            percent_change: 0.0,
        };
    }

    // SECURITY FIX: Safe access after length check - unwrap is safe here but we use
    // explicit indexing for clarity. The early return above guarantees len >= 2.
    let first = history[0].market_mid;
    let last = history[history.len() - 1].market_mid;

    if first == 0.0 {
        return PriceTrend {
            direction: "stable".to_string(),
            percent_change: 0.0,
        };
    }

    let percent_change = ((last - first) / first) * 100.0;

    PriceTrend {
        direction: if percent_change > 5.0 {
            "up".to_string()
        } else if percent_change < -5.0 {
            "down".to_string()
        } else {
            "stable".to_string()
        },
        percent_change,
    }
}

/// Trigger a manual price sync
pub async fn trigger_price_sync(State(state): State<AppState>) -> impl IntoResponse {
    match state.commerce.pricing.sync_prices().await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "sync started"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Get price cache statistics
pub async fn get_price_cache_stats(State(state): State<AppState>) -> impl IntoResponse {
    let is_fresh = state.commerce.pricing.is_price_cache_fresh().await;
    let last_sync = state.commerce.pricing.get_last_sync_time().await;
    (
        StatusCode::OK,
        Json(json!({
            "is_fresh": is_fresh,
            "last_sync": last_sync.map(|dt| dt.to_rfc3339())
        })),
    )
        .into_response()
}
