use crate::core::VectorTimestamp;
use crate::errors::Result;
use sqlx::{Row, SqlitePool};

#[derive(Clone)]
pub struct SyncRepository {
    pool: SqlitePool,
    node_id: String,
}

impl SyncRepository {
    pub fn new(pool: SqlitePool, node_id: String) -> Self {
        Self { pool, node_id }
    }

    pub async fn log_change(
        &self,
        record_id: &str,
        record_type: &str,
        operation: &str,
        data: &serde_json::Value,
    ) -> Result<()> {
        let vector = self
            .get_and_increment_vector(record_id, &self.node_id)
            .await?;

        sqlx::query(
            "INSERT OR REPLACE INTO Sync_Log 
            (record_id, record_type, operation, data, node_id, local_clock, version_vector, timestamp) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(record_id)
        .bind(record_type)
        .bind(operation)
        .bind(data.to_string())
        .bind(&self.node_id)
        .bind(vector.get_clock(&self.node_id) as i64)
        .bind(serde_json::to_string(&vector)?)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn log_change_with_tx<'a>(
        &self,
        tx: &mut sqlx::Transaction<'a, sqlx::Sqlite>,
        record_id: &str,
        record_type: &str,
        operation: &str,
        data: &serde_json::Value,
    ) -> Result<()> {
        // 1. Get Vector Timestamp (Atomic inside TX)
        let rows =
            sqlx::query("SELECT node_id, counter FROM Version_Vectors WHERE entity_uuid = ?")
                .bind(record_id)
                .fetch_all(&mut **tx)
                .await
                .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut entries = std::collections::HashMap::new();
        for row in rows {
            let node_id: String = row.try_get("node_id").unwrap_or_default();
            let counter: i64 = row.try_get("counter").unwrap_or_default();
            entries.insert(node_id, counter as u64);
        }
        let mut vector = VectorTimestamp::from_entries(entries);

        // 2. Increment
        vector.increment(self.node_id.clone());

        // 3. Update Vector
        for (node_id, counter) in &vector.entries {
            sqlx::query("INSERT OR REPLACE INTO Version_Vectors (entity_uuid, node_id, counter) VALUES (?, ?, ?)")
                .bind(record_id)
                .bind(node_id)
                .bind(*counter as i64)
                .execute(&mut **tx)
                .await
                .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        }

        // 4. Log Change
        sqlx::query(
            "INSERT OR REPLACE INTO Sync_Log 
            (record_id, record_type, operation, data, node_id, local_clock, version_vector, timestamp) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(record_id)
        .bind(record_type)
        .bind(operation)
        .bind(data.to_string())
        .bind(&self.node_id)
        .bind(vector.get_clock(&self.node_id) as i64)
        // SECURITY FIX: Use ? operator instead of unwrap
        .bind(serde_json::to_string(&vector)?)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_changes_since(
        &self,
        since_clock: i64,
        limit: i64,
    ) -> Result<
        Vec<(
            String,
            String,
            String,
            serde_json::Value,
            String,
            i64,
            String,
            String,
        )>,
    > {
        let rows = sqlx::query("SELECT record_id, record_type, operation, data, node_id, local_clock, version_vector, timestamp FROM Sync_Log WHERE local_clock > ? ORDER BY timestamp ASC LIMIT ?")
            .bind(since_clock)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut changes = Vec::new();
        for row in rows {
            let record_id: String = row.try_get("record_id").unwrap_or_default();
            let record_type: String = row.try_get("record_type").unwrap_or_default();
            let operation: String = row.try_get("operation").unwrap_or_default();
            let data_str: String = row.try_get("data").unwrap_or_default();
            let data: serde_json::Value = serde_json::from_str(&data_str)?;
            let node_id: String = row.try_get("node_id").unwrap_or_default();
            let local_clock: i64 = row.try_get("local_clock").unwrap_or_default();
            let version_vector_str: String = row.try_get("version_vector").unwrap_or_default();
            let timestamp: String = row.try_get("timestamp").unwrap_or_default();

            changes.push((
                record_id,
                record_type,
                operation,
                data,
                node_id,
                local_clock,
                version_vector_str,
                timestamp,
            ));
        }

        Ok(changes)
    }

    pub async fn get_version_vector(&self, entity_uuid: &str) -> Result<Option<VectorTimestamp>> {
        let rows =
            sqlx::query("SELECT node_id, counter FROM Version_Vectors WHERE entity_uuid = ?")
                .bind(entity_uuid)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if rows.is_empty() {
            return Ok(None);
        }

        let mut entries = std::collections::HashMap::new();
        for row in rows {
            let node_id: String = row.try_get("node_id").unwrap_or_default();
            let counter: i64 = row.try_get("counter").unwrap_or_default();
            entries.insert(node_id, counter as u64);
        }

        Ok(Some(VectorTimestamp::from_entries(entries)))
    }

    pub async fn update_version_vector(
        &self,
        entity_uuid: &str,
        vector: &VectorTimestamp,
    ) -> Result<()> {
        for (node_id, counter) in &vector.entries {
            sqlx::query("INSERT OR REPLACE INTO Version_Vectors (entity_uuid, node_id, counter) VALUES (?, ?, ?)")
                .bind(entity_uuid)
                .bind(node_id)
                .bind(*counter as i64)
                .execute(&self.pool)
                .await
                .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn get_last_sync_operation(&self, record_id: &str) -> Result<Option<String>> {
        let row = sqlx::query(
            "SELECT operation FROM Sync_Log WHERE record_id = ? ORDER BY local_clock DESC LIMIT 1",
        )
        .bind(record_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let op: String = row.try_get("operation").unwrap_or_default();
            Ok(Some(op))
        } else {
            Ok(None)
        }
    }

    pub async fn get_and_increment_vector(
        &self,
        entity_uuid: &str,
        node_id: &str,
    ) -> Result<VectorTimestamp> {
        let mut vector = self
            .get_version_vector(entity_uuid)
            .await?
            .unwrap_or_else(|| VectorTimestamp::new());
        vector.increment(node_id.to_string());
        self.update_version_vector(entity_uuid, &vector).await?;
        Ok(vector)
    }
}
