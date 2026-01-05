//! Sync Actor - Message-passing based sync service
//!
//! This module implements the Actor Pattern to eliminate the global Mutex convoy.
//! Instead of locking the entire SyncService for each operation, callers send
//! messages to a dedicated sync worker task.
//!
//! ## Why Actor Pattern?
//!
//! The previous design used `Arc<Mutex<SyncService>>` which caused:
//! - All sync operations to block each other (convoy effect)
//! - 10 concurrent clients = 10x latency
//! - Potential deadlocks if sync code called back into handlers
//!
//! With the actor pattern:
//! - Operations are queued and processed sequentially by a single worker
//! - Callers don't block waiting for a lock
//! - Natural backpressure via channel capacity

use crate::core::{InventoryItem, Ordering, Product, RecordType, SyncOperation, VectorTimestamp};
use crate::database::Database;
use crate::errors::Result;
use crate::network::NetworkService;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tracing;
use uuid::Uuid;

/// Commands that can be sent to the sync actor
#[derive(Debug)]
pub enum SyncCommand {
    /// Trigger a sync with all known peers
    SyncWithPeers {
        response: oneshot::Sender<Result<()>>,
    },

    /// Apply remote changes received from a peer
    ApplyChanges {
        changes: Vec<super::ChangeRecord>,
        response: oneshot::Sender<Result<()>>,
    },

    /// Get the current sync status
    GetStatus {
        response: oneshot::Sender<SyncActorStatus>,
    },

    /// Get discovered devices
    GetDevices {
        response: oneshot::Sender<Vec<crate::network::Device>>,
    },

    /// Manual device pairing
    ManualPair {
        name: String,
        address: std::net::IpAddr,
        port: u16,
        node_id: Option<String>,
        response: oneshot::Sender<Result<()>>,
    },
}

/// Status returned by the sync actor
#[derive(Debug, Clone)]
pub struct SyncActorStatus {
    pub last_sync: Option<DateTime<Utc>>,
    pub connected_peers: usize,
    pub pending_changes: usize,
    pub is_synced: bool,
}

/// Handle to communicate with the sync actor
#[derive(Clone)]
pub struct SyncActorHandle {
    sender: mpsc::Sender<SyncCommand>,
}

impl SyncActorHandle {
    /// Trigger a sync with peers (non-blocking)
    pub async fn sync_with_peers(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(SyncCommand::SyncWithPeers { response: tx })
            .await
            .map_err(|_| anyhow::anyhow!("Sync actor unavailable"))?;

        rx.await
            .map_err(|_| anyhow::anyhow!("Sync actor dropped"))?
    }

