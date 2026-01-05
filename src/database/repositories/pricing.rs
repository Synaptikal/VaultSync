use crate::core::PriceInfo;
use crate::errors::Result;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

#[derive(Clone)]
pub struct PricingRepository {
    pool: SqlitePool,
}

impl PricingRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert_matrix(&self, price: &PriceInfo) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO Pricing_Matrix 
            (price_uuid, product_uuid, market_mid, market_low, last_sync_timestamp) 
            VALUES (?, ?, ?, ?, ?)",
        )
        .bind(price.price_uuid.to_string())
        .bind(price.product_uuid.to_string())
        .bind(price.market_mid)
        .bind(price.market_low)
        .bind(price.last_sync_timestamp.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_for_product(&self, product_uuid: Uuid) -> Result<Option<PriceInfo>> {
        let row = sqlx::query("SELECT price_uuid, product_uuid, market_mid, market_low, last_sync_timestamp FROM Pricing_Matrix WHERE product_uuid = ?")
            .bind(product_uuid.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let price_uuid_str: String = row.try_get("price_uuid").unwrap_or_default();
            let price_uuid = Uuid::parse_str(&price_uuid_str).unwrap_or_default();
            // product_uuid is input
            let market_mid: f64 = row.try_get("market_mid").unwrap_or_default();
            let market_low: f64 = row.try_get("market_low").unwrap_or_default();
            let last_sync_timestamp_str: String =
                row.try_get("last_sync_timestamp").unwrap_or_default();
            let last_sync_timestamp =
                chrono::DateTime::parse_from_rfc3339(&last_sync_timestamp_str)?
                    .with_timezone(&chrono::Utc);

            Ok(Some(PriceInfo {
                price_uuid,
                product_uuid,
                market_mid,
                market_low,
                last_sync_timestamp,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_recent(&self, limit: i64) -> Result<Vec<PriceInfo>> {
        let rows = sqlx::query(
            "SELECT price_uuid, product_uuid, market_mid, market_low, last_sync_timestamp 
             FROM Pricing_Matrix 
             ORDER BY last_sync_timestamp DESC 
             LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let price_uuid_str: String = row.try_get("price_uuid").unwrap_or_default();
            let price_uuid = Uuid::parse_str(&price_uuid_str).unwrap_or_default();
            let product_uuid_str: String = row.try_get("product_uuid").unwrap_or_default();
            let product_uuid = Uuid::parse_str(&product_uuid_str).unwrap_or_default();
            let market_mid: f64 = row.try_get("market_mid").unwrap_or_default();
            let market_low: f64 = row.try_get("market_low").unwrap_or_default();
            let last_sync_timestamp_str: String =
                row.try_get("last_sync_timestamp").unwrap_or_default();
            let last_sync_timestamp =
                chrono::DateTime::parse_from_rfc3339(&last_sync_timestamp_str)?
                    .with_timezone(&chrono::Utc);

            results.push(PriceInfo {
                price_uuid,
                product_uuid,
                market_mid,
                market_low,
                last_sync_timestamp,
            });
        }
        Ok(results)
    }

    // --- Audited Price Override ---
    pub async fn log_price_override(
        &self,
        product_uuid: Uuid,
        new_price: f64,
        reason: &str,
        user_uuid: Option<Uuid>,
    ) -> Result<()> {
        // First ensure table exists (simple migration for now, usually done elsewhere)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS Price_Overrides (
                override_uuid TEXT PRIMARY KEY,
                product_uuid TEXT NOT NULL,
                new_price REAL NOT NULL,
                reason TEXT NOT NULL,
                user_uuid TEXT,
                timestamp TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        sqlx::query(
            "INSERT INTO Price_Overrides (override_uuid, product_uuid, new_price, reason, user_uuid, timestamp)
            VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(Uuid::new_v4().to_string())
        .bind(product_uuid.to_string())
        .bind(new_price)
        .bind(reason)
        .bind(user_uuid.map(|u| u.to_string()))
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        tracing::info!(
            "AUDIT: Price Override logged for product {}. New Price: {}",
            product_uuid,
            new_price
        );
        Ok(())
    }

    // --- Price History (Task 086) ---
    pub async fn record_price_history(&self, price: &PriceInfo, source: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO Price_History (history_uuid, product_uuid, market_mid, market_low, source, recorded_at) 
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(Uuid::new_v4().to_string())
        .bind(price.product_uuid.to_string())
        .bind(price.market_mid)
        .bind(price.market_low)
        .bind(source)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_price_history(
        &self,
        product_uuid: Uuid,
        days: i32,
    ) -> Result<Vec<crate::core::PriceHistoryEntry>> {
        let since = chrono::Utc::now() - chrono::Duration::days(days as i64);

        let rows = sqlx::query(
            "SELECT history_uuid, product_uuid, market_mid, market_low, source, recorded_at 
             FROM Price_History 
             WHERE product_uuid = ? AND recorded_at >= ?
             ORDER BY recorded_at ASC",
        )
        .bind(product_uuid.to_string())
        .bind(since.to_rfc3339())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        let mut history = Vec::new();
        for row in rows {
            let history_uuid_str: String = row.try_get("history_uuid").unwrap_or_default();
            let history_uuid = Uuid::parse_str(&history_uuid_str).unwrap_or_default();
            let market_mid: f64 = row.try_get("market_mid").unwrap_or_default();
            let market_low: f64 = row.try_get("market_low").unwrap_or_default();
            let source: String = row.try_get("source").unwrap_or_default();
            let recorded_at_str: String = row.try_get("recorded_at").unwrap_or_default();
            let recorded_at =
                chrono::DateTime::parse_from_rfc3339(&recorded_at_str)?.with_timezone(&chrono::Utc);

            history.push(crate::core::PriceHistoryEntry {
                history_uuid,
                product_uuid,
                market_mid,
                market_low,
                source,
                recorded_at,
            });
        }

        Ok(history)
    }
}
