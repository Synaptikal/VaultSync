//! Audit and Conflict Resolution Tests
//!
//! Tests for inventory auditing, blind counts, and conflict detection

use std::str::FromStr;
use vaultsync::audit::{ConflictType, ResolutionStatus};

mod conflict_type_tests {
    use super::*;

    #[test]
    fn test_conflict_type_display() {
        assert_eq!(ConflictType::Oversold.to_string(), "Oversold");
        assert_eq!(ConflictType::PriceMismatch.to_string(), "PriceMismatch");
        assert_eq!(ConflictType::CreditAnomaly.to_string(), "CreditAnomaly");
        assert_eq!(
            ConflictType::PhysicalMiscount.to_string(),
            "PhysicalMiscount"
        );
        assert_eq!(ConflictType::SyncConflict.to_string(), "SyncConflict");
    }

    #[test]
    fn test_conflict_type_from_str() {
        assert_eq!(
            ConflictType::from_str("Oversold").unwrap(),
            ConflictType::Oversold
        );
        assert_eq!(
            ConflictType::from_str("PriceMismatch").unwrap(),
            ConflictType::PriceMismatch
        );
        assert_eq!(
            ConflictType::from_str("CreditAnomaly").unwrap(),
            ConflictType::CreditAnomaly
        );
        assert_eq!(
            ConflictType::from_str("PhysicalMiscount").unwrap(),
            ConflictType::PhysicalMiscount
        );
        assert_eq!(
            ConflictType::from_str("SyncConflict").unwrap(),
            ConflictType::SyncConflict
        );
    }

    #[test]
    fn test_conflict_type_unknown_defaults() {
        // Unknown types should default to PhysicalMiscount per implementation
        let result = ConflictType::from_str("UnknownType");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConflictType::PhysicalMiscount);
    }

    #[test]
    fn test_conflict_type_case_sensitive() {
        // Should match exactly
        assert_eq!(
            ConflictType::from_str("oversold").unwrap(),
            ConflictType::PhysicalMiscount
        );
        assert_eq!(
            ConflictType::from_str("OVERSOLD").unwrap(),
            ConflictType::PhysicalMiscount
        );
    }

    #[test]
    fn test_conflict_type_serialization() {
        use serde_json;

        let conflict = ConflictType::Oversold;
        let json = serde_json::to_string(&conflict).expect("Should serialize");
        let parsed: ConflictType = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(conflict, parsed);
    }
}

mod resolution_status_tests {
    use super::*;

    #[test]
    fn test_resolution_status_display() {
        assert_eq!(ResolutionStatus::Pending.to_string(), "Pending");
        assert_eq!(ResolutionStatus::Resolved.to_string(), "Resolved");
        assert_eq!(ResolutionStatus::Ignored.to_string(), "Ignored");
    }

    #[test]
    fn test_resolution_status_from_str() {
        assert_eq!(
            ResolutionStatus::from_str("Pending").unwrap(),
            ResolutionStatus::Pending
        );
        assert_eq!(
            ResolutionStatus::from_str("Resolved").unwrap(),
            ResolutionStatus::Resolved
        );
        assert_eq!(
            ResolutionStatus::from_str("Ignored").unwrap(),
            ResolutionStatus::Ignored
        );
    }

    #[test]
    fn test_resolution_status_unknown_defaults() {
        // Unknown should default to Pending
        let result = ResolutionStatus::from_str("Unknown");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ResolutionStatus::Pending);
    }

    #[test]
    fn test_resolution_status_serialization() {
        use serde_json;

        for status in [
            ResolutionStatus::Pending,
            ResolutionStatus::Resolved,
            ResolutionStatus::Ignored,
        ] {
            let json = serde_json::to_string(&status).expect("Should serialize");
            let parsed: ResolutionStatus = serde_json::from_str(&json).expect("Should deserialize");
            assert_eq!(status, parsed);
        }
    }
}

mod conflict_detection_tests {
    use std::collections::HashMap;
    use uuid::Uuid;

    fn detect_miscount(
        expected: &HashMap<Uuid, i32>,
        counted: &HashMap<Uuid, i32>,
    ) -> Vec<(Uuid, i32, i32)> {
        let mut conflicts = Vec::new();

        // Check counted vs expected
        for (pid, counted_qty) in counted {
            let expected_qty = *expected.get(pid).unwrap_or(&0);
            if *counted_qty != expected_qty {
                conflicts.push((*pid, expected_qty, *counted_qty));
            }
        }

        // Check for items expected but not counted
        for (pid, expected_qty) in expected {
            if !counted.contains_key(pid) {
                conflicts.push((*pid, *expected_qty, 0));
            }
        }

        conflicts
    }