    /// Apply remote changes (non-blocking queue)
    pub async fn apply_changes(&self, changes: Vec<super::ChangeRecord>) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(SyncCommand::ApplyChanges {
                changes,
                response: tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Sync actor unavailable"))?;

        rx.await
            .map_err(|_| anyhow::anyhow!("Sync actor dropped"))?
    }

    /// Get sync status
    pub async fn get_status(&self) -> SyncActorStatus {
        let (tx, rx) = oneshot::channel();
        if self
            .sender
            .send(SyncCommand::GetStatus { response: tx })
            .await
            .is_ok()
        {
            rx.await.unwrap_or(SyncActorStatus {
                last_sync: None,
                connected_peers: 0,
                pending_changes: 0,
                is_synced: false,
            })
        } else {
            SyncActorStatus {
                last_sync: None,
                connected_peers: 0,
                pending_changes: 0,
                is_synced: false,
            }
        }
    }

    /// Get discovered devices
    pub async fn get_devices(&self) -> Vec<crate::network::Device> {
        let (tx, rx) = oneshot::channel();
        if self
            .sender
            .send(SyncCommand::GetDevices { response: tx })
            .await
            .is_ok()
        {
            rx.await.unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    /// Manual device pairing
    pub async fn manual_pair(
        &self,
        name: String,
        address: std::net::IpAddr,
        port: u16,
        node_id: Option<String>,
    ) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(SyncCommand::ManualPair {
                name,
                address,
                port,
                node_id,
                response: tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Sync actor unavailable"))?;

        rx.await
            .map_err(|_| anyhow::anyhow!("Sync actor dropped"))?
    }
}

/// The sync actor that processes commands sequentially
pub struct SyncActor {
    db: Arc<Database>,
    network: Option<NetworkService>,
    node_id: String,
    last_sync_time: Option<DateTime<Utc>>,
    receiver: mpsc::Receiver<SyncCommand>,
}

impl SyncActor {
    /// Create the actor and its handle
    pub fn new(
        db: Arc<Database>,
        network: Option<NetworkService>,
        node_id: String,
        buffer_size: usize,
    ) -> (SyncActorHandle, Self) {
        let (sender, receiver) = mpsc::channel(buffer_size);

        let handle = SyncActorHandle { sender };
        let actor = Self {
            db,
            network,
            node_id,
            last_sync_time: None,
            receiver,
        };

        (handle, actor)
    }

    /// Run the actor's main loop (call this in a spawned task)
    pub async fn run(mut self) {
        tracing::info!("SyncActor started for node {}", self.node_id);

        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                SyncCommand::SyncWithPeers { response } => {
                    let result = self.do_sync_with_peers().await;
                    let _ = response.send(result);
                }

                SyncCommand::ApplyChanges { changes, response } => {
                    let result = self.do_apply_changes(changes).await;
                    let _ = response.send(result);
                }

                SyncCommand::GetStatus { response } => {
                    let status = self.get_status().await;
                    let _ = response.send(status);
                }

                SyncCommand::GetDevices { response } => {
                    let devices = self.get_devices().await;
                    let _ = response.send(devices);
                }

                SyncCommand::ManualPair {
                    name,
                    address,
                    port,
                    node_id,
                    response,
                } => {
                    let result = self.do_manual_pair(name, address, port, node_id).await;
                    let _ = response.send(result);
                }
            }
        }

        tracing::info!("SyncActor shutting down");
    }

    async fn do_sync_with_peers(&mut self) -> Result<()> {
        tracing::info!("Starting sync with peer devices...");

        let peers = if let Some(network) = &self.network {
            network.get_connected_devices().await
        } else {
            tracing::warn!("Network service not available, skipping sync");
            Vec::new()
        };

        if peers.is_empty() {
            tracing::info!("No peers found to sync with.");
        }

        for peer in peers {
            if let Err(e) = self.sync_with_device(&peer).await {
                tracing::error!("Failed to sync with {}: {}", peer.name, e);
            }
        }

        self.last_sync_time = Some(Utc::now());
        tracing::info!("Completed sync with peer devices");
        Ok(())
    }

    async fn sync_with_device(&self, device: &crate::network::Device) -> Result<()> {
        tracing::info!("Syncing with device at {}:{}", device.address, device.port);

        let client = reqwest::Client::new();
        const SYNC_BATCH_SIZE: i64 = 100;

        // Push local changes
        let changes = self.db.sync.get_changes_since(0, SYNC_BATCH_SIZE).await?;

        if !changes.is_empty() {
            tracing::info!("Pushing {} changes to {}", changes.len(), device.name);

            let push_url = format!("http://{}:{}/api/sync/push", device.address, device.port);

            let payload: Vec<serde_json::Value> = changes
                .iter()
                .map(|(id, rtype, op, data, _node, local_clock, vv_str, ts)| {
                    let vector_timestamp: crate::core::VectorTimestamp =
                        serde_json::from_str(vv_str)
                            .unwrap_or_else(|_| crate::core::VectorTimestamp::new());
                    serde_json::json!({
                        "record_id": id,
                        "record_type": rtype,
                        "operation": op,
                        "data": data,
                        "vector_timestamp": vector_timestamp,
                        "timestamp": ts,
                        "sequence_number": local_clock
                    })
                })
                .collect();

            match client
                .post(&push_url)
                .json(&payload)
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    tracing::info!(
                        "Successfully pushed {} changes to {}",
                        changes.len(),
                        device.name
                    );
                }
                Ok(resp) => {
                    tracing::warn!("Failed to push to {}: {}", device.name, resp.status());
                }
                Err(e) => {
                    tracing::error!("Network error pushing to {}: {}", device.name, e);
                }
            }
        }

        // Pull remote changes
        let pull_url = format!(
            "http://{}:{}/api/sync/pull?since_clock=0",
            device.address, device.port
        );

        match client
            .get(&pull_url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(remote_changes) = resp.json::<Vec<super::ChangeRecord>>().await {
                    if !remote_changes.is_empty() {
                        tracing::info!(
                            "Received {} changes from {}",
                            remote_changes.len(),
                            device.name
                        );
                        self.do_apply_changes(remote_changes).await?;
                    }
                }
            }
            Ok(resp) => {
                tracing::warn!("Failed to pull from {}: {}", device.name, resp.status());
            }
            Err(e) => {
                tracing::error!("Network error pulling from {}: {}", device.name, e);
            }
        }

