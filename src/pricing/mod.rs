pub use crate::buylist::PricingServiceTrait;
use crate::core::PriceInfo;
use crate::database::Database;
use crate::errors::Result;
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use tracing;
use uuid::Uuid;

pub mod cache;
pub mod providers;
use crate::core::Category;
use std::collections::HashMap;
pub mod rules;
pub use rules::{RuleContext, RuleEngine};

use providers::{
    MockProvider, PokemonTcgProvider, PricingProvider, ScryfallProvider, SportsCardProvider,
};

#[derive(Clone)]
pub struct PricingService {
    db: Arc<Database>,
    // Registry: Primary Key = Category (e.g., SportsCard).
    // For TCG, we fallback to a specific map or logic.
    // Actually, let's keep it simple: A list or map of providers?
    // Let's use specific fields for clarity in this iteration, or a detailed map.
    // A HashMap<String, Arc...> where key is specific string like "TCG_Magic", "TCG_Pokemon", "Sports".
    providers: HashMap<String, Arc<dyn PricingProvider>>,
    last_sync_time: Arc<tokio::sync::Mutex<Option<DateTime<Utc>>>>,
    pub cache: Arc<cache::PriceCache>,
}

impl PricingService {
    pub fn new(db: Arc<Database>) -> Self {
        let mut providers: HashMap<String, Arc<dyn PricingProvider>> = HashMap::new();

        // Initialize Providers
        // 1. Magic (Default TCG)
        providers.insert("TCG_Magic".to_string(), Arc::new(ScryfallProvider::new()));

        // 2. Pokemon
        providers.insert(
            "TCG_Pokemon".to_string(),
            Arc::new(PokemonTcgProvider::new()),
        );

        // 3. Sports
        providers.insert(
            "SportsCard".to_string(),
            Arc::new(SportsCardProvider::new()),
        );

        // 4. Fallback / Mock
        providers.insert("default".to_string(), Arc::new(MockProvider));

        tracing::info!(
            "Initializing PricingService with {} providers",
            providers.len()
        );

        let ttl = std::env::var("PRICE_CACHE_TTL_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3600);

        let max_entries = std::env::var("PRICE_CACHE_MAX_ENTRIES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10000);

        Self {
            db,
            providers,
            last_sync_time: Arc::new(tokio::sync::Mutex::new(None)),
            cache: Arc::new(cache::PriceCache::new(ttl, max_entries)),
        }
    }

    pub fn with_scryfall(db: Arc<Database>) -> Self {
        // Legacy constructor for tests, defaulting to standard setup which includes Scryfall
        Self::new(db)
    }

    pub async fn warm_cache(&self) -> Result<()> {
        tracing::info!("Warming price cache from database...");
        // Load up to 1000 most recent prices
        match self.db.pricing.get_recent(1000).await {
            Ok(prices) => {
                let count = prices.len();
                for price in prices {
                    // Only cache if still fresh (< 24h)
                    let age = Utc::now() - price.last_sync_timestamp;
                    if age < Duration::hours(24) {
                        self.cache.set(price).await;
                    }
                }
                tracing::info!("Price cache warmed with {} items", count);
            }
            Err(e) => {
                tracing::warn!("Failed to warm price cache: {}", e);
            }
        }
        Ok(())
    }

    fn get_provider_key(&self, product: &crate::core::Product) -> String {
        match product.category {
            Category::SportsCard => "SportsCard".to_string(),
            Category::TCG => {
                // Check metadata for game
                if let Some(game) = product.metadata.get("game").and_then(|v| v.as_str()) {
                    match game.to_lowercase().as_str() {
                        "pokemon" => "TCG_Pokemon".to_string(),
                        "magic" | "mtg" => "TCG_Magic".to_string(),
                        _ => "default".to_string(),
                    }
                } else {
                    // Default to Magic if no game specified for now (History behavior)
                    "TCG_Magic".to_string()
                }
            }
            _ => "default".to_string(),
        }
    }

    /// MED-003 FIX: Price sync now uses concurrent batch processing
    /// Processes multiple products concurrently with a semaphore limit
    pub async fn sync_prices(&self) -> Result<()> {
        let products = self.db.products.get_all().await?;
        let total = products.len();
        tracing::info!("Syncing prices for {} products...", total);

        // Use a semaphore to limit concurrent requests (prevent rate limiting)
        const MAX_CONCURRENT: usize = 10;
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT));

        // Split products into batches for progress logging
        let batch_size = 50;
        let batches: Vec<_> = products.chunks(batch_size).collect();

        for (batch_idx, batch) in batches.iter().enumerate() {
            let mut handles = Vec::new();

            for product in batch.iter() {
                let product = product.clone();
                let key = self.get_provider_key(&product);
                let provider = self
                    .providers
                    .get(&key)
                    .or_else(|| self.providers.get("default"))
                    .cloned();
                let db = self.db.clone();
                let cache = self.cache.clone();
                let sem = semaphore.clone();

                let handle = tokio::spawn(async move {
                    // Acquire semaphore permit (limits concurrency)
                    let _permit = sem.acquire().await;

                    if let Some(provider) = provider {
                        match provider.get_price(&product).await {
                            Ok(price_info) => {
                                cache.set(price_info.clone()).await;
                                if let Err(e) = db.pricing.insert_matrix(&price_info).await {
                                    tracing::debug!(
                                        "Failed to store price for {}: {}",
                                        product.name,
                                        e
                                    );
                                }
                                // Record History (Task 086)
                                let source = provider.name();
                                if let Err(e) =
                                    db.pricing.record_price_history(&price_info, source).await
                                {
                                    tracing::warn!(
                                        "Failed to record price history for {}: {}",
                                        product.name,
                                        e
                                    );
                                }
                            }
                            Err(e) => {
                                tracing::debug!(
                                    "Failed to fetch price for {}: {}",
                                    product.name,
                                    e
                                );
                            }
                        }
                    }

                    // Small delay to avoid overwhelming external APIs
                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                });

                handles.push(handle);
            }

            // Wait for batch to complete
            for handle in handles {
                let _ = handle.await;
            }

            // Progress logging every batch
            let processed = ((batch_idx + 1) * batch_size).min(total);
            tracing::info!("Price sync progress: {}/{} products", processed, total);
        }

        let mut last_sync = self.last_sync_time.lock().await;
        *last_sync = Some(Utc::now());

        tracing::info!("Completed price sync for {} products", total);
        Ok(())
    }