    #[test]
    fn test_no_conflicts_when_counts_match() {
        let product = Uuid::new_v4();
        let mut expected = HashMap::new();
        let mut counted = HashMap::new();

        expected.insert(product, 10);
        counted.insert(product, 10);

        let conflicts = detect_miscount(&expected, &counted);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_conflict_when_count_too_low() {
        let product = Uuid::new_v4();
        let mut expected = HashMap::new();
        let mut counted = HashMap::new();

        expected.insert(product, 10);
        counted.insert(product, 8);

        let conflicts = detect_miscount(&expected, &counted);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0], (product, 10, 8));
    }

    #[test]
    fn test_conflict_when_count_too_high() {
        let product = Uuid::new_v4();
        let mut expected = HashMap::new();
        let mut counted = HashMap::new();

        expected.insert(product, 10);
        counted.insert(product, 12);

        let conflicts = detect_miscount(&expected, &counted);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0], (product, 10, 12));
    }

    #[test]
    fn test_conflict_when_item_missing() {
        let product = Uuid::new_v4();
        let mut expected = HashMap::new();
        let counted = HashMap::new();

        expected.insert(product, 10);
        // Nothing counted

        let conflicts = detect_miscount(&expected, &counted);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0], (product, 10, 0));
    }

    #[test]
    fn test_conflict_when_extra_item_found() {
        let expected_product = Uuid::new_v4();
        let extra_product = Uuid::new_v4();

        let mut expected = HashMap::new();
        let mut counted = HashMap::new();

        expected.insert(expected_product, 10);
        counted.insert(expected_product, 10);
        counted.insert(extra_product, 5); // Extra item not in system

        let conflicts = detect_miscount(&expected, &counted);
        assert_eq!(conflicts.len(), 1);
        // Extra item has expected 0
        assert_eq!(conflicts[0], (extra_product, 0, 5));
    }

    #[test]
    fn test_multiple_conflicts() {
        let product1 = Uuid::new_v4();
        let product2 = Uuid::new_v4();
        let product3 = Uuid::new_v4();

        let mut expected = HashMap::new();
        let mut counted = HashMap::new();

        expected.insert(product1, 10);
        expected.insert(product2, 5);
        expected.insert(product3, 20);

        counted.insert(product1, 8); // -2
        counted.insert(product2, 5); // Match
                                     // product3 not counted at all

        let conflicts = detect_miscount(&expected, &counted);
        assert_eq!(conflicts.len(), 2); // product1 mismatch + product3 missing
    }
}

mod oversold_detection_tests {
    use uuid::Uuid;

    struct InventoryState {
        product: Uuid,
        on_hand: i32,
        pending_sales: i32,
    }

    fn is_oversold(state: &InventoryState) -> bool {
        state.on_hand < state.pending_sales
    }

    fn oversold_amount(state: &InventoryState) -> i32 {
        if is_oversold(state) {
            state.pending_sales - state.on_hand
        } else {
            0
        }
    }

    #[test]
    fn test_not_oversold_when_sufficient_stock() {
        let state = InventoryState {
            product: Uuid::new_v4(),
            on_hand: 10,
            pending_sales: 5,
        };

        assert!(!is_oversold(&state));
        assert_eq!(oversold_amount(&state), 0);
    }

    #[test]
    fn test_oversold_when_insufficient_stock() {
        let state = InventoryState {
            product: Uuid::new_v4(),
            on_hand: 3,
            pending_sales: 5,
        };

        assert!(is_oversold(&state));
        assert_eq!(oversold_amount(&state), 2);
    }

    #[test]
    fn test_not_oversold_when_equal() {
        let state = InventoryState {
            product: Uuid::new_v4(),
            on_hand: 5,
            pending_sales: 5,
        };

        assert!(!is_oversold(&state));
    }

    #[test]
    fn test_zero_stock_with_sales() {
        let state = InventoryState {
            product: Uuid::new_v4(),
            on_hand: 0,
            pending_sales: 3,
        };

        assert!(is_oversold(&state));
        assert_eq!(oversold_amount(&state), 3);
    }
}
