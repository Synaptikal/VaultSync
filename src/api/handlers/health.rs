//! Health, monitoring, and audit log handlers

use crate::api::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

/// Basic health check endpoint - returns immediately with minimal check
/// This should be fast for load balancer probes
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    // Quick database check
    let db_ok = sqlx::query("SELECT 1")
        .fetch_one(&state.db.pool)
        .await
        .is_ok();

    let status = if db_ok { "healthy" } else { "unhealthy" };
    let status_code = if db_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(json!({
            "status": status,
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "database": if db_ok { "connected" } else { "disconnected" }
        })),
    )
}

pub async fn health_check_detailed(State(state): State<AppState>) -> impl IntoResponse {
    let health_service = crate::monitoring::HealthService::new(state.db.clone());
    let health = health_service.check_detailed().await;

    let status_code = match health.status {
        crate::monitoring::HealthStatus::Healthy => StatusCode::OK,
        crate::monitoring::HealthStatus::Degraded => StatusCode::OK,
        crate::monitoring::HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(health))
}

pub async fn get_alerts(State(state): State<AppState>) -> impl IntoResponse {
    let alerting_service = crate::monitoring::AlertingService::new(state.db.clone());
    let alerts = alerting_service.check_all().await;

    let severity_counts = alerts.iter().fold((0, 0, 0), |mut acc, alert| {
        match alert.severity {
            crate::monitoring::AlertSeverity::Info => acc.0 += 1,
            crate::monitoring::AlertSeverity::Warning => acc.1 += 1,
            crate::monitoring::AlertSeverity::Critical => acc.2 += 1,
        }
        acc
    });

    (
        StatusCode::OK,
        Json(json!({
            "count": alerts.len(),
            "info_count": severity_counts.0,
            "warning_count": severity_counts.1,
            "critical_count": severity_counts.2,
            "alerts": alerts
        })),
    )
}

#[derive(Deserialize)]
pub struct AuditLogQuery {
    pub table_name: Option<String>,
    pub record_uuid: Option<Uuid>,
    pub user_uuid: Option<Uuid>,
    pub limit: Option<i64>,
}

pub async fn get_audit_log(
    State(state): State<AppState>,
    Query(params): Query<AuditLogQuery>,
) -> impl IntoResponse {
    let audit_service = crate::monitoring::AuditLogService::new(state.db.pool.clone());

    if let Err(e) = audit_service.init().await {
        tracing::warn!("Failed to init audit log table: {}", e);
    }

    let limit = params.limit.unwrap_or(100).min(1000);

    match audit_service
        .query(
            params.table_name.as_deref(),
            params.record_uuid,
            params.user_uuid,
            None,
            limit,
        )
        .await
    {
        Ok(entries) => (
            StatusCode::OK,
            Json(json!({
                "count": entries.len(),
                "entries": entries
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_record_audit_history(
    State(state): State<AppState>,
    Path((table_name, record_uuid)): Path<(String, Uuid)>,
) -> impl IntoResponse {
    let audit_service = crate::monitoring::AuditLogService::new(state.db.pool.clone());

    match audit_service
        .get_record_history(&table_name, record_uuid)
        .await
    {
        Ok(entries) => (
            StatusCode::OK,
            Json(json!({
                "table_name": table_name,
                "record_uuid": record_uuid.to_string(),
                "count": entries.len(),
                "history": entries
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
