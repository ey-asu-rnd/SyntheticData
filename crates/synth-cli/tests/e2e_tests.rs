//! End-to-end CLI tests for synth-data.
//!
//! These tests exercise complete workflows from config creation through
//! data generation and output validation.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Get a Command for our binary.
fn synth_data() -> Command {
    Command::cargo_bin("synth-data").unwrap()
}

// ==========================================================================
// Full Workflow E2E Tests
// ==========================================================================

/// Test complete workflow: init -> validate -> generate
#[test]
fn test_full_workflow_init_validate_generate() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("workflow_config.yaml");
    let output_dir = temp_dir.path().join("output");

    // Step 1: Initialize config
    synth_data()
        .arg("init")
        .arg("-o")
        .arg(config_path.to_str().unwrap())
        .arg("-i")
        .arg("manufacturing")
        .arg("-c")
        .arg("small")
        .assert()
        .success();

    assert!(config_path.exists(), "Config file should be created");

    // Step 2: Validate config
    synth_data()
        .arg("validate")
        .arg("-c")
        .arg(config_path.to_str().unwrap())
        .assert()
        .success();

    // Step 3: Generate data
    synth_data()
        .arg("generate")
        .arg("-c")
        .arg(config_path.to_str().unwrap())
        .arg("-o")
        .arg(output_dir.to_str().unwrap())
        .arg("-s")
        .arg("42")
        .assert()
        .success();

    assert!(output_dir.exists(), "Output directory should be created");
}

/// Test workflow with each industry preset
#[test]
fn test_all_industry_presets_workflow() {
    let industries = [
        "manufacturing",
        "retail",
        "healthcare",
        "technology",
        "financial_services",
    ];

    for industry in industries {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(format!("{}_config.yaml", industry));
        let output_dir = temp_dir.path().join("output");

        // Init with industry preset
        synth_data()
            .arg("init")
            .arg("-o")
            .arg(config_path.to_str().unwrap())
            .arg("-i")
            .arg(industry)
            .assert()
            .success();

        // Validate
        synth_data()
            .arg("validate")
            .arg("-c")
            .arg(config_path.to_str().unwrap())
            .assert()
            .success();

        // Generate (demo mode for speed)
        synth_data()
            .arg("generate")
            .arg("--demo")
            .arg("-o")
            .arg(output_dir.to_str().unwrap())
            .assert()
            .success();

        assert!(
            output_dir.exists(),
            "Output directory should be created for {}",
            industry
        );
    }
}

/// Test workflow with different complexity levels
#[test]
fn test_all_complexity_levels() {
    let complexities = ["small", "medium", "large"];

    for complexity in complexities {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(format!("{}_config.yaml", complexity));

        // Init with complexity level
        synth_data()
            .arg("init")
            .arg("-o")
            .arg(config_path.to_str().unwrap())
            .arg("-c")
            .arg(complexity)
            .assert()
            .success();

        // Validate
        synth_data()
            .arg("validate")
            .arg("-c")
            .arg(config_path.to_str().unwrap())
            .assert()
            .success();

        // Verify config content reflects complexity
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(
            content.contains(complexity),
            "Config should contain complexity level: {}",
            complexity
        );
    }
}

// ==========================================================================
// Output Validation Tests
// ==========================================================================

/// Test that generated JSON output is valid
#[test]
fn test_generated_json_is_valid() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("output");

    synth_data()
        .arg("generate")
        .arg("--demo")
        .arg("-o")
        .arg(output_dir.to_str().unwrap())
        .assert()
        .success();

    // Find and validate sample_entries.json
    let sample_path = output_dir.join("sample_entries.json");
    assert!(sample_path.exists(), "Sample entries should be generated");

    let content = fs::read_to_string(&sample_path).unwrap();
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
    assert!(parsed.is_ok(), "Sample entries should be valid JSON");
}

/// Test that generated output contains expected structure
#[test]
fn test_generated_output_structure() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("output");

    synth_data()
        .arg("generate")
        .arg("--demo")
        .arg("-o")
        .arg(output_dir.to_str().unwrap())
        .arg("-s")
        .arg("12345")
        .assert()
        .success();

    // Check that output directory has expected files
    assert!(output_dir.exists(), "Output directory should exist");

    // List files in output
    let files: Vec<_> = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    println!("Generated files: {:?}", files);

    // Should have sample_entries.json
    assert!(
        files.iter().any(|f| f == "sample_entries.json"),
        "Should have sample_entries.json"
    );
}

