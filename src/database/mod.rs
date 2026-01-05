use crate::errors::Result;
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use std::sync::Arc;
use uuid::Uuid;

pub mod migrations;
pub mod repositories;

use repositories::audit::AuditRepository;
use repositories::auth::AuthRepository;

use repositories::customers::CustomerRepository;
use repositories::events::EventRepository;
use repositories::inventory::InventoryRepository;
use repositories::pricing::PricingRepository;
use repositories::products::ProductRepository;
use repositories::sync::SyncRepository;
use repositories::transactions::TransactionRepository;

pub struct Database {
    pub pool: SqlitePool,
    pub products: ProductRepository,
    pub inventory: InventoryRepository,
    pub transactions: TransactionRepository,
    pub customers: CustomerRepository,
    pub events: EventRepository,
    pub sync: SyncRepository,
    pub pricing: PricingRepository,
    pub auth: AuthRepository,
    pub audit: AuditRepository,
    pub node_id: String,
}

impl Database {
    pub async fn new(connection_string: &str, node_id: String) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    sqlx::query("PRAGMA foreign_keys = ON;")
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .connect(connection_string)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        // Node ID is now passed in from configuration
        let sync_repo = SyncRepository::new(pool.clone(), node_id.clone());

        Ok(Self {
            pool: pool.clone(),
            products: ProductRepository::new(pool.clone(), sync_repo.clone()),
            inventory: InventoryRepository::new(pool.clone(), sync_repo.clone()),
            transactions: TransactionRepository::new(pool.clone(), node_id.clone()),
            customers: CustomerRepository::new(pool.clone(), sync_repo.clone()),
            events: EventRepository::new(pool.clone(), sync_repo.clone()),
            sync: sync_repo,
            pricing: PricingRepository::new(pool.clone()),
            auth: AuthRepository::new(pool.clone()),
            audit: AuditRepository::new(pool.clone()),
            node_id,
        })
    }

    pub fn get_schema_migrations() -> Vec<(i32, &'static str, Vec<&'static str>)> {
        migrations::get_schema_migrations()
    }

    pub async fn preview_migrations(&self) -> Result<Vec<(i32, String)>> {
        let current_version: i32 = sqlx::query("SELECT MAX(version) FROM _migrations")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?
            .try_get::<Option<i64>, _>(0)
            .unwrap_or(Some(0))
            .unwrap_or(0) as i32;

        let migrations = Self::get_schema_migrations();
        let mut pending = Vec::new();

        for (version, description, _) in migrations {
            if version > current_version {
                pending.push((version, description.to_string()));
            }
        }
        Ok(pending)
    }

    pub async fn initialize_tables(&self) -> Result<()> {
        // Keep migration logic here for now but could be moved to repositories/migrations.rs
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                description TEXT NOT NULL,
                applied_at TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        // Check current version
        let current_version: i32 = sqlx::query("SELECT MAX(version) FROM _migrations")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?
            .try_get::<Option<i64>, _>(0)
            .unwrap_or(Some(0))
            .unwrap_or(0) as i32;

        let migrations = Self::get_schema_migrations();

        for (version, description, statements) in migrations {
            if version > current_version {
                tracing::info!("Applying migration {}: {}", version, description);
                for sql in statements {
                    sqlx::query(sql)
                        .execute(&self.pool)
                        .await
                        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
                }

                sqlx::query(
                    "INSERT INTO _migrations (version, description, applied_at) VALUES (?, ?, ?)",
                )
                .bind(version as i64)
                .bind(description)
                .bind(chrono::Utc::now().to_rfc3339())
                .execute(&self.pool)
                .await
                .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
            }
        }
        Ok(())
    }

    // MED-004 FIX: Pricing Rules
    pub async fn get_pricing_rules(&self) -> Result<Vec<crate::pricing::rules::PricingRule>> {
        let rows = sqlx::query(
            "SELECT rule_id, priority, category, condition, min_market_price, max_market_price, 
                    cash_multiplier, credit_multiplier, start_date, end_date, customer_tier, min_quantity
             FROM Pricing_Rules 
             WHERE is_active = 1 
             ORDER BY priority DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut rules = Vec::new();
        for row in rows {
            let rule_id: String = row.try_get("rule_id").unwrap_or_default();
            let priority: i32 = row.try_get("priority").unwrap_or(0);
            let category_str: Option<String> = row.try_get("category").ok();
            let condition_str: Option<String> = row.try_get("condition").ok();
            let min_market_price: Option<f64> = row.try_get("min_market_price").ok();
            let max_market_price: Option<f64> = row.try_get("max_market_price").ok();
            let cash_multiplier: f64 = row.try_get("cash_multiplier").unwrap_or(0.3);
            let credit_multiplier: f64 = row.try_get("credit_multiplier").unwrap_or(0.5);

            let category = category_str.and_then(|s| match s.as_str() {
                "TCG" => Some(crate::core::Category::TCG),
                "SportsCard" => Some(crate::core::Category::SportsCard),
                "Comic" => Some(crate::core::Category::Comic),
                "Bobblehead" => Some(crate::core::Category::Bobblehead),
                "Apparel" => Some(crate::core::Category::Apparel),
                "Figure" => Some(crate::core::Category::Figure),
                "Accessory" => Some(crate::core::Category::Accessory),
                "Other" => Some(crate::core::Category::Other),
                _ => None,
            });

            let condition = condition_str.and_then(|s| match s.as_str() {
                "NM" => Some(crate::core::Condition::NM),
                "LP" => Some(crate::core::Condition::LP),
                "MP" => Some(crate::core::Condition::MP),
                "HP" => Some(crate::core::Condition::HP),
                "DMG" => Some(crate::core::Condition::DMG),
                _ => None,
            });

            let start_date_str: Option<String> = row.try_get("start_date").ok();
            let start_date = start_date_str.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&chrono::Utc))
            });

            let end_date_str: Option<String> = row.try_get("end_date").ok();
            let end_date = end_date_str.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&chrono::Utc))
            });

            let customer_tier: Option<String> = row.try_get("customer_tier").ok();
            let min_quantity: Option<i32> = row.try_get("min_quantity").ok();

            rules.push(crate::pricing::rules::PricingRule {
                id: rule_id,
                priority,
                category,
                condition,
                min_market_price,
                max_market_price,
                start_date,
                end_date,
                customer_tier,
                min_quantity,
                cash_multiplier,
                credit_multiplier,
            });
        }

        Ok(rules)
    }

    pub async fn save_pricing_rule(&self, rule: &crate::pricing::rules::PricingRule) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let category_str = rule.category.as_ref().map(|c| format!("{:?}", c));
        let condition_str = rule.condition.as_ref().map(|c| format!("{:?}", c));

        sqlx::query(
            "INSERT OR REPLACE INTO Pricing_Rules 
             (rule_id, priority, category, condition, min_market_price, max_market_price, 
              cash_multiplier, credit_multiplier, start_date, end_date, customer_tier, min_quantity, is_active, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, COALESCE((SELECT created_at FROM Pricing_Rules WHERE rule_id = ?), ?), ?)"
        )
        .bind(&rule.id)
        .bind(rule.priority)
        .bind(&category_str)
        .bind(&condition_str)
        .bind(rule.min_market_price)
        .bind(rule.max_market_price)
        .bind(rule.cash_multiplier)
        .bind(rule.credit_multiplier)
        .bind(rule.start_date.map(|d| d.to_rfc3339()))
        .bind(rule.end_date.map(|d| d.to_rfc3339()))
        .bind(&rule.customer_tier)
        .bind(rule.min_quantity)
        .bind(&rule.id)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // MED-005 FIX: Refresh Token Management
    pub async fn save_refresh_token(
        &self,
        token_hash: &str,
        user_uuid: Uuid,
        expires_at: DateTime<Utc>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO Refresh_Tokens (token_hash, user_uuid, expires_at, created_at, is_revoked) VALUES (?, ?, ?, ?, 0)"
        )
        .bind(token_hash)
        .bind(user_uuid.to_string())
        .bind(expires_at.to_rfc3339())
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn get_refresh_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<(Uuid, DateTime<Utc>, bool)>> {
        let row = sqlx::query(
            "SELECT user_uuid, expires_at, is_revoked FROM Refresh_Tokens WHERE token_hash = ?",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let user_uuid_str: String = row.try_get("user_uuid").unwrap_or_default();
            let user_uuid = Uuid::parse_str(&user_uuid_str).unwrap_or_default();
            let expires_at_str: String = row.try_get("expires_at").unwrap_or_default();
            let expires_at = DateTime::parse_from_rfc3339(&expires_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let is_revoked: bool = row.try_get("is_revoked").unwrap_or(false);
            Ok(Some((user_uuid, expires_at, is_revoked)))
        } else {
            Ok(None)
        }
    }

    pub async fn revoke_refresh_token(&self, token_hash: &str) -> Result<()> {
        sqlx::query("UPDATE Refresh_Tokens SET is_revoked = 1 WHERE token_hash = ?")
            .bind(token_hash)
            .execute(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // --- Sync Conflict Methods (TASK-121) ---

    pub async fn get_sync_conflicts(&self) -> Result<Vec<serde_json::Value>> {
        // Query the dedicated Sync_Conflicts table
        let rows = sqlx::query(
            "SELECT c.conflict_uuid, c.resource_type, c.resource_uuid, c.conflict_type, 
                    c.resolution_status, c.detected_at, c.resolved_at, c.resolved_by_user,
                    c.resolution_strategy,
                    s.snapshot_uuid, s.state_data, s.node_id as remote_node_id
             FROM Sync_Conflicts c
             LEFT JOIN Conflict_Snapshots s ON c.conflict_uuid = s.conflict_uuid
             WHERE c.resolution_status = 'Pending'
             ORDER BY c.detected_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut conflicts = Vec::new();
        for row in rows {
            let conflict_uuid: String = row.try_get("conflict_uuid").unwrap_or_default();
            let resource_type: String = row.try_get("resource_type").unwrap_or_default();
            let resource_uuid: String = row.try_get("resource_uuid").unwrap_or_default();
            let conflict_type: String = row.try_get("conflict_type").unwrap_or_default();
            let resolution_status: String = row.try_get("resolution_status").unwrap_or_default();
            let detected_at: String = row.try_get("detected_at").unwrap_or_default();

            // Snapshot data (Remote state)
            let state_data: String = row
                .try_get("state_data")
                .unwrap_or_else(|_| "{}".to_string());
            let remote_node: String = row.try_get("remote_node_id").unwrap_or_default();

            // Fetch current local state for comparison
            let local_state = match resource_type.as_str() {
                "Product" => {
                    if let Ok(uuid) = Uuid::parse_str(&resource_uuid) {
                        self.products
                            .get_by_id(uuid)
                            .await?
                            .map(|p| serde_json::to_value(p).unwrap_or_default())
                    } else {
                        None
                    }
                }
                "InventoryItem" => {
                    if let Ok(uuid) = Uuid::parse_str(&resource_uuid) {
                        self.inventory
                            .get_by_id(uuid)
                            .await?
                            .map(|i| serde_json::to_value(i).unwrap_or_default())
                    } else {
                        None
                    }
                }
                _ => None,
            };

            conflicts.push(serde_json::json!({
                "conflict_uuid": conflict_uuid,
                "resource_type": resource_type,
                "resource_uuid": resource_uuid,
                "conflict_type": conflict_type, // 'Concurrent_Mod', etc.
                "status": resolution_status,
                "detected_at": detected_at,
                "remote_node_id": remote_node,
                "remote_state": serde_json::from_str::<serde_json::Value>(&state_data).unwrap_or_default(),
                "local_state": local_state.unwrap_or(serde_json::json!({"status": "deleted_or_missing"}))
            }));
        }
        Ok(conflicts)
    }

    pub async fn resolve_sync_conflict(
        &self,
        conflict_uuid: &str,
        resolution_strategy: &str, // 'LocalWins', 'RemoteWins'
    ) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        // 1. Get the conflict details including snapshot
        let row = sqlx::query(
            "SELECT c.resource_type, c.resource_uuid, s.state_data 
             FROM Sync_Conflicts c
             JOIN Conflict_Snapshots s ON c.conflict_uuid = s.conflict_uuid
             WHERE c.conflict_uuid = ?",
        )
        .bind(conflict_uuid)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if let Some(r) = row {
            let resource_type: String = r.try_get("resource_type").unwrap_or_default();
            let _resource_uuid: String = r.try_get("resource_uuid").unwrap_or_default();
            let state_data: String = r.try_get("state_data").unwrap_or_default();

            // 2. Apply resolution logic
            if resolution_strategy == "RemoteWins" {
                // Apply the Snapshot data to the local DB
                // We reuse the existing insert methods but need to parse JSON
                match resource_type.as_str() {
                    "Product" => {
                        if let Ok(p) = serde_json::from_str::<crate::core::Product>(&state_data) {
                            if let Err(e) = self.products.insert_with_tx(&mut tx, &p).await {
                                tracing::error!("Failed to apply resolved product: {}", e);
                                return Err(e);
                            }
                        }
                    }
                    "InventoryItem" => {
                        if let Ok(i) =
                            serde_json::from_str::<crate::core::InventoryItem>(&state_data)
                        {
                            if let Err(e) = self.inventory.insert_with_tx(&mut tx, &i).await {
                                tracing::error!("Failed to apply resolved inventory: {}", e);
                                return Err(e);
                            }
                        }
                    }
                    _ => {
                        // Other types not auto-resolvable via this strategy yet
                    }
                }
            }
        }

        // 3. Mark as resolved
        sqlx::query(
            "UPDATE Sync_Conflicts 
             SET resolution_status = 'Resolved', 
                 resolved_at = ?, 
                 resolution_strategy = ?
             WHERE conflict_uuid = ?",
        )
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(resolution_strategy)
        .bind(conflict_uuid)
        .execute(&mut *tx)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        tracing::info!(
            "Resolved sync conflict {} with strategy: {}",
            conflict_uuid,
            resolution_strategy
        );
        Ok(())
    }

    /// Record a new conflict detected by the SyncService
    pub async fn record_sync_conflict(
        &self,
        resource_type: &str,
        resource_uuid: &str,
        conflict_type: &str,
        remote_node_id: &str,
        remote_data: &serde_json::Value,
        remote_vector: &crate::core::VectorTimestamp,
    ) -> Result<()> {
        let conflict_uuid = Uuid::new_v4().to_string();
        let snapshot_uuid = Uuid::new_v4().to_string();

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        // 1. Create Conflict Record
        sqlx::query(
            "INSERT INTO Sync_Conflicts (
                conflict_uuid, resource_type, resource_uuid, conflict_type, 
                resolution_status, detected_at
             ) VALUES (?, ?, ?, ?, 'Pending', ?)",
        )
        .bind(&conflict_uuid)
        .bind(resource_type)
        .bind(resource_uuid)
        .bind(conflict_type)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&mut *tx)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        // 2. Create Snapshot (Remote State which caused conflict)
        sqlx::query(
            "INSERT INTO Conflict_Snapshots (
                snapshot_uuid, conflict_uuid, node_id, state_data, vector_clock
             ) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(snapshot_uuid)
        .bind(&conflict_uuid)
        .bind(remote_node_id)
        .bind(remote_data.to_string())
        .bind(serde_json::to_string(remote_vector).unwrap_or_default())
        .execute(&mut *tx)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        tracing::warn!(
            "Recorded new sync conflict: {} for {}",
            conflict_type,
            resource_uuid
        );
        Ok(())
    }

    // --- Offline Queue Stats (TASK-131) ---

    pub async fn get_offline_queue_stats(&self) -> Result<serde_json::Value> {
        let pending: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM Offline_Queue WHERE status = 'pending'")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

        let processing: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM Offline_Queue WHERE status = 'processing'")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

        let failed: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM Offline_Queue WHERE status = 'failed'")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

        Ok(serde_json::json!({
            "pending": pending,
            "processing": processing,
            "failed": failed
        }))
    }
}

