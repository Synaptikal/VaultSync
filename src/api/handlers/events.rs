//! Event-related API handlers
//!
//! Handles tournament/event creation, listing, and participant registration.

use crate::api::AppState;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateEventRequest {
    pub name: String,
    pub event_type: String,
    pub date: chrono::DateTime<Utc>,
    pub entry_fee: f64,
    pub max_participants: Option<i32>,
}

/// Create a new event/tournament
pub async fn create_event(
    State(state): State<AppState>,
    Json(req): Json<CreateEventRequest>,
) -> impl IntoResponse {
    match state
        .system
        .events
        .create_event(
            req.name,
            req.event_type,
            req.date,
            req.entry_fee,
            req.max_participants,
        )
        .await
    {
        Ok(event) => (StatusCode::CREATED, Json(event)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Get all upcoming events
pub async fn get_events(State(state): State<AppState>) -> impl IntoResponse {
    match state.system.events.get_upcoming_events().await {
        Ok(events) => (StatusCode::OK, Json(events)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct RegisterParticipantRequest {
    pub player_name: String,
    pub customer_uuid: Option<Uuid>,
    pub pay_with_credit: bool,
}

/// Register a participant for an event
pub async fn register_participant(
    State(state): State<AppState>,
    Path(event_uuid): Path<Uuid>,
    Json(req): Json<RegisterParticipantRequest>,
) -> impl IntoResponse {
    match state
        .system
        .events
        .register_player(
            event_uuid,
            req.player_name,
            req.customer_uuid,
            req.pay_with_credit,
        )
        .await
    {
        Ok(participant) => (StatusCode::CREATED, Json(participant)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
