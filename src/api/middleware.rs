use crate::api::AppState;
use crate::auth::{verify_jwt, UserRole};
use axum::{
    extract::Request,
    extract::State,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::headers::{authorization::Bearer, Authorization};
use axum_extra::TypedHeader;
use serde_json::json;

/// Extension type to store authenticated user claims in request
#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub user_uuid: String,
    pub username: String,
    pub role: UserRole,
}

/// CRIT-005 FIX: Auth middleware now extracts and stores claims for role checking
pub async fn auth_middleware(
    auth: Option<TypedHeader<Authorization<Bearer>>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth = auth.ok_or(StatusCode::UNAUTHORIZED)?;
    let token = auth.token();

    match verify_jwt(token) {
        Ok(claims) => {
            // Store claims in request extensions for downstream handlers/middleware
            let user = AuthenticatedUser {
                user_uuid: claims.sub.clone(),
                username: claims.username.clone(),
                role: claims.role.clone(),
            };
            request.extensions_mut().insert(user);

            let response = next.run(request).await;
            Ok(response)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Middleware guard for Admin-only routes
pub async fn require_admin(request: Request, next: Next) -> Result<Response, Response> {
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Not authenticated"})),
            )
                .into_response()
        })?;

    if user.role != UserRole::Admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Admin access required", "current_role": user.role.to_string()})),
        )
            .into_response());
    }

    Ok(next.run(request).await)
}

pub async fn metrics_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let endpoint = request.uri().path().to_string();
    let response = next.run(request).await;
    let status = response.status();
    let is_error = status.is_client_error() || status.is_server_error();

    state.metrics.record_request(&endpoint, is_error);
    state.alerting.record_request(status.is_server_error());

    response
}

/// Middleware guard for Manager-or-higher routes (Admin or Manager)
pub async fn require_manager(request: Request, next: Next) -> Result<Response, Response> {
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Not authenticated"})),
            )
                .into_response()
        })?;

    match user.role {
        UserRole::Admin | UserRole::Manager => Ok(next.run(request).await),
        UserRole::Employee => Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Manager or Admin access required", "current_role": user.role.to_string()}))
        ).into_response()),
    }
}

/// Helper to extract authenticated user from request extensions
pub fn get_current_user(request: &Request) -> Option<AuthenticatedUser> {
    request.extensions().get::<AuthenticatedUser>().cloned()
}
