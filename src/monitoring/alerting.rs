//! Alerting System
//!
//! TASK-215 to TASK-218: Provides alerting for error rates, sync failures,
//! disk space, and database issues

use crate::database::Database;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use sysinfo::Disks;
use tokio::sync::RwLock;

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// An alert that has been triggered
#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    pub id: String,
    pub severity: AlertSeverity,
    pub category: String,
    pub message: String,
    pub triggered_at: DateTime<Utc>,
    pub details: Option<serde_json::Value>,
}

/// Alert thresholds configuration
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    /// Error rate threshold (percentage) - default 5%
    pub error_rate_warning: f64,
    pub error_rate_critical: f64,

    /// Disk space thresholds in GB
    pub disk_space_warning_gb: f64,
    pub disk_space_critical_gb: f64,

    /// Sync backlog thresholds
    pub sync_backlog_warning: u64,
    pub sync_backlog_critical: u64,

    /// Sync stale threshold (minutes since last sync)
    pub sync_stale_minutes: i64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            error_rate_warning: 1.0,     // 1% error rate warning
            error_rate_critical: 5.0,    // 5% error rate critical
            disk_space_warning_gb: 5.0,  // 5 GB warning
            disk_space_critical_gb: 1.0, // 1 GB critical
            sync_backlog_warning: 100,   // 100 pending changes warning
            sync_backlog_critical: 500,  // 500 pending changes critical
            sync_stale_minutes: 60,      // 1 hour stale sync warning
        }
    }
}

/// Alerting service that monitors system health and triggers alerts
pub struct AlertingService {
    db: Arc<Database>,
    thresholds: AlertThresholds,

    // Sliding window for error rate calculation
    request_count: AtomicU64,
    error_count: AtomicU64,

    // Active alerts
    active_alerts: RwLock<Vec<Alert>>,
}

