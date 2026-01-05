use crate::core::{Category, Condition, Product};
use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingRule {
    pub id: String,
    pub priority: i32, // Higher number = Higher priority. If multiple rules match, highest priority wins.

    // Matchers (None = Wildcard)
    pub category: Option<Category>,
    pub condition: Option<Condition>,
    pub min_market_price: Option<f64>,
    pub max_market_price: Option<f64>,

    // Advanced Matchers (Tasks 083, 084, 085)
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub customer_tier: Option<String>,
    pub min_quantity: Option<i32>,

    // Multipliers
    pub cash_multiplier: f64,
    pub credit_multiplier: f64,
}

pub struct RuleContext<'a> {
    pub product: Option<&'a Product>,
    pub condition: &'a Condition,
    pub market_price: f64,
    pub quantity: i32,
    pub customer_tier: Option<&'a str>,
}

#[derive(Clone)]
pub struct RuleEngine {
    rules: Vec<PricingRule>,
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleEngine {
    /// Create with default hardcoded rules
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
        }
    }

    /// MED-004 FIX: Load rules from database, falling back to defaults if empty
    pub async fn load_from_db(db: &crate::database::Database) -> Self {
        match db.get_pricing_rules().await {
            Ok(rules) if !rules.is_empty() => {
                tracing::info!("Loaded {} pricing rules from database", rules.len());
                Self { rules }
            }
            Ok(_) => {
                tracing::info!("No pricing rules in database, using defaults");
                Self::new()
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to load pricing rules from DB: {}, using defaults",
                    e
                );
                Self::new()
            }
        }
    }

    /// Save current rules to database
    pub async fn save_to_db(&self, db: &crate::database::Database) -> anyhow::Result<()> {
        for rule in &self.rules {
            db.save_pricing_rule(rule).await?;
        }
        tracing::info!("Saved {} pricing rules to database", self.rules.len());
        Ok(())
    }

    /// Get all rules
    pub fn get_rules(&self) -> &[PricingRule] {
        &self.rules
    }

    /// Add or update a rule
    pub fn upsert_rule(&mut self, rule: PricingRule) {
        // Remove existing rule with same ID if present
        self.rules.retain(|r| r.id != rule.id);
        self.rules.push(rule);
        // Sort by priority (descending)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Remove a rule by ID
    pub fn remove_rule(&mut self, rule_id: &str) -> bool {
        let len_before = self.rules.len();
        self.rules.retain(|r| r.id != rule_id);
        self.rules.len() < len_before
    }

    /// Default hardcoded rules (used as fallback)
    fn default_rules() -> Vec<PricingRule> {
        vec![
            // Base Rules for TCG (Low End)
            PricingRule {
                id: "tcg_bulk".to_string(),
                priority: 10,
                category: Some(Category::TCG),
                condition: None, // Apply to all checks unless overridden
                min_market_price: None,
                max_market_price: Some(10.0), // Under $10
                start_date: None,
                end_date: None,
                customer_tier: None,
                min_quantity: None,
                cash_multiplier: 0.40,
                credit_multiplier: 0.55,
            },
            // Base Rule for TCG (Mid Range)
            PricingRule {
                id: "tcg_mid".to_string(),
                priority: 20,
                category: Some(Category::TCG),
                condition: None,
                min_market_price: Some(10.0),
                max_market_price: Some(50.0),
                start_date: None,
                end_date: None,
                customer_tier: None,
                min_quantity: None,
                cash_multiplier: 0.50,
                credit_multiplier: 0.65,
            },
            // High End Rule
            PricingRule {
                id: "high_end".to_string(),
                priority: 30,
                category: None, // Any category
                condition: None,
                min_market_price: Some(50.0),
                max_market_price: None,
                start_date: None,
                end_date: None,
                customer_tier: None,
                min_quantity: None,
                cash_multiplier: 0.60,
                credit_multiplier: 0.75,
            },
            // Damaged Cards Punishment
            PricingRule {
                id: "damaged".to_string(),
                priority: 100, // Very specific
                category: None,
                condition: Some(Condition::DMG),
                min_market_price: None,
                max_market_price: None,
                start_date: None,
                end_date: None,
                customer_tier: None,
                min_quantity: None,
                cash_multiplier: 0.20,
                credit_multiplier: 0.30,
            },
            // Fallback Global
            PricingRule {
                id: "global_default".to_string(),
                priority: 0,
                category: None,
                condition: None,
                min_market_price: None,
                max_market_price: None,
                start_date: None,
                end_date: None,
                customer_tier: None,
                min_quantity: None,
                cash_multiplier: 0.30,
                credit_multiplier: 0.50,
            },
            // LOW-002 FIX: Sports Cards Default
            PricingRule {
                id: "sports_default".to_string(),
                priority: 15,
                category: Some(Category::SportsCard),
                condition: None,
                min_market_price: None,
                max_market_price: None,
                start_date: None,
                end_date: None,
                customer_tier: None,
                min_quantity: None,
                cash_multiplier: 0.40,
                credit_multiplier: 0.55,
            },
        ]
    }

    pub fn calculate_multipliers(&self, context: RuleContext) -> (f64, f64) {
        // Find best matching rule
        let mut best_rule: Option<&PricingRule> = None;

        for rule in &self.rules {
            if !self.matches(rule, &context) {
                continue;
            }

            if let Some(current_best) = best_rule {
                if rule.priority > current_best.priority {
                    best_rule = Some(rule);
                }
            } else {
                best_rule = Some(rule);
            }
        }

        if let Some(rule) = best_rule {
            (rule.cash_multiplier, rule.credit_multiplier)
        } else {
            (0.3, 0.5) // Emergency fallback
        }
    }

    fn matches(&self, rule: &PricingRule, context: &RuleContext) -> bool {
        // Check Category
        if let Some(cat) = &rule.category {
            if let Some(p) = context.product {
                if p.category != *cat {
                    return false;
                }
            }
        }

        // Check Condition
        if let Some(cond) = &rule.condition {
            if cond != context.condition {
                return false;
            }
        }

        // Check Price Range
        if let Some(min) = rule.min_market_price {
            if context.market_price < min {
                return false;
            }
        }
        if let Some(max) = rule.max_market_price {
            if context.market_price > max {
                return false;
            }
        }

        // Check Date Range (TASKS 083)
        let now = Utc::now();
        if let Some(start) = rule.start_date {
            if now < start {
                return false;
            }
        }
        if let Some(end) = rule.end_date {
            if now > end {
                return false;
            }
        }

        // Check Customer Tier (TASK 084)
        if let Some(rule_tier) = &rule.customer_tier {
            match context.customer_tier {
                Some(tier) if tier == rule_tier => {}
                _ => return false,
            }
        }

        // Check Quantity (TASK 085)
        if let Some(min_q) = rule.min_quantity {
            if context.quantity < min_q {
                return false;
            }
        }

        true
    }
}
