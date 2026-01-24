# datasynth-fingerprint

Privacy-preserving fingerprint extraction from real data and synthesis of matching synthetic data.

## Overview

The `datasynth-fingerprint` crate provides tools for extracting statistical fingerprints from real datasets while preserving privacy through differential privacy mechanisms and k-anonymity. These fingerprints can then be used to generate synthetic data that matches the statistical properties of the original data without exposing sensitive information.

## Architecture

```
Real Data → Extract → .dsf File → Generate → Synthetic Data → Evaluate
```

The fingerprinting workflow consists of three main stages:

1. **Extraction**: Analyze real data and extract statistical properties
2. **Synthesis**: Generate configuration and synthetic data from fingerprints
3. **Evaluation**: Validate synthetic data fidelity against fingerprints

## Key Components

### Models (`models/`)

| Model | Description |
|-------|-------------|
| **Fingerprint** | Root container with manifest, schema, statistics, correlations, integrity, rules, anomalies, privacy_audit |
| **Manifest** | Version, format, created_at, source metadata, privacy metadata, checksums, optional signature |
| **SchemaFingerprint** | Tables with columns, data types, cardinalities, relationships |
| **StatisticsFingerprint** | Numeric stats (distribution, percentiles, Benford), categorical stats (frequencies, entropy) |
| **CorrelationFingerprint** | Correlation matrices with copula parameters |
| **IntegrityFingerprint** | Foreign key definitions, cardinality rules |
| **RulesFingerprint** | Balance rules, approval thresholds |
| **AnomalyFingerprint** | Anomaly rates, type distributions, temporal patterns |
| **PrivacyAudit** | Actions log, epsilon spent, k-anonymity, warnings |

### Privacy Engine (`privacy/`)

| Component | Description |
|-----------|-------------|
| **LaplaceMechanism** | Differential privacy with configurable epsilon |
| **GaussianMechanism** | Alternative DP mechanism for (ε,δ)-privacy |
| **KAnonymity** | Suppression of rare categorical values below k threshold |
| **PrivacyEngine** | Unified interface combining DP, k-anonymity, winsorization |
| **PrivacyAuditBuilder** | Build privacy audit with actions and warnings |

#### Privacy Levels

| Level | Epsilon | k | Outlier % | Use Case |
|-------|---------|---|-----------|----------|
| Minimal | 5.0 | 3 | 99% | Low privacy, high utility |
| Standard | 1.0 | 5 | 95% | Balanced (default) |
| High | 0.5 | 10 | 90% | Higher privacy |
| Maximum | 0.1 | 20 | 85% | Maximum privacy |

### Extraction Engine (`extraction/`)

| Extractor | Description |
|-----------|-------------|
| **FingerprintExtractor** | Main coordinator for all extraction |
| **SchemaExtractor** | Infer data types, cardinalities, relationships |
| **StatsExtractor** | Compute distributions, percentiles, Benford analysis |
| **CorrelationExtractor** | Pearson correlations, copula fitting |
| **IntegrityExtractor** | Detect foreign key relationships |
| **RulesExtractor** | Detect balance rules, approval patterns |
| **AnomalyExtractor** | Analyze anomaly rates and patterns |

### I/O (`io/`)

| Component | Description |
|-----------|-------------|
| **FingerprintWriter** | Write .dsf files (ZIP with YAML/JSON components) |
| **FingerprintReader** | Read .dsf files with checksum verification |
| **FingerprintValidator** | Validate DSF structure and integrity |
| **validate_dsf()** | Convenience function for CLI validation |

### Synthesis (`synthesis/`)

| Component | Description |
|-----------|-------------|
| **ConfigSynthesizer** | Convert fingerprint to GeneratorConfig |
| **DistributionFitter** | Fit AmountSampler parameters from statistics |
| **GaussianCopula** | Generate correlated values preserving multivariate structure |

### Evaluation (`evaluation/`)

| Component | Description |
|-----------|-------------|
| **FidelityEvaluator** | Compare synthetic data against fingerprint |
| **FidelityReport** | Overall score, component scores, pass/fail status |
| **FidelityConfig** | Thresholds and weights for evaluation |

## DSF File Format

The DataSynth Fingerprint (`.dsf`) file is a ZIP archive containing:

