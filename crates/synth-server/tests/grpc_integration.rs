//! gRPC integration tests.
//!
//! Tests the gRPC service implementation using direct trait method calls.

use tonic::Request;

use synth_server::grpc::service::default_generator_config;
use synth_server::grpc::{
    BulkGenerateRequest, ConfigRequest, ControlAction, ControlCommand, GenerationConfig,
    SyntheticDataService, SynthService,
};

/// Helper to create test service.
fn test_service() -> SynthService {
    let config = default_generator_config();
    SynthService::new(config)
}

// ==========================================================================
// Health Check Tests
// ==========================================================================

#[tokio::test]
async fn test_health_check() {
    let service = test_service();
    let request = Request::new(());

    let response = service.health_check(request).await.unwrap();
    let health = response.into_inner();

    assert!(health.healthy);
    assert!(!health.version.is_empty());
}

#[tokio::test]
async fn test_health_check_returns_uptime() {
    let service = test_service();
    let request = Request::new(());

    let response = service.health_check(request).await.unwrap();
    let health = response.into_inner();

    // Uptime should be 0 or a small number for a fresh service
    assert!(health.uptime_seconds < 10);
}

// ==========================================================================
// Metrics Tests
// ==========================================================================

#[tokio::test]
async fn test_get_metrics() {
    let service = test_service();
    let request = Request::new(());

    let response = service.get_metrics(request).await.unwrap();
    let metrics = response.into_inner();

    assert_eq!(metrics.total_entries_generated, 0);
    assert_eq!(metrics.total_anomalies_injected, 0);
    assert_eq!(metrics.active_streams, 0);
}

#[tokio::test]
async fn test_metrics_track_generation() {
    let service = test_service();

    // Generate some entries first
    let gen_request = Request::new(BulkGenerateRequest {
        config: None,
        entry_count: 50,
        output_format: 0,
        include_master_data: false,
        inject_anomalies: false,
    });
    let _ = service.bulk_generate(gen_request).await.unwrap();

    // Now check metrics
    let request = Request::new(());
    let response = service.get_metrics(request).await.unwrap();
    let metrics = response.into_inner();

    assert!(metrics.total_entries_generated > 0);
}

// ==========================================================================
// Configuration Tests
// ==========================================================================

#[tokio::test]
async fn test_get_config() {
    let service = test_service();
    let request = Request::new(());

    let response = service.get_config(request).await.unwrap();
    let config_response = response.into_inner();

    assert!(config_response.success);
    assert!(config_response.current_config.is_some());
}

#[tokio::test]
async fn test_get_config_returns_industry() {
    let service = test_service();
    let request = Request::new(());

    let response = service.get_config(request).await.unwrap();
    let config = response.into_inner().current_config.unwrap();

    // Default config should have an industry
    assert!(!config.industry.is_empty());
}

#[tokio::test]
async fn test_set_config() {
    let service = test_service();

    let new_config = GenerationConfig {
        industry: "retail".to_string(),
        start_date: "2024-01-01".to_string(),
        period_months: 6,
        seed: 42,
        coa_complexity: "medium".to_string(),
        companies: vec![],
        fraud_enabled: false,
        fraud_rate: 0.0,
        generate_master_data: false,
        generate_document_flows: false,
    };

    let request = Request::new(ConfigRequest {
        config: Some(new_config),
    });

    let response = service.set_config(request).await.unwrap();
    let config_response = response.into_inner();

    assert!(config_response.success);
}

#[tokio::test]
async fn test_set_config_updates_industry() {
    let service = test_service();

    let new_config = GenerationConfig {
        industry: "technology".to_string(),
        start_date: "2024-01-01".to_string(),
        period_months: 12,
        seed: 0,
        coa_complexity: "large".to_string(),
        companies: vec![],
        fraud_enabled: true,
        fraud_rate: 0.05,
        generate_master_data: true,
        generate_document_flows: true,
    };

    let set_request = Request::new(ConfigRequest {
        config: Some(new_config),
    });
    let _ = service.set_config(set_request).await.unwrap();

    // Verify config was updated
    let get_request = Request::new(());
    let response = service.get_config(get_request).await.unwrap();
    let config = response.into_inner().current_config.unwrap();

    assert_eq!(config.industry.to_lowercase(), "technology");
}

// ==========================================================================
// Bulk Generate Tests
// ==========================================================================

#[tokio::test]
async fn test_bulk_generate_returns_entries() {
    let service = test_service();

    let request = Request::new(BulkGenerateRequest {
        config: None,
        entry_count: 100,
        output_format: 0,
        include_master_data: false,
        inject_anomalies: false,
    });

    let response = service.bulk_generate(request).await.unwrap();
    let result = response.into_inner();

    assert!(result.entries_generated > 0);
    assert!(result.duration_ms > 0);
    assert!(result.stats.is_some());
}

#[tokio::test]
async fn test_bulk_generate_respects_count() {
    let service = test_service();

    let request = Request::new(BulkGenerateRequest {
        config: None,
        entry_count: 50,
        output_format: 0,
        include_master_data: false,
        inject_anomalies: false,
    });

    let response = service.bulk_generate(request).await.unwrap();
    let result = response.into_inner();

    // Should generate at least the requested amount
    assert!(result.entries_generated >= 50);
}

#[tokio::test]
async fn test_bulk_generate_with_anomalies() {
    let service = test_service();

    let request = Request::new(BulkGenerateRequest {
        config: None,
        entry_count: 100,
        output_format: 0,
        include_master_data: false,
        inject_anomalies: true,
    });

    let response = service.bulk_generate(request).await.unwrap();
    let result = response.into_inner();

    assert!(result.entries_generated > 0);
    // Anomaly count may be 0 due to probabilistic injection
    let stats = result.stats.unwrap();
    assert!(stats.total_entries > 0);
}

