//! REST API routes.

use std::sync::Arc;

use axum::{
    extract::{State, WebSocketUpgrade},
    http::{header, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::info;

use crate::grpc::service::{ServerState, SynthService};
use synth_runtime::{EnhancedOrchestrator, PhaseConfig};

use super::websocket;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub service: Arc<SynthService>,
    pub server_state: Arc<ServerState>,
}

/// CORS configuration for the REST API.
#[derive(Clone)]
pub struct CorsConfig {
    /// Allowed origins. If empty, only localhost is allowed.
    pub allowed_origins: Vec<String>,
    /// Allow any origin (development mode only - NOT recommended for production).
    pub allow_any_origin: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec![
                "http://localhost:5173".to_string(),  // Vite dev server
                "http://localhost:3000".to_string(),  // Common dev server
                "http://127.0.0.1:5173".to_string(),
                "http://127.0.0.1:3000".to_string(),
                "tauri://localhost".to_string(),      // Tauri app
            ],
            allow_any_origin: false,
        }
    }
}

/// Create the REST API router with default CORS settings.
pub fn create_router(service: SynthService) -> Router {
    create_router_with_cors(service, CorsConfig::default())
}

/// Create the REST API router with custom CORS settings.
pub fn create_router_with_cors(service: SynthService, cors_config: CorsConfig) -> Router {
    let server_state = service.state.clone();
    let state = AppState {
        service: Arc::new(service),
        server_state,
    };

    let cors = if cors_config.allow_any_origin {
        // Development mode - allow any origin (use with caution)
        CorsLayer::permissive()
    } else {
        // Production mode - restricted origins
        let origins: Vec<_> = cors_config
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();

        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
    };

    Router::new()
        // Health and metrics
        .route("/health", get(health_check))
        .route("/api/metrics", get(get_metrics))
        // Configuration
        .route("/api/config", get(get_config))
        .route("/api/config", post(set_config))
        // Generation
        .route("/api/generate/bulk", post(bulk_generate))
        .route("/api/stream/start", post(start_stream))
        .route("/api/stream/stop", post(stop_stream))
        .route("/api/stream/pause", post(pause_stream))
        .route("/api/stream/resume", post(resume_stream))
        .route("/api/stream/trigger/:pattern", post(trigger_pattern))
        // WebSocket
        .route("/ws/metrics", get(websocket_metrics))
        .route("/ws/events", get(websocket_events))
        .layer(cors)
        .with_state(state)
}

// ===========================================================================
// Request/Response types
// ===========================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub healthy: bool,
    pub version: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub total_entries_generated: u64,
    pub total_anomalies_injected: u64,
    pub uptime_seconds: u64,
    pub session_entries: u64,
    pub session_entries_per_second: f64,
    pub active_streams: u32,
    pub total_stream_events: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub success: bool,
    pub message: String,
    pub config: Option<GenerationConfigDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfigDto {
    pub industry: String,
    pub start_date: String,
    pub period_months: u32,
    pub seed: Option<u64>,
    pub coa_complexity: String,
    pub companies: Vec<CompanyConfigDto>,
    pub fraud_enabled: bool,
    pub fraud_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyConfigDto {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub country: String,
    pub annual_transaction_volume: u64,
    pub volume_weight: f32,
}

#[derive(Debug, Deserialize)]
pub struct BulkGenerateRequest {
    pub entry_count: Option<u64>,
    pub include_master_data: Option<bool>,
    pub inject_anomalies: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct BulkGenerateResponse {
    pub success: bool,
    pub entries_generated: u64,
    pub duration_ms: u64,
    pub anomaly_count: u64,
}

#[derive(Debug, Deserialize)]
pub struct StreamRequest {
    pub events_per_second: Option<u32>,
    pub max_events: Option<u64>,
    pub inject_anomalies: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct StreamResponse {
    pub success: bool,
    pub message: String,
}

// ===========================================================================
// Handlers
// ===========================================================================

/// Health check endpoint.
async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        healthy: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.server_state.uptime_seconds(),
    })
}

/// Get server metrics.
async fn get_metrics(State(state): State<AppState>) -> Json<MetricsResponse> {
    let uptime = state.server_state.uptime_seconds();
    let total_entries = state.server_state.total_entries.load(std::sync::atomic::Ordering::Relaxed);

    let entries_per_second = if uptime > 0 {
        total_entries as f64 / uptime as f64
    } else {
        0.0
    };

    Json(MetricsResponse {
        total_entries_generated: total_entries,
        total_anomalies_injected: state.server_state.total_anomalies.load(std::sync::atomic::Ordering::Relaxed),
        uptime_seconds: uptime,
        session_entries: total_entries,
        session_entries_per_second: entries_per_second,
        active_streams: state.server_state.active_streams.load(std::sync::atomic::Ordering::Relaxed) as u32,
        total_stream_events: state.server_state.total_stream_events.load(std::sync::atomic::Ordering::Relaxed),
    })
}

