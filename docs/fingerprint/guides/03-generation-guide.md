# Generation Guide

This guide covers generating synthetic data from fingerprints using DataSynth.

---

## How Generation Works

When you generate from a fingerprint, DataSynth:

1. **Reads the fingerprint** and extracts schema, statistics, correlations, and rules
2. **Synthesizes configuration** by mapping fingerprint parameters to DataSynth config
3. **Fits generators** to match the fingerprint's distributions
4. **Enforces correlations** using copula-based generation
5. **Applies business rules** to ensure semantic validity
6. **Injects anomalies** matching the fingerprint's anomaly profile

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  .dsf           │────▶│  Config         │────▶│  DataSynth      │
│  Fingerprint    │     │  Synthesizer    │     │  Generators     │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                        │
                                                        ▼
                                                ┌─────────────────┐
                                                │  Synthetic      │
                                                │  Data           │
                                                └─────────────────┘
```

---

## Basic Generation

### Simple Generation

```bash
# Generate with defaults (same volume as original)
datasynth-data generate \
    --fingerprint ./my_fingerprint.dsf \
    --output ./synthetic_data/
```

### Common Options

```bash
# Scale the data (2x the original volume)
datasynth-data generate \
    --fingerprint ./my_fingerprint.dsf \
    --output ./synthetic_data/ \
    --scale 2.0

# Generate for a specific time period
datasynth-data generate \
    --fingerprint ./my_fingerprint.dsf \
    --output ./synthetic_data/ \
    --start-date 2025-01-01 \
    --end-date 2025-12-31

# Specify output format
datasynth-data generate \
    --fingerprint ./my_fingerprint.dsf \
    --output ./synthetic_data/ \
    --format parquet

# Set random seed for reproducibility
datasynth-data generate \
    --fingerprint ./my_fingerprint.dsf \
    --output ./synthetic_data/ \
    --seed 42
```

---

## Configuration File

For advanced control, use a configuration file:

```yaml
# generation_config.yaml

# Fingerprint source
fingerprint:
  path: "./my_fingerprint.dsf"

  # Optional: verify fingerprint integrity
  verify_checksum: true
  verify_signature: true

# Output settings
output:
  path: "./synthetic_data/"
  format: "csv"  # csv, json, parquet

  # Compression (optional)
  compression:
    enabled: true
    algorithm: "gzip"
    level: 6

  # File splitting (optional)
  splitting:
    enabled: true
    max_rows_per_file: 1000000

# Generation parameters
generation:
  # Scale relative to fingerprint
  scale: 1.0

  # Date range override
  date_range:
    start: "2025-01-01"
    end: "2025-12-31"

  # Random seed
  seed: 42

  # Parallelism
  threads: 8

# Override fingerprint settings
overrides:
  # Add/modify companies
  companies:
    - code: "SYNTH001"
      name: "Synthetic Corp"
      currency: "USD"
      country: "US"
      volume_weight: 1.0

  # Modify anomaly injection
  anomaly_injection:
    enabled: true
    rate_multiplier: 2.0  # Double the anomaly rate for ML training

  # Modify specific distributions
  distributions:
    "journal_entries.amount":
      shift: 1.1  # 10% higher amounts
```

Run with config:

```bash
datasynth-data generate --config generation_config.yaml
```

---

## Scaling Options

### Volume Scaling

Control the amount of data generated:

```yaml
generation:
  # Relative scale (1.0 = same as original)
  scale: 0.5  # 50% of original volume

  # OR absolute count
  # row_count:
  #   journal_entries: 500000
  #   vendors: 2000
```

### Time Period Scaling

Shift or expand the time range:

```yaml
generation:
  date_range:
    # Option 1: Explicit dates
    start: "2025-01-01"
    end: "2025-12-31"

    # Option 2: Shift from original
    # shift_years: 1  # Move 1 year forward

    # Option 3: Expand/contract
    # months: 24  # Generate 24 months (vs original)
```

### Entity Scaling

Scale master data entities:

```yaml
generation:
  entity_scaling:
    vendors:
      scale: 1.5  # 50% more vendors
    customers:
      scale: 2.0  # Double the customers
    employees:
      count: 500  # Exactly 500 employees
```

---

## Distribution Overrides

### Modifying Distributions

Override fingerprint distributions for specific use cases:

```yaml
overrides:
  distributions:
    # Shift amount distribution
    "journal_entries.amount":
      type: "shift"
      multiplier: 1.2  # 20% higher

    # Replace distribution entirely
    "journal_entries.line_count":
      type: "replace"
      distribution:
        type: "fixed"
        values: [2, 4, 6]
        probabilities: [0.6, 0.3, 0.1]

    # Add variation
    "journal_entries.posting_date":
      type: "add_noise"
      noise_days: 2  # ±2 days random variation