// ==========================================================================
// Determinism Tests
// ==========================================================================

/// Test that same seed produces identical output
#[test]
fn test_deterministic_generation_with_seed() {
    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();
    let output_dir1 = temp_dir1.path().join("output");
    let output_dir2 = temp_dir2.path().join("output");

    // Generate with same seed twice
    for (output_dir, _name) in [
        (output_dir1.clone(), "first"),
        (output_dir2.clone(), "second"),
    ] {
        synth_data()
            .arg("generate")
            .arg("--demo")
            .arg("-o")
            .arg(output_dir.to_str().unwrap())
            .arg("-s")
            .arg("99999")
            .assert()
            .success();
    }

    // Compare outputs
    let content1 = fs::read_to_string(output_dir1.join("sample_entries.json")).unwrap();
    let content2 = fs::read_to_string(output_dir2.join("sample_entries.json")).unwrap();

    assert_eq!(
        content1, content2,
        "Same seed should produce identical output"
    );
}

/// Test that different seeds produce different output
#[test]
fn test_different_seeds_different_output() {
    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();
    let output_dir1 = temp_dir1.path().join("output");
    let output_dir2 = temp_dir2.path().join("output");

    // Generate with different seeds
    synth_data()
        .arg("generate")
        .arg("--demo")
        .arg("-o")
        .arg(output_dir1.to_str().unwrap())
        .arg("-s")
        .arg("11111")
        .assert()
        .success();

    synth_data()
        .arg("generate")
        .arg("--demo")
        .arg("-o")
        .arg(output_dir2.to_str().unwrap())
        .arg("-s")
        .arg("22222")
        .assert()
        .success();

    // Compare outputs
    let content1 = fs::read_to_string(output_dir1.join("sample_entries.json")).unwrap();
    let content2 = fs::read_to_string(output_dir2.join("sample_entries.json")).unwrap();

    assert_ne!(
        content1, content2,
        "Different seeds should produce different output"
    );
}

// ==========================================================================
// Config Modification Tests
// ==========================================================================

/// Test modifying config and regenerating
#[test]
fn test_config_modification_and_regenerate() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("modify_config.yaml");
    let output_dir = temp_dir.path().join("output");

    // Create initial config
    synth_data()
        .arg("init")
        .arg("-o")
        .arg(config_path.to_str().unwrap())
        .assert()
        .success();

    // Read and parse config
    let content = fs::read_to_string(&config_path).unwrap();
    let mut config: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();

    // Modify seed in config
    if let Some(global) = config.get_mut("global") {
        global["seed"] = serde_yaml::Value::Number(serde_yaml::Number::from(42));
    }

    // Write modified config
    fs::write(&config_path, serde_yaml::to_string(&config).unwrap()).unwrap();

    // Validate modified config
    synth_data()
        .arg("validate")
        .arg("-c")
        .arg(config_path.to_str().unwrap())
        .assert()
        .success();

    // Generate with modified config
    synth_data()
        .arg("generate")
        .arg("-c")
        .arg(config_path.to_str().unwrap())
        .arg("-o")
        .arg(output_dir.to_str().unwrap())
        .assert()
        .success();

    assert!(output_dir.exists(), "Should generate with modified config");
}

// ==========================================================================
// Error Handling Tests
// ==========================================================================

/// Test handling of invalid config file
#[test]
fn test_invalid_config_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid_config.yaml");

    // Write invalid YAML
    fs::write(
        &config_path,
        "global:\n  seed: invalid_value\n  bogus: field",
    )
    .unwrap();

    synth_data()
        .arg("validate")
        .arg("-c")
        .arg(config_path.to_str().unwrap())
        .assert()
        .failure();
}

/// Test handling of missing config file
#[test]
fn test_missing_config_handling() {
    synth_data()
        .arg("generate")
        .arg("-c")
        .arg("/nonexistent/config.yaml")
        .arg("-o")
        .arg("/tmp/output")
        .assert()
        .failure();
}

