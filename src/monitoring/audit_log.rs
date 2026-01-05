//! Data Modification Audit Log
//!
//! TASK-203: Comprehensive audit logging for INSERT/UPDATE/DELETE operations
//! on critical tables (Transactions, Inventory, Customers)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Types of data operations that can be audited
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AuditOperation {
    Insert,
    Update,
    Delete,
}

impl std::fmt::Display for AuditOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditOperation::Insert => write!(f, "INSERT"),
            AuditOperation::Update => write!(f, "UPDATE"),
            AuditOperation::Delete => write!(f, "DELETE"),
        }
    }
}

/// An audit log entry recording a data modification
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLogEntry {
    pub id: i64,
    pub table_name: String,
    pub record_uuid: String,
    pub operation: String,
    pub user_uuid: Option<String>,
    pub old_values: Option<String>, // JSON
    pub new_values: Option<String>, // JSON
    pub request_id: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Request to create an audit log entry
#[derive(Debug, Clone)]
pub struct CreateAuditLogRequest {
    pub table_name: String,
    pub record_uuid: Uuid,
    pub operation: AuditOperation,
    pub user_uuid: Option<Uuid>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub request_id: Option<String>,
    pub ip_address: Option<String>,
}

/// Audit log service for tracking data modifications
pub struct AuditLogService {
    pool: sqlx::SqlitePool,
}

impl AuditLogService {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }

    /// Initialize the audit log table
    pub async fn init(&self) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                table_name TEXT NOT NULL,
                record_uuid TEXT NOT NULL,
                operation TEXT NOT NULL,
                user_uuid TEXT,
                old_values TEXT,
                new_values TEXT,
                request_id TEXT,
                ip_address TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for common queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_table ON audit_log(table_name)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_record ON audit_log(record_uuid)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_user ON audit_log(user_uuid)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_created ON audit_log(created_at)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Log a data modification
    pub async fn log(&self, request: CreateAuditLogRequest) -> anyhow::Result<i64> {
        let old_values_json = request.old_values.map(|v| v.to_string());
        let new_values_json = request.new_values.map(|v| v.to_string());

        let result = sqlx::query(
            r#"
            INSERT INTO audit_log (table_name, record_uuid, operation, user_uuid, old_values, new_values, request_id, ip_address)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&request.table_name)
        .bind(request.record_uuid.to_string())
        .bind(request.operation.to_string())
        .bind(request.user_uuid.map(|u| u.to_string()))
        .bind(old_values_json)
        .bind(new_values_json)
        .bind(&request.request_id)
        .bind(&request.ip_address)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Log an INSERT operation
    pub async fn log_insert(
        &self,
        table_name: &str,
        record_uuid: Uuid,
        new_values: serde_json::Value,
        user_uuid: Option<Uuid>,
        request_id: Option<String>,
    ) -> anyhow::Result<i64> {
        self.log(CreateAuditLogRequest {
            table_name: table_name.to_string(),
            record_uuid,
            operation: AuditOperation::Insert,
            user_uuid,
            old_values: None,
            new_values: Some(new_values),
            request_id,
            ip_address: None,
        })
        .await
    }

    /// Log an UPDATE operation
    pub async fn log_update(
        &self,
        table_name: &str,
        record_uuid: Uuid,
        old_values: serde_json::Value,
        new_values: serde_json::Value,
        user_uuid: Option<Uuid>,
        request_id: Option<String>,
    ) -> anyhow::Result<i64> {
        self.log(CreateAuditLogRequest {
            table_name: table_name.to_string(),
            record_uuid,
            operation: AuditOperation::Update,
            user_uuid,
            old_values: Some(old_values),
            new_values: Some(new_values),
            request_id,
            ip_address: None,
        })
        .await
    }

    /// Log a DELETE operation
    pub async fn log_delete(
        &self,
        table_name: &str,
        record_uuid: Uuid,
        old_values: serde_json::Value,
        user_uuid: Option<Uuid>,
        request_id: Option<String>,
    ) -> anyhow::Result<i64> {
        self.log(CreateAuditLogRequest {
            table_name: table_name.to_string(),
            record_uuid,
            operation: AuditOperation::Delete,
            user_uuid,
            old_values: Some(old_values),
            new_values: None,
            request_id,
            ip_address: None,
        })
        .await
    }

    /// Query audit log entries
    pub async fn query(
        &self,
        table_name: Option<&str>,
        record_uuid: Option<Uuid>,
        user_uuid: Option<Uuid>,
        since: Option<DateTime<Utc>>,
        limit: i64,
    ) -> anyhow::Result<Vec<AuditLogEntry>> {
        let mut query = String::from("SELECT * FROM audit_log WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();

        if let Some(table) = table_name {
            query.push_str(" AND table_name = ?");
            bindings.push(table.to_string());
        }

        if let Some(uuid) = record_uuid {
            query.push_str(" AND record_uuid = ?");
            bindings.push(uuid.to_string());
        }

        if let Some(uuid) = user_uuid {
            query.push_str(" AND user_uuid = ?");
            bindings.push(uuid.to_string());
        }

        if let Some(dt) = since {
            query.push_str(" AND created_at >= ?");
            bindings.push(dt.to_rfc3339());
        }

        query.push_str(" ORDER BY created_at DESC LIMIT ?");
        bindings.push(limit.to_string());

        // Build dynamic query
        let mut sqlx_query = sqlx::query_as::<_, AuditLogEntry>(&query);
        for binding in bindings {
            sqlx_query = sqlx_query.bind(binding);
        }

        let entries = sqlx_query.fetch_all(&self.pool).await?;
        Ok(entries)
    }

    /// Get audit history for a specific record
    pub async fn get_record_history(
        &self,
        table_name: &str,
        record_uuid: Uuid,
    ) -> anyhow::Result<Vec<AuditLogEntry>> {
        let entries = sqlx::query_as::<_, AuditLogEntry>(
            r#"
            SELECT * FROM audit_log 
            WHERE table_name = ? AND record_uuid = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(table_name)
        .bind(record_uuid.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    /// Get recent audit entries
    pub async fn get_recent(&self, limit: i64) -> anyhow::Result<Vec<AuditLogEntry>> {
        let entries = sqlx::query_as::<_, AuditLogEntry>(
            r#"
            SELECT * FROM audit_log 
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    /// Purge old audit entries (for maintenance)
    pub async fn purge_old_entries(&self, days_to_keep: i64) -> anyhow::Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM audit_log 
            WHERE created_at < datetime('now', ? || ' days')
            "#,
        )
        .bind(-days_to_keep)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_operation_display() {
        assert_eq!(AuditOperation::Insert.to_string(), "INSERT");
        assert_eq!(AuditOperation::Update.to_string(), "UPDATE");
        assert_eq!(AuditOperation::Delete.to_string(), "DELETE");
    }
}