pub async fn initialize_db(node_id: String) -> Result<Arc<Database>> {
    let db_path = "vaultsync.db";
    if !std::path::Path::new(db_path).exists() {
        std::fs::File::create(db_path)?;
    }

    let connection_string = format!("sqlite://{}", db_path);
    let db = Database::new(&connection_string, node_id).await?;
    db.initialize_tables().await?;
    tracing::info!("Database initialized at {}", db_path);
    Ok(Arc::new(db))
}

pub async fn initialize_test_db() -> Result<Arc<Database>> {
    let test_node_id = format!("test_node_{}", uuid::Uuid::new_v4());
    let db = Database::new("sqlite::memory:", test_node_id).await?;
    db.initialize_tables().await?;
    Ok(Arc::new(db))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migrations_apply_cleanly() {
        let db = initialize_test_db().await.expect("Failed to init test db");

        // Verify version
        let version: i32 = sqlx::query_scalar("SELECT MAX(version) FROM _migrations")
            .fetch_one(&db.pool)
            .await
            .expect("Failed to fetch version");

        // This relies on get_schema_migrations being available (public)
        let schema_migrations = Database::get_schema_migrations();
        let expected_version = schema_migrations.last().map(|(v, _, _)| *v).unwrap_or(0);

        assert_eq!(
            version, expected_version,
            "Database version should match latest migration"
        );
    }

    #[tokio::test]
    async fn test_preview_migrations() {
        // 1. Create DB but don't init tables yet
        let test_node_id = format!("test_node_{}", uuid::Uuid::new_v4());
        let db = Database::new("sqlite::memory:", test_node_id)
            .await
            .expect("Failed to create db");

        // Manually init only _migrations table to simulate fresh state
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                description TEXT NOT NULL,
                applied_at TEXT NOT NULL
            )",
        )
        .execute(&db.pool)
        .await
        .expect("Failed to create migrations table");

        // 2. Preview
        let pending = db.preview_migrations().await.expect("Failed to preview");
        let all_migrations = Database::get_schema_migrations();

        assert_eq!(
            pending.len(),
            all_migrations.len(),
            "All migrations should be pending"
        );
        assert_eq!(pending[0].1, "Initial Schema");
    }
}
