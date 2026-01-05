//! Request ID Middleware
//!
//! Generates unique request IDs for tracing requests through the system
//! (TASK-202)

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Header name for request ID
pub static REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Middleware that adds a unique request ID to each request
/// If the client provides an X-Request-Id header, it will be preserved
/// Otherwise, a new UUID will be generated
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    // Check if request already has an ID (from client or load balancer)
    let request_id = request
        .headers()
        .get(&REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Add request ID to extensions for use in handlers
    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    // Create a span with the request ID for tracing
    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri().path(),
    );

    // Execute the request within the span
    let _guard = span.enter();

    let mut response = next.run(request).await;

    // Add request ID to response headers
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response
            .headers_mut()
            .insert(REQUEST_ID_HEADER.clone(), header_value);
    }

    response
}

/// Request ID extractor for use in handlers
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Extension trait to easily get request ID in handlers
impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
