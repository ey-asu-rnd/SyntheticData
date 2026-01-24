# Fingerprinting

Privacy-preserving fingerprint extraction enables generating synthetic data that matches the statistical properties of real data without exposing sensitive information.

## Overview

Fingerprinting is a three-stage process:

1. **Extract**: Analyze real data and capture statistical properties into a `.dsf` fingerprint file
2. **Synthesize**: Generate synthetic data configuration from the fingerprint
3. **Evaluate**: Validate that synthetic data matches the fingerprint's statistical properties

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Real Data  │────▶│   Extract   │────▶│ .dsf File   │────▶│  Evaluate   │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
                          │                    │                    │
                          ▼                    ▼                    ▼
                    Privacy Engine      Config Synthesizer    Fidelity Report
```

## Privacy Mechanisms

### Differential Privacy

The extraction process applies differential privacy to protect individual records:

- **Laplace Mechanism**: Adds calibrated noise to numeric statistics
- **Gaussian Mechanism**: Alternative for (ε,δ)-differential privacy
- **Epsilon Budget**: Tracks privacy budget across all operations

```
Privacy Guarantee: For any two datasets D and D' differing in one record,
the probability ratio of any output is bounded by e^ε
```

### K-Anonymity

Categorical values are protected through suppression:

- Values appearing fewer than k times are replaced with `<suppressed>`
- Prevents identification of rare categories
- Configurable threshold per privacy level

### Winsorization

Numeric outliers are clipped to prevent identification:

- Values beyond the configured percentile are capped
- Prevents extreme values from leaking individual information
- Outlier percentile varies by privacy level (85%-99%)

## Privacy Levels

| Level | Epsilon | k | Outlier % | Description |
|-------|---------|---|-----------|-------------|
| **Minimal** | 5.0 | 3 | 99% | Highest utility, lower privacy |
| **Standard** | 1.0 | 5 | 95% | Balanced (recommended default) |
| **High** | 0.5 | 10 | 90% | Higher privacy for sensitive data |
| **Maximum** | 0.1 | 20 | 85% | Maximum privacy, some utility loss |

### Choosing a Privacy Level

- **Minimal**: Internal testing, non-sensitive data
- **Standard**: General use, moderate sensitivity
- **High**: Personal financial data, healthcare
- **Maximum**: Highly sensitive data, regulatory compliance

## DSF File Format

The DataSynth Fingerprint (`.dsf`) file is a ZIP archive containing:

```
fingerprint.dsf
├── manifest.json       # Metadata, checksums, privacy config
├── schema.yaml         # Table/column structure, relationships
├── statistics.yaml     # Distributions, percentiles, Benford
├── correlations.yaml   # Correlation matrices, copula params
├── integrity.yaml      # Foreign keys, cardinality rules
├── rules.yaml          # Balance constraints, thresholds
├── anomalies.yaml      # Anomaly rates, patterns
└── privacy_audit.json  # All privacy decisions logged
```

### Manifest Structure

```json
{
  "version": "1.0",
  "format": "dsf",
  "created_at": "2026-01-23T10:30:00Z",
  "source": {
    "row_count": 100000,
    "column_count": 25,
    "tables": ["journal_entries", "vendors"]
  },
  "privacy": {
    "level": "standard",
    "epsilon": 1.0,
    "k": 5
  },
  "checksums": {
    "schema": "sha256:...",
    "statistics": "sha256:...",
    "correlations": "sha256:..."
  }
}
```

## Extraction Process

### Step 1: Schema Extraction

Analyzes data structure:
- Infers column data types (numeric, categorical, date, text)
- Computes cardinalities
- Detects foreign key relationships
- Identifies primary keys

### Step 2: Statistical Extraction

Computes distributions with privacy:
- **Numeric columns**: Mean, std, min, max, percentiles (with DP noise)
- **Categorical columns**: Frequencies (with k-anonymity)
- **Temporal columns**: Date ranges, seasonality patterns
- **Benford analysis**: First-digit distribution compliance

### Step 3: Correlation Extraction

Captures multivariate relationships:
- Pearson correlation matrices (with DP)
- Copula parameters for joint distributions
- Cross-table relationship strengths

### Step 4: Rules Extraction

Detects business rules:
- Balance equations (debits = credits)
- Approval thresholds
- Validation constraints

### Step 5: Anomaly Pattern Extraction

Captures anomaly characteristics:
- Overall anomaly rate
- Type distribution
- Temporal patterns

## Synthesis Process

### Configuration Generation

The `ConfigSynthesizer` converts fingerprints to generation configuration:

```rust
// From fingerprint statistics, generate:
AmountSampler {
    distribution: LogNormal,
    mean: fp.statistics.amount.mean,
    std: fp.statistics.amount.std,
    round_number_bias: 0.15,
}
```

### Copula-Based Generation

For correlated columns, the `GaussianCopula` preserves relationships:

1. Generate independent uniform samples
2. Apply correlation structure
3. Transform to target marginal distributions

## Fidelity Evaluation

### Metrics

| Metric | Description | Target |
|--------|-------------|--------|
| **KS Statistic** | Max CDF difference | < 0.1 |
| **Wasserstein Distance** | Earth mover's distance | < 0.1 |
| **Benford MAD** | Mean absolute deviation from Benford | < 0.015 |
| **Correlation RMSE** | Correlation matrix difference | < 0.1 |
| **Schema Match** | Column type agreement | > 0.95 |

### Fidelity Report

```
Fidelity Evaluation Report
==========================
Overall Score: 0.87
Status: PASSED (threshold: 0.80)

