//! Authentication middleware for REST API.
//!
//! Provides API key authentication for protecting endpoints.

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashSet;

/// Authentication configuration.
#[derive(Clone, Debug)]
pub struct AuthConfig {
    /// Whether authentication is enabled.
    pub enabled: bool,
    /// Valid API keys.
    pub api_keys: HashSet<String>,
    /// Paths that don't require authentication (e.g., health checks).
    pub exempt_paths: HashSet<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_keys: HashSet::new(),
            exempt_paths: HashSet::from([
                "/health".to_string(),
                "/ready".to_string(),
                "/live".to_string(),
                "/metrics".to_string(),
            ]),
        }
    }
}

impl AuthConfig {
    /// Create a new auth config with API key authentication enabled.
    pub fn with_api_keys(api_keys: Vec<String>) -> Self {
        Self {
            enabled: true,
            api_keys: api_keys.into_iter().collect(),
            exempt_paths: HashSet::from([
                "/health".to_string(),
                "/ready".to_string(),
                "/live".to_string(),
                "/metrics".to_string(),
            ]),
        }
    }

    /// Add exempt paths that don't require authentication.
    pub fn with_exempt_paths(mut self, paths: Vec<String>) -> Self {
        for path in paths {
            self.exempt_paths.insert(path);
        }
        self
    }
}

/// Authentication middleware that checks for valid API key.
///
/// Checks for API key in:
/// 1. `Authorization: Bearer <key>` header
/// 2. `X-API-Key: <key>` header
pub async fn auth_middleware(
    axum::Extension(config): axum::Extension<AuthConfig>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Skip if auth is disabled
    if !config.enabled {
        return next.run(request).await;
    }

    // Check if path is exempt
    let path = request.uri().path();
    if config.exempt_paths.contains(path) {
        return next.run(request).await;
    }

    // Extract API key from headers
    let api_key = extract_api_key(&request);

    match api_key {
        Some(key) if config.api_keys.contains(&key) => {
            // Valid API key, proceed
            next.run(request).await
        }
        Some(_) => {
            // Invalid API key
            (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Bearer")],
                "Invalid API key",
            )
                .into_response()
        }
        None => {
            // No API key provided
            (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Bearer")],
                "API key required. Provide via 'Authorization: Bearer <key>' or 'X-API-Key' header",
            )
                .into_response()
        }
    }
}

/// Extract API key from request headers.
fn extract_api_key(request: &Request<Body>) -> Option<String> {
    // Try Authorization: Bearer <key>
    if let Some(auth_header) = request.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(key) = auth_str.strip_prefix("Bearer ") {
                return Some(key.to_string());
            }
        }
    }

    // Try X-API-Key header
    if let Some(api_key_header) = request.headers().get("X-API-Key") {
        if let Ok(key) = api_key_header.to_str() {
            return Some(key.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "ok"
    }

    fn test_router(config: AuthConfig) -> Router {
        Router::new()
            .route("/api/test", get(test_handler))
            .route("/health", get(test_handler))
            .layer(middleware::from_fn(auth_middleware))
            .layer(axum::Extension(config))
    }

    #[tokio::test]
    async fn test_auth_disabled() {
        let config = AuthConfig::default();
        let router = test_router(config);

        let request = Request::builder()
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_valid_bearer_token() {
        let config = AuthConfig::with_api_keys(vec!["test-key-123".to_string()]);
        let router = test_router(config);

        let request = Request::builder()
            .uri("/api/test")
            .header("Authorization", "Bearer test-key-123")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_valid_x_api_key() {
        let config = AuthConfig::with_api_keys(vec!["test-key-456".to_string()]);
        let router = test_router(config);

        let request = Request::builder()
            .uri("/api/test")
            .header("X-API-Key", "test-key-456")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_invalid_api_key() {
        let config = AuthConfig::with_api_keys(vec!["valid-key".to_string()]);
        let router = test_router(config);

        let request = Request::builder()
            .uri("/api/test")
            .header("Authorization", "Bearer wrong-key")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_missing_api_key() {
        let config = AuthConfig::with_api_keys(vec!["valid-key".to_string()]);
        let router = test_router(config);

        let request = Request::builder()
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_exempt_path() {
        let config = AuthConfig::with_api_keys(vec!["valid-key".to_string()]);
        let router = test_router(config);

        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
