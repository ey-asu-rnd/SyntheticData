//! Tauri application for synthetic data generator UI.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Application state shared across Tauri commands.
pub struct AppState {
    pub server_url: RwLock<String>,
    pub client: reqwest::Client,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            server_url: RwLock::new("http://localhost:3000".to_string()),
            client: reqwest::Client::new(),
        }
    }
}

/// Health check response from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub healthy: bool,
    pub version: String,
    pub uptime_seconds: u64,
}

/// Metrics response from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub total_entries_generated: u64,
    pub total_anomalies_injected: u64,
    pub uptime_seconds: u64,
    pub session_entries: u64,
    pub session_entries_per_second: f64,
    pub active_streams: u32,
    pub total_stream_events: u64,
}

/// Configuration DTO.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub success: bool,
    pub message: String,
    pub config: Option<GenerationConfigDto>,
}

/// Generation configuration.
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

/// Company configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyConfigDto {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub country: String,
    pub annual_transaction_volume: u64,
    pub volume_weight: f32,
}

/// Bulk generation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkGenerateRequest {
    pub entry_count: Option<u64>,
    pub include_master_data: Option<bool>,
    pub inject_anomalies: Option<bool>,
}

/// Bulk generation response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkGenerateResponse {
    pub success: bool,
    pub entries_generated: u64,
    pub duration_ms: u64,
    pub anomaly_count: u64,
}

/// Stream control response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamResponse {
    pub success: bool,
    pub message: String,
}

// ===========================================================================
// Tauri Commands
// ===========================================================================

/// Set the server URL.
#[tauri::command]
async fn set_server_url(state: State<'_, Arc<AppState>>, url: String) -> Result<(), String> {
    let mut server_url = state.server_url.write().await;
    *server_url = url;
    Ok(())
}

/// Get the current server URL.
#[tauri::command]
async fn get_server_url(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let url = state.server_url.read().await.clone();
    Ok(url)
}

/// Check server health.
#[tauri::command]
async fn check_health(state: State<'_, Arc<AppState>>) -> Result<HealthResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .get(format!("{}/health", url))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    response
        .json::<HealthResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Get server metrics.
#[tauri::command]
async fn get_metrics(state: State<'_, Arc<AppState>>) -> Result<MetricsResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .get(format!("{}/api/metrics", url))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    response
        .json::<MetricsResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Get current configuration.
#[tauri::command]
async fn get_config(state: State<'_, Arc<AppState>>) -> Result<ConfigResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .get(format!("{}/api/config", url))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    response
        .json::<ConfigResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Update configuration.
#[tauri::command]
async fn set_config(
    state: State<'_, Arc<AppState>>,
    config: GenerationConfigDto,
) -> Result<ConfigResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .post(format!("{}/api/config", url))
        .json(&config)
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    response
        .json::<ConfigResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Start bulk generation.
#[tauri::command]
async fn bulk_generate(
    state: State<'_, Arc<AppState>>,
    request: BulkGenerateRequest,
) -> Result<BulkGenerateResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .post(format!("{}/api/generate/bulk", url))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    response
        .json::<BulkGenerateResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Start streaming.
#[tauri::command]
async fn start_stream(state: State<'_, Arc<AppState>>) -> Result<StreamResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .post(format!("{}/api/stream/start", url))
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    response
        .json::<StreamResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Stop streaming.
#[tauri::command]
async fn stop_stream(state: State<'_, Arc<AppState>>) -> Result<StreamResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .post(format!("{}/api/stream/stop", url))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    response
        .json::<StreamResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Pause streaming.
#[tauri::command]
async fn pause_stream(state: State<'_, Arc<AppState>>) -> Result<StreamResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .post(format!("{}/api/stream/pause", url))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    response
        .json::<StreamResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Resume streaming.
#[tauri::command]
async fn resume_stream(state: State<'_, Arc<AppState>>) -> Result<StreamResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .post(format!("{}/api/stream/resume", url))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    response
        .json::<StreamResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Trigger a specific pattern.
#[tauri::command]
async fn trigger_pattern(
    state: State<'_, Arc<AppState>>,
    pattern: String,
) -> Result<StreamResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .post(format!("{}/api/stream/trigger/{}", url, pattern))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    response
        .json::<StreamResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

fn main() {
    let app_state = Arc::new(AppState::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            set_server_url,
            get_server_url,
            check_health,
            get_metrics,
            get_config,
            set_config,
            bulk_generate,
            start_stream,
            stop_stream,
            pause_stream,
            resume_stream,
            trigger_pattern,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
