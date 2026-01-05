//! Sync-related API handlers
//!
//! P0-3 Fix: These handlers now use `SyncActor` (message passing) instead of
//! `Mutex<SyncService>` to eliminate the global lock convoy effect.

use crate::api::AppState;
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Push sync changes from a peer (apply remotely-received changes)
///
/// P0-3: Now uses actor pattern - no lock held during processing
pub async fn push_sync_changes(
    State(state): State<AppState>,
    Json(changes): Json<Vec<crate::sync::ChangeRecord>>,
) -> impl IntoResponse {
    // Use the actor (non-blocking queue)
    match state.sync_actor.apply_changes(changes).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "synced"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Pull sync changes for a peer to consume
///
/// This reads directly from the database - no lock needed
pub async fn pull_sync_changes(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let since: i64 = params
        .get("since")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let limit: i64 = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    match state.db.sync.get_changes_since(since, limit).await {
        Ok(changes) => {
            let records: Vec<crate::sync::ChangeRecord> = changes
                .into_iter()
                .map(|(id, rtype, op, data, _node, local_clock, vv_str, ts)| {
                    let record_type = match rtype.as_str() {
                        "Product" => crate::core::RecordType::Product,
                        "InventoryItem" => crate::core::RecordType::InventoryItem,
                        "PriceInfo" => crate::core::RecordType::PriceInfo,
                        "Transaction" => crate::core::RecordType::Transaction,
                        "Customer" => crate::core::RecordType::Customer,
                        _ => crate::core::RecordType::Product,
                    };

                    let operation = match op.as_str() {
                        "Insert" => crate::core::SyncOperation::Insert,
                        "Update" => crate::core::SyncOperation::Update,
                        "Delete" => crate::core::SyncOperation::Delete,
                        _ => crate::core::SyncOperation::Update,
                    };

                    let vector_timestamp: crate::core::VectorTimestamp =
                        serde_json::from_str(&vv_str)
                            .unwrap_or_else(|_| crate::core::VectorTimestamp::new());

                    crate::sync::ChangeRecord {
                        record_id: id,
                        record_type,
                        operation,
                        data,
                        vector_timestamp,
                        timestamp: chrono::DateTime::parse_from_rfc3339(&ts)
                            .unwrap_or(chrono::Utc::now().into())
                            .with_timezone(&chrono::Utc),
                        sequence_number: Some(local_clock as u64),
                        checksum: None,
                    }
                })
                .collect();

            (StatusCode::OK, Json(records)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Serialize)]
pub struct SyncStatusResponse {
    pub is_synced: bool,
    pub last_sync: Option<String>,
    pub pending_changes: i64,
    pub connected_peers: i32,
    pub node_id: String,
}

/// Get current sync status
///
/// P0-3: Now uses actor pattern - no lock held
pub async fn get_sync_status(State(state): State<AppState>) -> impl IntoResponse {
    // Use the actor to get status (non-blocking)
    let status = state.sync_actor.get_status().await;

    let response = SyncStatusResponse {
        is_synced: status.is_synced,
        last_sync: status.last_sync.map(|dt| dt.to_rfc3339()),
        pending_changes: status.pending_changes as i64,
        connected_peers: status.connected_peers as i32,
        node_id: state.db.node_id.clone(),
    };

    (StatusCode::OK, Json(response)).into_response()
}

pub async fn get_sync_conflicts(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.get_sync_conflicts().await {
        Ok(conflicts) => (StatusCode::OK, Json(conflicts)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct SyncConflictResolution {
    pub record_id: String,
    pub resolution: String,
}

pub async fn resolve_sync_conflict(
    State(state): State<AppState>,
    Json(req): Json<SyncConflictResolution>,
) -> impl IntoResponse {
    match state
        .db
        .resolve_sync_conflict(&req.record_id, &req.resolution)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "resolved"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Get detailed sync progress
///
/// P0-3: Now uses actor pattern - no lock held
pub async fn get_sync_progress(State(state): State<AppState>) -> impl IntoResponse {
    // Use the actor for status
    let status = state.sync_actor.get_status().await;

    let queue_stats = match state.db.get_offline_queue_stats().await {
        Ok(stats) => stats,
        Err(_) => serde_json::json!({"pending": 0, "processing": 0, "failed": 0}),
    };

    (
        StatusCode::OK,
        Json(json!({
            "last_sync": status.last_sync,
            "connected_peers": status.connected_peers,
            "pending_changes": status.pending_changes,
            "is_synced": status.is_synced,
            "offline_queue": queue_stats
        })),
    )
}

/// Trigger a sync with all discovered peers
///
/// P0-3: Now uses actor pattern - request is queued, no lock held
pub async fn trigger_peer_sync(State(state): State<AppState>) -> impl IntoResponse {
    // Use the actor (queues the sync, doesn't block)
    match state.sync_actor.sync_with_peers().await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "sync triggered"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Get list of discovered peer devices
///
/// P0-3: Now uses actor pattern - no lock held
pub async fn get_discovered_devices(State(state): State<AppState>) -> impl IntoResponse {
    // Use the actor to get devices
    let devices = state.sync_actor.get_devices().await;

    (
        StatusCode::OK,
        Json(json!({
            "devices": devices,
            "count": devices.len()
        })),
    )
}

#[derive(Deserialize)]
pub struct ManualPairRequest {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub node_id: Option<String>,
}

/// Manually pair a device by IP address
///
/// P0-3: Now uses actor pattern - no lock held
pub async fn manual_pair_device(
    State(state): State<AppState>,
    Json(req): Json<ManualPairRequest>,
) -> impl IntoResponse {
    let addr: std::net::IpAddr = match req.address.parse() {
        Ok(a) => a,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid IP address"})),
            )
                .into_response()
        }
    };

    // Use the actor for manual pairing
    match state
        .sync_actor
        .manual_pair(req.name, addr, req.port, req.node_id)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "paired"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