impl AlertingService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            thresholds: AlertThresholds::default(),
            request_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            active_alerts: RwLock::new(Vec::new()),
        }
    }

    pub fn with_thresholds(mut self, thresholds: AlertThresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    /// Record a request (for error rate calculation)
    pub fn record_request(&self, is_error: bool) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
        if is_error {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get current error rate percentage
    pub fn get_error_rate(&self) -> f64 {
        let requests = self.request_count.load(Ordering::Relaxed);
        let errors = self.error_count.load(Ordering::Relaxed);

        if requests == 0 {
            return 0.0;
        }

        (errors as f64 / requests as f64) * 100.0
    }

    /// Reset counters (call periodically, e.g., every hour)
    pub fn reset_counters(&self) {
        self.request_count.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
    }

    /// Run all alert checks and return any triggered alerts
    pub async fn check_all(&self) -> Vec<Alert> {
        let mut alerts = Vec::new();

        // TASK-215: Error rate alerting
        if let Some(alert) = self.check_error_rate().await {
            alerts.push(alert);
        }

        // TASK-216: Sync failure alerts
        if let Some(alert) = self.check_sync_status().await {
            alerts.push(alert);
        }

        // TASK-217: Low disk space alerts
        if let Some(alert) = self.check_disk_space().await {
            alerts.push(alert);
        }

        // TASK-218: Database connection alerts
        if let Some(alert) = self.check_database().await {
            alerts.push(alert);
        }

        // Store active alerts
        {
            let mut active = self.active_alerts.write().await;
            *active = alerts.clone();
        }

        alerts
    }

    /// Get currently active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        self.active_alerts.read().await.clone()
    }

    /// TASK-215: Check error rate
    async fn check_error_rate(&self) -> Option<Alert> {
        let error_rate = self.get_error_rate();

        if error_rate >= self.thresholds.error_rate_critical {
            Some(Alert {
                id: "error-rate-critical".to_string(),
                severity: AlertSeverity::Critical,
                category: "errors".to_string(),
                message: format!("Critical error rate: {:.1}%", error_rate),
                triggered_at: Utc::now(),
                details: Some(serde_json::json!({
                    "error_rate": error_rate,
                    "threshold": self.thresholds.error_rate_critical,
                    "requests": self.request_count.load(Ordering::Relaxed),
                    "errors": self.error_count.load(Ordering::Relaxed)
                })),
            })
        } else if error_rate >= self.thresholds.error_rate_warning {
            Some(Alert {
                id: "error-rate-warning".to_string(),
                severity: AlertSeverity::Warning,
                category: "errors".to_string(),
                message: format!("Elevated error rate: {:.1}%", error_rate),
                triggered_at: Utc::now(),
                details: Some(serde_json::json!({
                    "error_rate": error_rate,
                    "threshold": self.thresholds.error_rate_warning
                })),
            })
        } else {
            None
        }
    }

    /// TASK-216: Check sync status
    async fn check_sync_status(&self) -> Option<Alert> {
        // Check pending changes count
        match self.db.sync.get_changes_since(0, 1000).await {
            Ok(changes) => {
                let pending_count = changes.len() as u64;

                if pending_count >= self.thresholds.sync_backlog_critical {
                    return Some(Alert {
                        id: "sync-backlog-critical".to_string(),
                        severity: AlertSeverity::Critical,
                        category: "sync".to_string(),
                        message: format!(
                            "Critical sync backlog: {} pending changes",
                            pending_count
                        ),
                        triggered_at: Utc::now(),
                        details: Some(serde_json::json!({
                            "pending_changes": pending_count,
                            "threshold": self.thresholds.sync_backlog_critical
                        })),
                    });
                } else if pending_count >= self.thresholds.sync_backlog_warning {
                    return Some(Alert {
                        id: "sync-backlog-warning".to_string(),
                        severity: AlertSeverity::Warning,
                        category: "sync".to_string(),
                        message: format!("High sync backlog: {} pending changes", pending_count),
                        triggered_at: Utc::now(),
                        details: Some(serde_json::json!({
                            "pending_changes": pending_count,
                            "threshold": self.thresholds.sync_backlog_warning
                        })),
                    });
                }
            }
            Err(e) => {
                return Some(Alert {
                    id: "sync-check-failed".to_string(),
                    severity: AlertSeverity::Warning,
                    category: "sync".to_string(),
                    message: format!("Failed to check sync status: {}", e),
                    triggered_at: Utc::now(),
                    details: None,
                });
            }
        }

        None
    }

    /// TASK-217: Check disk space
    async fn check_disk_space(&self) -> Option<Alert> {
        let disks = Disks::new_with_refreshed_list();

        let mut min_free_gb = f64::MAX;
        let mut critical_disk = None;

        for disk in disks.list() {
            let free_gb = disk.available_space() as f64 / 1_073_741_824.0;
            if free_gb < min_free_gb {
                min_free_gb = free_gb;
                critical_disk = Some(disk.mount_point().to_string_lossy().to_string());
            }
        }

        if min_free_gb < self.thresholds.disk_space_critical_gb {
            Some(Alert {
                id: "disk-space-critical".to_string(),
                severity: AlertSeverity::Critical,
                category: "disk".to_string(),
                message: format!("Critical: Only {:.1} GB disk space remaining", min_free_gb),
                triggered_at: Utc::now(),
                details: Some(serde_json::json!({
                    "free_gb": min_free_gb,
                    "mount_point": critical_disk,
                    "threshold": self.thresholds.disk_space_critical_gb
                })),
            })
        } else if min_free_gb < self.thresholds.disk_space_warning_gb {
            Some(Alert {
                id: "disk-space-warning".to_string(),
                severity: AlertSeverity::Warning,
                category: "disk".to_string(),
                message: format!("Warning: Only {:.1} GB disk space remaining", min_free_gb),
                triggered_at: Utc::now(),
                details: Some(serde_json::json!({
                    "free_gb": min_free_gb,
                    "mount_point": critical_disk,
                    "threshold": self.thresholds.disk_space_warning_gb
                })),
            })
        } else {
            None
        }
    }

    /// TASK-218: Check database connection
    async fn check_database(&self) -> Option<Alert> {
        let start = std::time::Instant::now();

        match sqlx::query("SELECT 1").fetch_one(&self.db.pool).await {
            Ok(_) => {
                let latency_ms = start.elapsed().as_millis() as u64;

                // Alert if database is very slow (> 5 seconds)
                if latency_ms > 5000 {
                    return Some(Alert {
                        id: "database-slow".to_string(),
                        severity: AlertSeverity::Warning,
                        category: "database".to_string(),
                        message: format!("Database responding slowly: {}ms", latency_ms),
                        triggered_at: Utc::now(),
                        details: Some(serde_json::json!({
                            "latency_ms": latency_ms
                        })),
                    });
                }
                None
            }
            Err(e) => Some(Alert {
                id: "database-unreachable".to_string(),
                severity: AlertSeverity::Critical,
                category: "database".to_string(),
                message: format!("Database connection failed: {}", e),
                triggered_at: Utc::now(),
                details: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_error_rate_calculation() {
        let db = crate::database::initialize_test_db()
            .await
            .expect("Failed to init db");
        let service = AlertingService {
            db,
            thresholds: AlertThresholds::default(),
            request_count: AtomicU64::new(100),
            error_count: AtomicU64::new(5),
            active_alerts: RwLock::new(Vec::new()),
        };

        // This test would need proper setup, just showing structure
        assert_eq!(service.get_error_rate(), 5.0);
    }
}