Component Scores:
  Statistical:   0.89
  Correlation:   0.85
  Schema:        0.98
  Rules:         0.76

Details:
  - KS statistic (amount): 0.05
  - Benford MAD: 0.008
  - Correlation RMSE: 0.07
```

## CLI Usage

### Basic Workflow

```bash
# 1. Extract fingerprint from real data
datasynth-data fingerprint extract \
    --input ./real_data.csv \
    --output ./fingerprint.dsf \
    --privacy-level standard

# 2. Validate fingerprint integrity
datasynth-data fingerprint validate ./fingerprint.dsf

# 3. View fingerprint details
datasynth-data fingerprint info ./fingerprint.dsf --detailed

# 4. Generate synthetic data (using derived config)
datasynth-data generate \
    --config ./derived_config.yaml \
    --output ./synthetic_data

# 5. Evaluate fidelity
datasynth-data fingerprint evaluate \
    --fingerprint ./fingerprint.dsf \
    --synthetic ./synthetic_data \
    --threshold 0.85 \
    --report ./report.html
```

### Comparing Fingerprints

```bash
# Compare two versions
datasynth-data fingerprint diff ./fp_v1.dsf ./fp_v2.dsf
```

### Custom Privacy Parameters

```bash
# Override privacy level with custom values
datasynth-data fingerprint extract \
    --input ./sensitive_data.csv \
    --output ./fingerprint.dsf \
    --epsilon 0.3 \
    --k 15
```

## Best Practices

### Data Preparation

1. **Clean data first**: Remove obvious errors before extraction
2. **Consistent formats**: Standardize date and number formats
3. **Document exclusions**: Note any columns excluded from extraction

### Privacy Selection

1. **Start with standard**: Adjust based on fidelity evaluation
2. **Consider sensitivity**: Use higher privacy for personal data
3. **Review audit log**: Check privacy decisions in `privacy_audit.json`

### Fidelity Optimization

1. **Check component scores**: Identify weak areas
2. **Adjust generation config**: Tune parameters for low-scoring metrics
3. **Iterate**: Re-evaluate after adjustments

### Compliance

1. **Preserve audit trail**: Keep `.dsf` files for compliance review
2. **Document privacy choices**: Record rationale for privacy level
3. **Version fingerprints**: Track changes over time

## Troubleshooting

### Low Fidelity Score

**Cause**: Statistical differences between synthetic and fingerprint

**Solutions**:
- Review component scores to identify specific issues
- Adjust generation configuration parameters
- Consider using auto-tuning recommendations

### Fingerprint Validation Errors

**Cause**: Corrupted or modified DSF file

**Solutions**:
- Re-extract from source data
- Check file transfer integrity
- Verify checksums match manifest

### Privacy Budget Exceeded

**Cause**: Too many queries on sensitive data

**Solutions**:
- Reduce number of extracted statistics
- Use higher epsilon (lower privacy)
- Aggregate fine-grained statistics

## See Also

- [CLI Reference - Fingerprint Commands](../user-guide/cli-reference.md#fingerprint)
- [datasynth-fingerprint Crate](../crates/datasynth-fingerprint.md)
- [Fingerprint Concepts](../../fingerprint/concepts/01-overview.md)
- [Privacy Model](../../fingerprint/concepts/03-privacy-model.md)
