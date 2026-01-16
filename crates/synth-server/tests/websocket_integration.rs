//! WebSocket integration tests.
//!
//! Tests the WebSocket endpoints for metrics and events streaming.
//!
//! Note: Full WebSocket upgrade testing requires an actual server with a WebSocket client.
//! These tests verify the endpoint exists and handles basic HTTP behavior correctly.

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use tower::ServiceExt;

use synth_server::grpc::service::{default_generator_config, SynthService};
use synth_server::rest::create_router;

/// Helper to create test router.
fn test_router() -> axum::Router {
    let config = default_generator_config();
    let service = SynthService::new(config);
    create_router(service)
}

// ==========================================================================
// WebSocket Endpoint Existence Tests
// ==========================================================================

#[tokio::test]
async fn test_metrics_websocket_endpoint_exists() {
    let router = test_router();

    // Send a regular GET request (not an upgrade)
    // Should fail since it's a WebSocket-only endpoint
    let request = Request::builder()
        .uri("/ws/metrics")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    // Without proper WebSocket upgrade headers, should return an error
    // This confirms the endpoint exists but requires WebSocket upgrade
    assert!(
        response.status().is_client_error(),
        "Non-upgrade request to WebSocket endpoint should fail"
    );
}

#[tokio::test]
async fn test_events_websocket_endpoint_exists() {
    let router = test_router();

    // Send a regular GET request (not an upgrade)
    let request = Request::builder()
        .uri("/ws/events")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    // Without proper WebSocket upgrade headers, should return an error
    assert!(
        response.status().is_client_error(),
        "Non-upgrade request to WebSocket endpoint should fail"
    );
}

// ==========================================================================
// WebSocket Upgrade Header Validation
// ==========================================================================

#[tokio::test]
async fn test_metrics_websocket_requires_upgrade_header() {
    let router = test_router();

    // Request with connection header but wrong value
    let request = Request::builder()
        .uri("/ws/metrics")
        .method("GET")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    // Should reject non-upgrade connections
    assert!(
        response.status().is_client_error(),
        "Should require upgrade header"
    );
}

#[tokio::test]
async fn test_events_websocket_requires_upgrade_header() {
    let router = test_router();

    // Request with connection header but wrong value
    let request = Request::builder()
        .uri("/ws/events")
        .method("GET")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    // Should reject non-upgrade connections
    assert!(
        response.status().is_client_error(),
        "Should require upgrade header"
    );
}

#[tokio::test]
async fn test_websocket_partial_upgrade_headers_rejected() {
    let router = test_router();

    // Request with only Connection: Upgrade but no Sec-WebSocket-* headers
    let request = Request::builder()
        .uri("/ws/metrics")
        .method("GET")
        .header(header::CONNECTION, "upgrade")
        .header(header::UPGRADE, "websocket")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    // Should reject without full WebSocket headers
    assert!(
        response.status().is_client_error(),
        "Should require complete WebSocket headers"
    );
}

// ==========================================================================
// HTTP Method Tests
// ==========================================================================

#[tokio::test]
async fn test_metrics_websocket_rejects_post() {
    let router = test_router();

    let request = Request::builder()
        .uri("/ws/metrics")
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    // POST should not be allowed on WebSocket endpoints
    assert_eq!(
        response.status(),
        StatusCode::METHOD_NOT_ALLOWED,
        "POST should not be allowed on WebSocket endpoint"
    );
}

#[tokio::test]
async fn test_events_websocket_rejects_post() {
    let router = test_router();

    let request = Request::builder()
        .uri("/ws/events")
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    // POST should not be allowed on WebSocket endpoints
    assert_eq!(
        response.status(),
        StatusCode::METHOD_NOT_ALLOWED,
        "POST should not be allowed on WebSocket endpoint"
    );
}

#[tokio::test]
async fn test_metrics_websocket_rejects_put() {
    let router = test_router();

    let request = Request::builder()
        .uri("/ws/metrics")
        .method("PUT")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::METHOD_NOT_ALLOWED,
        "PUT should not be allowed on WebSocket endpoint"
    );
}

#[tokio::test]
async fn test_events_websocket_rejects_delete() {
    let router = test_router();

    let request = Request::builder()
        .uri("/ws/events")
        .method("DELETE")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::METHOD_NOT_ALLOWED,
        "DELETE should not be allowed on WebSocket endpoint"
    );
}

// ==========================================================================
// WebSocket Upgrade Request (426 Expected in Test Context)
// Note: In tower's oneshot(), WebSocket upgrade can't complete,
// so we expect 426 Upgrade Required status code.
// ==========================================================================

#[tokio::test]
async fn test_metrics_websocket_upgrade_request_recognized() {
    let router = test_router();

    // Proper WebSocket upgrade request
    let request = Request::builder()
        .uri("/ws/metrics")
        .method("GET")
        .header(header::CONNECTION, "upgrade")
        .header(header::UPGRADE, "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    // In tower's oneshot testing context, WebSocket upgrade returns 426
    // This is expected behavior - actual upgrade requires real server
    assert!(
        response.status() == StatusCode::SWITCHING_PROTOCOLS
            || response.status() == StatusCode::UPGRADE_REQUIRED,
        "WebSocket upgrade request should be recognized (got {})",
        response.status()
    );
}

#[tokio::test]
async fn test_events_websocket_upgrade_request_recognized() {
    let router = test_router();

    // Proper WebSocket upgrade request
    let request = Request::builder()
        .uri("/ws/events")
        .method("GET")
        .header(header::CONNECTION, "upgrade")
        .header(header::UPGRADE, "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    // In tower's oneshot testing context, WebSocket upgrade returns 426
    assert!(
        response.status() == StatusCode::SWITCHING_PROTOCOLS
            || response.status() == StatusCode::UPGRADE_REQUIRED,
        "WebSocket upgrade request should be recognized (got {})",
        response.status()
    );
}

// ==========================================================================
// Wrong WebSocket Version Tests
// ==========================================================================

#[tokio::test]
async fn test_websocket_wrong_version_rejected() {
    let router = test_router();

    // WebSocket version 8 (old version, should be rejected)
    let request = Request::builder()
        .uri("/ws/metrics")
        .method("GET")
        .header(header::CONNECTION, "upgrade")
        .header(header::UPGRADE, "websocket")
        .header("sec-websocket-version", "8")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    // Should reject old WebSocket version
    assert!(
        response.status().is_client_error(),
        "Should reject old WebSocket version"
    );
}