/// Test handling of invalid output directory
#[test]
fn test_invalid_output_directory() {
    // Try to generate to a path where we can't create directories
    // On Linux, /proc is read-only
    synth_data()
        .arg("generate")
        .arg("--demo")
        .arg("-o")
        .arg("/proc/invalid_output_dir")
        .assert()
        .failure();
}

// ==========================================================================
// Multi-Company Workflow Tests
// ==========================================================================

/// Test generating for multi-company configuration
#[test]
fn test_multi_company_generation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("multi_company.yaml");
    let output_dir = temp_dir.path().join("output");

    // Create config
    synth_data()
        .arg("init")
        .arg("-o")
        .arg(config_path.to_str().unwrap())
        .assert()
        .success();

    // Read config and add more companies
    let content = fs::read_to_string(&config_path).unwrap();
    let mut config: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();

    // Add second company
    if let Some(companies) = config
        .get_mut("companies")
        .and_then(|c| c.as_sequence_mut())
    {
        let second_company = serde_yaml::from_str::<serde_yaml::Value>(
            r#"
            code: "2000"
            name: "Subsidiary Company"
            currency: "EUR"
            country: "DE"
            annual_transaction_volume: "ten_k"
            volume_weight: 0.3
            fiscal_year_variant: "K4"
            "#,
        )
        .unwrap();
        companies.push(second_company);
    }

    // Update first company weight
    if let Some(companies) = config
        .get_mut("companies")
        .and_then(|c| c.as_sequence_mut())
    {
        if let Some(first) = companies.get_mut(0) {
            first["volume_weight"] = serde_yaml::Value::Number(serde_yaml::Number::from(0.7f64));
        }
    }

    fs::write(&config_path, serde_yaml::to_string(&config).unwrap()).unwrap();

    // Validate
    synth_data()
        .arg("validate")
        .arg("-c")
        .arg(config_path.to_str().unwrap())
        .assert()
        .success();

    // Generate
    synth_data()
        .arg("generate")
        .arg("-c")
        .arg(config_path.to_str().unwrap())
        .arg("-o")
        .arg(output_dir.to_str().unwrap())
        .assert()
        .success();

    assert!(output_dir.exists(), "Should generate multi-company output");
}

// ==========================================================================
// Config Export/Import Tests
// ==========================================================================

/// Test that generated configs can be parsed correctly
#[test]
fn test_config_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("roundtrip_config.yaml");

    // Generate config
    synth_data()
        .arg("init")
        .arg("-o")
        .arg(config_path.to_str().unwrap())
        .arg("-i")
        .arg("manufacturing")
        .arg("-c")
        .arg("medium")
        .assert()
        .success();

    // Read and parse
    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: synth_config::GeneratorConfig =
        serde_yaml::from_str(&content).expect("Should parse as GeneratorConfig");

    // Serialize back
    let serialized = serde_yaml::to_string(&parsed).expect("Should serialize back");

    // Parse again to verify roundtrip
    let _reparsed: synth_config::GeneratorConfig =
        serde_yaml::from_str(&serialized).expect("Should parse after roundtrip");
}

// ==========================================================================
// Performance/Stress Tests
// ==========================================================================

/// Test that demo generation completes in reasonable time
#[test]
fn test_demo_generation_performance() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("output");

    let start = std::time::Instant::now();

    synth_data()
        .arg("generate")
        .arg("--demo")
        .arg("-o")
        .arg(output_dir.to_str().unwrap())
        .timeout(std::time::Duration::from_secs(60))
        .assert()
        .success();

    let duration = start.elapsed();
    println!("Demo generation completed in {:?}", duration);

    // Demo generation should complete in under 60 seconds
    assert!(
        duration < std::time::Duration::from_secs(60),
        "Demo generation should complete in under 60 seconds"
    );
}

// ==========================================================================
// CLI Argument Tests
// ==========================================================================

/// Test verbose mode provides additional output
#[test]
fn test_verbose_mode() {
    synth_data()
        .arg("-v")
        .arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("Industry Presets"));
}

/// Test verbose flag with long form
#[test]
fn test_verbose_long_form() {
    synth_data().arg("--verbose").arg("info").assert().success();
}

/// Test help for each subcommand
#[test]
fn test_subcommand_help() {
    for subcommand in ["generate", "init", "validate", "info"] {
        synth_data()
            .arg(subcommand)
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage:"));
    }
}
