//! Pricing and Rule Engine Tests
//!
//! Tests for pricing calculations, condition scaling, and rule engine

use serde_json::json;
use uuid::Uuid;
use vaultsync::core::{Category, Condition, Product};
use vaultsync::pricing::rules::PricingRule;
use vaultsync::pricing::{RuleContext, RuleEngine};

mod condition_scaling_tests {
    use super::*;

    fn get_condition_scale(condition: &Condition) -> f64 {
        // Standard industry condition scaling (as used in buylist)
        match condition {
            Condition::NM
            | Condition::NearMintMint
            | Condition::Mint
            | Condition::GemMint
            | Condition::New => 1.0,
            Condition::LP | Condition::VeryFine | Condition::OpenBox => 0.8,
            Condition::MP | Condition::Fine | Condition::Used => 0.6,
            Condition::HP | Condition::Good => 0.4,
            Condition::DMG | Condition::Poor => 0.2,
        }
    }

    #[test]
    fn test_nm_condition_full_value() {
        let scale = get_condition_scale(&Condition::NM);
        assert_eq!(scale, 1.0);
    }

    #[test]
    fn test_lp_condition_80_percent() {
        let scale = get_condition_scale(&Condition::LP);
        assert!((scale - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_mp_condition_60_percent() {
        let scale = get_condition_scale(&Condition::MP);
        assert!((scale - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_hp_condition_40_percent() {
        let scale = get_condition_scale(&Condition::HP);
        assert!((scale - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_dmg_condition_20_percent() {
        let scale = get_condition_scale(&Condition::DMG);
        assert!((scale - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_condition_scaling_applied_to_price() {
        let base_price = 100.0;

        for condition in [
            Condition::NM,
            Condition::LP,
            Condition::MP,
            Condition::HP,
            Condition::DMG,
        ] {
            let scaled = base_price * get_condition_scale(&condition);
            assert!(scaled <= base_price);
            assert!(scaled > 0.0);
        }
    }

    #[test]
    fn test_condition_scaling_order() {
        // Better conditions should always have higher scaling
        assert!(get_condition_scale(&Condition::NM) > get_condition_scale(&Condition::LP));
        assert!(get_condition_scale(&Condition::LP) > get_condition_scale(&Condition::MP));
        assert!(get_condition_scale(&Condition::MP) > get_condition_scale(&Condition::HP));
        assert!(get_condition_scale(&Condition::HP) > get_condition_scale(&Condition::DMG));
    }
}

mod rule_engine_tests {
    use super::*;

    fn create_test_product() -> Product {
        Product {
            product_uuid: Uuid::new_v4(),
            name: "Test Card".to_string(),
            category: Category::TCG,
            set_code: None,
            collector_number: None,
            barcode: None,
            release_year: None,
            metadata: json!({}),
            weight_oz: None,
            length_in: None,
            width_in: None,
            height_in: None,
            upc: None,
            isbn: None,
            manufacturer: None,
            msrp: None,
            deleted_at: None,
        }
    }

    #[test]
    fn test_rule_engine_default_creation() {
        let engine = RuleEngine::new();
        let rules = engine.get_rules();
        assert!(rules.len() > 0, "Should have default rules");
    }

    #[test]
    fn test_rule_engine_calculate_multipliers() {
        let engine = RuleEngine::new();
        let product = create_test_product();

        let context = RuleContext {
            product: Some(&product),
            condition: &Condition::NM,
            market_price: 5.0, // Low value TCG
            quantity: 1,
            customer_tier: None,
        };

        let (cash, credit) = engine.calculate_multipliers(context);

        // Should return valid multipliers
        assert!(
            cash > 0.0 && cash <= 1.0,
            "Cash rate should be between 0 and 1"
        );
        assert!(
            credit > 0.0 && credit <= 1.0,
            "Credit rate should be between 0 and 1"
        );
        assert!(cash <= credit, "Credit rate should be >= cash rate");
    }

    #[test]
    fn test_high_value_items_get_better_rates() {
        let engine = RuleEngine::new();
        let product = create_test_product();

        let low_value_context = RuleContext {
            product: Some(&product),
            condition: &Condition::NM,
            market_price: 5.0,
            quantity: 1,
            customer_tier: None,
        };

        // Use different market prices in the context, not the product
        // The product struct doesn't have market_price field

        let high_value_context = RuleContext {
            product: Some(&product),
            condition: &Condition::NM,
            market_price: 100.0,
            quantity: 1,
            customer_tier: None,
        };

        let (low_cash, _) = engine.calculate_multipliers(low_value_context);
        let (high_cash, _) = engine.calculate_multipliers(high_value_context);

        // High value items should get better rates
        assert!(
            high_cash >= low_cash,
            "High value items should get equal or better rates"
        );
    }

    #[test]
    fn test_damaged_condition_gets_lower_rate() {
        let engine = RuleEngine::new();
        let product = create_test_product();

        let nm_context = RuleContext {
            product: Some(&product),
            condition: &Condition::NM,
            market_price: 50.0,
            quantity: 1,
            customer_tier: None,
        };

        let dmg_context = RuleContext {
            product: Some(&product),
            condition: &Condition::DMG,
            market_price: 50.0,
            quantity: 1,
            customer_tier: None,
        };

        let (nm_cash, _) = engine.calculate_multipliers(nm_context);
        let (dmg_cash, _) = engine.calculate_multipliers(dmg_context);

        // Damaged items should get worse rates
        assert!(dmg_cash < nm_cash, "Damaged items should get lower rates");
    }

    #[test]
    fn test_rule_priority_works() {
        let mut engine = RuleEngine::new();

        // Add a custom high-priority rule
        let custom_rule = PricingRule {
            id: "custom_test".to_string(),
            priority: 1000, // Very high priority
            category: Some(Category::TCG),
            condition: None,
            min_market_price: None,
            max_market_price: None,
            start_date: None,
            end_date: None,
            customer_tier: None,
            min_quantity: None,
            cash_multiplier: 0.99,
            credit_multiplier: 0.99,
        };

        engine.upsert_rule(custom_rule);

        let product = create_test_product();
        let context = RuleContext {
            product: Some(&product),
            condition: &Condition::NM,
            market_price: 50.0,
            quantity: 1,
            customer_tier: None,
        };

        let (cash, _) = engine.calculate_multipliers(context);

        // Should use our custom high-priority rule
        assert!(
            (cash - 0.99).abs() < 0.01,
            "Should use high priority custom rule"
        );
    }

    #[test]
    fn test_rule_removal() {
        let mut engine = RuleEngine::new();
        let initial_count = engine.get_rules().len();

        let custom_rule = PricingRule {
            id: "removable_rule".to_string(),
            priority: 50,
            category: None,
            condition: None,
            min_market_price: None,
            max_market_price: None,
            start_date: None,
            end_date: None,
            customer_tier: None,
            min_quantity: None,
            cash_multiplier: 0.50,
            credit_multiplier: 0.70,
        };

        engine.upsert_rule(custom_rule);
        assert_eq!(engine.get_rules().len(), initial_count + 1);

        let removed = engine.remove_rule("removable_rule");
        assert!(removed, "Should successfully remove rule");
        assert_eq!(engine.get_rules().len(), initial_count);
    }

    #[test]
    fn test_customer_tier_matching() {
        let mut engine = RuleEngine::new();

        // Add a tier-specific rule
        let vip_rule = PricingRule {
            id: "vip_bonus".to_string(),
            priority: 500,
            category: None,
            condition: None,
            min_market_price: None,
            max_market_price: None,
            start_date: None,
            end_date: None,
            customer_tier: Some("VIP".to_string()),
            min_quantity: None,
            cash_multiplier: 0.80,
            credit_multiplier: 0.90,
        };

        engine.upsert_rule(vip_rule);

        let product = create_test_product();

        let vip_context = RuleContext {
            product: Some(&product),
            condition: &Condition::NM,
            market_price: 50.0,
            quantity: 1,
            customer_tier: Some("VIP"),
        };

        let regular_context = RuleContext {
            product: Some(&product),
            condition: &Condition::NM,
            market_price: 50.0,
            quantity: 1,
            customer_tier: None,
        };

        let (vip_cash, _) = engine.calculate_multipliers(vip_context);
        let (regular_cash, _) = engine.calculate_multipliers(regular_context);

        // VIP should get better rates
        assert!(
            vip_cash > regular_cash,
            "VIP customers should get better rates"
        );
    }
}

mod buy_price_calculation_tests {
    use super::*;

    fn calculate_buy_price(market_price: f64, condition: &Condition, cash_rate: f64) -> f64 {
        let condition_scale = match condition {
            Condition::NM => 1.0,
            Condition::LP => 0.8,
            Condition::MP => 0.6,
            Condition::HP => 0.4,
            Condition::DMG => 0.2,
            _ => 0.8, // Default for other conditions
        };

        market_price * condition_scale * cash_rate
    }

    #[test]
    fn test_full_buy_price_calculation_nm() {
        let market = 100.0;
        let cash_rate = 0.50;
        let price = calculate_buy_price(market, &Condition::NM, cash_rate);

        assert!((price - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_full_buy_price_calculation_lp() {
        let market = 100.0;
        let cash_rate = 0.50;
        let price = calculate_buy_price(market, &Condition::LP, cash_rate);

        // 100 * 0.8 * 0.5 = 40
        assert!((price - 40.0).abs() < 0.01);
    }

    #[test]
    fn test_full_buy_price_calculation_hp() {
        let market = 100.0;
        let cash_rate = 0.50;
        let price = calculate_buy_price(market, &Condition::HP, cash_rate);

        // 100 * 0.4 * 0.5 = 20
        assert!((price - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_high_value_card_calculation() {
        let market = 500.0;
        let cash_rate = 0.60; // Better rate for high value
        let price = calculate_buy_price(market, &Condition::NM, cash_rate);

        assert!((price - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_low_value_card_calculation() {
        let market = 0.25;
        let cash_rate = 0.50;
        let price = calculate_buy_price(market, &Condition::NM, cash_rate);

        assert!((price - 0.125).abs() < 0.001);
    }
}