/// Get current configuration.
async fn get_config(State(state): State<AppState>) -> Json<ConfigResponse> {
    let config = state.server_state.config.read().await;

    Json(ConfigResponse {
        success: true,
        message: "Current configuration".to_string(),
        config: Some(GenerationConfigDto {
            industry: format!("{:?}", config.global.industry),
            start_date: config.global.start_date.clone(),
            period_months: config.global.period_months,
            seed: config.global.seed,
            coa_complexity: format!("{:?}", config.chart_of_accounts.complexity),
            companies: config
                .companies
                .iter()
                .map(|c| CompanyConfigDto {
                    code: c.code.clone(),
                    name: c.name.clone(),
                    currency: c.currency.clone(),
                    country: c.country.clone(),
                    annual_transaction_volume: c.annual_transaction_volume.count(),
                    volume_weight: c.volume_weight as f32,
                })
                .collect(),
            fraud_enabled: config.fraud.enabled,
            fraud_rate: config.fraud.fraud_rate as f32,
        }),
    })
}

/// Set configuration.
async fn set_config(
    State(_state): State<AppState>,
    Json(new_config): Json<GenerationConfigDto>,
) -> Json<ConfigResponse> {
    // For now, we just acknowledge the config update
    // Full implementation would convert DTO back to GeneratorConfig
    info!("Configuration update requested");

    Json(ConfigResponse {
        success: true,
        message: "Configuration updated".to_string(),
        config: Some(new_config),
    })
}

/// Bulk generation endpoint.
async fn bulk_generate(
    State(state): State<AppState>,
    Json(req): Json<BulkGenerateRequest>,
) -> Result<Json<BulkGenerateResponse>, (StatusCode, String)> {
    // Validate entry_count bounds
    const MAX_ENTRY_COUNT: u64 = 1_000_000;
    if let Some(count) = req.entry_count {
        if count > MAX_ENTRY_COUNT {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "entry_count ({}) exceeds maximum allowed value ({})",
                    count, MAX_ENTRY_COUNT
                ),
            ));
        }
    }

    let config = state.server_state.config.read().await.clone();
    let start_time = std::time::Instant::now();

    let phase_config = PhaseConfig {
        generate_master_data: req.include_master_data.unwrap_or(false),
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: req.inject_anomalies.unwrap_or(false),
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator = EnhancedOrchestrator::new(config, phase_config)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create orchestrator: {}", e)))?;

    let result = orchestrator
        .generate()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Generation failed: {}", e)))?;

    let duration_ms = start_time.elapsed().as_millis() as u64;
    let entries_count = result.journal_entries.len() as u64;
    let anomaly_count = result.anomaly_labels.labels.len() as u64;

    // Update metrics
    state.server_state.total_entries.fetch_add(entries_count, std::sync::atomic::Ordering::Relaxed);
    state.server_state.total_anomalies.fetch_add(anomaly_count, std::sync::atomic::Ordering::Relaxed);

    Ok(Json(BulkGenerateResponse {
        success: true,
        entries_generated: entries_count,
        duration_ms,
        anomaly_count,
    }))
}

/// Start streaming.
async fn start_stream(
    State(state): State<AppState>,
    Json(_req): Json<StreamRequest>,
) -> Json<StreamResponse> {
    state.server_state.stream_stopped.store(false, std::sync::atomic::Ordering::Relaxed);
    state.server_state.stream_paused.store(false, std::sync::atomic::Ordering::Relaxed);

    Json(StreamResponse {
        success: true,
        message: "Stream started".to_string(),
    })
}

/// Stop streaming.
async fn stop_stream(State(state): State<AppState>) -> Json<StreamResponse> {
    state.server_state.stream_stopped.store(true, std::sync::atomic::Ordering::Relaxed);

    Json(StreamResponse {
        success: true,
        message: "Stream stopped".to_string(),
    })
}

/// Pause streaming.
async fn pause_stream(State(state): State<AppState>) -> Json<StreamResponse> {
    state.server_state.stream_paused.store(true, std::sync::atomic::Ordering::Relaxed);

    Json(StreamResponse {
        success: true,
        message: "Stream paused".to_string(),
    })
}

/// Resume streaming.
async fn resume_stream(State(state): State<AppState>) -> Json<StreamResponse> {
    state.server_state.stream_paused.store(false, std::sync::atomic::Ordering::Relaxed);

    Json(StreamResponse {
        success: true,
        message: "Stream resumed".to_string(),
    })
}

