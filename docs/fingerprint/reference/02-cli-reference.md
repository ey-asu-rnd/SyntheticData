# CLI Reference

Complete reference for fingerprint-related command-line tools.

---

## Overview

Fingerprinting functionality is provided by two CLI tools:

| Tool | Purpose |
|------|---------|
| `datasynth-fingerprint` | Extract, validate, and manage fingerprints |
| `datasynth-data` | Generate synthetic data from fingerprints, evaluate fidelity |

---

## datasynth-fingerprint

### Synopsis

```
datasynth-fingerprint <COMMAND> [OPTIONS]
```

### Commands

| Command | Description |
|---------|-------------|
| `extract` | Extract fingerprint from data source |
| `validate` | Validate fingerprint privacy compliance |
| `info` | Display fingerprint information |
| `diff` | Compare two fingerprints |
| `sign` | Sign a fingerprint |
| `verify` | Verify fingerprint signature |
| `merge` | Merge multiple fingerprints |
| `inspect` | Inspect data source before extraction |

---

### extract

Extract a fingerprint from a data source.

#### Synopsis

```
datasynth-fingerprint extract [OPTIONS] --output <PATH>
```

#### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--input <PATH>` | `-i` | Input file or directory | Required* |
| `--connection <URL>` | `-c` | Database connection string | Required* |
| `--tables <TABLES>` | `-t` | Comma-separated table names | All tables |
| `--output <PATH>` | `-o` | Output fingerprint path | Required |
| `--config <PATH>` | | Configuration file | None |
| `--privacy-level <LEVEL>` | `-p` | Privacy preset | `standard` |
| `--privacy-epsilon <FLOAT>` | | Differential privacy epsilon | 1.0 |
| `--privacy-k <INT>` | | k-anonymity threshold | 5 |
| `--where <EXPR>` | `-w` | SQL WHERE clause filter | None |
| `--sample <FLOAT>` | | Sample rate (0.0-1.0) | 1.0 |
| `--threads <INT>` | `-j` | Parallel threads | CPU count |
| `--verbose` | `-v` | Verbose output | Off |
| `--quiet` | `-q` | Suppress progress | Off |

*Either `--input` or `--connection` is required.

#### Privacy Levels

| Level | Epsilon | k | Description |
|-------|---------|---|-------------|
| `minimal` | 5.0 | 3 | Low privacy, high utility |
| `standard` | 1.0 | 5 | Balanced (default) |
| `high` | 0.5 | 10 | Higher privacy |
| `maximum` | 0.1 | 20 | Maximum privacy |

#### Examples

```bash
# Extract from CSV files
datasynth-fingerprint extract \
    --input ./data/ \
    --output ./fingerprint.dsf

# Extract from database with privacy settings
datasynth-fingerprint extract \
    --connection "postgresql://user:pass@localhost/db" \
    --tables "orders,customers,products" \
    --output ./fingerprint.dsf \
    --privacy-level high

# Extract with date filter and sampling
datasynth-fingerprint extract \
    --connection "postgresql://user:pass@localhost/db" \
    --tables "transactions" \
    --where "created_at >= '2023-01-01'" \
    --sample 0.1 \
    --output ./fingerprint.dsf

# Extract using config file
datasynth-fingerprint extract \
    --config ./extraction_config.yaml
```

---

### validate

Validate fingerprint privacy compliance.

#### Synopsis

```
datasynth-fingerprint validate <FINGERPRINT> [OPTIONS]
```

#### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--strict` | | Fail on warnings | Off |
| `--output <PATH>` | `-o` | Output validation report | stdout |
| `--format <FMT>` | `-f` | Output format (text, json) | text |

#### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Valid, no issues |
| 1 | Valid, with warnings |
| 2 | Invalid, privacy violations |
| 3 | Invalid, file error |

#### Examples

```bash
# Basic validation
datasynth-fingerprint validate ./fingerprint.dsf

# Strict validation (fail on warnings)
datasynth-fingerprint validate ./fingerprint.dsf --strict

# Output JSON report
datasynth-fingerprint validate ./fingerprint.dsf \
    --format json \
    --output ./validation_report.json
```

#### Sample Output

```
Validating fingerprint: ./fingerprint.dsf

Manifest:
  ✓ Version: 1.0.0
  ✓ Created: 2024-12-15T10:30:00Z
  ✓ Checksums valid

Privacy Audit:
  ✓ Differential privacy: ε=1.0
  ✓ k-anonymity: k=5
  ✓ No individual values
  ✓ All group sizes >= 5
  ✓ Epsilon budget: 0.87/1.0

Warnings:
  ⚠ 12 rare categories merged into 'Other'
  ⚠ 3 fields suppressed (PII)

Status: VALID (2 warnings)
```

---

### info

Display fingerprint information.

#### Synopsis

