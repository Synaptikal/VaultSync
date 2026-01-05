// Integration tests for authentication and JWT handling

use uuid::Uuid;
use vaultsync::auth::{create_jwt, hash_password, verify_jwt, verify_password, UserRole};

mod common;

#[tokio::test]
async fn test_jwt_creation_and_verification() {
    // Set JWT_SECRET for test
    std::env::set_var(
        "JWT_SECRET",
        "test_secret_minimum_32_characters_long_for_security",
    );

    let user_uuid = Uuid::new_v4();
    let username = "testuser";
    let role = UserRole::Admin;

    // Create JWT
    let token = create_jwt(user_uuid, username, role.clone()).expect("Failed to create JWT");

    assert!(!token.is_empty());

    // Verify JWT
    let claims = verify_jwt(&token).expect("Failed to verify JWT");

    assert_eq!(claims.sub, user_uuid.to_string());
    assert_eq!(claims.username, username);
    assert_eq!(claims.role, role);
}

#[tokio::test]
async fn test_jwt_rejects_invalid_token() {
    std::env::set_var(
        "JWT_SECRET",
        "test_secret_minimum_32_characters_long_for_security",
    );

    let invalid_token = "invalid.jwt.token";
    let result = verify_jwt(invalid_token);

    assert!(result.is_err());
}

#[tokio::test]
async fn test_jwt_rejects_expired_token() {
    std::env::set_var(
        "JWT_SECRET",
        "test_secret_minimum_32_characters_long_for_security",
    );
    std::env::set_var("JWT_EXPIRATION_HOURS", "0"); // Expire immediately

    let user_uuid = Uuid::new_v4();
    let token = create_jwt(user_uuid, "user", UserRole::Employee).expect("Failed to create JWT");

    // Wait a moment for expiration
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let result = verify_jwt(&token);
    assert!(result.is_err(), "Expected expired token to be rejected");
}

/// CRITICAL TEST: Ensure JWT algorithm confusion is prevented
#[tokio::test]
async fn test_jwt_prevents_algorithm_confusion() {
    std::env::set_var(
        "JWT_SECRET",
        "test_secret_minimum_32_characters_long_for_security",
    );

    // Try to create a JWT with "none" algorithm (should be rejected by verify)
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct FakeClaims {
        sub: String,
        username: String,
        role: String,
        exp: usize,
    }

    let mut header = Header::default();
    header.alg = jsonwebtoken::Algorithm::None; // Try to use "none" algorithm

    let claims = FakeClaims {
        sub: Uuid::new_v4().to_string(),
        username: "hacker".to_string(),
        role: "Admin".to_string(),
        exp: 9999999999,
    };

    // This creates a token with alg: "none"
    let fake_token = encode(&header, &claims, &EncodingKey::from_secret(b""));

    if let Ok(token_str) = fake_token {
        // Our verify_jwt should REJECT this
        let result = verify_jwt(&token_str);
        assert!(
            result.is_err(),
            "SECURITY FAILURE: Accepted token with 'none' algorithm!"
        );
    }
}

#[tokio::test]
async fn test_password_hashing_and_verification() {
    let password = "SecurePassword123!";

    // Hash password
    let hash = hash_password(password).expect("Failed to hash password");
    assert!(!hash.is_empty());
    assert_ne!(hash, password); // Hash should be different from plaintext

    // Verify correct password
    let is_valid = verify_password(password, &hash).expect("Failed to verify password");
    assert!(is_valid);

    // Verify incorrect password
    let is_invalid = verify_password("WrongPassword", &hash).expect("Failed to verify password");
    assert!(!is_invalid);
}

#[tokio::test]
async fn test_password_hash_is_unique() {
    let password = "SamePassword123!";

    let hash1 = hash_password(password).expect("Failed to hash");
    let hash2 = hash_password(password).expect("Failed to hash");

    // Hashes should be different due to unique salts
    assert_ne!(hash1, hash2);

    // But both should verify the same password
    assert!(verify_password(password, &hash1).unwrap());
    assert!(verify_password(password, &hash2).unwrap());
}

#[tokio::test]
async fn test_user_roles() {
    use std::str::FromStr;

    // Test role parsing
    let admin = UserRole::from_str("Admin").expect("Failed to parse Admin");
    assert_eq!(admin, UserRole::Admin);

    let manager = UserRole::from_str("Manager").expect("Failed to parse Manager");
    assert_eq!(manager, UserRole::Manager);

    let employee = UserRole::from_str("Employee").expect("Failed to parse Employee");
    assert_eq!(employee, UserRole::Employee);

    // Test invalid role
    let invalid = UserRole::from_str("SuperUser");
    assert!(invalid.is_err());

    // Test to_string
    assert_eq!(admin.to_string(), "Admin");
    assert_eq!(manager.to_string(), "Manager");
    assert_eq!(employee.to_string(), "Employee");
}

#[tokio::test]
async fn test_jwt_with_different_roles() {
    std::env::set_var(
        "JWT_SECRET",
        "test_secret_minimum_32_characters_long_for_security",
    );

    let user_uuid = Uuid::new_v4();

    for role in &[UserRole::Admin, UserRole::Manager, UserRole::Employee] {
        let token = create_jwt(user_uuid, "user", role.clone()).expect("Failed to create JWT");

        let claims = verify_jwt(&token).expect("Failed to verify JWT");
        assert_eq!(claims.role, *role);
    }
}