```

### Temporal Pattern Overrides

```yaml
overrides:
  temporal:
    # Modify month-end spike
    month_end_spike: 3.0  # Increase from fingerprint value

    # Modify weekday distribution
    weekday_distribution: [0.20, 0.20, 0.20, 0.20, 0.20, 0.0, 0.0]

    # Add specific holidays
    holidays:
      - date: "2025-12-25"
        effect: 0.0  # No transactions
      - date: "2025-11-28"
        effect: 0.5  # 50% volume (Thanksgiving)
```

---

## Correlation Preservation

### Automatic Correlation

By default, DataSynth preserves correlations from the fingerprint:

```yaml
generation:
  correlations:
    # Enable correlation preservation (default: true)
    enabled: true

    # Method for correlation preservation
    method: "copula"  # copula, conditional, none

    # Tolerance for correlation matching
    tolerance: 0.1  # Accept if within ±0.1 of target
```

### Copula-Based Generation

For high-fidelity correlation preservation:

```yaml
generation:
  correlations:
    method: "copula"

    # Copula settings
    copula:
      type: "gaussian"  # gaussian, t, clayton, frank

      # Columns to include in copula
      columns:
        - "amount"
        - "line_count"
        - "approval_level"
```

### Conditional Generation

For categorical dependencies:

```yaml
generation:
  correlations:
    method: "conditional"

    conditionals:
      # When business_process is AP, use specific account distribution
      - condition:
          column: "business_process"
          value: "accounts_payable"
        then:
          column: "account_type"
          distribution:
            expense: 0.65
            asset: 0.25
            liability: 0.10
```

---

## Business Rule Enforcement

### Balance Equations

```yaml
generation:
  rules:
    # Ensure debits = credits (always enabled for financial data)
    balance_equations: true

    # Tolerance for balance
    balance_tolerance: 0.01
```

### Approval Workflow

```yaml
generation:
  rules:
    approval:
      enabled: true

      # Use fingerprint thresholds or override
      thresholds:
        - max_amount: 1000
          approvers: 1
        - max_amount: 10000
          approvers: 2
        - max_amount: 100000
          approvers: 3
        - max_amount: null
          approvers: 4
```

### Referential Integrity

```yaml
generation:
  rules:
    referential_integrity:
      enabled: true

      # Ensure FK references exist
      enforce_foreign_keys: true

      # Allow orphan records (matching fingerprint rate)
      allow_orphans: true
      orphan_rate: 0.001  # 0.1% orphans (from fingerprint)
```

### Temporal Ordering

```yaml
generation:
  rules:
    temporal_ordering:
      enabled: true

      # Ensure document dates are in correct order
      orderings:
        - before: "order_date"
          after: "invoice_date"
        - before: "invoice_date"
          after: "payment_date"
```

---

## Anomaly Injection

### From Fingerprint

By default, anomalies match the fingerprint profile:

```yaml
generation:
  anomalies:
    # Use fingerprint anomaly profile
    source: "fingerprint"

    # Optional: scale the rate
    rate_multiplier: 1.0  # Same rate as fingerprint
```

### Enhanced for ML Training

For ML training, you may want more anomalies:

```yaml
generation:
  anomalies:
    source: "fingerprint"

    # Increase anomaly rate for training data
    rate_multiplier: 5.0  # 5x the normal rate

    # Focus on specific types
    type_weights:
      fraud: 2.0      # Double fraud anomalies
      errors: 1.0     # Normal error rate
      process: 0.5    # Half process anomalies
```

### Custom Anomaly Profile

```yaml
generation:
  anomalies:
    source: "custom"

    profile:
      overall_rate: 0.05  # 5% anomalies

      types:
        fraud:
          rate: 0.02
          subtypes:
            duplicate_payment: 0.4
            fictitious_vendor: 0.3
            expense_manipulation: 0.3

        errors:
          rate: 0.02
          subtypes:
            wrong_account: 0.5
            wrong_period: 0.3
            wrong_amount: 0.2

        statistical:
          rate: 0.01
          subtypes:
            outlier: 0.6
            benford_violation: 0.4
```

---

## Multi-Company Generation

### From Fingerprint Companies

If the fingerprint captured multiple companies:

```yaml
generation:
  companies:
    # Use fingerprint company distribution
    source: "fingerprint"

    # Optional: filter to specific companies
    include: ["C001", "C002"]
```

### Custom Company Structure

```yaml
generation:
  companies:
    source: "custom"

    list:
      - code: "PARENT"
        name: "Parent Corp"
        currency: "USD"
        country: "US"
        volume_weight: 0.4

      - code: "SUB_EU"
        name: "European Subsidiary"
        currency: "EUR"
        country: "DE"
        volume_weight: 0.35

      - code: "SUB_ASIA"
        name: "Asian Subsidiary"
        currency: "JPY"
        country: "JP"
        volume_weight: 0.25

    # Intercompany transactions
    intercompany:
      enabled: true
      rate: 0.15  # 15% of transactions are IC
