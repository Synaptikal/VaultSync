//! Dashboard API handlers
//!
//! Handles dashboard statistics and metrics.

use crate::api::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
pub struct DashboardStats {
    pub total_products: i64,
    pub total_customers: i64,
    pub total_inventory_items: i64,
    pub total_inventory_value: f64,
    pub low_stock_count: i64,
    pub today_sales: f64,
    pub today_transactions: i64,
    pub pending_sync_changes: i64,
}

/// Get dashboard statistics
pub async fn get_dashboard_stats(State(state): State<AppState>) -> impl IntoResponse {
    // Return real stats using proper aggregation queries
    match state.db.transactions.get_dashboard_metrics().await {
        Ok(metrics) => {
            // Products count
            let products_count = match state.db.products.get_all().await {
                Ok(p) => p.len() as i64,
                Err(_) => 0,
            };

            // Customers count
            let customers_count = match state.db.customers.get_all().await {
                Ok(c) => c.len() as i64,
                Err(_) => 0,
            };

            // Inventory count using optimized query
            let inventory_count = state.db.inventory.get_total_count().await.unwrap_or(0);

            // Low stock count using optimized query
            let low_stock_count = state.db.inventory.get_low_stock_count(5).await.unwrap_or(0);

            // Sync pending changes
            let sync_pending = match state.db.sync.get_changes_since(0, 10000).await {
                Ok(c) => c.len() as i64,
                Err(_) => 0,
            };

            // Calculate inventory value (sum of market_mid * quantity)
            let total_inventory_value = match state.commerce.inventory.get_all_items().await {
                Ok(items) => {
                    let mut total = 0.0;
                    for item in items.iter().take(1000) {
                        // Cap to avoid memory issues
                        if let Some(price) = state
                            .commerce
                            .pricing
                            .get_cached_price(item.product_uuid)
                            .await
                        {
                            total += price.market_mid * item.quantity_on_hand as f64;
                        }
                    }
                    total
                }
                Err(_) => 0.0,
            };

            (
                StatusCode::OK,
                Json(json!({
                    "total_products": products_count,
                    "total_customers": customers_count,
                    "total_inventory_items": inventory_count,
                    "total_inventory_value": total_inventory_value,
                    "low_stock_count": low_stock_count,
                    "today_sales": metrics.total_sales_today,
                    "today_transactions": metrics.transaction_count_today,
                    "pending_sync_changes": sync_pending,
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

/// Get system alerts - placeholder implementation
pub async fn get_alerts(State(_state): State<AppState>) -> impl IntoResponse {
    // Return empty alerts for now - monitoring service not yet available
    (StatusCode::OK, Json(json!({"alerts": []}))).into_response()
}
