use crate::errors::Result;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    /// JWT secret for token signing (minimum 32 characters)
    pub jwt_secret: String,

    /// Database connection URL
    pub database_url: String,

    /// Unique node identifier for multi-terminal sync
    pub node_id: String,

    /// API server port
    pub api_port: u16,

    /// CORS allowed origins (None = permissive mode for development)
    pub cors_origins: Option<Vec<String>>,

    /// JWT token expiration in hours
    pub jwt_expiration_hours: u64,

    /// Maximum database connections
    pub db_max_connections: u32,

    /// API rate limit per second
    pub rate_limit_per_second: u64,

    /// API rate limit burst size
    pub rate_limit_burst: u32,

    /// Auth rate limit per second (stricter)
    pub auth_rate_limit_per_second: u64,

    /// Auth rate limit burst size
    pub auth_rate_limit_burst: u32,

    /// Price cache TTL in seconds
    pub price_cache_ttl_seconds: i64,

    /// Maximum price cache entries
    pub price_cache_max_entries: usize,

    /// Store Name for Receipts
    pub store_name: String,

    /// Store Address for Receipts
    pub store_address: String,

    /// Store Phone (Optional)
    pub store_phone: Option<String>,

    /// Store Website (Optional)
    pub store_website: Option<String>,

    /// Pricing volatility threshold for flagging (default: 0.15 = 15%)
    pub pricing_volatility_threshold: f64,

    /// Sync batch size for fetching changes (default: 100)
    pub sync_batch_size: i64,

    /// Thermal printer line width in characters (default: 42)
    pub thermal_line_width: usize,

    /// Price cache freshness in hours (default: 24)
    pub price_freshness_hours: i64,

    /// Maximum concurrent pricing API requests (default: 10)
    pub pricing_max_concurrent: usize,

    /// API request delay in milliseconds (default: 20)
    pub pricing_api_delay_ms: u64,

    /// Pricing sync batch size (default: 50)
    pub pricing_sync_batch_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            jwt_secret: "test_secret_key_must_be_at_least_32_chars_long_for_security".to_string(),
            database_url: "sqlite::memory:".to_string(),
            node_id: "test_node_default".to_string(),
            api_port: 0,
            cors_origins: None,
            jwt_expiration_hours: 24,
            db_max_connections: 5,
            rate_limit_per_second: 1000,
            rate_limit_burst: 2000,
            auth_rate_limit_per_second: 1000,
            auth_rate_limit_burst: 2000,
            price_cache_ttl_seconds: 3600,
            price_cache_max_entries: 1000,
            store_name: "Test Store".to_string(),
            store_address: "123 Test St".to_string(),
            store_phone: None,
            store_website: None,
            pricing_volatility_threshold: 0.15,
            sync_batch_size: 100,
            thermal_line_width: 42,
            price_freshness_hours: 24,
            pricing_max_concurrent: 10,
            pricing_api_delay_ms: 0,
            pricing_sync_batch_size: 50,
        }
    }
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// # Errors
    /// Returns an error if required variables are missing or invalid.
    /// This enables fail-fast behavior at application startup.
    pub fn from_env() -> Result<Self> {
        // JWT Secret - REQUIRED and must be secure
        let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| {
            anyhow::anyhow!(
                "JWT_SECRET environment variable is required. \
                 Generate one with: openssl rand -base64 32"
            )
        })?;

        if jwt_secret.len() < 32 {
            return Err(anyhow::anyhow!(
                "JWT_SECRET must be at least 32 characters (got {}). \
                 Generate a secure secret with: openssl rand -base64 32",
                jwt_secret.len()
            ));
        }

        // Warn if using obvious development secret
        if jwt_secret.contains("dev")
            || jwt_secret.contains("test")
            || jwt_secret.contains("CHANGE")
        {
            tracing::warn!(
                "JWT_SECRET appears to be a development secret. \
                 Please use a secure random secret in production!"
            );
        }

        // Database URL
        let database_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:vaultsync.db".to_string());

        // Node ID - generate unique ID if not provided
        let node_id = std::env::var("NODE_ID").unwrap_or_else(|_| generate_node_id());

        // API Port
        let api_port = std::env::var("API_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .map_err(|_| anyhow::anyhow!("API_PORT must be a valid port number"))?;

        // CORS origins - parse comma-separated list
        let cors_origins = std::env::var("CORS_ALLOWED_ORIGINS")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(|s| {
                s.split(',')
                    .map(|origin| origin.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            });

        // JWT expiration
        let jwt_expiration_hours = std::env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse()
            .unwrap_or(24);

        // Database pool size
        let db_max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5);

        // Rate limiting
        let rate_limit_per_second = std::env::var("RATE_LIMIT_PER_SECOND")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .unwrap_or(100);

        let rate_limit_burst = std::env::var("RATE_LIMIT_BURST")
            .unwrap_or_else(|_| "200".to_string())
            .parse()
            .unwrap_or(200);

        let auth_rate_limit_per_second = std::env::var("AUTH_RATE_LIMIT_PER_SECOND")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5);

        let auth_rate_limit_burst = std::env::var("AUTH_RATE_LIMIT_BURST")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);

        // Cache settings
        let price_cache_ttl_seconds = std::env::var("PRICE_CACHE_TTL_SECONDS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .unwrap_or(3600);

        let price_cache_max_entries = std::env::var("PRICE_CACHE_MAX_ENTRIES")
            .unwrap_or_else(|_| "10000".to_string())
            .parse()
            .unwrap_or(10000);

        // Store Details
        let store_name =
            std::env::var("STORE_NAME").unwrap_or_else(|_| "VaultSync Store".to_string());
        let store_address =
            std::env::var("STORE_ADDRESS").unwrap_or_else(|_| "123 Local St".to_string());
        let store_phone = std::env::var("STORE_PHONE").ok();
        let store_website = std::env::var("STORE_WEBSITE").ok();

        // Pricing volatility threshold
        let pricing_volatility_threshold = std::env::var("PRICING_VOLATILITY_THRESHOLD")
            .unwrap_or_else(|_| "0.15".to_string())
            .parse()
            .unwrap_or(0.15);

        // Sync batch size
        let sync_batch_size = std::env::var("SYNC_BATCH_SIZE")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .unwrap_or(100);

        // Thermal printer line width
        let thermal_line_width = std::env::var("THERMAL_LINE_WIDTH")
            .unwrap_or_else(|_| "42".to_string())
            .parse()
            .unwrap_or(42);

        // Price freshness hours
        let price_freshness_hours = std::env::var("PRICE_FRESHNESS_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse()
            .unwrap_or(24);

        // Pricing max concurrent requests
        let pricing_max_concurrent = std::env::var("PRICING_MAX_CONCURRENT")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);

        // Pricing API delay
        let pricing_api_delay_ms = std::env::var("PRICING_API_DELAY_MS")
            .unwrap_or_else(|_| "20".to_string())
            .parse()
            .unwrap_or(20);

        // Pricing sync batch size
        let pricing_sync_batch_size = std::env::var("PRICING_SYNC_BATCH_SIZE")
            .unwrap_or_else(|_| "50".to_string())
            .parse()
            .unwrap_or(50);

        Ok(Self {
            jwt_secret,
            database_url,
            node_id,
            api_port,
            cors_origins,
            jwt_expiration_hours,
            db_max_connections,
            rate_limit_per_second,
            rate_limit_burst,
            auth_rate_limit_per_second,
            auth_rate_limit_burst,
            price_cache_ttl_seconds,
            price_cache_max_entries,
            store_name,
            store_address,
            store_phone,
            store_website,
            pricing_volatility_threshold,
            sync_batch_size,
            thermal_line_width,
            price_freshness_hours,
            pricing_max_concurrent,
            pricing_api_delay_ms,
            pricing_sync_batch_size,
        })
    }

    /// Check if running in production mode (CORS configured)
    pub fn is_production(&self) -> bool {
        self.cors_origins.is_some()
    }
}

/// Generate a unique node ID based on random data
fn generate_node_id() -> String {
    use rand::Rng;
    let random: u64 = rand::thread_rng().gen();
    let id = format!("node_{:016x}", random);
    tracing::info!("Generated node ID: {}", id);
    id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_node_id_is_unique() {
        let id1 = generate_node_id();
        let id2 = generate_node_id();
        assert_ne!(id1, id2);
        assert!(id1.starts_with("node_"));
    }
}
