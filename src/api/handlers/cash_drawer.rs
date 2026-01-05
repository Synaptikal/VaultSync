//! Cash drawer API handlers
//!
//! Handles cash drawer operations, shift management, and cash counts.

use crate::api::AppState;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

/// Open the cash drawer (returns ESC/POS command)
pub async fn open_cash_drawer(State(state): State<AppState>) -> impl IntoResponse {
    let command = state.system.cash_drawer.get_open_drawer_command();

    // Return the ESC/POS command bytes as base64 for client to send to hardware
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&command);

    (
        StatusCode::OK,
        Json(json!({
            "status": "command_generated",
            "command_base64": encoded,
            "message": "Send this to the cash drawer device"
        })),
    )
}

#[derive(Deserialize)]
pub struct CashCountRequest {
    pub shift_uuid: Option<Uuid>,
    pub count_type: String,
    pub pennies: i32,
    pub nickels: i32,
    pub dimes: i32,
    pub quarters: i32,
    pub ones: i32,
    pub fives: i32,
    pub tens: i32,
    pub twenties: i32,
    pub fifties: i32,
    pub hundreds: i32,
    pub notes: Option<String>,
    pub counted_by: Option<Uuid>,
}

/// Record a cash count
pub async fn record_cash_count(
    State(state): State<AppState>,
    Json(req): Json<CashCountRequest>,
) -> impl IntoResponse {
    let total = crate::services::CashDrawerService::calculate_total(
        req.pennies,
        req.nickels,
        req.dimes,
        req.quarters,
        req.ones,
        req.fives,
        req.tens,
        req.twenties,
        req.fifties,
        req.hundreds,
    );

    let count_type = match req.count_type.as_str() {
        "shift_open" => crate::services::CashCountType::ShiftOpen,
        "shift_close" => crate::services::CashCountType::ShiftClose,
        "drop_safe" => crate::services::CashCountType::DropSafe,
        "audit" => crate::services::CashCountType::Audit,
        _ => crate::services::CashCountType::Adjustment,
    };

    let count = crate::services::CashCount {
        count_uuid: Uuid::new_v4(),
        shift_uuid: req.shift_uuid,
        count_type,
        pennies: req.pennies,
        nickels: req.nickels,
        dimes: req.dimes,
        quarters: req.quarters,
        ones: req.ones,
        fives: req.fives,
        tens: req.tens,
        twenties: req.twenties,
        fifties: req.fifties,
        hundreds: req.hundreds,
        total_amount: total,
        counted_by: req.counted_by,
        counted_at: chrono::Utc::now(),
        notes: req.notes,
    };

    match state.system.cash_drawer.record_count(&count).await {
        Ok(uuid) => (
            StatusCode::OK,
            Json(json!({
                "count_uuid": uuid,
                "total_amount": total
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

#[derive(Deserialize)]
pub struct OpenShiftRequest {
    pub user_uuid: Uuid,
    pub terminal_id: String,
    pub opening_count: Option<CashCountRequest>,
}

/// Open a new shift
pub async fn open_shift(
    State(state): State<AppState>,
    Json(req): Json<OpenShiftRequest>,
) -> impl IntoResponse {
    let opening_count = req.opening_count.map(|c| {
        let total = crate::services::CashDrawerService::calculate_total(
            c.pennies, c.nickels, c.dimes, c.quarters, c.ones, c.fives, c.tens, c.twenties,
            c.fifties, c.hundreds,
        );
        crate::services::CashCount {
            count_uuid: Uuid::new_v4(),
            shift_uuid: None,
            count_type: crate::services::CashCountType::ShiftOpen,
            pennies: c.pennies,
            nickels: c.nickels,
            dimes: c.dimes,
            quarters: c.quarters,
            ones: c.ones,
            fives: c.fives,
            tens: c.tens,
            twenties: c.twenties,
            fifties: c.fifties,
            hundreds: c.hundreds,
            total_amount: total,
            counted_by: c.counted_by,
            counted_at: chrono::Utc::now(),
            notes: c.notes,
        }
    });

    match state
        .system
        .cash_drawer
        .open_shift(req.user_uuid, &req.terminal_id, opening_count.as_ref())
        .await
    {
        Ok(shift) => (StatusCode::OK, Json(shift)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct CloseShiftRequest {
    pub closing_count: CashCountRequest,
}

/// Close a shift
pub async fn close_shift(
    State(state): State<AppState>,
    Path(shift_uuid): Path<Uuid>,
    Json(req): Json<CloseShiftRequest>,
) -> impl IntoResponse {
    let c = req.closing_count;
    let total = crate::services::CashDrawerService::calculate_total(
        c.pennies, c.nickels, c.dimes, c.quarters, c.ones, c.fives, c.tens, c.twenties, c.fifties,
        c.hundreds,
    );

    let count = crate::services::CashCount {
        count_uuid: Uuid::new_v4(),
        shift_uuid: Some(shift_uuid),
        count_type: crate::services::CashCountType::ShiftClose,
        pennies: c.pennies,
        nickels: c.nickels,
        dimes: c.dimes,
        quarters: c.quarters,
        ones: c.ones,
        fives: c.fives,
        tens: c.tens,
        twenties: c.twenties,
        fifties: c.fifties,
        hundreds: c.hundreds,
        total_amount: total,
        counted_by: c.counted_by,
        counted_at: chrono::Utc::now(),
        notes: c.notes,
    };

    match state
        .system
        .cash_drawer
        .close_shift(shift_uuid, &count)
        .await
    {
        Ok(shift) => (StatusCode::OK, Json(shift)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Get current shift for a terminal
pub async fn get_current_shift(
    State(state): State<AppState>,
    Path(terminal_id): Path<String>,
) -> impl IntoResponse {
    match state.system.cash_drawer.get_open_shift(&terminal_id).await {
        Ok(Some(shift)) => (StatusCode::OK, Json(shift)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "No open shift for this terminal"})),
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
pub struct VarianceReportQuery {
    pub start_date: String,
    pub end_date: String,
}

/// Get cash variance report
pub async fn get_cash_variance_report(
    State(state): State<AppState>,
    Query(params): Query<VarianceReportQuery>,
) -> impl IntoResponse {
    let start = chrono::DateTime::parse_from_rfc3339(&params.start_date)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now() - chrono::Duration::days(7));

    let end = chrono::DateTime::parse_from_rfc3339(&params.end_date)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now());

    match state
        .system
        .cash_drawer
        .get_variance_report(start, end)
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

/// Get Z-report for a shift
pub async fn get_shift_z_report(
    State(state): State<AppState>,
    Path(shift_uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state.system.reporting.get_shift_z_report(shift_uuid).await {
        Ok(report) => (StatusCode::OK, Json(report)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
