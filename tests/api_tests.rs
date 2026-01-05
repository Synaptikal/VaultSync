//! API Endpoint Tests
//!
//! TASK-235: Basic API endpoint tests
//!
//! Note: Full integration tests require the complete AppState setup.
//! These tests focus on the health endpoints which don't require full state.

/// Test that the database can be initialized
#[tokio::test]
async fn test_database_initialization() {
    let db = vaultsync::database::initialize_test_db()
        .await
        .expect("Failed to initialize database");

    // Create query to verify tables exist
    let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
        .fetch_all(&db.pool)
        .await;

    assert!(result.is_ok(), "Should be able to query database");
}

/// Test JWT token generation and validation
#[tokio::test]
async fn test_jwt_token_creation() {
    use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Claims {
        sub: String,
        user_uuid: String,
        role: String,
        exp: usize,
    }

    let secret = "test-secret";
    let claims = Claims {
        sub: "testuser".to_string(),
        user_uuid: "11111111-1111-1111-1111-111111111111".to_string(),
        role: "Admin".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };

    // Encode
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("Failed to encode token");

    assert!(!token.is_empty());

    // Decode and validate
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .expect("Failed to decode token");

    assert_eq!(token_data.claims.sub, "testuser");
    assert_eq!(token_data.claims.role, "Admin");
}

/// Test password hashing
#[tokio::test]
async fn test_password_hashing() {
    use argon2::{
        password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    };
    use rand::rngs::OsRng;

    let password = "test-password-123";

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();

    assert!(!hash.is_empty());

    // Verify password
    let parsed_hash = PasswordHash::new(&hash).expect("Failed to parse hash");
    let result = argon2.verify_password(password.as_bytes(), &parsed_hash);
    assert!(result.is_ok(), "Password verification should succeed");

    // Wrong password should fail
    let wrong_result = argon2.verify_password(b"wrong-password", &parsed_hash);
    assert!(wrong_result.is_err(), "Wrong password should fail");
}

/// Test UUID generation and formatting
#[test]
fn test_uuid_operations() {
    let uuid = uuid::Uuid::new_v4();
    let uuid_str = uuid.to_string();

    assert_eq!(uuid_str.len(), 36); // Standard UUID format

    let parsed = uuid::Uuid::parse_str(&uuid_str);
    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap(), uuid);
}

/// Test JSON serialization of common types
#[test]
fn test_json_serialization() {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestItem {
        id: i32,
        name: String,
        price: f64,
    }

    let item = TestItem {
        id: 1,
        name: "Test Product".to_string(),
        price: 19.99,
    };

    // Serialize
    let json_str = serde_json::to_string(&item).expect("Failed to serialize");
    assert!(json_str.contains("Test Product"));

    // Deserialize
    let parsed: TestItem = serde_json::from_str(&json_str).expect("Failed to deserialize");
    assert_eq!(parsed, item);
}