    pub async fn get_last_sync_time(&self) -> Option<DateTime<Utc>> {
        *self.last_sync_time.lock().await
    }

    pub async fn is_price_cache_fresh(&self) -> bool {
        if let Some(last_sync) = *self.last_sync_time.lock().await {
            let age = Utc::now() - last_sync;
            age < Duration::hours(24)
        } else {
            false
        }
    }

    pub async fn get_cached_price(&self, product_uuid: Uuid) -> Option<PriceInfo> {
        match self.db.pricing.get_for_product(product_uuid).await {
            Ok(price_info) => price_info,
            Err(e) => {
                tracing::error!(
                    "Database error checking cached price for {}: {}",
                    product_uuid,
                    e
                );
                None
            }
        }
    }

    pub async fn get_price_for_product(&self, product_uuid: Uuid) -> Option<PriceInfo> {
        // 0. Check In-Memory Cache first
        if let Some(price) = self.cache.get(product_uuid).await {
            return Some(price);
        }

        // 1. Try to get from DB
        match self.db.pricing.get_for_product(product_uuid).await {
            Ok(Some(price_info)) => {
                // Check if price is fresh (e.g., < 24 hours old)
                let age = Utc::now() - price_info.last_sync_timestamp;
                if age < Duration::hours(24) {
                    // Populate in-memory cache for next time
                    self.cache.set(price_info.clone()).await;
                    return Some(price_info);
                }

                // If stale, we'll try to fetch fresh, but keep this as fallback
                tracing::info!(
                    "Price for {} is stale ({} hours old), fetching fresh...",
                    product_uuid,
                    age.num_hours()
                );
            }
            Ok(None) => {
                tracing::info!("No price found for {}, fetching...", product_uuid);
            }
            Err(e) => {
                tracing::error!("Database error checking price for {}: {}", product_uuid, e);
            }
        }

        // 2. Fetch from Provider
        match self.db.products.get_by_id(product_uuid).await {
            Ok(Some(product)) => {
                let key = self.get_provider_key(&product);
                // SECURITY FIX: Safe provider lookup with fallback
                let provider = match self
                    .providers
                    .get(&key)
                    .or_else(|| self.providers.get("default"))
                {
                    Some(p) => p,
                    None => {
                        tracing::error!(
                            "No pricing provider available for key '{}' or 'default'",
                            key
                        );
                        return None;
                    }
                };

                match provider.get_price(&product).await {
                    Ok(new_price) => {
                        // Update caches
                        self.cache.set(new_price.clone()).await;

                        if let Err(e) = self.db.pricing.insert_matrix(&new_price).await {
                            tracing::error!("Failed to cache price for {}: {}", product.name, e);
                        }

                        // Record History (Task 086)
                        if let Err(e) = self
                            .db
                            .pricing
                            .record_price_history(&new_price, provider.name())
                            .await
                        {
                            tracing::warn!(
                                "Failed to record price history for {}: {}",
                                product.name,
                                e
                            );
                        }
                        return Some(new_price);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to fetch price for {}: {}", product.name, e);
                    }
                }
            }
            Ok(None) => {
                tracing::warn!(
                    "Product {} not found in DB, cannot fetch price",
                    product_uuid
                );
            }
            Err(e) => {
                tracing::error!("Database error fetching product {}: {}", product_uuid, e);
            }
        }

        // 3. Fallback: Return stale price if we have it
        if let Ok(Some(price_info)) = self.db.pricing.get_for_product(product_uuid).await {
            return Some(price_info);
        }

        None
    }