/// Trigger a specific pattern.
async fn trigger_pattern(
    State(_state): State<AppState>,
    axum::extract::Path(pattern): axum::extract::Path<String>,
) -> Json<StreamResponse> {
    info!("Pattern trigger requested: {}", pattern);

    Json(StreamResponse {
        success: false,
        message: format!("Pattern '{}' trigger not yet implemented", pattern),
    })
}

/// WebSocket endpoint for metrics stream.
async fn websocket_metrics(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket::handle_metrics_socket(socket, state))
}

/// WebSocket endpoint for event stream.
async fn websocket_events(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket::handle_events_socket(socket, state))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // Response Serialization Tests
    // ==========================================================================

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            healthy: true,
            version: "0.1.0".to_string(),
            uptime_seconds: 100,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("version"));
        assert!(json.contains("uptime_seconds"));
    }

    #[test]
    fn test_health_response_deserialization() {
        let json = r#"{"healthy":true,"version":"0.1.0","uptime_seconds":100}"#;
        let response: HealthResponse = serde_json::from_str(json).unwrap();
        assert!(response.healthy);
        assert_eq!(response.version, "0.1.0");
        assert_eq!(response.uptime_seconds, 100);
    }

    #[test]
    fn test_metrics_response_serialization() {
        let response = MetricsResponse {
            total_entries_generated: 1000,
            total_anomalies_injected: 10,
            uptime_seconds: 60,
            session_entries: 1000,
            session_entries_per_second: 16.67,
            active_streams: 1,
            total_stream_events: 500,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("total_entries_generated"));
        assert!(json.contains("session_entries_per_second"));
    }

    #[test]
    fn test_metrics_response_deserialization() {
        let json = r#"{
            "total_entries_generated": 5000,
            "total_anomalies_injected": 50,
            "uptime_seconds": 300,
            "session_entries": 5000,
            "session_entries_per_second": 16.67,
            "active_streams": 2,
            "total_stream_events": 10000
        }"#;
        let response: MetricsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total_entries_generated, 5000);
        assert_eq!(response.active_streams, 2);
    }

    #[test]
    fn test_config_response_serialization() {
        let response = ConfigResponse {
            success: true,
            message: "Configuration loaded".to_string(),
            config: Some(GenerationConfigDto {
                industry: "manufacturing".to_string(),
                start_date: "2024-01-01".to_string(),
                period_months: 12,
                seed: Some(42),
                coa_complexity: "medium".to_string(),
                companies: vec![],
                fraud_enabled: false,
                fraud_rate: 0.0,
            }),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("config"));
    }

    #[test]
    fn test_config_response_without_config() {
        let response = ConfigResponse {
            success: false,
            message: "No configuration available".to_string(),
            config: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("null") || json.contains("config\":null"));
    }

    #[test]
    fn test_generation_config_dto_roundtrip() {
        let original = GenerationConfigDto {
            industry: "retail".to_string(),
            start_date: "2024-06-01".to_string(),
            period_months: 6,
            seed: Some(12345),
            coa_complexity: "large".to_string(),
            companies: vec![CompanyConfigDto {
                code: "1000".to_string(),
                name: "Test Corp".to_string(),
                currency: "USD".to_string(),
                country: "US".to_string(),
                annual_transaction_volume: 100000,
                volume_weight: 1.0,
            }],
            fraud_enabled: true,
            fraud_rate: 0.05,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: GenerationConfigDto = serde_json::from_str(&json).unwrap();

        assert_eq!(original.industry, deserialized.industry);
        assert_eq!(original.seed, deserialized.seed);
        assert_eq!(original.companies.len(), deserialized.companies.len());
    }

    #[test]
    fn test_company_config_dto_serialization() {
        let company = CompanyConfigDto {
            code: "2000".to_string(),
            name: "European Subsidiary".to_string(),
            currency: "EUR".to_string(),
            country: "DE".to_string(),
            annual_transaction_volume: 50000,
            volume_weight: 0.5,
        };
        let json = serde_json::to_string(&company).unwrap();
        assert!(json.contains("2000"));
        assert!(json.contains("EUR"));
        assert!(json.contains("DE"));
    }

    #[test]
    fn test_bulk_generate_request_deserialization() {
        let json = r#"{
            "entry_count": 5000,
            "include_master_data": true,
            "inject_anomalies": true
        }"#;
        let request: BulkGenerateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.entry_count, Some(5000));
        assert_eq!(request.include_master_data, Some(true));
        assert_eq!(request.inject_anomalies, Some(true));
    }

    #[test]
    fn test_bulk_generate_request_with_defaults() {
        let json = r#"{}"#;
        let request: BulkGenerateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.entry_count, None);
        assert_eq!(request.include_master_data, None);
        assert_eq!(request.inject_anomalies, None);
    }

    #[test]
    fn test_bulk_generate_response_serialization() {
        let response = BulkGenerateResponse {
            success: true,
            entries_generated: 1000,
            duration_ms: 250,
            anomaly_count: 20,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("entries_generated"));
        assert!(json.contains("1000"));
        assert!(json.contains("duration_ms"));
    }

    #[test]
    fn test_stream_response_serialization() {
        let response = StreamResponse {
            success: true,
            message: "Stream started successfully".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("Stream started"));
    }

    #[test]
    fn test_stream_response_failure() {
        let response = StreamResponse {
            success: false,
            message: "Stream failed to start".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("false"));
        assert!(json.contains("failed"));
    }

    // ==========================================================================
    // CORS Configuration Tests
    // ==========================================================================

    #[test]
    fn test_cors_config_default() {
        let config = CorsConfig::default();
        assert!(!config.allow_any_origin);
        assert!(!config.allowed_origins.is_empty());
        assert!(config.allowed_origins.contains(&"http://localhost:5173".to_string()));
        assert!(config.allowed_origins.contains(&"tauri://localhost".to_string()));
    }

    #[test]
    fn test_cors_config_custom_origins() {
        let config = CorsConfig {
            allowed_origins: vec![
                "https://example.com".to_string(),
                "https://app.example.com".to_string(),
            ],
            allow_any_origin: false,
        };
        assert_eq!(config.allowed_origins.len(), 2);
        assert!(config.allowed_origins.contains(&"https://example.com".to_string()));
    }

    #[test]
    fn test_cors_config_permissive() {
        let config = CorsConfig {
            allowed_origins: vec![],
            allow_any_origin: true,
        };
        assert!(config.allow_any_origin);
    }

    // ==========================================================================
    // Request Validation Tests (edge cases)
    // ==========================================================================

    #[test]
    fn test_bulk_generate_request_partial() {
        let json = r#"{"entry_count": 100}"#;
        let request: BulkGenerateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.entry_count, Some(100));
        assert!(request.include_master_data.is_none());
    }

    #[test]
    fn test_generation_config_no_seed() {
        let config = GenerationConfigDto {
            industry: "technology".to_string(),
            start_date: "2024-01-01".to_string(),
            period_months: 3,
            seed: None,
            coa_complexity: "small".to_string(),
            companies: vec![],
            fraud_enabled: false,
            fraud_rate: 0.0,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("seed"));
    }

    #[test]
    fn test_generation_config_multiple_companies() {
        let config = GenerationConfigDto {
            industry: "manufacturing".to_string(),
            start_date: "2024-01-01".to_string(),
            period_months: 12,
            seed: Some(42),
            coa_complexity: "large".to_string(),
            companies: vec![
                CompanyConfigDto {
                    code: "1000".to_string(),
                    name: "Headquarters".to_string(),
                    currency: "USD".to_string(),
                    country: "US".to_string(),
                    annual_transaction_volume: 100000,
                    volume_weight: 1.0,
                },
                CompanyConfigDto {
                    code: "2000".to_string(),
                    name: "European Sub".to_string(),
                    currency: "EUR".to_string(),
                    country: "DE".to_string(),
                    annual_transaction_volume: 50000,
                    volume_weight: 0.5,
                },
                CompanyConfigDto {
                    code: "3000".to_string(),
                    name: "APAC Sub".to_string(),
                    currency: "JPY".to_string(),
                    country: "JP".to_string(),
                    annual_transaction_volume: 30000,
                    volume_weight: 0.3,
                },
            ],
            fraud_enabled: true,
            fraud_rate: 0.02,
        };
        assert_eq!(config.companies.len(), 3);
    }

    // ==========================================================================
    // Metrics Calculation Tests
    // ==========================================================================

    #[test]
    fn test_metrics_entries_per_second_calculation() {
        // Test that we can represent the expected calculation
        let total_entries: u64 = 1000;
        let uptime: u64 = 60;
        let eps = if uptime > 0 {
            total_entries as f64 / uptime as f64
        } else {
            0.0
        };
        assert!((eps - 16.67).abs() < 0.1);
    }

    #[test]
    fn test_metrics_entries_per_second_zero_uptime() {
        let total_entries: u64 = 1000;
        let uptime: u64 = 0;
        let eps = if uptime > 0 {
            total_entries as f64 / uptime as f64
        } else {
            0.0
        };
        assert_eq!(eps, 0.0);
    }
}