```

---

## Output Formats

### CSV Output

```yaml
output:
  format: "csv"
  path: "./synthetic_data/"

  csv:
    delimiter: ","
    quote_char: '"'
    include_header: true
    null_value: ""
```

### Parquet Output

```yaml
output:
  format: "parquet"
  path: "./synthetic_data/"

  parquet:
    compression: "snappy"  # snappy, gzip, lz4, zstd
    row_group_size: 100000
```

### JSON Output

```yaml
output:
  format: "json"
  path: "./synthetic_data/"

  json:
    style: "records"  # records, lines (jsonl)
    pretty: false
```

### Multiple Formats

```yaml
output:
  path: "./synthetic_data/"

  formats:
    - format: "csv"
      path: "./csv/"
    - format: "parquet"
      path: "./parquet/"
```

---

## Performance Tuning

### Parallel Generation

```yaml
generation:
  performance:
    # Number of worker threads
    threads: 8

    # Batch size for generation
    batch_size: 10000

    # Memory limit
    max_memory_mb: 4096
```

### Streaming Output

For very large datasets:

```yaml
generation:
  performance:
    streaming: true
    flush_interval: 100000  # Flush every 100K records
```

### Progress Reporting

```bash
# Enable progress bar
datasynth-data generate \
    --fingerprint ./fp.dsf \
    --output ./out/ \
    --progress

# Output:
# Generating synthetic data...
# [████████████████████████████████░░░░░░░░] 78% (780,000/1,000,000)
# ETA: 2m 15s
```

---

## Validation During Generation

### Runtime Validation

```yaml
generation:
  validation:
    # Validate during generation
    runtime: true

    # Checks to perform
    checks:
      - balance_equations
      - referential_integrity
      - temporal_ordering

    # Action on failure
    on_failure: "warn"  # warn, skip, abort
```

### Post-Generation Validation

```bash
# Automatically validate after generation
datasynth-data generate \
    --fingerprint ./fp.dsf \
    --output ./out/ \
    --validate

# Or separately
datasynth-data validate ./out/
```

---

## Reproducibility

### Seed Management

```yaml
generation:
  # Global seed
  seed: 42

  # Per-table seeds (optional, derived from global if not specified)
  table_seeds:
    journal_entries: 42001
    vendors: 42002
    customers: 42003
```

### Versioning

```bash
# Include generation metadata
datasynth-data generate \
    --fingerprint ./fp.dsf \
    --output ./out/ \
    --include-metadata

# Creates: ./out/generation_metadata.json
# {
#   "fingerprint": "fp.dsf",
#   "fingerprint_checksum": "sha256:abc...",
#   "seed": 42,
#   "generated_at": "2024-12-15T10:30:00Z",
#   "config": { ... }
# }
```

---

## Example: Complete Generation Config

```yaml
# production_generation.yaml

fingerprint:
  path: "./erp_fingerprint_v2.dsf"
  verify_checksum: true

output:
  path: "./synthetic_erp_data/"
  format: "parquet"

  parquet:
    compression: "snappy"

  splitting:
    enabled: true
    max_rows_per_file: 500000

generation:
  scale: 1.0
  seed: 20241215

  date_range:
    start: "2025-01-01"
    end: "2025-12-31"

  threads: 8

  correlations:
    enabled: true
    method: "copula"

  rules:
    balance_equations: true
    referential_integrity: true
    temporal_ordering: true

  anomalies:
    source: "fingerprint"
    rate_multiplier: 1.0

  validation:
    runtime: true
    checks:
      - balance_equations
      - referential_integrity

overrides:
  companies:
    - code: "SYNTH"
      name: "Synthetic Corp"
      currency: "USD"
      country: "US"
      volume_weight: 1.0
```

---

## Troubleshooting

### "Fingerprint schema mismatch"

```
Error: Fingerprint expects table 'vendors' but no matching generator found.
```

**Solution**: Ensure all fingerprint tables have corresponding generators. Check if any tables were suppressed in the fingerprint.

### "Correlation target not achievable"

```
Warning: Cannot achieve correlation 0.67 between amount and line_count.
         Achieved: 0.52
```

**Solution**: This happens when the marginal distributions constrain possible correlations. Consider:
- Using copula-based generation
- Relaxing the correlation tolerance
- Adjusting distribution parameters

### "Memory limit exceeded"

```
Error: Generation exceeded memory limit of 4096 MB.
```

**Solution**: Enable streaming mode or increase memory:

```yaml
generation:
  performance:
    streaming: true
    max_memory_mb: 8192
```

---

## Next Steps

- [Privacy Configuration](./04-privacy-configuration.md): Understanding privacy settings
- [Fidelity Tuning](./05-fidelity-tuning.md): Improving synthetic data quality
- [Integration Patterns](./06-integration-patterns.md): Common integration scenarios