    pub fn calculate_safety_status(
        &self,
        cached_price: f64,
        market_update: f64,
    ) -> crate::core::PriceStatus {
        let variance = (market_update - cached_price).abs() / cached_price;

        if variance > 0.15 {
            // If price moved > 15%, flag for manager review
            crate::core::PriceStatus::Flagged
        } else {
            crate::core::PriceStatus::Safe
        }
    }

    pub async fn clear_cache(&self) {
        self.cache.clear().await;
        tracing::info!("Price cache cleared manually");
    }

    pub async fn invalidate_product(&self, product_uuid: Uuid) {
        self.cache.invalidate(product_uuid).await;
        tracing::info!("Invalidated price cache for product {}", product_uuid);
    }

    pub async fn invalidate_category(&self, category: Category) -> Result<()> {
        let products = self.db.products.get_by_category(category.clone()).await?;
        let count = products.len();
        for product in products {
            self.cache.invalidate(product.product_uuid).await;
        }
        tracing::info!(
            "Invalidated price cache for category {:?} ({} products)",
            category,
            count
        );
        Ok(())
    }
}

#[async_trait::async_trait]
impl PricingServiceTrait for PricingService {
    async fn get_price_for_card(&self, product_uuid: Uuid) -> Option<PriceInfo> {
        self.get_price_for_product(product_uuid).await
    }

    async fn get_cached_price(&self, product_uuid: Uuid) -> Option<PriceInfo> {
        self.get_cached_price(product_uuid).await
    }

    fn calculate_safety_status(
        &self,
        cached_price: f64,
        market_update: f64,
    ) -> crate::core::PriceStatus {
        let variance = (market_update - cached_price).abs() / cached_price;

        if variance > 0.15 {
            crate::core::PriceStatus::Flagged
        } else {
            crate::core::PriceStatus::Safe
        }
    }
}