```
datasynth-fingerprint info <FINGERPRINT> [OPTIONS]
```

#### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--component <NAME>` | `-c` | Show specific component | All |
| `--format <FMT>` | `-f` | Output format (text, json, yaml) | text |
| `--check-completeness` | | Check for missing components | Off |

#### Components

| Name | Description |
|------|-------------|
| `schema` | Schema fingerprint |
| `statistics` | Statistical fingerprint |
| `correlations` | Correlation fingerprint |
| `integrity` | Referential integrity |
| `rules` | Business rules |
| `anomalies` | Anomaly profile |
| `privacy` | Privacy audit |

#### Examples

```bash
# Summary information
datasynth-fingerprint info ./fingerprint.dsf

# Detailed statistics
datasynth-fingerprint info ./fingerprint.dsf --component statistics

# JSON output
datasynth-fingerprint info ./fingerprint.dsf --format json

# Check completeness
datasynth-fingerprint info ./fingerprint.dsf --check-completeness
```

---

### diff

Compare two fingerprints.

#### Synopsis

```
datasynth-fingerprint diff <FINGERPRINT1> <FINGERPRINT2> [OPTIONS]
```

#### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--component <NAME>` | `-c` | Compare specific component | All |
| `--threshold <FLOAT>` | | Significance threshold | 0.05 |
| `--output <PATH>` | `-o` | Output diff report | stdout |

#### Examples

```bash
# Compare two fingerprints
datasynth-fingerprint diff ./fp_v1.dsf ./fp_v2.dsf

# Compare only statistics
datasynth-fingerprint diff ./fp_v1.dsf ./fp_v2.dsf --component statistics
```

---

### sign

Sign a fingerprint.

#### Synopsis

```
datasynth-fingerprint sign <FINGERPRINT> [OPTIONS]
```

#### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--key <PATH>` | `-k` | Private key file | Required |
| `--output <PATH>` | `-o` | Output signed fingerprint | In-place |
| `--signer <NAME>` | | Signer identifier | From key |

#### Examples

```bash
# Sign fingerprint
datasynth-fingerprint sign ./fingerprint.dsf \
    --key ./private_key.pem

# Sign to new file
datasynth-fingerprint sign ./fingerprint.dsf \
    --key ./private_key.pem \
    --output ./fingerprint_signed.dsf \
    --signer "EU Data Team"
```

---

### verify

Verify fingerprint signature.

#### Synopsis

```
datasynth-fingerprint verify <FINGERPRINT> [OPTIONS]
```

#### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--key <PATH>` | `-k` | Public key file | Required |

#### Examples

```bash
# Verify signature
datasynth-fingerprint verify ./fingerprint.dsf \
    --key ./public_key.pem
```

---

### merge

Merge multiple fingerprints.

#### Synopsis

```
datasynth-fingerprint merge <FINGERPRINTS...> --output <PATH> [OPTIONS]
```

#### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--output <PATH>` | `-o` | Output merged fingerprint | Required |
| `--strategy <STR>` | | Merge strategy | `union` |
| `--privacy-level <LEVEL>` | | Privacy for merged FP | `standard` |

#### Merge Strategies

| Strategy | Description |
|----------|-------------|
| `union` | Include all tables from all fingerprints |
| `intersection` | Only common tables |
| `weighted` | Weight statistics by source row counts |

#### Examples

```bash
# Merge fingerprints from different domains
datasynth-fingerprint merge \
    ./sales_fp.dsf ./finance_fp.dsf ./hr_fp.dsf \
    --output ./merged_fp.dsf \
    --strategy weighted
```

---

### inspect

Inspect a data source before extraction.

#### Synopsis

```
datasynth-fingerprint inspect [OPTIONS]
```

#### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--input <PATH>` | `-i` | Input file or directory | Required* |
| `--connection <URL>` | `-c` | Database connection string | Required* |
| `--suggest-config` | | Suggest extraction config | Off |

#### Examples

```bash
# Inspect files
datasynth-fingerprint inspect --input ./data/

# Inspect database and suggest config
datasynth-fingerprint inspect \
    --connection "postgresql://user:pass@localhost/db" \
    --suggest-config
```

---

## datasynth-data (Fingerprint Mode)

### generate (with fingerprint)

Generate synthetic data from a fingerprint.

#### Synopsis

```
datasynth-data generate --fingerprint <PATH> --output <PATH> [OPTIONS]
```

#### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--fingerprint <PATH>` | | Input fingerprint | Required |
| `--output <PATH>` | `-o` | Output directory | Required |
| `--config <PATH>` | | Generation config file | None |
| `--format <FMT>` | `-f` | Output format (csv, json, parquet) | csv |
| `--scale <FLOAT>` | | Scale factor | 1.0 |
| `--seed <INT>` | | Random seed | Random |
| `--start-date <DATE>` | | Override start date | From FP |
| `--end-date <DATE>` | | Override end date | From FP |
| `--anomaly-rate <FLOAT>` | | Override anomaly rate | From FP |
| `--threads <INT>` | `-j` | Parallel threads | CPU count |
| `--validate` | | Validate after generation | Off |
| `--progress` | | Show progress bar | Off |
| `--include-metadata` | | Include generation metadata | Off |

