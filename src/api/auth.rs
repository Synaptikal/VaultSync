use crate::api::AppState;
use crate::auth::{create_jwt, hash_password, verify_password, UserRole};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RegisterRequest {
    username: String,
    password: String,
    role: String, // "admin", "employee"
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    token: String,
    refresh_token: String, // MED-005: Added refresh token
}

#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    refresh_token: String,
}

#[axum::debug_handler]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Check if user exists
    match state.db.auth.get_user_by_username(&payload.username).await {
        Ok(Some(_)) => {
            return (StatusCode::CONFLICT, "User already exists").into_response();
        }
        Ok(None) => {} // Continue
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error checking user",
            )
                .into_response();
        }
    }

    let user_uuid = Uuid::new_v4();
    let password_hash = match hash_password(&payload.password) {
        Ok(hash) => hash,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password").into_response();
        }
    };

    // Parse role
    let role = match UserRole::from_str(&payload.role) {
        Ok(r) => r,
        Err(_) => {
            // Default to Employee if invalid or fail - here failing is safer
            return (StatusCode::BAD_REQUEST, "Invalid role").into_response();
        }
    };

    if state
        .db
        .auth
        .insert_user(user_uuid, &payload.username, &password_hash, role)
        .await
        .is_err()
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user").into_response();
    }

    (StatusCode::CREATED, "User created successfully").into_response()
}

#[axum::debug_handler]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let user_option = match state.db.auth.get_user_by_username(&payload.username).await {
        Ok(option) => option,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    let (user_uuid, _username, password_hash, role) = match user_option {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
    };

    match verify_password(&payload.password, &password_hash) {
        Ok(valid) => {
            if !valid {
                return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response();
            }
        }
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Auth error").into_response(),
    }

    // Generate Access Token (JWT)
    let token = match create_jwt(user_uuid, &payload.username, role) {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate token",
            )
                .into_response()
        }
    };

    // MED-005 FIX: Generate Refresh Token
    let refresh_token = crate::auth::create_refresh_token();
    let refresh_token_hash = crate::auth::hash_token(&refresh_token);

    // Save refresh token to DB (valid for 30 days)
    let expires_at = chrono::Utc::now() + chrono::Duration::days(30);
    if let Err(e) = state
        .db
        .save_refresh_token(&refresh_token_hash, user_uuid, expires_at)
        .await
    {
        tracing::error!("Failed to save refresh token: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save session").into_response();
    }

    Json(AuthResponse {
        token,
        refresh_token,
    })
    .into_response()
}

/// MED-005 FIX: Refresh token endpoint
#[axum::debug_handler]
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> impl IntoResponse {
    let refresh_token_hash = crate::auth::hash_token(&payload.refresh_token);

    // Verify refresh token
    match state.db.get_refresh_token(&refresh_token_hash).await {
        Ok(Some((user_uuid, expires_at, is_revoked))) => {
            // Check if revoked
            if is_revoked {
                return (StatusCode::UNAUTHORIZED, "Token revoked").into_response();
            }

            // Check if expired
            if chrono::Utc::now() > expires_at {
                return (StatusCode::UNAUTHORIZED, "Token expired").into_response();
            }

            // Get user details for new JWT
            let (jwt_user_uuid, username, _, role) =
                match state.db.auth.get_user_by_uuid(user_uuid).await {
                    Ok(Some(u)) => u,
                    _ => return (StatusCode::UNAUTHORIZED, "User not found").into_response(),
                };

            // Generate new JWT
            let new_token = match create_jwt(jwt_user_uuid, &username, role) {
                Ok(t) => t,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to generate token",
                    )
                        .into_response()
                }
            };

            // Rotate refresh token (Security best practice)
            let new_refresh_token = crate::auth::create_refresh_token();
            let new_refresh_hash = crate::auth::hash_token(&new_refresh_token);
            let new_expires_at = chrono::Utc::now() + chrono::Duration::days(30);

            if let Err(e) = state
                .db
                .save_refresh_token(&new_refresh_hash, user_uuid, new_expires_at)
                .await
            {
                tracing::error!("Failed to save new refresh token: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
            }

            // Revoke old token
            let _ = state.db.revoke_refresh_token(&refresh_token_hash).await;

            Json(AuthResponse {
                token: new_token,
                refresh_token: new_refresh_token,
            })
            .into_response()
        }
        Ok(None) => (StatusCode::UNAUTHORIZED, "Invalid refresh token").into_response(),
        Err(e) => {
            tracing::error!("Database error checking refresh token: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}
