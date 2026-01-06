use crate::auth::UserRole;
use crate::errors::Result;
use sqlx::{Row, SqlitePool};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthRepository {
    pool: SqlitePool,
}

impl AuthRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert_user(
        &self,
        user_uuid: Uuid,
        username: &str,
        password_hash: &str,
        role: UserRole,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO Users 
            (user_uuid, username, password_hash, role, created_at) 
            VALUES (?, ?, ?, ?, ?)",
        )
        .bind(user_uuid.to_string())
        .bind(username)
        .bind(password_hash)
        .bind(role.to_string())
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<(Uuid, String, String, UserRole)>> {
        let row = sqlx::query(
            "SELECT user_uuid, username, password_hash, role FROM Users WHERE username = ?",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let user_uuid_str: String = row.try_get("user_uuid").unwrap_or_default();
            let user_uuid = Uuid::parse_str(&user_uuid_str).unwrap_or_default();
            let username: String = row.try_get("username").unwrap_or_default();
            let password_hash: String = row.try_get("password_hash").unwrap_or_default();
            let role_str: String = row.try_get("role").unwrap_or_default();
            let role = UserRole::from_str(&role_str).unwrap_or(UserRole::Employee); // Default to low priv on error

            Ok(Some((user_uuid, username, password_hash, role)))
        } else {
            Ok(None)
        }
    }

    pub async fn get_user_by_uuid(
        &self,
        uuid: Uuid,
    ) -> Result<Option<(Uuid, String, String, UserRole)>> {
        let row = sqlx::query(
            "SELECT user_uuid, username, password_hash, role FROM Users WHERE user_uuid = ?",
        )
        .bind(uuid.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| crate::errors::VaultSyncError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let user_uuid_str: String = row.try_get("user_uuid").unwrap_or_default();
            let user_uuid = Uuid::parse_str(&user_uuid_str).unwrap_or_default();
            let username: String = row.try_get("username").unwrap_or_default();
            let password_hash: String = row.try_get("password_hash").unwrap_or_default();
            let role_str: String = row.try_get("role").unwrap_or_default();
            let role = UserRole::from_str(&role_str).unwrap_or(UserRole::Employee);

            Ok(Some((user_uuid, username, password_hash, role)))
        } else {
            Ok(None)
        }
    }
}
