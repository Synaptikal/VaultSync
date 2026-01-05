//! Backup-related API handlers
//!
//! Handles database backup creation, listing, verification, and retention.

use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use serde_json::json;

/// Create a new database backup
pub async fn create_backup() -> impl IntoResponse {
    let backup_service = crate::services::backup::BackupService::from_env();

    match backup_service.create_backup().await {
        Ok(result) => {
            if result.success {
                (StatusCode::OK, Json(result)).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(result)).into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// List all available backups
pub async fn list_backups() -> impl IntoResponse {
    let backup_service = crate::services::backup::BackupService::from_env();

    match backup_service.list_backups().await {
        Ok(backups) => (
            StatusCode::OK,
            Json(json!({
                "count": backups.len(),
                "backups": backups
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

/// Apply backup retention policy (delete old backups)
pub async fn apply_backup_retention() -> impl IntoResponse {
    let backup_service = crate::services::backup::BackupService::from_env();

    match backup_service.apply_retention_policy().await {
        Ok(deleted) => (
            StatusCode::OK,
            Json(json!({
                "deleted_count": deleted.len(),
                "deleted_files": deleted.iter().map(|p| p.display().to_string()).collect::<Vec<_>>()
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
pub struct VerifyBackupRequest {
    pub backup_path: String,
}

/// Verify a specific backup
pub async fn verify_backup(Json(req): Json<VerifyBackupRequest>) -> impl IntoResponse {
    let backup_service = crate::services::backup::BackupService::from_env();
    let path = std::path::PathBuf::from(&req.backup_path);

    match backup_service.verify_backup(&path).await {
        Ok(valid) => (
            StatusCode::OK,
            Json(json!({
                "valid": valid,
                "path": req.backup_path
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
