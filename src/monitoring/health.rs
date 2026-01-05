//! Health Check System
//!
//! Provides comprehensive health checks for VaultSync components

use crate::database::Database;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;
use sysinfo::Disks;

/// Overall system health status
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Individual component check result
#[derive(Debug, Clone, Serialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ComponentHealth {
    pub fn ok() -> Self {
        Self {
            status: HealthStatus::Healthy,
            latency_ms: None,
            message: None,
            details: None,
        }
    }

    pub fn ok_with_latency(latency_ms: u64) -> Self {
        Self {
            status: HealthStatus::Healthy,
            latency_ms: Some(latency_ms),
            message: None,
            details: None,
        }
    }

    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            latency_ms: None,
            message: Some(message.into()),
            details: None,
        }
    }

    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            latency_ms: None,
            message: Some(message.into()),
            details: None,
        }
    }
}

/// Comprehensive health check response
#[derive(Debug, Clone, Serialize)]
pub struct HealthCheckResponse {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: DateTime<Utc>,
    pub checks: HealthChecks,
}

/// Individual health checks
#[derive(Debug, Clone, Serialize)]
pub struct HealthChecks {
    pub database: ComponentHealth,
    pub disk: ComponentHealth,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<ComponentHealth>,
}

/// Health check service
pub struct HealthService {
    db: Arc<Database>,
    start_time: Instant,
    version: String,
    disk_warning_threshold_gb: f64,
    disk_critical_threshold_gb: f64,
}

impl HealthService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            start_time: Instant::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            disk_warning_threshold_gb: 5.0,
            disk_critical_threshold_gb: 1.0,
        }
    }

    /// Simple health check (just returns OK if server is running)
    pub fn check_basic(&self) -> serde_json::Value {
        serde_json::json!({
            "status": "ok",
            "version": self.version,
            "uptime_seconds": self.start_time.elapsed().as_secs()
        })
    }

    /// Comprehensive health check with all components
    pub async fn check_detailed(&self) -> HealthCheckResponse {
        let db_health = self.check_database().await;
        let disk_health = self.check_disk();
        let sync_health = self.check_sync().await;

        // Determine overall status (worst of all checks)
        let overall_status = self.determine_overall_status(&db_health, &disk_health, &sync_health);

        HealthCheckResponse {
            status: overall_status,
            version: self.version.clone(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            timestamp: Utc::now(),
            checks: HealthChecks {
                database: db_health,
                disk: disk_health,
                sync: sync_health,
            },
        }
    }

    /// Check database connectivity
    async fn check_database(&self) -> ComponentHealth {
        let start = Instant::now();

        // Simple query to verify database is responsive
        match sqlx::query("SELECT 1").fetch_one(&self.db.pool).await {
            Ok(_) => {
                let latency = start.elapsed().as_millis() as u64;
                if latency > 1000 {
                    ComponentHealth {
                        status: HealthStatus::Degraded,
                        latency_ms: Some(latency),
                        message: Some("Database responding slowly".to_string()),
                        details: None,
                    }
                } else {
                    ComponentHealth::ok_with_latency(latency)
                }
            }
            Err(e) => ComponentHealth::unhealthy(format!("Database error: {}", e)),
        }
    }

    /// Check available disk space
    fn check_disk(&self) -> ComponentHealth {
        let disks = Disks::new_with_refreshed_list();

        // Find the disk where the database is located (typically the system disk)
        // For simplicity, check the first disk with the most space
        let mut min_free_gb = f64::MAX;
        let mut disk_details = Vec::new();

        for disk in disks.list() {
            let free_gb = disk.available_space() as f64 / 1_073_741_824.0; // Convert to GB
            let total_gb = disk.total_space() as f64 / 1_073_741_824.0;
            let used_percent = ((total_gb - free_gb) / total_gb * 100.0).round();

            min_free_gb = min_free_gb.min(free_gb);

            disk_details.push(serde_json::json!({
                "mount": disk.mount_point().to_string_lossy(),
                "free_gb": (free_gb * 10.0).round() / 10.0,
                "total_gb": (total_gb * 10.0).round() / 10.0,
                "used_percent": used_percent
            }));
        }

        if min_free_gb < self.disk_critical_threshold_gb {
            ComponentHealth {
                status: HealthStatus::Unhealthy,
                latency_ms: None,
                message: Some(format!("Critical: Only {:.1} GB free", min_free_gb)),
                details: Some(serde_json::json!(disk_details)),
            }
        } else if min_free_gb < self.disk_warning_threshold_gb {
            ComponentHealth {
                status: HealthStatus::Degraded,
                latency_ms: None,
                message: Some(format!("Warning: Only {:.1} GB free", min_free_gb)),
                details: Some(serde_json::json!(disk_details)),
            }
        } else {
            ComponentHealth {
                status: HealthStatus::Healthy,
                latency_ms: None,
                message: None,
                details: Some(serde_json::json!({"free_gb": (min_free_gb * 10.0).round() / 10.0})),
            }
        }
    }

    /// Check sync service status
    async fn check_sync(&self) -> Option<ComponentHealth> {
        // Check pending sync changes
        match self.db.sync.get_changes_since(0, 1000).await {
            Ok(changes) => {
                let pending_count = changes.len();

                if pending_count > 500 {
                    Some(ComponentHealth {
                        status: HealthStatus::Degraded,
                        latency_ms: None,
                        message: Some(format!(
                            "High sync backlog: {} pending changes",
                            pending_count
                        )),
                        details: Some(serde_json::json!({"pending_changes": pending_count})),
                    })
                } else {
                    Some(ComponentHealth {
                        status: HealthStatus::Healthy,
                        latency_ms: None,
                        message: None,
                        details: Some(serde_json::json!({"pending_changes": pending_count})),
                    })
                }
            }
            Err(e) => Some(ComponentHealth::degraded(format!(
                "Sync check failed: {}",
                e
            ))),
        }
    }

    fn determine_overall_status(
        &self,
        db: &ComponentHealth,
        disk: &ComponentHealth,
        sync: &Option<ComponentHealth>,
    ) -> HealthStatus {
        // Any unhealthy component makes the whole system unhealthy
        if matches!(db.status, HealthStatus::Unhealthy)
            || matches!(disk.status, HealthStatus::Unhealthy)
        {
            return HealthStatus::Unhealthy;
        }

        // Any degraded component makes the whole system degraded
        if matches!(db.status, HealthStatus::Degraded)
            || matches!(disk.status, HealthStatus::Degraded)
            || sync
                .as_ref()
                .map(|s| matches!(s.status, HealthStatus::Degraded))
                .unwrap_or(false)
        {
            return HealthStatus::Degraded;
        }

        HealthStatus::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_health_constructors() {
        let ok = ComponentHealth::ok();
        assert!(matches!(ok.status, HealthStatus::Healthy));

        let degraded = ComponentHealth::degraded("slow");
        assert!(matches!(degraded.status, HealthStatus::Degraded));
        assert_eq!(degraded.message, Some("slow".to_string()));

        let unhealthy = ComponentHealth::unhealthy("failed");
        assert!(matches!(unhealthy.status, HealthStatus::Unhealthy));
    }
}
