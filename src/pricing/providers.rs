use crate::core::{PriceInfo, Product};
use crate::errors::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

#[async_trait]
pub trait PricingProvider: Send + Sync {
    async fn get_price(&self, product: &Product) -> Result<PriceInfo>;
    fn name(&self) -> &str;
}

pub struct ScryfallProvider {
    client: reqwest::Client,
}

impl ScryfallProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl PricingProvider for ScryfallProvider {
    async fn get_price(&self, product: &Product) -> Result<PriceInfo> {
        // Only works for TCG category
        if product.category != crate::core::Category::TCG {
            return Err(crate::errors::VaultSyncError::PricingError(
                "Scryfall only supports TCG cards".to_string(),
            )
            .into());
        }

        // Try to find by set code and collector number first (most accurate)
        if let (Some(set), Some(cn)) = (&product.set_code, &product.collector_number) {
            let url = format!(
                "https://api.scryfall.com/cards/{}/{}",
                set.to_lowercase(),
                cn
            );

            // Note: In a real app, we should respect Scryfall's rate limits (100ms delay between requests)
            // and headers requirements.
            let resp = self
                .client
                .get(&url)
                .header("User-Agent", "VaultSync/0.1.0")
                .header("Accept", "application/json")
                .send()
                .await?;

            if resp.status().is_success() {
                let body: Value = resp.json().await?;

                // Parse price from prices.usd or prices.eur
                let usd_price = body["prices"]["usd"]
                    .as_str()
                    .or_else(|| body["prices"]["usd_foil"].as_str())
                    .unwrap_or("0.0");

                let market_mid = usd_price.parse::<f64>().unwrap_or(0.0);

                return Ok(PriceInfo {
                    price_uuid: Uuid::new_v4(),
                    product_uuid: product.product_uuid,
                    market_mid,
                    market_low: market_mid * 0.85, // Rough estimate
                    last_sync_timestamp: Utc::now(),
                });
            }
        } else if !product.name.is_empty() {
            // Fallback to fuzzy search by name
            let url = format!(
                "https://api.scryfall.com/cards/named?fuzzy={}",
                product.name
            );
            let resp = self
                .client
                .get(&url)
                .header("User-Agent", "VaultSync/0.1.0")
                .header("Accept", "application/json")
                .send()
                .await?;

            if resp.status().is_success() {
                let body: Value = resp.json().await?;
                let usd_price = body["prices"]["usd"]
                    .as_str()
                    .or_else(|| body["prices"]["usd_foil"].as_str())
                    .unwrap_or("0.0");
                let market_mid = usd_price.parse::<f64>().unwrap_or(0.0);

                return Ok(PriceInfo {
                    price_uuid: Uuid::new_v4(),
                    product_uuid: product.product_uuid,
                    market_mid,
                    market_low: market_mid * 0.85,
                    last_sync_timestamp: Utc::now(),
                });
            }
        }

        Err(crate::errors::VaultSyncError::PricingError(format!(
            "Could not find price for {}",
            product.name
        ))
        .into())
    }

    fn name(&self) -> &str {
        "Scryfall"
    }
}

pub struct MockProvider;

#[async_trait]
impl PricingProvider for MockProvider {
    async fn get_price(&self, product: &Product) -> Result<PriceInfo> {
        // Deterministic price based on UUID
        let bytes = product.product_uuid.as_bytes();
        let val = ((bytes[0] as u32) << 8) | (bytes[1] as u32);
        let price = (val % 10000) as f64 / 100.0 + 1.0;

        Ok(PriceInfo {
            price_uuid: Uuid::new_v4(),
            product_uuid: product.product_uuid,
            market_mid: price,
            market_low: price * 0.8,
            last_sync_timestamp: Utc::now(),
        })
    }

    fn name(&self) -> &str {
        "Mock"
    }
}

pub struct PokemonTcgProvider {
    client: reqwest::Client,
    api_key: Option<String>,
}

impl PokemonTcgProvider {
    pub fn new() -> Self {
        let api_key = std::env::var("POKEMON_TCG_API_KEY").ok();
        if api_key.is_none() {
            tracing::warn!("POKEMON_TCG_API_KEY not set - Pokemon pricing will use fallback");
        }
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }
}

