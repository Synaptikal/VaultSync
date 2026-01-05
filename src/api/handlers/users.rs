//! User-related API handlers
//!
//! Handles current user info extraction from JWT.

use axum::{http::StatusCode, response::IntoResponse};
use serde_json::json;

/// Get information about the currently authenticated user
pub async fn get_current_user(headers: axum::http::HeaderMap) -> impl IntoResponse {
    // Extract token from Authorization header
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                match crate::auth::verify_jwt(token) {
                    Ok(claims) => {
                        return (
                            StatusCode::OK,
                            axum::extract::Json(json!({
                                "user_uuid": claims.sub,
                                "username": claims.username,
                                "role": claims.role.to_string()
                            })),
                        )
                            .into_response();
                    }
                    Err(_) => {}
                }
            }
        }
    }

    (
        StatusCode::UNAUTHORIZED,
        axum::extract::Json(json!({"error": "Not authenticated"})),
    )
        .into_response()
}
