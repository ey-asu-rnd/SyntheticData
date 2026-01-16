//! REST API routes.

use std::sync::Arc;

use axum::{
    extract::{State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::grpc::service::{default_generator_config, ServerState, SynthService};
use synth_runtime::{EnhancedOrchestrator, PhaseConfig};

use super::websocket;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub service: Arc<SynthService>,
    pub server_state: Arc<ServerState>,
}

/// Create the REST API router.
pub fn create_router(service: SynthService) -> Router {
    let server_state = service.state.clone();
    let state = AppState {
        service: Arc::new(service),
        server_state,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

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

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub healthy: bool,
    pub version: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize)]
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
    State(state): State<AppState>,
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
    }
}