#[async_trait]
impl PricingProvider for PokemonTcgProvider {
    async fn get_price(&self, product: &Product) -> Result<PriceInfo> {
        // Only works for Pokemon category (which is also TCG but Pokemon specific)
        // Check metadata for pokemon indicator or use TCG category

        // Try the Pokemon TCG API (pokemontcg.io)
        // API docs: https://docs.pokemontcg.io/
        if let (Some(set), Some(cn)) = (&product.set_code, &product.collector_number) {
            // Format: set_id-collector_number
            let query = format!("set.id:{} number:{}", set.to_lowercase(), cn);

            let mut request = self
                .client
                .get("https://api.pokemontcg.io/v2/cards")
                .query(&[("q", &query)])
                .header("User-Agent", "VaultSync/0.1.0");

            if let Some(ref key) = self.api_key {
                request = request.header("X-Api-Key", key);
            }

            if let Ok(resp) = request.send().await {
                if resp.status().is_success() {
                    if let Ok(body) = resp.json::<Value>().await {
                        if let Some(cards) = body["data"].as_array() {
                            if let Some(card) = cards.first() {
                                // TCGPlayer prices are in cardmarket and tcgplayer objects
                                let tcgplayer = &card["tcgplayer"]["prices"];

                                // Try normal, then holofoil, then reverseHolofoil
                                let market_price = tcgplayer["normal"]["market"]
                                    .as_f64()
                                    .or_else(|| tcgplayer["holofoil"]["market"].as_f64())
                                    .or_else(|| tcgplayer["reverseHolofoil"]["market"].as_f64())
                                    .unwrap_or(0.0);

                                let low_price = tcgplayer["normal"]["low"]
                                    .as_f64()
                                    .or_else(|| tcgplayer["holofoil"]["low"].as_f64())
                                    .or_else(|| tcgplayer["reverseHolofoil"]["low"].as_f64())
                                    .unwrap_or(market_price * 0.7);

                                if market_price > 0.0 {
                                    return Ok(PriceInfo {
                                        price_uuid: Uuid::new_v4(),
                                        product_uuid: product.product_uuid,
                                        market_mid: market_price,
                                        market_low: low_price,
                                        last_sync_timestamp: Utc::now(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback to name search
        if !product.name.is_empty() {
            let query = format!("name:\"{}\"", product.name);

            let mut request = self
                .client
                .get("https://api.pokemontcg.io/v2/cards")
                .query(&[("q", &query), ("pageSize", &"1".to_string())])
                .header("User-Agent", "VaultSync/0.1.0");

            if let Some(ref key) = self.api_key {
                request = request.header("X-Api-Key", key);
            }

            if let Ok(resp) = request.send().await {
                if resp.status().is_success() {
                    if let Ok(body) = resp.json::<Value>().await {
                        if let Some(cards) = body["data"].as_array() {
                            if let Some(card) = cards.first() {
                                let tcgplayer = &card["tcgplayer"]["prices"];
                                let market_price = tcgplayer["normal"]["market"]
                                    .as_f64()
                                    .or_else(|| tcgplayer["holofoil"]["market"].as_f64())
                                    .unwrap_or(0.0);

                                if market_price > 0.0 {
                                    let low_price = tcgplayer["normal"]["low"]
                                        .as_f64()
                                        .or_else(|| tcgplayer["holofoil"]["low"].as_f64())
                                        .unwrap_or(market_price * 0.7);

                                    return Ok(PriceInfo {
                                        price_uuid: Uuid::new_v4(),
                                        product_uuid: product.product_uuid,
                                        market_mid: market_price,
                                        market_low: low_price,
                                        last_sync_timestamp: Utc::now(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Final fallback: deterministic mock based on UUID for consistency
        tracing::debug!(
            "Pokemon API lookup failed for {}, using fallback",
            product.name
        );
        let bytes = product.product_uuid.as_bytes();
        let val = ((bytes[0] as u32) << 8) | (bytes[1] as u32);
        let price = (val % 500) as f64 / 10.0 + 1.0; // $1-$51 range

        Ok(PriceInfo {
            price_uuid: Uuid::new_v4(),
            product_uuid: product.product_uuid,
            market_mid: price,
            market_low: price * 0.7,
            last_sync_timestamp: Utc::now(),
        })
    }

    fn name(&self) -> &str {
        "PokemonTCG"
    }
}

#[derive(Deserialize)]
struct PriceChartingResponse {
    #[serde(rename = "loose-price")]
    loose_price: Option<f64>,
}

pub struct PriceChartingProvider {
    client: reqwest::Client,
    api_key: String,
}

impl PriceChartingProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }

    pub async fn get_price(&self, product: &Product) -> Result<PriceInfo> {
        let resp = self
            .client
            .get("https://www.pricecharting.com/api/product")
            .query(&[("t", &self.api_key), ("q", &product.name)])
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(crate::errors::VaultSyncError::PricingError(format!(
                "PriceCharting API Error: {}",
                resp.status()
            ))
            .into());
        }

        let body: PriceChartingResponse = resp.json().await?;
        let market_mid = body.loose_price.unwrap_or(0.0) / 100.0;

        Ok(PriceInfo {
            price_uuid: Uuid::new_v4(),
            product_uuid: product.product_uuid,
            market_mid,
            market_low: market_mid * 0.9,
            last_sync_timestamp: Utc::now(),
        })
    }
}

pub struct SportsCardProvider {
    price_charting: Option<PriceChartingProvider>,
}

impl SportsCardProvider {
    pub fn new() -> Self {
        let pc_key = std::env::var("PRICECHARTING_API_KEY").ok();
        let price_charting = pc_key.map(PriceChartingProvider::new);

        if price_charting.is_none() {
            tracing::warn!("PRICECHARTING_API_KEY not set - SportsCard pricing will use fallback");
        }

        Self { price_charting }
    }
}

#[async_trait]
impl PricingProvider for SportsCardProvider {
    async fn get_price(&self, product: &Product) -> Result<PriceInfo> {
        if let Some(pc) = &self.price_charting {
            return pc.get_price(product).await;
        }

        Err(crate::errors::VaultSyncError::PricingError(
            "Sports Card pricing API not configured".to_string(),
        )
        .into())
    }

    fn name(&self) -> &str {
        if self.price_charting.is_some() {
            "SportsCard (PriceCharting)"
        } else {
            "SportsCard (Mock)"
        }
    }
}
