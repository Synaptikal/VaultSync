//! Validation Tests
//!
//! Comprehensive tests for input validation rules

mod validation_rules_tests {
    use vaultsync::api::validation::{rules, ValidationError, Validator};

    // not_empty tests
    #[test]
    fn test_not_empty_with_valid_string() {
        let result = rules::not_empty("name", "John");
        assert!(result.is_ok());
    }

    #[test]
    fn test_not_empty_with_empty_string() {
        let result = rules::not_empty("name", "");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.field, "name");
    }

    #[test]
    fn test_not_empty_with_whitespace_only() {
        let result = rules::not_empty("name", "   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_not_empty_with_whitespace_and_content() {
        let result = rules::not_empty("name", "  John  ");
        assert!(result.is_ok());
    }

    // length_range tests
    #[test]
    fn test_length_range_within_bounds() {
        let result = rules::length_range("name", "John", 2, 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_length_range_too_short() {
        let result = rules::length_range("name", "J", 2, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_length_range_too_long() {
        let result = rules::length_range("name", "ThisNameIsTooLong", 2, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_length_range_at_minimum() {
        let result = rules::length_range("name", "Jo", 2, 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_length_range_at_maximum() {
        let result = rules::length_range("name", "JohnSmith!", 2, 10);
        assert!(result.is_ok());
    }

    // positive tests
    #[test]
    fn test_positive_with_positive_integer() {
        let result = rules::positive("quantity", 5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_positive_with_zero() {
        let result = rules::positive("quantity", 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_positive_with_negative() {
        let result = rules::positive("quantity", -1);
        assert!(result.is_err());
    }

    #[test]
    fn test_positive_with_float() {
        let result = rules::positive("price", 0.01f64);
        assert!(result.is_ok());
    }

    #[test]
    fn test_positive_with_negative_float() {
        let result = rules::positive("price", -0.01f64);
        assert!(result.is_err());
    }

    // non_negative tests
    #[test]
    fn test_non_negative_with_positive() {
        let result = rules::non_negative("stock", 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_non_negative_with_zero() {
        let result = rules::non_negative("stock", 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_non_negative_with_negative() {
        let result = rules::non_negative("stock", -1);
        assert!(result.is_err());
    }

    // in_range tests
    #[test]
    fn test_in_range_within_bounds() {
        let result = rules::in_range("year", 2023, 1900, 2100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_in_range_below_minimum() {
        let result = rules::in_range("year", 1800, 1900, 2100);
        assert!(result.is_err());
    }

    #[test]
    fn test_in_range_above_maximum() {
        let result = rules::in_range("year", 2200, 1900, 2100);
        assert!(result.is_err());
    }

    #[test]
    fn test_in_range_at_minimum() {
        let result = rules::in_range("year", 1900, 1900, 2100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_in_range_at_maximum() {
        let result = rules::in_range("year", 2100, 1900, 2100);
        assert!(result.is_ok());
    }

    // email_format tests
    #[test]
    fn test_email_format_valid() {
        let result = rules::email_format("email", "user@example.com");
        assert!(result.is_ok());
    }

    #[test]
    fn test_email_format_empty_allowed() {
        let result = rules::email_format("email", "");
        assert!(result.is_ok(), "Empty email should be allowed");
    }

    #[test]
    fn test_email_format_missing_at() {
        let result = rules::email_format("email", "userexample.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_email_format_missing_dot() {
        let result = rules::email_format("email", "user@examplecom");
        assert!(result.is_err());
    }

    #[test]
    fn test_email_format_complex_valid() {
        let result = rules::email_format("email", "user.name+tag@subdomain.example.co.uk");
        assert!(result.is_ok());
    }

    // valid_year tests
    #[test]
    fn test_valid_year_current() {
        let result = rules::valid_year("year", 2024);
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_year_future() {
        let result = rules::valid_year("year", 2050);
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_year_too_old() {
        let result = rules::valid_year("year", 1800);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_year_too_far_future() {
        let result = rules::valid_year("year", 2200);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_year_boundary_low() {
        let result = rules::valid_year("year", 1900);
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_year_boundary_high() {
        let result = rules::valid_year("year", 2100);
        assert!(result.is_ok());
    }
}

mod validator_builder_tests {
    use vaultsync::api::validation::{rules, Validator};

    #[test]
    fn test_validator_all_pass() {
        let result = Validator::new()
            .check(rules::not_empty("name", "John"))
            .check(rules::positive("age", 25))
            .check(rules::email_format("email", "john@example.com"))
            .finish();

        assert!(result.is_ok());
    }

    #[test]
    fn test_validator_one_fails() {
        let result = Validator::new()
            .check(rules::not_empty("name", "John"))
            .check(rules::positive("age", -5)) // This fails
            .check(rules::email_format("email", "john@example.com"))
            .finish();

        assert!(result.is_err());
    }

    #[test]
    fn test_validator_finish_all_collects_errors() {
        let result = Validator::new()
            .check(rules::not_empty("name", "")) // Fail
            .check(rules::positive("age", -5)) // Fail
            .check(rules::email_format("email", "bad")) // Fail
            .finish_all();

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 3, "Should collect all 3 errors");
    }

    #[test]
    fn test_validator_finish_returns_first_error() {
        let result = Validator::new()
            .check(rules::not_empty("name", "")) // Fail first
            .check(rules::positive("age", -5)) // Fail second
            .finish();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.field, "name", "Should return first error");
    }

    #[test]
    fn test_validator_empty_passes() {
        let result = Validator::new().finish();
        assert!(result.is_ok());
    }
}

mod validation_error_tests {
    use vaultsync::api::validation::ValidationError;

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError {
            field: "username".to_string(),
            message: "must not be empty".to_string(),
        };

        let display = error.to_string();
        assert!(display.contains("username"));
        assert!(display.contains("must not be empty"));
    }

    #[test]
    fn test_validation_error_to_vaultsync_error() {
        use vaultsync::errors::VaultSyncError;

        let error = ValidationError {
            field: "price".to_string(),
            message: "must be positive".to_string(),
        };

        let vs_error: VaultSyncError = error.into();
        if let VaultSyncError::ValidationError(msg) = vs_error {
            assert!(msg.contains("price"));
            assert!(msg.contains("must be positive"));
        } else {
            panic!("Expected ValidationError variant");
        }
    }
}

mod edge_case_tests {
    use vaultsync::api::validation::rules;

    #[test]
    fn test_unicode_in_not_empty() {
        let result = rules::not_empty("name", "日本語");
        assert!(result.is_ok());
    }

    #[test]
    fn test_unicode_length_range() {
        // Unicode characters count as multiple bytes but single chars
        let result = rules::length_range("name", "日本語", 1, 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_email_with_unicode_domain() {
        // Basic check - has @ and .
        let result = rules::email_format("email", "user@例え.jp");
        assert!(result.is_ok());
    }

    #[test]
    fn test_very_long_string() {
        let long_string = "a".repeat(10000);
        let result = rules::not_empty("data", &long_string);
        assert!(result.is_ok());
    }

    #[test]
    fn test_special_characters() {
        let special = "!@#$%^&*(){}[]|\\:\";<>?,./";
        let result = rules::not_empty("data", special);
        assert!(result.is_ok());
    }
}
