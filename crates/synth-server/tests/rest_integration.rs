//! REST API integration tests.
//!
//! Tests the REST endpoints using axum's built-in test utilities.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

use synth_server::grpc::service::{default_generator_config, SynthService};
use synth_server::rest::{create_router, create_router_with_cors, CorsConfig};

/// Helper to create test router.
fn test_router() -> axum::Router {
    let config = default_generator_config();
    let service = SynthService::new(config);
    create_router(service)
}

/// Helper to create test router with custom CORS.
fn test_router_with_cors(cors_config: CorsConfig) -> axum::Router {
    let config = default_generator_config();
    let service = SynthService::new(config);
    create_router_with_cors(service, cors_config)
}

/// Helper to send a request and get the response body as JSON.
async fn json_response(router: axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    (status, json)
}

// ==========================================================================
// Health and Probe Endpoint Tests
// ==========================================================================

#[tokio::test]
async fn test_liveness_probe() {
    let router = test_router();
    let request = Request::builder().uri("/live").body(Body::empty()).unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["alive"], true);
    assert!(json["timestamp"].is_string());
}

#[tokio::test]
async fn test_readiness_probe() {
    let router = test_router();
    let request = Request::builder()
        .uri("/ready")
        .body(Body::empty())
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["ready"], true);
    assert!(json["checks"].is_array());
}

#[tokio::test]
async fn test_health_endpoint() {
    let router = test_router();
    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["healthy"], true);
    assert!(json["version"].is_string());
    assert!(json["uptime_seconds"].is_number());
}

#[tokio::test]
async fn test_health_endpoint_returns_version() {
    let router = test_router();
    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(!json["version"].as_str().unwrap().is_empty());
}

// ==========================================================================
// Metrics Endpoint Tests
// ==========================================================================

#[tokio::test]
async fn test_prometheus_metrics_endpoint() {
    let router = test_router();
    let request = Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(status, StatusCode::OK);
    // Check for Prometheus format markers
    assert!(text.contains("# HELP synth_entries_generated_total"));
    assert!(text.contains("# TYPE synth_entries_generated_total counter"));
    assert!(text.contains("synth_uptime_seconds"));
    assert!(text.contains("synth_info{version="));
}

#[tokio::test]
async fn test_prometheus_metrics_content_type() {
    let router = test_router();
    let request = Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.contains("text/plain"));
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let router = test_router();
    let request = Request::builder()
        .uri("/api/metrics")
        .body(Body::empty())
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(json["total_entries_generated"].is_number());
    assert!(json["uptime_seconds"].is_number());
    assert!(json["active_streams"].is_number());
}

#[tokio::test]
async fn test_metrics_initial_values() {
    let router = test_router();
    let request = Request::builder()
        .uri("/api/metrics")
        .body(Body::empty())
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["total_entries_generated"], 0);
    assert_eq!(json["total_anomalies_injected"], 0);
    assert_eq!(json["active_streams"], 0);
}

// ==========================================================================
// Config Endpoint Tests
// ==========================================================================

#[tokio::test]
async fn test_get_config_endpoint() {
    let router = test_router();
    let request = Request::builder()
        .uri("/api/config")
        .body(Body::empty())
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert!(json["config"].is_object());
}

#[tokio::test]
async fn test_get_config_returns_industry() {
    let router = test_router();
    let request = Request::builder()
        .uri("/api/config")
        .body(Body::empty())
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(json["config"]["industry"].is_string());
    assert!(json["config"]["period_months"].is_number());
}

#[tokio::test]
async fn test_set_config_endpoint() {
    let router = test_router();
    let config_json = serde_json::json!({
        "industry": "retail",
        "start_date": "2024-01-01",
        "period_months": 6,
        "seed": 42,
        "coa_complexity": "medium",
        "companies": [],
        "fraud_enabled": false,
        "fraud_rate": 0.0
    });

    let request = Request::builder()
        .uri("/api/config")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(config_json.to_string()))
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    // Verify the message indicates config was applied
    assert!(json["message"].as_str().unwrap().contains("applied"));
}

#[tokio::test]
async fn test_set_config_invalid_industry() {
    let router = test_router();
    let config_json = serde_json::json!({
        "industry": "invalid_industry",
        "start_date": "2024-01-01",
        "period_months": 6,
        "seed": 42,
        "coa_complexity": "medium",
        "companies": [],
        "fraud_enabled": false,
        "fraud_rate": 0.0
    });

    let request = Request::builder()
        .uri("/api/config")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(config_json.to_string()))
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["success"], false);
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("Unknown industry"));
}

#[tokio::test]
async fn test_set_config_invalid_complexity() {
    let router = test_router();
    let config_json = serde_json::json!({
        "industry": "retail",
        "start_date": "2024-01-01",
        "period_months": 6,
        "seed": 42,
        "coa_complexity": "invalid_complexity",
        "companies": [],
        "fraud_enabled": false,
        "fraud_rate": 0.0
    });

    let request = Request::builder()
        .uri("/api/config")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(config_json.to_string()))
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["success"], false);
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("Unknown CoA complexity"));
}

// ==========================================================================
// Bulk Generate Endpoint Tests
// ==========================================================================

#[tokio::test]
async fn test_bulk_generate_endpoint() {
    let router = test_router();
    let request_json = serde_json::json!({
        "entry_count": 100,
        "include_master_data": false,
        "inject_anomalies": false
    });

    let request = Request::builder()
        .uri("/api/generate/bulk")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(request_json.to_string()))
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert!(json["entries_generated"].as_u64().unwrap() > 0);
    assert!(json["duration_ms"].is_number());
}

