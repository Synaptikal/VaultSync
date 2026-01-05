// P0-3 Fix: Actor pattern for sync to eliminate global Mutex convoy
pub mod actor;
pub use actor::{SyncActor, SyncActorHandle, SyncActorStatus, SyncCommand};

use crate::core::{RecordType, SyncOperation, VectorTimestamp};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChangeRecord {
    pub record_id: String,
    pub record_type: RecordType,
    pub operation: SyncOperation,
    pub data: serde_json::Value,
    pub vector_timestamp: VectorTimestamp,
    pub timestamp: DateTime<Utc>,
    /// Server-local sequence number for delta sync
    pub sequence_number: Option<u64>,
    /// TASK-123: Checksum for data integrity verification
    #[serde(default)]
    pub checksum: Option<String>,
}

impl ChangeRecord {
    /// TASK-123: Calculate checksum of record data
    pub fn calculate_checksum(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.record_id.hash(&mut hasher);
        format!("{:?}", self.record_type).hash(&mut hasher);
        format!("{:?}", self.operation).hash(&mut hasher);
        self.data.to_string().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// TASK-123: Verify checksum matches data
    pub fn verify_checksum(&self) -> bool {
        match &self.checksum {
            Some(cs) => cs == &self.calculate_checksum(),
            None => true, // No checksum = skip verification
        }
    }
}

pub struct SyncDataConflict {
    pub record_id: String,
    pub conflicting_values: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct SyncStatus {
    pub last_sync: Option<DateTime<Utc>>,
    pub connected_peers: usize,
    pub pending_changes: usize,
    pub is_synced: bool,
}

#[cfg(test)]
mod tests {
    // Legacy tests removed.
}
