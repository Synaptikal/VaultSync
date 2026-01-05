/// AppError is a thin wrapper for API handlers to return VaultSyncError
/// This module is provided for handler ergonomics with the `?` operator.
pub use crate::errors::VaultSyncError as AppError;

// Additional conversions for anyhow compatibility
impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::GenericError(e.to_string())
    }
}