#### Examples

```bash
# Basic generation
datasynth-data generate \
    --fingerprint ./fingerprint.dsf \
    --output ./synthetic_data/

# Scaled generation with different date range
datasynth-data generate \
    --fingerprint ./fingerprint.dsf \
    --output ./synthetic_data/ \
    --scale 2.0 \
    --start-date 2025-01-01 \
    --end-date 2025-12-31 \
    --seed 42

# ML training data with higher anomaly rate
datasynth-data generate \
    --fingerprint ./fingerprint.dsf \
    --output ./ml_training_data/ \
    --anomaly-rate 0.05 \
    --format parquet \
    --validate

# Using config file
datasynth-data generate \
    --config ./generation_config.yaml
```

---

### evaluate

Evaluate synthetic data fidelity against a fingerprint.

#### Synopsis

```
datasynth-data evaluate --fingerprint <PATH> --synthetic <PATH> [OPTIONS]
```

#### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--fingerprint <PATH>` | | Reference fingerprint | Required |
| `--synthetic <PATH>` | | Synthetic data directory | Required |
| `--output <PATH>` | `-o` | Output report path | stdout |
| `--format <FMT>` | `-f` | Report format (text, html, json) | text |
| `--detailed` | `-d` | Include detailed metrics | Off |
| `--threshold <FLOAT>` | | Pass/fail threshold | 0.80 |

#### Examples

```bash
# Basic evaluation
datasynth-data evaluate \
    --fingerprint ./fingerprint.dsf \
    --synthetic ./synthetic_data/

# Detailed HTML report
datasynth-data evaluate \
    --fingerprint ./fingerprint.dsf \
    --synthetic ./synthetic_data/ \
    --output ./fidelity_report.html \
    --format html \
    --detailed

# CI/CD evaluation with threshold
datasynth-data evaluate \
    --fingerprint ./fingerprint.dsf \
    --synthetic ./synthetic_data/ \
    --threshold 0.90 \
    --format json
```

#### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Fidelity above threshold |
| 1 | Fidelity below threshold |
| 2 | Evaluation error |

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATASYNTH_LOG_LEVEL` | Log verbosity (trace, debug, info, warn, error) | info |
| `DATASYNTH_CONFIG_DIR` | Default config directory | `~/.datasynth/` |
| `DATASYNTH_CACHE_DIR` | Cache directory | `~/.cache/datasynth/` |
| `DATASYNTH_THREADS` | Default thread count | CPU count |
| `DATASYNTH_PRIVACY_LEVEL` | Default privacy level | standard |

---

## Configuration Files

### Extraction Config

```yaml
# ~/.datasynth/extraction.yaml
input:
  type: "database"
  connection: "${DATABASE_URL}"

output:
  path: "./fingerprint.dsf"

privacy:
  level: "standard"
  differential_privacy:
    epsilon: 1.0
  k_anonymity:
    k: 5
  suppression:
    always_suppress:
      - "*.ssn"
      - "*.email"

extraction:
  statistics:
    numeric:
      percentiles: [1, 5, 10, 25, 50, 75, 90, 95, 99]
    categorical:
      max_categories: 100
    temporal:
      detect_seasonality: true
```

### Generation Config

```yaml
# ~/.datasynth/generation.yaml
fingerprint:
  path: "./fingerprint.dsf"

output:
  path: "./synthetic_data/"
  format: "parquet"

generation:
  scale: 1.0
  seed: 42
  threads: 8
  correlations:
    enabled: true
    method: "copula"
```

---

## Shell Completion

### Bash

```bash
# Add to ~/.bashrc
eval "$(datasynth-fingerprint completions bash)"
eval "$(datasynth-data completions bash)"
```

### Zsh

```zsh
# Add to ~/.zshrc
eval "$(datasynth-fingerprint completions zsh)"
eval "$(datasynth-data completions zsh)"
```

### Fish

```fish
# Add to ~/.config/fish/config.fish
datasynth-fingerprint completions fish | source
datasynth-data completions fish | source
```

---

## Exit Codes Summary

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error / warnings |
| 2 | Invalid input / validation failure |
| 3 | File/IO error |
| 4 | Network/connection error |
| 5 | Privacy violation |
| 64-78 | Reserved for future use |

---

## Next Steps

- [API Reference](./03-api-reference.md): Rust API documentation
- [Fingerprint Specification](./01-fingerprint-spec.md): Format details
- [Examples](../examples/): Practical examples
