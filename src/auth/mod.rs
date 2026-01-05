use crate::errors::{Result, VaultSyncError};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum UserRole {
    Admin,
    Manager,
    Employee,
}

impl ToString for UserRole {
    fn to_string(&self) -> String {
        match self {
            UserRole::Admin => "Admin".to_string(),
            UserRole::Manager => "Manager".to_string(),
            UserRole::Employee => "Employee".to_string(),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Admin" => Ok(UserRole::Admin),
            "Manager" => Ok(UserRole::Manager),
            "Employee" => Ok(UserRole::Employee),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub user_uuid: Uuid,
    pub username: String,
    pub role: UserRole,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_uuid
    pub username: String,
    pub role: UserRole, // Strongly typed
    pub exp: usize,
}

// Helper to get secret from env or default
fn get_jwt_secret() -> Result<Vec<u8>> {
    let s = std::env::var("JWT_SECRET").map_err(|_| {
        VaultSyncError::ConfigError("JWT_SECRET environment variable is not set".to_string())
    })?;
    Ok(s.into_bytes())
}

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| VaultSyncError::AuthError(e.to_string()))?
        .to_string();
    Ok(password_hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed_hash =
        PasswordHash::new(password_hash).map_err(|e| VaultSyncError::AuthError(e.to_string()))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn create_jwt(user_uuid: Uuid, username: &str, role: UserRole) -> Result<String> {
    let expiration_hours: u64 = std::env::var("JWT_EXPIRATION_HOURS")
        .unwrap_or_else(|_| "24".to_string())
        .parse()
        .unwrap_or(24);

    // SECURITY FIX: UNIX_EPOCH is always in the past, so this should never fail.
    // Using expect() for clarity about the invariant.
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System clock is before UNIX epoch - this should never happen")
        .as_secs() as usize
        + (expiration_hours as usize * 3600);

    let claims = Claims {
        sub: user_uuid.to_string(),
        username: username.to_string(),
        role,
        exp: expiration,
    };

    let secret = get_jwt_secret()?;
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&secret),
    )
    .map_err(|e| VaultSyncError::AuthError(e.to_string()))?;

    Ok(token)
}

pub fn verify_jwt(token: &str) -> Result<Claims> {
    let secret = get_jwt_secret()?;

    // SECURITY FIX (CRIT-07): Explicitly enforce HS256 algorithm
    // Prevents algorithm confusion attacks (e.g., "none", RS256 with public key)
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.leeway = 60; // 1 minute clock skew tolerance

    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(&secret), &validation)
        .map_err(|e| VaultSyncError::AuthError(e.to_string()))?;

    Ok(token_data.claims)
}

/// MED-005 FIX: Generate a secure random refresh token
pub fn create_refresh_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let token: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    hex::encode(token)
}

/// Hash a refresh token for storage (SHA256)
pub fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
