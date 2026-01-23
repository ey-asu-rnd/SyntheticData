# Synthetic Data Fingerprinting

> Generate privacy-preserving synthetic data that mirrors real datasets without exposing sensitive information.

## What is Fingerprinting?

Synthetic Data Fingerprinting is a technique that extracts the statistical "DNA" of a dataset—its structure, distributions, correlations, and business rules—without capturing any individual records. This fingerprint can then be used by DataSynth to generate synthetic data that is statistically equivalent to the original.

```
Real Data → Fingerprint Tool → .dsf File → DataSynth → Synthetic Data
   (PII)      (On-premise)      (Safe)     (Anywhere)    (No PII)
```

## Why Fingerprinting?

| Traditional Approach | Fingerprint Approach |
|---------------------|---------------------|
| Anonymize real data | Extract statistical properties |
| Risk of re-identification | No individual records stored |
| Destroys referential integrity | Preserves relationships |
| Corrupts statistical properties | Maintains distributions |
| Limited sharing options | Safe to share fingerprints |

## Quick Start

```bash
# 1. Extract fingerprint from real data (on-premise)
datasynth-fingerprint extract \
    --input ./real_erp_data/ \
    --output ./erp_fingerprint.dsf \
    --privacy-epsilon 1.0

# 2. Validate privacy compliance
datasynth-fingerprint validate ./erp_fingerprint.dsf

# 3. Generate synthetic data (anywhere)
datasynth-data generate \
    --fingerprint ./erp_fingerprint.dsf \
    --output ./synthetic_data/

# 4. Evaluate fidelity
datasynth-data evaluate \
    --fingerprint ./erp_fingerprint.dsf \
    --synthetic ./synthetic_data/ \
    --output ./fidelity_report.html
```

## Documentation Structure

### Concepts
- [Overview](./concepts/01-overview.md) - Problem statement and solution approach
- [Architecture](./concepts/02-architecture.md) - System design and components
- [Privacy Model](./concepts/03-privacy-model.md) - Privacy guarantees and mechanisms
- [Fidelity Model](./concepts/04-fidelity-model.md) - Quality metrics and validation

### Guides
- [Getting Started](./guides/01-getting-started.md) - First steps with fingerprinting
- [Extraction Guide](./guides/02-extraction-guide.md) - Extracting fingerprints from data
- [Generation Guide](./guides/03-generation-guide.md) - Generating synthetic data
- [Privacy Configuration](./guides/04-privacy-configuration.md) - Tuning privacy settings
- [Fidelity Tuning](./guides/05-fidelity-tuning.md) - Improving synthetic data quality
- [Integration Patterns](./guides/06-integration-patterns.md) - Common integration scenarios

### Reference
- [Fingerprint Specification](./reference/01-fingerprint-spec.md) - Complete .dsf format
- [CLI Reference](./reference/02-cli-reference.md) - Command-line interface
- [API Reference](./reference/03-api-reference.md) - Rust API documentation
- [Configuration Reference](./reference/04-configuration.md) - All configuration options
- [Metrics Reference](./reference/05-metrics-reference.md) - Fidelity metrics

### Examples
- [Cross-Border Sharing](./examples/01-cross-border.md) - GDPR-compliant data sharing
- [Vendor Collaboration](./examples/02-vendor-collaboration.md) - Sharing with software vendors
- [ML Training Pipeline](./examples/03-ml-training.md) - Training ML models
- [Audit Procedures](./examples/04-audit-procedures.md) - Audit firm collaboration

## Key Features

- **Privacy by Design**: Differential privacy, k-anonymity, and outlier suppression
- **Statistical Fidelity**: Preserves distributions, correlations, and patterns
- **Structural Integrity**: Maintains schema, relationships, and cardinalities
- **Business Rule Compliance**: Captures and enforces domain constraints
- **Anomaly Preservation**: Reproduces realistic anomaly profiles
- **Audit Trail**: Complete privacy audit documentation

## Version

This documentation covers DataSynth Fingerprinting v1.0.

## License

Apache 2.0 - See [LICENSE](../../LICENSE) for details.