        Ok(())
    }

    async fn do_apply_changes(&self, changes: Vec<super::ChangeRecord>) -> Result<()> {
        for change in changes {
            tracing::debug!(
                "Applying change: {} ({:?})",
                change.record_id,
                change.operation
            );

            // Conflict Detection
            let local_vector = self
                .db
                .sync
                .get_version_vector(&change.record_id)
                .await?
                .unwrap_or_else(VectorTimestamp::new);

            let ordering = local_vector.compare(&change.vector_timestamp);

            match ordering {
                Ordering::Less => {
                    // Local < Remote: Fast forward (Apply change)
                    self.apply_change_db(&change).await?;
                    // Update local vector
                    self.db
                        .sync
                        .update_version_vector(&change.record_id, &change.vector_timestamp)
                        .await?;
                }
                Ordering::Greater => {
                    // Local > Remote: Stale update, ignore
                    tracing::debug!("Ignoring stale update for {}", change.record_id);
                }
                Ordering::Equal => {
                    // Already have this state, ignore
                }
                Ordering::Concurrent => {
                    // Conflict!
                    tracing::warn!("Conflict detected for {}!", change.record_id);

                    // RECORD THE CONFLICT
                    if let Err(e) = self
                        .db
                        .record_sync_conflict(
                            &format!("{:?}", change.record_type),
                            &change.record_id,
                            "Concurrent_Mod",
                            "Remote_Peer",
                            &change.data,
                            &change.vector_timestamp,
                        )
                        .await
                    {
                        tracing::error!("Failed to persist conflict record: {}", e);
                    }

                    // Resolve Conflict (Auto-resolution via LWW or Merge)
                    let resolved_change = self.resolve_conflict(&change, &local_vector).await?;

                    if let Some(final_change) = resolved_change {
                        self.apply_change_db(&final_change).await?;
                    }

                    // Always merge vectors after resolution
                    let mut merged_vector = local_vector.clone();
                    merged_vector.merge(&change.vector_timestamp);
                    self.db
                        .sync
                        .update_version_vector(&change.record_id, &merged_vector)
                        .await?;
                }
            }
        }
        Ok(())
    }

    async fn apply_change_db(&self, change: &super::ChangeRecord) -> Result<()> {
        match change.record_type {
            RecordType::Product => {
                if let Ok(product) = serde_json::from_value::<Product>(change.data.clone()) {
                    self.db.products.insert(&product).await?;
                }
            }
            RecordType::InventoryItem => match change.operation {
                SyncOperation::Delete => {
                    if let Some(uuid_str) =
                        change.data.get("inventory_uuid").and_then(|v| v.as_str())
                    {
                        if let Ok(uuid) = Uuid::parse_str(uuid_str) {
                            self.db.inventory.delete(uuid).await?;
                        }
                    }
                }
                _ => {
                    if let Ok(item) = serde_json::from_value::<InventoryItem>(change.data.clone()) {
                        self.db.inventory.insert(&item).await?;
                    }
                }
            },
            RecordType::Transaction => {
                if let Ok(transaction) =
                    serde_json::from_value::<crate::core::Transaction>(change.data.clone())
                {
                    self.db.transactions.insert(&transaction).await?;
                }
            }
            RecordType::WantsList => {
                if let Ok(list) =
                    serde_json::from_value::<crate::core::WantsList>(change.data.clone())
                {
                    self.db.customers.save_wants_list(&list).await?;
                }
            }
            RecordType::Event => {
                if let Ok(event) = serde_json::from_value::<crate::core::Event>(change.data.clone())
                {
                    self.db.events.insert(&event).await?;
                }
            }
            RecordType::EventParticipant => {
                if let Ok(participant) =
                    serde_json::from_value::<crate::core::EventParticipant>(change.data.clone())
                {
                    self.db.events.register_participant(&participant).await?;
                }
            }
            _ => {
                tracing::warn!("Unsupported record type for sync: {:?}", change.record_type);
            }
        }
        Ok(())
    }

    async fn resolve_conflict(
        &self,
        remote_change: &super::ChangeRecord,
        _local_vector: &VectorTimestamp,
    ) -> Result<Option<super::ChangeRecord>> {
        match remote_change.record_type {
            RecordType::Product => {
                if let Ok(uuid) = Uuid::parse_str(&remote_change.record_id) {
                    if let Some(local_product) = self.db.products.get_by_id(uuid).await? {
                        if let Ok(remote_product) =
                            serde_json::from_value::<Product>(remote_change.data.clone())
                        {
                            tracing::info!(
                                "Merging product metadata for conflict resolution: {}",
                                remote_product.name
                            );

                            let mut final_product = remote_product.clone();
                            if let serde_json::Value::Object(mut remote_meta) =
                                final_product.metadata
                            {
                                if let serde_json::Value::Object(local_meta) =
                                    local_product.metadata
                                {
                                    for (k, v) in local_meta {
                                        remote_meta.entry(k).or_insert(v);
                                    }
                                }
                                final_product.metadata = serde_json::Value::Object(remote_meta);
                            } else if final_product.metadata.is_null() {
                                final_product.metadata = local_product.metadata;
                            }

                            let mut resolved = remote_change.clone();
                            resolved.data = serde_json::to_value(final_product)?;
                            return Ok(Some(resolved));
                        }
                    }
                }
                Ok(Some(remote_change.clone()))
            }
            RecordType::InventoryItem => {
                if remote_change.operation == SyncOperation::Delete {
                    return Ok(Some(remote_change.clone()));
                }
                if let Some(local_op_str) = self
                    .db
                    .sync
                    .get_last_sync_operation(&remote_change.record_id)
                    .await?
                {
                    if local_op_str == "Delete" || local_op_str == "SoftDelete" {
                        tracing::info!(
                            "Conflict: Local delete wins for {}",
                            remote_change.record_id
                        );
                        return Ok(None);
                    }
                }
                // LWW Fallback
                tracing::warn!(
                    "Resolving Inventory conflict for {} (LWW).",
                    remote_change.record_id
                );
                Ok(Some(remote_change.clone()))
            }
            _ => Ok(Some(remote_change.clone())),
        }
    }

    async fn get_status(&self) -> SyncActorStatus {
        let pending_changes = match self.db.sync.get_changes_since(0, 10000).await {
            Ok(changes) => changes.len(),
            Err(_) => 0,
        };

        let connected_peers = if let Some(network) = &self.network {
            network.get_connected_devices().await.len()
        } else {
            0
        };

        let is_synced = if let Some(last) = self.last_sync_time {
            let age = Utc::now() - last;
            age.num_minutes() < 5 && pending_changes == 0
        } else {
            false
        };

        SyncActorStatus {
            last_sync: self.last_sync_time,
            connected_peers,
            pending_changes,
            is_synced,
        }
    }

    async fn get_devices(&self) -> Vec<crate::network::Device> {
        if let Some(network) = &self.network {
            network.get_connected_devices().await
        } else {
            Vec::new()
        }
    }

    async fn do_manual_pair(
        &mut self,
        name: String,
        address: std::net::IpAddr,
        port: u16,
        node_id: Option<String>,
    ) -> Result<()> {
        if let Some(network) = &self.network {
            network
                .manual_add_device(name, address, port, node_id)
                .await
        } else {
            Err(anyhow::anyhow!("Network service not available"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_actor_status() {
        // This would require a mock database, but we can at least test the types compile
        let status = SyncActorStatus {
            last_sync: Some(Utc::now()),
            connected_peers: 0,
            pending_changes: 0,
            is_synced: true,
        };
        assert!(status.is_synced);
    }
}