#[tokio::test]
async fn test_bulk_generate_returns_stats() {
    let service = test_service();

    let request = Request::new(BulkGenerateRequest {
        config: None,
        entry_count: 100,
        output_format: 0,
        include_master_data: false,
        inject_anomalies: false,
    });

    let response = service.bulk_generate(request).await.unwrap();
    let result = response.into_inner();

    let stats = result.stats.unwrap();
    assert!(stats.total_entries > 0);
    assert!(stats.total_lines > 0);
    assert!(!stats.entries_by_company.is_empty());
}

#[tokio::test]
async fn test_bulk_generate_entry_count_validation() {
    let service = test_service();

    // Request exceeds maximum (1,000,000)
    let request = Request::new(BulkGenerateRequest {
        config: None,
        entry_count: 2_000_000,
        output_format: 0,
        include_master_data: false,
        inject_anomalies: false,
    });

    let response = service.bulk_generate(request).await;
    assert!(response.is_err());
    let status = response.err().unwrap();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn test_bulk_generate_with_custom_config() {
    let service = test_service();

    let custom_config = GenerationConfig {
        industry: "retail".to_string(),
        start_date: "2024-01-01".to_string(),
        period_months: 3,
        seed: 12345,
        coa_complexity: "small".to_string(),
        companies: vec![],
        fraud_enabled: false,
        fraud_rate: 0.0,
        generate_master_data: false,
        generate_document_flows: false,
    };

    let request = Request::new(BulkGenerateRequest {
        config: Some(custom_config),
        entry_count: 50,
        output_format: 0,
        include_master_data: false,
        inject_anomalies: false,
    });

    let response = service.bulk_generate(request).await.unwrap();
    let result = response.into_inner();

    assert!(result.entries_generated > 0);
}

// ==========================================================================
// Control Tests
// ==========================================================================

#[tokio::test]
async fn test_control_pause() {
    let service = test_service();

    let request = Request::new(ControlCommand {
        action: ControlAction::Pause as i32,
        pattern_name: None,
    });

    let response = service.control(request).await.unwrap();
    let result = response.into_inner();

    assert!(result.success);
    assert!(result.message.to_lowercase().contains("pause"));
}

#[tokio::test]
async fn test_control_resume() {
    let service = test_service();

    let request = Request::new(ControlCommand {
        action: ControlAction::Resume as i32,
        pattern_name: None,
    });

    let response = service.control(request).await.unwrap();
    let result = response.into_inner();

    assert!(result.success);
    assert!(result.message.to_lowercase().contains("resum"));
}

#[tokio::test]
async fn test_control_stop() {
    let service = test_service();

    let request = Request::new(ControlCommand {
        action: ControlAction::Stop as i32,
        pattern_name: None,
    });

    let response = service.control(request).await.unwrap();
    let result = response.into_inner();

    assert!(result.success);
    assert!(result.message.to_lowercase().contains("stop"));
}

#[tokio::test]
async fn test_control_trigger_pattern() {
    let service = test_service();

    let request = Request::new(ControlCommand {
        action: ControlAction::TriggerPattern as i32,
        pattern_name: Some("year_end".to_string()),
    });

    let response = service.control(request).await.unwrap();
    let result = response.into_inner();

    // Pattern trigger may not be implemented
    // Just verify it doesn't crash
    assert!(result.message.len() > 0);
}

#[tokio::test]
async fn test_control_lifecycle() {
    let service = test_service();

    // Pause
    let pause_request = Request::new(ControlCommand {
        action: ControlAction::Pause as i32,
        pattern_name: None,
    });
    let pause_response = service.control(pause_request).await.unwrap();
    assert!(pause_response.into_inner().success);

    // Resume
    let resume_request = Request::new(ControlCommand {
        action: ControlAction::Resume as i32,
        pattern_name: None,
    });
    let resume_response = service.control(resume_request).await.unwrap();
    assert!(resume_response.into_inner().success);

    // Stop
    let stop_request = Request::new(ControlCommand {
        action: ControlAction::Stop as i32,
        pattern_name: None,
    });
    let stop_response = service.control(stop_request).await.unwrap();
    assert!(stop_response.into_inner().success);
}

// ==========================================================================
// Concurrent Access Tests
// ==========================================================================

#[tokio::test]
async fn test_concurrent_bulk_generate() {
    use std::sync::Arc;

    let service = Arc::new(test_service());

    let handles: Vec<_> = (0..3)
        .map(|_| {
            let svc = Arc::clone(&service);
            tokio::spawn(async move {
                let request = Request::new(BulkGenerateRequest {
                    config: None,
                    entry_count: 25,
                    output_format: 0,
                    include_master_data: false,
                    inject_anomalies: false,
                });
                svc.bulk_generate(request).await
            })
        })
        .collect();

    for handle in handles {
        let response = handle.await.unwrap();
        assert!(response.is_ok());
    }
}

#[tokio::test]
async fn test_concurrent_health_checks() {
    use std::sync::Arc;

    let service = Arc::new(test_service());

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let svc = Arc::clone(&service);
            tokio::spawn(async move {
                let request = Request::new(());
                svc.health_check(request).await
            })
        })
        .collect();

    for handle in handles {
        let response = handle.await.unwrap();
        assert!(response.is_ok());
        assert!(response.unwrap().into_inner().healthy);
    }
}