#[tokio::test]
async fn test_bulk_generate_with_anomalies() {
    let router = test_router();
    let request_json = serde_json::json!({
        "entry_count": 100,
        "include_master_data": false,
        "inject_anomalies": true
    });

    let request = Request::builder()
        .uri("/api/generate/bulk")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(request_json.to_string()))
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    // anomaly_count may be 0 due to probabilistic injection
    assert!(json["anomaly_count"].is_number());
}

#[tokio::test]
async fn test_bulk_generate_entry_count_too_high() {
    let router = test_router();
    let request_json = serde_json::json!({
        "entry_count": 2000000,  // Exceeds limit
        "include_master_data": false,
        "inject_anomalies": false
    });

    let request = Request::builder()
        .uri("/api/generate/bulk")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(request_json.to_string()))
        .unwrap();

    let (status, _json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_bulk_generate_empty_request() {
    let router = test_router();
    let request_json = serde_json::json!({});

    let request = Request::builder()
        .uri("/api/generate/bulk")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(request_json.to_string()))
        .unwrap();

    let (status, json) = json_response(router, request).await;

    // Should succeed with defaults
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
}

// ==========================================================================
// Stream Control Endpoint Tests
// ==========================================================================

#[tokio::test]
async fn test_stream_start_endpoint() {
    let router = test_router();
    let request_json = serde_json::json!({
        "events_per_second": 10,
        "max_events": 100
    });

    let request = Request::builder()
        .uri("/api/stream/start")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(request_json.to_string()))
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert!(json["message"].as_str().unwrap().contains("started"));
}

#[tokio::test]
async fn test_stream_stop_endpoint() {
    let router = test_router();
    let request = Request::builder()
        .uri("/api/stream/stop")
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert!(json["message"].as_str().unwrap().contains("stopped"));
}

#[tokio::test]
async fn test_stream_pause_endpoint() {
    let router = test_router();
    let request = Request::builder()
        .uri("/api/stream/pause")
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert!(json["message"].as_str().unwrap().contains("paused"));
}

#[tokio::test]
async fn test_stream_resume_endpoint() {
    let router = test_router();
    let request = Request::builder()
        .uri("/api/stream/resume")
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert!(json["message"].as_str().unwrap().contains("resumed"));
}

#[tokio::test]
async fn test_stream_control_lifecycle() {
    let config = default_generator_config();
    let service = SynthService::new(config);
    let router = create_router(service);

    // Start
    let request = Request::builder()
        .uri("/api/stream/start")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from("{}"))
        .unwrap();
    let (status, _) = json_response(router.clone(), request).await;
    assert_eq!(status, StatusCode::OK);

    // Pause
    let request = Request::builder()
        .uri("/api/stream/pause")
        .method("POST")
        .body(Body::empty())
        .unwrap();
    let (status, _) = json_response(router.clone(), request).await;
    assert_eq!(status, StatusCode::OK);

    // Resume
    let request = Request::builder()
        .uri("/api/stream/resume")
        .method("POST")
        .body(Body::empty())
        .unwrap();
    let (status, _) = json_response(router.clone(), request).await;
    assert_eq!(status, StatusCode::OK);

    // Stop
    let request = Request::builder()
        .uri("/api/stream/stop")
        .method("POST")
        .body(Body::empty())
        .unwrap();
    let (status, _) = json_response(router, request).await;
    assert_eq!(status, StatusCode::OK);
}

// ==========================================================================
// Pattern Trigger Endpoint Tests
// ==========================================================================

#[tokio::test]
async fn test_trigger_pattern_endpoint() {
    let router = test_router();
    let request = Request::builder()
        .uri("/api/stream/trigger/year_end_spike")
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("will be applied"));
}

#[tokio::test]
async fn test_trigger_pattern_invalid() {
    let router = test_router();
    let request = Request::builder()
        .uri("/api/stream/trigger/invalid_pattern")
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], false);
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("Unknown pattern"));
}

#[tokio::test]
async fn test_trigger_pattern_custom() {
    let router = test_router();
    let request = Request::builder()
        .uri("/api/stream/trigger/custom:my_pattern")
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let (status, json) = json_response(router, request).await;

    assert_eq!(status, StatusCode::OK);
    // Custom patterns starting with "custom:" are allowed
    assert_eq!(json["success"], true);
}

// ==========================================================================
// CORS Tests
// ==========================================================================

#[tokio::test]
async fn test_cors_default_origins() {
    let cors_config = CorsConfig::default();
    let router = test_router_with_cors(cors_config);

    let request = Request::builder()
        .uri("/health")
        .method("OPTIONS")
        .header("origin", "http://localhost:5173")
        .header("access-control-request-method", "GET")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    // CORS preflight should return 200
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_cors_permissive_mode() {
    let cors_config = CorsConfig {
        allowed_origins: vec![],
        allow_any_origin: true,
    };
    let router = test_router_with_cors(cors_config);

    let request = Request::builder()
        .uri("/health")
        .method("OPTIONS")
        .header("origin", "https://any-domain.com")
        .header("access-control-request-method", "GET")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    // Permissive CORS should allow any origin
    assert_eq!(response.status(), StatusCode::OK);
}

// ==========================================================================
// Error Handling Tests
// ==========================================================================

#[tokio::test]
async fn test_invalid_json_request() {
    let router = test_router();
    let request = Request::builder()
        .uri("/api/generate/bulk")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from("not valid json"))
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    // Should return 4xx error for invalid JSON
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn test_not_found_endpoint() {
    let router = test_router();
    let request = Request::builder()
        .uri("/api/nonexistent")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_method_not_allowed() {
    let router = test_router();
    let request = Request::builder()
        .uri("/health")
        .method("DELETE")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}
