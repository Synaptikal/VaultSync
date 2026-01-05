use crate::audit::{Conflict, ConflictType, ResolutionStatus};
use crate::errors::Result;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuditRepository {
    pool: SqlitePool,
}

impl AuditRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn update_status(&self, conflict_uuid: Uuid, status: ResolutionStatus) -> Result<()> {
        let now = Utc::now();
        sqlx::query("UPDATE Inventory_Conflicts SET resolution_status = ?, resolved_at = ? WHERE conflict_uuid = ?")
            .bind(status.to_string())
            .bind(now.to_rfc3339())
            .bind(conflict_uuid.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn insert_conflict(&self, conflict: &Conflict) -> Result<()> {
        let terminal_ids_json = conflict
            .terminal_ids
            .as_ref()
            .map(|ids| serde_json::to_string(ids).unwrap_or("[]".to_string()));

        sqlx::query(
            "INSERT INTO Inventory_Conflicts (conflict_uuid, product_uuid, conflict_type, terminal_ids, expected_quantity, actual_quantity, resolution_status, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(conflict.conflict_uuid.to_string())
        .bind(conflict.product_uuid.to_string())
        .bind(conflict.conflict_type.to_string())
        .bind(terminal_ids_json)
        .bind(conflict.expected_quantity)
        .bind(conflict.actual_quantity)
        .bind(conflict.resolution_status.to_string())
        .bind(conflict.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_pending_conflicts(&self) -> Result<Vec<Conflict>> {
        let rows =
            sqlx::query("SELECT * FROM Inventory_Conflicts WHERE resolution_status = 'Pending'")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut conflicts = Vec::new();
        for row in rows {
            let conflict_type_str: String = row.try_get("conflict_type").unwrap_or_default();
            let status_str: String = row.try_get("resolution_status").unwrap_or_default();
            let terminal_ids_json: Option<String> = row.try_get("terminal_ids").ok();

            let terminal_ids = if let Some(json) = terminal_ids_json {
                serde_json::from_str(&json).ok()
            } else {
                None
            };

            conflicts.push(Conflict {
                conflict_uuid: Uuid::parse_str(
                    &row.try_get::<String, _>("conflict_uuid")
                        .unwrap_or_default(),
                )
                .unwrap_or_default(),
                product_uuid: Uuid::parse_str(
                    &row.try_get::<String, _>("product_uuid").unwrap_or_default(),
                )
                .unwrap_or_default(),
                conflict_type: ConflictType::from_str(&conflict_type_str)
                    .unwrap_or(ConflictType::PhysicalMiscount),
                terminal_ids,
                expected_quantity: row.try_get("expected_quantity").unwrap_or(0),
                actual_quantity: row.try_get("actual_quantity").unwrap_or(0),
                resolution_status: ResolutionStatus::from_str(&status_str)
                    .unwrap_or(ResolutionStatus::Pending),
                resolved_by: row
                    .try_get::<Option<String>, _>("resolved_by")
                    .ok()
                    .flatten()
                    .map(|s| Uuid::parse_str(&s).unwrap_or_default()),
                resolution_notes: row.try_get("resolution_notes").ok(),
                created_at: DateTime::parse_from_rfc3339(
                    &row.try_get::<String, _>("created_at").unwrap_or_default(),
                )
                .unwrap_or(Utc::now().into())
                .with_timezone(&Utc),
                resolved_at: row
                    .try_get::<Option<String>, _>("resolved_at")
                    .ok()
                    .flatten()
                    .map(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .unwrap_or(Utc::now().into())
                            .with_timezone(&Utc)
                    }),
            });
        }

        Ok(conflicts)
    }
}
