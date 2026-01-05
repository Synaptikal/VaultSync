//! Metrics System
//!
//! Provides simple metrics collection and Prometheus-compatible output
//! Uses a lightweight custom implementation to avoid heavy dependencies

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

/// Simple counter metric
pub struct Counter {
    value: AtomicU64,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    pub fn increment(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add(&self, n: u64) {
        self.value.fetch_add(n, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple gauge metric
pub struct Gauge {
    value: AtomicU64,
}

impl Gauge {
    pub fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    pub fn set(&self, value: f64) {
        self.value.store(value.to_bits(), Ordering::Relaxed);
    }

    pub fn get(&self) -> f64 {
        f64::from_bits(self.value.load(Ordering::Relaxed))
    }
}

impl Default for Gauge {
    fn default() -> Self {
        Self::new()
    }
}

/// Application metrics registry
pub struct MetricsRegistry {
    // HTTP Metrics
    pub http_requests_total: Counter,
    pub http_errors_total: Counter,

    // Business Metrics
    pub transactions_total: Counter,
    pub daily_sales_total: Gauge,
    pub daily_transactions_count: Gauge,

    // Inventory Metrics
    pub inventory_items_total: Gauge,
    pub inventory_value_total: Gauge,
    pub low_stock_items: Gauge,

    // Sync Metrics
    pub sync_pending_changes: Gauge,
    pub sync_connected_peers: Gauge,

    // System Metrics
    pub uptime_seconds: Gauge,
    pub disk_free_gb: Gauge,

    // Labeled counters (endpoint -> count)
    endpoint_requests: RwLock<HashMap<String, u64>>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            http_requests_total: Counter::new(),
            http_errors_total: Counter::new(),
            transactions_total: Counter::new(),
            daily_sales_total: Gauge::new(),
            daily_transactions_count: Gauge::new(),
            inventory_items_total: Gauge::new(),
            inventory_value_total: Gauge::new(),
            low_stock_items: Gauge::new(),
            sync_pending_changes: Gauge::new(),
            sync_connected_peers: Gauge::new(),
            uptime_seconds: Gauge::new(),
            disk_free_gb: Gauge::new(),
            endpoint_requests: RwLock::new(HashMap::new()),
        }
    }

    /// Record an HTTP request by endpoint
    pub fn record_request(&self, endpoint: &str, is_error: bool) {
        self.http_requests_total.increment();
        if is_error {
            self.http_errors_total.increment();
        }

        if let Ok(mut map) = self.endpoint_requests.write() {
            *map.entry(endpoint.to_string()).or_insert(0) += 1;
        }
    }

    /// Record a transaction
    pub fn record_transaction(&self, amount: f64) {
        self.transactions_total.increment();

        // Update daily totals
        let current_sales = self.daily_sales_total.get();
        self.daily_sales_total.set(current_sales + amount);

        let current_count = self.daily_transactions_count.get();
        self.daily_transactions_count.set(current_count + 1.0);
    }

    /// Render metrics in Prometheus text format
    pub fn render_prometheus(&self) -> String {
        let mut output = String::new();

        // HTTP Metrics
        output.push_str("# HELP vaultsync_http_requests_total Total HTTP requests\n");
        output.push_str("# TYPE vaultsync_http_requests_total counter\n");
        output.push_str(&format!(
            "vaultsync_http_requests_total {}\n",
            self.http_requests_total.get()
        ));

        output.push_str("# HELP vaultsync_http_errors_total Total HTTP errors\n");
        output.push_str("# TYPE vaultsync_http_errors_total counter\n");
        output.push_str(&format!(
            "vaultsync_http_errors_total {}\n",
            self.http_errors_total.get()
        ));

        // Business Metrics
        output.push_str("\n# HELP vaultsync_transactions_total Total transactions processed\n");
        output.push_str("# TYPE vaultsync_transactions_total counter\n");
        output.push_str(&format!(
            "vaultsync_transactions_total {}\n",
            self.transactions_total.get()
        ));

        output.push_str("# HELP vaultsync_daily_sales_total Today's sales total\n");
        output.push_str("# TYPE vaultsync_daily_sales_total gauge\n");
        output.push_str(&format!(
            "vaultsync_daily_sales_total {:.2}\n",
            self.daily_sales_total.get()
        ));

        output.push_str("# HELP vaultsync_daily_transactions_count Today's transaction count\n");
        output.push_str("# TYPE vaultsync_daily_transactions_count gauge\n");
        output.push_str(&format!(
            "vaultsync_daily_transactions_count {}\n",
            self.daily_transactions_count.get() as u64
        ));

        // Inventory Metrics
        output.push_str("\n# HELP vaultsync_inventory_items_total Total inventory items\n");
        output.push_str("# TYPE vaultsync_inventory_items_total gauge\n");
        output.push_str(&format!(
            "vaultsync_inventory_items_total {}\n",
            self.inventory_items_total.get() as u64
        ));

        output.push_str("# HELP vaultsync_inventory_value_total Total inventory value\n");
        output.push_str("# TYPE vaultsync_inventory_value_total gauge\n");
        output.push_str(&format!(
            "vaultsync_inventory_value_total {:.2}\n",
            self.inventory_value_total.get()
        ));

        output.push_str("# HELP vaultsync_low_stock_items Items below stock threshold\n");
        output.push_str("# TYPE vaultsync_low_stock_items gauge\n");
        output.push_str(&format!(
            "vaultsync_low_stock_items {}\n",
            self.low_stock_items.get() as u64
        ));

        // Sync Metrics
        output.push_str("\n# HELP vaultsync_sync_pending_changes Pending sync changes\n");
        output.push_str("# TYPE vaultsync_sync_pending_changes gauge\n");
        output.push_str(&format!(
            "vaultsync_sync_pending_changes {}\n",
            self.sync_pending_changes.get() as u64
        ));

        output.push_str("# HELP vaultsync_sync_connected_peers Connected sync peers\n");
        output.push_str("# TYPE vaultsync_sync_connected_peers gauge\n");
        output.push_str(&format!(
            "vaultsync_sync_connected_peers {}\n",
            self.sync_connected_peers.get() as u64
        ));

        // System Metrics
        output.push_str("\n# HELP vaultsync_uptime_seconds Application uptime in seconds\n");
        output.push_str("# TYPE vaultsync_uptime_seconds gauge\n");
        output.push_str(&format!(
            "vaultsync_uptime_seconds {}\n",
            self.uptime_seconds.get() as u64
        ));

        output.push_str("# HELP vaultsync_disk_free_gb Free disk space in GB\n");
        output.push_str("# TYPE vaultsync_disk_free_gb gauge\n");
        output.push_str(&format!(
            "vaultsync_disk_free_gb {:.2}\n",
            self.disk_free_gb.get()
        ));

        // Per-endpoint requests
        if let Ok(map) = self.endpoint_requests.read() {
            if !map.is_empty() {
                output
                    .push_str("\n# HELP vaultsync_endpoint_requests_total Requests per endpoint\n");
                output.push_str("# TYPE vaultsync_endpoint_requests_total counter\n");
                for (endpoint, count) in map.iter() {
                    output.push_str(&format!(
                        "vaultsync_endpoint_requests_total{{endpoint=\"{}\"}} {}\n",
                        endpoint, count
                    ));
                }
            }
        }

        output
    }

    /// Render metrics as JSON (for easy consumption by frontend dashboards)
    pub fn render_json(&self) -> serde_json::Value {
        serde_json::json!({
            "http": {
                "requests_total": self.http_requests_total.get(),
                "errors_total": self.http_errors_total.get()
            },
            "business": {
                "transactions_total": self.transactions_total.get(),
                "daily_sales": self.daily_sales_total.get(),
                "daily_transactions": self.daily_transactions_count.get() as u64
            },
            "inventory": {
                "items_total": self.inventory_items_total.get() as u64,
                "value_total": self.inventory_value_total.get(),
                "low_stock_count": self.low_stock_items.get() as u64
            },
            "sync": {
                "pending_changes": self.sync_pending_changes.get() as u64,
                "connected_peers": self.sync_connected_peers.get() as u64
            },
            "system": {
                "uptime_seconds": self.uptime_seconds.get() as u64,
                "disk_free_gb": self.disk_free_gb.get()
            }
        })
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Handler for /metrics endpoint (Prometheus format)
pub async fn metrics_prometheus_handler(
    State(registry): State<std::sync::Arc<MetricsRegistry>>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        [("Content-Type", "text/plain; charset=utf-8")],
        registry.render_prometheus(),
    )
}

/// Handler for /metrics/json endpoint (JSON format)
pub async fn metrics_json_handler(
    State(registry): State<std::sync::Arc<MetricsRegistry>>,
) -> impl IntoResponse {
    (StatusCode::OK, axum::Json(registry.render_json()))
}
