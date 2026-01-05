use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VaultSyncError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Inventory error: {0}")]
    InventoryError(String),

    #[error("Pricing error: {0}")]
    PricingError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Sync error: {0}")]
    SyncError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Insufficient stock: {0}")]
    InsufficientStock(String),

    #[error("Insufficient credit: {0}")]
    InsufficientCredit(String),

    #[error("Payment error: {0}")]
    PaymentError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("UUID error: {0}")]
    UuidError(#[from] uuid::Error),

    #[error("Generic error: {0}")]
    GenericError(String),
}

impl IntoResponse for VaultSyncError {
    fn into_response(self) -> Response {
        // CRIT-03 FIX: Log the actual error but return sanitized messages to client
        let (status, internal_msg, client_msg) = match &self {
            VaultSyncError::DatabaseError(msg) => {
                tracing::error!("Database error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    msg.clone(),
                    "Internal database error".to_string(),
                )
            }
            VaultSyncError::NetworkError(msg) => {
                tracing::error!("Network error: {}", msg);
                (
                    StatusCode::BAD_GATEWAY,
                    msg.clone(),
                    "Network error".to_string(),
                )
            }
            VaultSyncError::InventoryError(msg) => {
                // This is a client-facing error, safe to show
                (StatusCode::BAD_REQUEST, msg.clone(), msg.clone())
            }
            VaultSyncError::PricingError(msg) => {
                tracing::error!("Pricing error: {}", msg);
                (
                    StatusCode::BAD_GATEWAY,
                    msg.clone(),
                    "Pricing service error".to_string(),
                )
            }
            VaultSyncError::TransactionError(msg) => {
                // Client-facing, safe to show
                (StatusCode::BAD_REQUEST, msg.clone(), msg.clone())
            }
            VaultSyncError::SyncError(msg) => {
                tracing::warn!("Sync error: {}", msg);
                (
                    StatusCode::CONFLICT,
                    msg.clone(),
                    "Sync conflict occurred".to_string(),
                )
            }
            VaultSyncError::AuthError(msg) => {
                tracing::warn!("Auth error: {}", msg);
                (
                    StatusCode::UNAUTHORIZED,
                    msg.clone(),
                    "Authentication failed".to_string(),
                )
            }
            VaultSyncError::ConfigError(msg) => {
                tracing::error!("Config error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    msg.clone(),
                    "Configuration error".to_string(),
                )
            }
            VaultSyncError::ValidationError(msg) => {
                // Client-facing, safe to show
                (StatusCode::BAD_REQUEST, msg.clone(), msg.clone())
            }
            VaultSyncError::InsufficientStock(msg) => {
                (StatusCode::CONFLICT, msg.clone(), msg.clone())
            }
            VaultSyncError::InsufficientCredit(msg) => {
                (StatusCode::PAYMENT_REQUIRED, msg.clone(), msg.clone())
            }
            VaultSyncError::PaymentError(msg) => {
                tracing::error!("Payment error: {}", msg);
                (
                    StatusCode::PAYMENT_REQUIRED,
                    msg.clone(),
                    "Payment processing error".to_string(),
                )
            }
            VaultSyncError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone(), msg.clone()),
            VaultSyncError::SerializationError(e) => {
                tracing::error!("Serialization error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    e.to_string(),
                    "Data format error".to_string(),
                )
            }
            VaultSyncError::IoError(e) => {
                tracing::error!("IO error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    e.to_string(),
                    "Internal error".to_string(),
                )
            }
            VaultSyncError::UuidError(e) => (
                StatusCode::BAD_REQUEST,
                e.to_string(),
                format!("Invalid UUID: {}", e),
            ),
            VaultSyncError::GenericError(msg) => {
                tracing::error!("Generic error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    msg.clone(),
                    "Internal error".to_string(),
                )
            }
            VaultSyncError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    msg.clone(),
                    "Internal error".to_string(),
                )
            }
        };

        let _ = internal_msg; // Suppress unused warning - logged above

        let body = Json(serde_json::json!({
            "error": client_msg,
        }));

        (status, body).into_response()
    }
}

// Re-export anyhow for compatibility where needed
pub use anyhow::Result;
