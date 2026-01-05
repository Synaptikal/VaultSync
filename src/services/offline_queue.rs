use crate::database::Database;
use crate::errors::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedOperation {
    pub queue_uuid: Uuid,
    pub operation_type: String,
    pub record_type: String,
    pub record_uuid: Uuid,
    pub payload: serde_json::Value,
    pub status: QueueStatus,
    pub retry_count: i32,
    pub error_message: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub processed_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QueueStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl std::fmt::Display for QueueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueStatus::Pending => write!(f, "pending"),
            QueueStatus::Processing => write!(f, "processing"),
            QueueStatus::Completed => write!(f, "completed"),
            QueueStatus::Failed => write!(f, "failed"),
        }
    }
}

pub struct OfflineQueueService {
    db: Arc<Database>,
}

impl OfflineQueueService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Queue an operation for later processing (TASK-127)
    pub async fn enqueue(
        &self,
        operation_type: &str,
        record_type: &str,
        record_uuid: Uuid,
        payload: serde_json::Value,
    ) -> Result<Uuid> {
        let queue_uuid = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO Offline_Queue (queue_uuid, operation_type, record_type, record_uuid, payload, status, retry_count, created_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(queue_uuid.to_string())
        .bind(operation_type)
        .bind(record_type)
        .bind(record_uuid.to_string())
        .bind(serde_json::to_string(&payload)?)
        .bind("pending")
        .bind(0)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.db.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        tracing::info!(
            "Queued offline operation: {} {} ({})",
            operation_type,
            record_type,
            queue_uuid
        );
        Ok(queue_uuid)
    }

    /// Get pending operations (TASK-128)
    pub async fn get_pending(&self, limit: i64) -> Result<Vec<QueuedOperation>> {
        let rows = sqlx::query(
            "SELECT queue_uuid, operation_type, record_type, record_uuid, payload, status, retry_count, error_message, created_at, processed_at 
             FROM Offline_Queue WHERE status = 'pending' ORDER BY created_at ASC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.db.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut operations = Vec::new();
        for row in rows {
            if let Ok(op) = self.parse_row(&row) {
                operations.push(op);
            }
        }
        Ok(operations)
    }

    /// Mark operation as processing
    pub async fn mark_processing(&self, queue_uuid: Uuid) -> Result<()> {
        sqlx::query("UPDATE Offline_Queue SET status = 'processing' WHERE queue_uuid = ?")
            .bind(queue_uuid.to_string())
            .execute(&self.db.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Mark operation as completed
    pub async fn mark_completed(&self, queue_uuid: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE Offline_Queue SET status = 'completed', processed_at = ? WHERE queue_uuid = ?",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(queue_uuid.to_string())
        .execute(&self.db.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Mark operation as failed with retry logic (TASK-129)
    pub async fn mark_failed(
        &self,
        queue_uuid: Uuid,
        error: &str,
        max_retries: i32,
    ) -> Result<bool> {
        // Get current retry count
        let row = sqlx::query("SELECT retry_count FROM Offline_Queue WHERE queue_uuid = ?")
            .bind(queue_uuid.to_string())
            .fetch_optional(&self.db.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let current_retries: i32 = if let Some(r) = row {
            sqlx::Row::try_get(&r, "retry_count").unwrap_or(0)
        } else {
            return Ok(false);
        };

        let new_retries = current_retries + 1;
        let new_status = if new_retries >= max_retries {
            "failed"
        } else {
            "pending"
        };

        sqlx::query("UPDATE Offline_Queue SET status = ?, retry_count = ?, error_message = ? WHERE queue_uuid = ?")
            .bind(new_status)
            .bind(new_retries)
            .bind(error)
            .bind(queue_uuid.to_string())
            .execute(&self.db.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        // Will retry if not exceeded max
        Ok(new_retries < max_retries)
    }

    /// Get queue stats (TASK-131)
    pub async fn get_stats(&self) -> Result<serde_json::Value> {
        let pending: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM Offline_Queue WHERE status = 'pending'")
                .fetch_one(&self.db.pool)
                .await
                .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let processing: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM Offline_Queue WHERE status = 'processing'")
                .fetch_one(&self.db.pool)
                .await
                .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let completed: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM Offline_Queue WHERE status = 'completed'")
                .fetch_one(&self.db.pool)
                .await
                .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let failed: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM Offline_Queue WHERE status = 'failed'")
                .fetch_one(&self.db.pool)
                .await
                .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(serde_json::json!({
            "pending": pending,
            "processing": processing,
            "completed": completed,
            "failed": failed,
            "total": pending + processing + completed + failed
        }))
    }

    fn parse_row(&self, row: &sqlx::sqlite::SqliteRow) -> Result<QueuedOperation> {
        use sqlx::Row;

        let queue_uuid_str: String = row.try_get("queue_uuid").unwrap_or_default();
        let record_uuid_str: String = row.try_get("record_uuid").unwrap_or_default();
        let payload_str: String = row.try_get("payload").unwrap_or_default();
        let status_str: String = row.try_get("status").unwrap_or_default();
        let created_str: String = row.try_get("created_at").unwrap_or_default();
        let processed_str: Option<String> = row.try_get("processed_at").ok();

        Ok(QueuedOperation {
            queue_uuid: Uuid::parse_str(&queue_uuid_str).unwrap_or_default(),
            operation_type: row.try_get("operation_type").unwrap_or_default(),
            record_type: row.try_get("record_type").unwrap_or_default(),
            record_uuid: Uuid::parse_str(&record_uuid_str).unwrap_or_default(),
            payload: serde_json::from_str(&payload_str).unwrap_or_default(),
            status: match status_str.as_str() {
                "pending" => QueueStatus::Pending,
                "processing" => QueueStatus::Processing,
                "completed" => QueueStatus::Completed,
                "failed" => QueueStatus::Failed,
                _ => QueueStatus::Pending,
            },
            retry_count: row.try_get("retry_count").unwrap_or(0),
            error_message: row.try_get("error_message").ok(),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(Utc::now()),
            processed_at: processed_str.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            }),
        })
    }
}
