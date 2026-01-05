//! Comprehensive Auth Module Tests
//!
//! Tests for authentication, password hashing, JWT tokens, and role management

use std::str::FromStr;

mod password_tests {
    use vaultsync::auth::{hash_password, verify_password};

    #[test]
    fn test_password_hashing_returns_valid_hash() {
        let password = "secure_password_123!";
        let hash = hash_password(password).expect("Should hash password");

        // Verify hash format (Argon2 hashes start with $argon2)
        assert!(hash.starts_with("$argon2"), "Hash should be Argon2 format");
        assert!(hash.len() > 50, "Hash should be reasonably long");
    }

    #[test]
    fn test_password_verification_correct_password() {
        let password = "correct_password";
        let hash = hash_password(password).expect("Should hash password");

        let result = verify_password(password, &hash).expect("Should verify password");
        assert!(result, "Correct password should verify");
    }

    #[test]
    fn test_password_verification_wrong_password() {
        let password = "correct_password";
        let hash = hash_password(password).expect("Should hash password");

        let result = verify_password("wrong_password", &hash).expect("Should verify password");
        assert!(!result, "Wrong password should not verify");
    }

    #[test]
    fn test_password_hashing_produces_unique_hashes() {
        let password = "same_password";
        let hash1 = hash_password(password).expect("Should hash password");
        let hash2 = hash_password(password).expect("Should hash password");

        // Due to random salt, hashes should be different
        assert_ne!(
            hash1, hash2,
            "Same password should produce different hashes due to salt"
        );
    }

    #[test]
    fn test_password_verification_with_invalid_hash_format() {
        let result = verify_password("password", "not_a_valid_hash");
        assert!(result.is_err(), "Invalid hash format should return error");
    }

    #[test]
    fn test_empty_password_can_be_hashed() {
        // While not recommended, empty passwords should still work
        let hash = hash_password("").expect("Should hash empty password");
        let result = verify_password("", &hash).expect("Should verify");
        assert!(result, "Empty password should verify");
    }

    #[test]
    fn test_unicode_password() {
        let password = "密码🔐パスワード";
        let hash = hash_password(password).expect("Should hash unicode password");

        let result = verify_password(password, &hash).expect("Should verify");
        assert!(result, "Unicode password should verify");
    }

    #[test]
    fn test_very_long_password() {
        let password = "a".repeat(1000);
        let hash = hash_password(&password).expect("Should hash long password");

        let result = verify_password(&password, &hash).expect("Should verify");
        assert!(result, "Long password should verify");
    }
}

mod jwt_tests {
    use uuid::Uuid;
    use vaultsync::auth::{create_jwt, verify_jwt, UserRole};

    #[test]
    fn test_create_jwt_returns_token() {
        std::env::set_var("JWT_SECRET", "test_secret_key_that_is_long_enough_for_jwt");

        let user_uuid = Uuid::new_v4();
        let token =
            create_jwt(user_uuid, "testuser", UserRole::Employee).expect("Should create JWT");

        // JWT format: header.payload.signature
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3, "JWT should have 3 parts");
    }

    #[test]
    fn test_verify_jwt_valid_token() {
        std::env::set_var("JWT_SECRET", "test_secret_key_that_is_long_enough_for_jwt");

        let user_uuid = Uuid::new_v4();
        let token = create_jwt(user_uuid, "testuser", UserRole::Admin).expect("Should create JWT");

        let claims = verify_jwt(&token).expect("Should verify JWT");

        assert_eq!(claims.sub, user_uuid.to_string());
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.role, UserRole::Admin);
    }

    #[test]
    fn test_verify_jwt_invalid_token() {
        std::env::set_var("JWT_SECRET", "test_secret_key_that_is_long_enough_for_jwt");

        let result = verify_jwt("invalid.token.here");
        assert!(result.is_err(), "Invalid token should fail verification");
    }

    #[test]
    fn test_verify_jwt_tampered_token() {
        std::env::set_var("JWT_SECRET", "test_secret_key_that_is_long_enough_for_jwt");

        let user_uuid = Uuid::new_v4();
        let token =
            create_jwt(user_uuid, "testuser", UserRole::Employee).expect("Should create JWT");

        // Tamper with the token
        let mut tampered = token.clone();
        if tampered.len() > 10 {
            tampered.replace_range(5..8, "XXX");
        }

        let result = verify_jwt(&tampered);
        assert!(result.is_err(), "Tampered token should fail verification");
    }

    #[test]
    fn test_jwt_with_different_roles() {
        std::env::set_var("JWT_SECRET", "test_secret_key_that_is_long_enough_for_jwt");

        let user_uuid = Uuid::new_v4();

        for role in [UserRole::Admin, UserRole::Manager, UserRole::Employee] {
            let token = create_jwt(user_uuid, "user", role.clone()).expect("Should create JWT");

            let claims = verify_jwt(&token).expect("Should verify");
            assert_eq!(claims.role, role, "Role should match");
        }
    }
}

mod role_tests {
    use std::str::FromStr;
    use vaultsync::auth::UserRole;

    #[test]
    fn test_user_role_to_string() {
        assert_eq!(UserRole::Admin.to_string(), "Admin");
        assert_eq!(UserRole::Manager.to_string(), "Manager");
        assert_eq!(UserRole::Employee.to_string(), "Employee");
    }

    #[test]
    fn test_user_role_from_string_valid() {
        assert_eq!(UserRole::from_str("Admin").unwrap(), UserRole::Admin);
        assert_eq!(UserRole::from_str("Manager").unwrap(), UserRole::Manager);
        assert_eq!(UserRole::from_str("Employee").unwrap(), UserRole::Employee);
    }

    #[test]
    fn test_user_role_from_string_invalid() {
        let result = UserRole::from_str("InvalidRole");
        assert!(result.is_err(), "Invalid role should return error");
    }

    #[test]
    fn test_user_role_from_string_case_sensitive() {
        // These should fail because wrong case
        assert!(UserRole::from_str("admin").is_err());
        assert!(UserRole::from_str("ADMIN").is_err());
        assert!(UserRole::from_str("manager").is_err());
    }
}

mod refresh_token_tests {
    use vaultsync::auth::{create_refresh_token, hash_token};

    #[test]
    fn test_refresh_token_generation() {
        let token = create_refresh_token();

        // Should be 64 hex characters (32 bytes as hex)
        assert_eq!(token.len(), 64, "Refresh token should be 64 hex chars");
        assert!(
            token.chars().all(|c| c.is_ascii_hexdigit()),
            "Token should only contain hex chars"
        );
    }

    #[test]
    fn test_refresh_tokens_are_unique() {
        let token1 = create_refresh_token();
        let token2 = create_refresh_token();

        assert_ne!(token1, token2, "Each refresh token should be unique");
    }

    #[test]
    fn test_hash_token_produces_consistent_hash() {
        let token = "test_token_value";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);

        assert_eq!(hash1, hash2, "Same token should produce same hash");
    }

    #[test]
    fn test_hash_token_different_inputs() {
        let hash1 = hash_token("token1");
        let hash2 = hash_token("token2");

        assert_ne!(
            hash1, hash2,
            "Different tokens should produce different hashes"
        );
    }

    #[test]
    fn test_hash_token_format() {
        let hash = hash_token("test");

        // SHA256 produces 32 bytes = 64 hex characters
        assert_eq!(hash.len(), 64, "Hash should be 64 hex chars");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should only contain hex chars"
        );
    }
}
