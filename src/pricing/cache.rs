use crate::core::PriceInfo;
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
struct CacheEntry {
    price_info: PriceInfo,
    cached_at: DateTime<Utc>,
    ttl_seconds: i64,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        Utc::now() > self.cached_at + Duration::seconds(self.ttl_seconds)
    }
}

pub struct PriceCache {
    entries: RwLock<HashMap<Uuid, CacheEntry>>,
    default_ttl_seconds: i64,
    max_entries: usize,
}

#[derive(Serialize)]
pub struct CacheStats {
    pub entries: usize,
    pub max_entries: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

impl PriceCache {
    pub fn new(ttl_seconds: i64, max_entries: usize) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            default_ttl_seconds: ttl_seconds,
            max_entries,
        }
    }

    pub async fn get(&self, product_uuid: Uuid) -> Option<PriceInfo> {
        let entries = self.entries.read().await;

        if let Some(entry) = entries.get(&product_uuid) {
            if !entry.is_expired() {
                // Tracking hits/misses would require atomic counters or write lock
                // For simplicity avoiding write lock on read path
                return Some(entry.price_info.clone());
            }
        }

        None
    }

    pub async fn set(&self, price_info: PriceInfo) {
        let mut entries = self.entries.write().await;

        // Periodic cleanup check? Or strict limit?
        if entries.len() >= self.max_entries {
            // Quick check if we have space after random/old eviction?
            // Clean up ANY expired entries first
            entries.retain(|_, v| !v.is_expired());

            // If still full, remove oldest
            if entries.len() >= self.max_entries {
                // O(N) scan for oldest cached_at
                let oldest = entries
                    .iter()
                    .min_by_key(|(_, v)| v.cached_at)
                    .map(|(k, _)| *k);

                if let Some(k) = oldest {
                    entries.remove(&k);
                }
            }
        }

        entries.insert(
            price_info.product_uuid,
            CacheEntry {
                price_info,
                cached_at: Utc::now(),
                ttl_seconds: self.default_ttl_seconds,
            },
        );
    }

    pub async fn get_stats(&self) -> CacheStats {
        let entries = self.entries.read().await;
        CacheStats {
            entries: entries.len(),
            max_entries: self.max_entries,
            hits: 0, // Not implementing atomic counters yet
            misses: 0,
            hit_rate: 0.0,
        }
    }

    pub async fn invalidate(&self, product_uuid: Uuid) {
        let mut entries = self.entries.write().await;
        entries.remove(&product_uuid);
    }

    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }
}
