//! Tax calculation service
//!
//! Handles tax rate lookup and calculation for transactions,
//! including customer tax-exempt status and category-based rates.

use crate::database::Database;
use crate::errors::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Tax rate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxRate {
    pub rate_id: String,
    pub name: String,
    pub rate: f64, // 0.0 to 1.0 (e.g., 0.08 for 8%)
    pub applies_to_category: Option<String>,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

impl TaxRate {
    pub fn new(name: String, rate: f64) -> Self {
        let now = Utc::now();
        Self {
            rate_id: Uuid::new_v4().to_string(),
            name,
            rate,
            applies_to_category: None,
            is_default: false,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn default_rate(rate: f64) -> Self {
        let mut tax_rate = Self::new("Default".to_string(), rate);
        tax_rate.is_default = true;
        tax_rate
    }
}

/// Tax calculation result for a single item
#[derive(Debug, Clone, Serialize)]
pub struct ItemTax {
    pub item_uuid: Uuid,
    pub taxable_amount: f64,
    pub tax_rate: f64,
    pub tax_amount: f64,
}

/// Tax breakdown for an entire transaction
#[derive(Debug, Clone, Serialize)]
pub struct TaxBreakdown {
    pub items: Vec<ItemTax>,
    pub total_taxable: f64,
    pub total_tax: f64,
    pub effective_rate: f64,
}

impl TaxBreakdown {
    pub fn zero() -> Self {
        Self {
            items: Vec::new(),
            total_taxable: 0.0,
            total_tax: 0.0,
            effective_rate: 0.0,
        }
    }
}

/// Service for calculating taxes on transactions
pub struct TaxService {
    db: Arc<Database>,
    default_rate: f64,
}

impl TaxService {
    pub fn new(db: Arc<Database>) -> Self {
        // Default tax rate - should be loaded from config/database
        let default_rate = std::env::var("DEFAULT_TAX_RATE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0); // 0% if not configured

        Self { db, default_rate }
    }

    /// Get the applicable tax rate for a product category
    pub async fn get_rate_for_category(&self, category: &str) -> Result<f64> {
        // Try to find category-specific rate from database
        let result = sqlx::query(
            "SELECT rate FROM Tax_Rates 
             WHERE applies_to_category = ? AND is_active = 1
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(category)
        .fetch_optional(&self.db.pool)
        .await;

        if let Ok(Some(row)) = result {
            if let Ok(rate) = sqlx::Row::try_get::<f64, _>(&row, "rate") {
                return Ok(rate);
            }
        }

        // Fall back to default rate
        self.get_default_rate().await
    }

    /// Get the default tax rate
    pub async fn get_default_rate(&self) -> Result<f64> {
        // Try to get from database first
        let result = sqlx::query(
            "SELECT rate FROM Tax_Rates 
             WHERE is_default = 1 AND is_active = 1
             LIMIT 1",
        )
        .fetch_optional(&self.db.pool)
        .await;

        if let Ok(Some(row)) = result {
            if let Ok(rate) = sqlx::Row::try_get::<f64, _>(&row, "rate") {
                return Ok(rate);
            }
        }

        // Fall back to environment/hardcoded default
        Ok(self.default_rate)
    }

    /// Calculate tax for a single item
    pub async fn calculate_item_tax(
        &self,
        item_uuid: Uuid,
        amount: f64,
        category: Option<&str>,
        customer_tax_exempt: bool,
    ) -> Result<ItemTax> {
        // Tax-exempt customers pay no tax
        if customer_tax_exempt {
            return Ok(ItemTax {
                item_uuid,
                taxable_amount: amount,
                tax_rate: 0.0,
                tax_amount: 0.0,
            });
        }

        // Get applicable rate
        let rate = match category {
            Some(cat) => self.get_rate_for_category(cat).await?,
            None => self.get_default_rate().await?,
        };

        // Calculate tax (round to 2 decimal places)
        let tax_amount = (amount * rate * 100.0).round() / 100.0;

        Ok(ItemTax {
            item_uuid,
            taxable_amount: amount,
            tax_rate: rate,
            tax_amount,
        })
    }

    /// Calculate tax for multiple items in a transaction
    pub async fn calculate_transaction_tax(
        &self,
        items: &[(Uuid, f64, Option<String>)], // (item_uuid, amount, category)
        customer_tax_exempt: bool,
    ) -> Result<TaxBreakdown> {
        if customer_tax_exempt {
            let total_taxable: f64 = items.iter().map(|(_, amount, _)| amount).sum();
            return Ok(TaxBreakdown {
                items: items
                    .iter()
                    .map(|(uuid, amount, _)| ItemTax {
                        item_uuid: *uuid,
                        taxable_amount: *amount,
                        tax_rate: 0.0,
                        tax_amount: 0.0,
                    })
                    .collect(),
                total_taxable,
                total_tax: 0.0,
                effective_rate: 0.0,
            });
        }

        let mut item_taxes = Vec::new();
        let mut total_taxable = 0.0;
        let mut total_tax = 0.0;

        for (item_uuid, amount, category) in items {
            let item_tax = self
                .calculate_item_tax(*item_uuid, *amount, category.as_deref(), false)
                .await?;

            total_taxable += item_tax.taxable_amount;
            total_tax += item_tax.tax_amount;
            item_taxes.push(item_tax);
        }

        let effective_rate = if total_taxable > 0.0 {
            total_tax / total_taxable
        } else {
            0.0
        };

        Ok(TaxBreakdown {
            items: item_taxes,
            total_taxable,
            total_tax,
            effective_rate,
        })
    }

    /// Create a new tax rate
    pub async fn create_tax_rate(&self, rate: &TaxRate) -> Result<()> {
        sqlx::query(
            "INSERT INTO Tax_Rates 
             (rate_id, name, rate, applies_to_category, is_default, is_active, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&rate.rate_id)
        .bind(&rate.name)
        .bind(rate.rate)
        .bind(&rate.applies_to_category)
        .bind(rate.is_default as i32)
        .bind(rate.is_active as i32)
        .bind(rate.created_at.to_rfc3339())
        .bind(rate.updated_at.to_rfc3339())
        .execute(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(())
    }

    /// Get all active tax rates
    pub async fn get_all_rates(&self) -> Result<Vec<TaxRate>> {
        let rows = sqlx::query(
            "SELECT rate_id, name, rate, applies_to_category, is_default, is_active, created_at, updated_at
             FROM Tax_Rates WHERE is_active = 1
             ORDER BY is_default DESC, name ASC",
        )
        .fetch_all(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        let mut rates = Vec::new();
        for row in rows {
            rates.push(TaxRate {
                rate_id: sqlx::Row::try_get(&row, "rate_id").unwrap_or_default(),
                name: sqlx::Row::try_get(&row, "name").unwrap_or_default(),
                rate: sqlx::Row::try_get(&row, "rate").unwrap_or(0.0),
                applies_to_category: sqlx::Row::try_get(&row, "applies_to_category").ok(),
                is_default: sqlx::Row::try_get::<i32, _>(&row, "is_default").unwrap_or(0) == 1,
                is_active: sqlx::Row::try_get::<i32, _>(&row, "is_active").unwrap_or(1) == 1,
                created_at: sqlx::Row::try_get::<String, _>(&row, "created_at")
                    .ok()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
                updated_at: sqlx::Row::try_get::<String, _>(&row, "updated_at")
                    .ok()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
            });
        }

        Ok(rates)
    }

    /// Update an existing tax rate
    pub async fn update_tax_rate(&self, rate: &TaxRate) -> Result<()> {
        sqlx::query(
            "UPDATE Tax_Rates 
             SET name = ?, rate = ?, applies_to_category = ?, is_default = ?, is_active = ?, updated_at = ?
             WHERE rate_id = ?",
        )
        .bind(&rate.name)
        .bind(rate.rate)
        .bind(&rate.applies_to_category)
        .bind(rate.is_default as i32)
        .bind(rate.is_active as i32)
        .bind(Utc::now().to_rfc3339())
        .bind(&rate.rate_id)
        .execute(&self.db.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(())
    }

    /// Deactivate a tax rate (soft delete)
    pub async fn deactivate_tax_rate(&self, rate_id: &str) -> Result<()> {
        sqlx::query("UPDATE Tax_Rates SET is_active = 0, updated_at = ? WHERE rate_id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(rate_id)
            .execute(&self.db.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tax_rate_creation() {
        let rate = TaxRate::new("Sales Tax".to_string(), 0.08);
        assert_eq!(rate.rate, 0.08);
        assert_eq!(rate.name, "Sales Tax");
        assert!(!rate.is_default);
        assert!(rate.is_active);
    }

    #[test]
    fn test_default_tax_rate() {
        let rate = TaxRate::default_rate(0.0825);
        assert_eq!(rate.rate, 0.0825);
        assert!(rate.is_default);
    }

    #[test]
    fn test_zero_breakdown() {
        let breakdown = TaxBreakdown::zero();
        assert_eq!(breakdown.total_tax, 0.0);
        assert_eq!(breakdown.total_taxable, 0.0);
        assert!(breakdown.items.is_empty());
    }
}
