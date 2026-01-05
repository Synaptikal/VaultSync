use crate::database::Database;
use crate::errors::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub conflict_uuid: Uuid,
    pub product_uuid: Uuid,
    pub conflict_type: ConflictType,
    pub terminal_ids: Option<Vec<String>>,
    pub expected_quantity: i32,
    pub actual_quantity: i32,
    pub resolution_status: ResolutionStatus,
    pub resolved_by: Option<Uuid>,
    pub resolution_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    Oversold,
    PriceMismatch,
    CreditAnomaly,
    PhysicalMiscount,
    SyncConflict,
}

impl std::fmt::Display for ConflictType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConflictType::Oversold => write!(f, "Oversold"),
            ConflictType::PriceMismatch => write!(f, "PriceMismatch"),
            ConflictType::CreditAnomaly => write!(f, "CreditAnomaly"),
            ConflictType::PhysicalMiscount => write!(f, "PhysicalMiscount"),
            ConflictType::SyncConflict => write!(f, "SyncConflict"),
        }
    }
}

impl std::str::FromStr for ConflictType {
    type Err = ();
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Oversold" => Ok(ConflictType::Oversold),
            "PriceMismatch" => Ok(ConflictType::PriceMismatch),
            "CreditAnomaly" => Ok(ConflictType::CreditAnomaly),
            "PhysicalMiscount" => Ok(ConflictType::PhysicalMiscount),
            "SyncConflict" => Ok(ConflictType::SyncConflict),
            _ => Ok(ConflictType::PhysicalMiscount), // Default
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResolutionStatus {
    Pending,
    Resolved,
    Ignored,
}

impl std::fmt::Display for ResolutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolutionStatus::Pending => write!(f, "Pending"),
            ResolutionStatus::Resolved => write!(f, "Resolved"),
            ResolutionStatus::Ignored => write!(f, "Ignored"),
        }
    }
}

impl std::str::FromStr for ResolutionStatus {
    type Err = ();
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(ResolutionStatus::Pending),
            "Resolved" => Ok(ResolutionStatus::Resolved),
            "Ignored" => Ok(ResolutionStatus::Ignored),
            _ => Ok(ResolutionStatus::Pending),
        }
    }
}

#[derive(Clone)]
pub struct AuditService {
    db: Arc<Database>,
}

impl AuditService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn create_conflict(
        &self,
        product_uuid: Uuid,
        conflict_type: ConflictType,
        expected: i32,
        actual: i32,
        terminal_ids: Option<Vec<String>>,
    ) -> Result<Conflict> {
        let conflict = Conflict {
            conflict_uuid: Uuid::new_v4(),
            product_uuid,
            conflict_type,
            terminal_ids,
            expected_quantity: expected,
            actual_quantity: actual,
            resolution_status: ResolutionStatus::Pending,
            resolved_by: None,
            resolution_notes: None,
            created_at: Utc::now(),
            resolved_at: None,
        };

        self.db.audit.insert_conflict(&conflict).await?;
        tracing::info!("Conflict Recorded: {}", conflict.conflict_uuid);

        Ok(conflict)
    }

    pub async fn get_pending_conflicts(&self) -> Result<Vec<Conflict>> {
        self.db.audit.get_pending_conflicts().await
    }

    pub async fn submit_blind_count(
        &self,
        location_tag: String,
        counted_items: Vec<(Uuid, i32)>,
    ) -> Result<Vec<Conflict>> {
        let expected_items = self.db.inventory.get_by_location(&location_tag).await?;

        // Aggregate expected by ProductUUID
        let mut expected_map: std::collections::HashMap<Uuid, i32> =
            std::collections::HashMap::new();
        for item in expected_items {
            *expected_map.entry(item.product_uuid).or_insert(0) += item.quantity_on_hand;
        }

        let mut conflicts = Vec::new();
        let mut counted_map: std::collections::HashMap<Uuid, i32> =
            std::collections::HashMap::new();

        for (pid, qty) in counted_items {
            *counted_map.entry(pid).or_insert(0) += qty;
        }

        // Check counted vs expected
        for (pid, counted_qty) in &counted_map {
            let expected_qty = *expected_map.get(pid).unwrap_or(&0);
            if *counted_qty != expected_qty {
                // Conflict
                conflicts.push(
                    self.create_conflict(
                        *pid,
                        ConflictType::PhysicalMiscount,
                        expected_qty,
                        *counted_qty,
                        None,
                    )
                    .await?,
                );
            }
        }

        // Check for items expected but not counted
        for (pid, expected_qty) in &expected_map {
            if !counted_map.contains_key(pid) {
                // Missing entire pile
                conflicts.push(
                    self.create_conflict(
                        *pid,
                        ConflictType::PhysicalMiscount,
                        *expected_qty,
                        0,
                        None,
                    )
                    .await?,
                );
            }
        }

        Ok(conflicts)
    }

    pub async fn resolve_conflict(
        &self,
        conflict_uuid: Uuid,
        resolution: ResolutionStatus,
    ) -> Result<()> {
        self.db.audit.update_status(conflict_uuid, resolution).await
    }
}