```
fingerprint.dsf (ZIP)
├── manifest.json       # Version, checksums, privacy config
├── schema.yaml         # Tables, columns, relationships
├── statistics.yaml     # Distributions, percentiles, Benford
├── correlations.yaml   # Correlation matrices, copulas
├── integrity.yaml      # FK relationships, cardinality
├── rules.yaml          # Balance constraints, approval thresholds
├── anomalies.yaml      # Anomaly rates, type distribution
└── privacy_audit.json  # Privacy decisions, epsilon spent
```

## Usage

### Extracting a Fingerprint

```rust
use datasynth_fingerprint::{
    extraction::FingerprintExtractor,
    privacy::{PrivacyEngine, PrivacyLevel},
    io::FingerprintWriter,
};

// Create privacy engine with standard level
let privacy = PrivacyEngine::new(PrivacyLevel::Standard);

// Extract fingerprint from CSV data
let extractor = FingerprintExtractor::new(privacy);
let fingerprint = extractor.extract_from_csv("data.csv")?;

// Write to DSF file
let writer = FingerprintWriter::new();
writer.write(&fingerprint, "fingerprint.dsf")?;
```

### Reading a Fingerprint

```rust
use datasynth_fingerprint::io::FingerprintReader;

let reader = FingerprintReader::new();
let fingerprint = reader.read("fingerprint.dsf")?;

println!("Tables: {:?}", fingerprint.schema.tables.len());
println!("Privacy epsilon spent: {}", fingerprint.privacy_audit.epsilon_spent);
```

### Validating a Fingerprint

```rust
use datasynth_fingerprint::io::validate_dsf;

match validate_dsf("fingerprint.dsf") {
    Ok(report) => println!("Valid: {:?}", report),
    Err(e) => eprintln!("Invalid: {}", e),
}
```

### Synthesizing Configuration

```rust
use datasynth_fingerprint::synthesis::ConfigSynthesizer;

let synthesizer = ConfigSynthesizer::new();
let config = synthesizer.synthesize(&fingerprint)?;

// Use config with datasynth-generators
```

### Evaluating Fidelity

```rust
use datasynth_fingerprint::evaluation::{FidelityEvaluator, FidelityConfig};

let config = FidelityConfig::default();
let evaluator = FidelityEvaluator::new(config);

let report = evaluator.evaluate(&fingerprint, "./synthetic_data/")?;

println!("Overall score: {:.2}", report.overall_score);
println!("Pass: {}", report.passed);

for (metric, score) in &report.component_scores {
    println!("  {}: {:.2}", metric, score);
}
```

## Fidelity Metrics

| Category | Metrics |
|----------|---------|
| **Statistical** | KS statistic, Wasserstein distance, Benford MAD |
| **Correlation** | Correlation matrix RMSE |
| **Schema** | Column type match, row count ratio |
| **Rules** | Balance equation compliance rate |

## Privacy Guarantees

The fingerprint extraction process provides the following privacy guarantees:

1. **Differential Privacy**: Numeric statistics are perturbed using Laplace or Gaussian mechanisms with configurable epsilon budget
2. **K-Anonymity**: Categorical values appearing fewer than k times are suppressed
3. **Winsorization**: Outliers are clipped to prevent identification of extreme values
4. **Audit Trail**: All privacy decisions are logged for compliance verification

## CLI Commands

```bash
# Extract fingerprint
datasynth-data fingerprint extract \
    --input ./data.csv \
    --output ./fp.dsf \
    --privacy-level standard

# Validate
datasynth-data fingerprint validate ./fp.dsf

# Show info
datasynth-data fingerprint info ./fp.dsf --detailed

# Compare
datasynth-data fingerprint diff ./fp1.dsf ./fp2.dsf

# Evaluate fidelity
datasynth-data fingerprint evaluate \
    --fingerprint ./fp.dsf \
    --synthetic ./synthetic/ \
    --threshold 0.8
```

## Dependencies

```toml
[dependencies]
datasynth-core = { path = "../datasynth-core" }
datasynth-config = { path = "../datasynth-config" }
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"
zip = "0.6"
sha2 = "0.10"
rand = "0.8"
statrs = "0.16"
```

## See Also

- [Fingerprinting Guide](../advanced/fingerprinting.md)
- [CLI Reference](../user-guide/cli-reference.md#fingerprint)
- [Privacy Model](../../fingerprint/concepts/03-privacy-model.md)
- [Fidelity Model](../../fingerprint/concepts/04-fidelity-model.md)
