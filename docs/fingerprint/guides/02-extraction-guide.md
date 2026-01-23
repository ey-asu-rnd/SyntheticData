# Extraction Guide

This guide covers all aspects of extracting fingerprints from real data sources.

---

## Supported Data Sources

### File-Based Sources

| Format | Extension | Notes |
|--------|-----------|-------|
| CSV | `.csv` | Comma, tab, pipe delimiters supported |
| Parquet | `.parquet` | Recommended for large datasets |
| JSON | `.json`, `.jsonl` | Object arrays or newline-delimited |
| Excel | `.xlsx`, `.xls` | First sheet by default |

### Database Sources

| Database | Connection String Format |
|----------|-------------------------|
| PostgreSQL | `postgresql://user:pass@host:port/database` |
| MySQL | `mysql://user:pass@host:port/database` |
| SQL Server | `mssql://user:pass@host:port/database` |
| Oracle | `oracle://user:pass@host:port/service` |
| SQLite | `sqlite:///path/to/file.db` |

---

## Basic Extraction

### From Files

```bash
# Single file
datasynth-fingerprint extract \
    --input ./journal_entries.csv \
    --output ./fingerprint.dsf

# Directory of files
datasynth-fingerprint extract \
    --input ./data_directory/ \
    --output ./fingerprint.dsf

# Specific files
datasynth-fingerprint extract \
    --input ./data/journal_entries.csv \
    --input ./data/vendors.csv \
    --input ./data/customers.csv \
    --output ./fingerprint.dsf
```

### From Database

```bash
# All tables
datasynth-fingerprint extract \
    --connection "postgresql://user:pass@localhost/erp" \
    --output ./fingerprint.dsf

# Specific tables
datasynth-fingerprint extract \
    --connection "postgresql://user:pass@localhost/erp" \
    --tables "journal_entries,vendors,customers" \
    --output ./fingerprint.dsf

# With query filter
datasynth-fingerprint extract \
    --connection "postgresql://user:pass@localhost/erp" \
    --tables "journal_entries" \
    --where "posting_date >= '2023-01-01'" \
    --output ./fingerprint.dsf
```

---

## Configuration File

For complex extractions, use a configuration file:

```yaml
# extraction.yaml

# Input configuration
input:
  # File-based input
  type: "files"
  path: "./data/"
  format: "auto"  # auto-detect from extension

  # OR database input
  # type: "database"
  # connection: "postgresql://user:pass@localhost/erp"
  # tables:
  #   - "journal_entries"
  #   - "vendors"
  #   - "customers"
  # where:
  #   journal_entries: "posting_date >= '2023-01-01'"

  # File selection (for directory input)
  include:
    - "journal_entries.csv"
    - "vendors.csv"
    - "customers.csv"
  exclude:
    - "*_backup.csv"
    - "temp_*.csv"

  # Column mappings (rename columns)
  column_mappings:
    journal_entries:
      "doc_id": "document_id"
      "post_dt": "posting_date"
      "amt": "amount"

  # Column type hints (override auto-detection)
  type_hints:
    journal_entries:
      document_id: "uuid"
      posting_date: "date"
      amount: "decimal"
      company_code: "categorical"

# Output configuration
output:
  path: "./fingerprint.dsf"

  # Optional: compression
  compression: "gzip"

  # Optional: encryption
  encryption:
    enabled: true
    algorithm: "AES-256-GCM"
    key_file: "./encryption.key"

# Privacy configuration
privacy:
  level: "standard"  # minimal, standard, high, maximum

  differential_privacy:
    enabled: true
    epsilon: 1.0

  k_anonymity:
    enabled: true
    k: 5

  suppression:
    always_suppress:
      - "*.ssn"
      - "*.email"
      - "employee.name"

# Extraction settings
extraction:
  # Schema extraction
  schema:
    detect_relationships: true
    infer_primary_keys: true

  # Statistical extraction
  statistics:
    numeric:
      percentiles: [1, 5, 10, 25, 50, 75, 90, 95, 99]
      detect_distribution: true
      distribution_candidates:
        - "normal"
        - "log_normal"
        - "gamma"
        - "pareto"
        - "exponential"
      benford_analysis: true
      round_number_detection: true

    categorical:
      max_categories: 100
      rare_threshold: 0.01
      semantic_detection: true  # Detect business meaning

    temporal:
      detect_seasonality: true
      detect_trends: true
      detect_patterns: true  # weekday, month-end, etc.
      granularity: "daily"

  # Correlation extraction
  correlations:
    compute_pairwise: true
    pairwise_threshold: 0.1  # Only store if |r| > 0.1
    compute_conditionals: true
    conditional_max_categories: 10
    compute_copulas: false  # Expensive, enable for high fidelity
    min_samples: 50

  # Integrity extraction
  integrity:
    detect_foreign_keys: true
    compute_cardinalities: true
    detect_temporal_ordering: true

  # Business rule inference
  rules:
    infer_balance_equations: true
    infer_thresholds: true
    infer_constraints: true
    custom_rules:
      - name: "debits_equal_credits"
        type: "balance"
        group_by: "document_id"
        columns: ["debit_amount", "credit_amount"]
      - name: "invoice_before_payment"
        type: "temporal_order"
        before: "invoice_date"
        after: "payment_date"

  # Anomaly profiling
  anomalies:
    detect_anomalies: true
    methods:
      - "isolation_forest"
      - "statistical"
    label_column: "is_fraud"  # If labeled data exists
```

Run with config:

```bash
datasynth-fingerprint extract --config extraction.yaml
```

---

## Schema Extraction

### Automatic Detection

The extractor automatically detects:

- **Column names and types**: From file headers or database schema
- **Primary keys**: Unique columns with naming patterns (e.g., `*_id`)
- **Foreign keys**: Matching column names across tables
- **Constraints**: NOT NULL, unique values

### Relationship Detection

```yaml
# Detected automatically based on column names
relationships:
  - from:
      table: "journal_entries"
      column: "vendor_id"
    to:
      table: "vendors"
      column: "vendor_id"
    type: "many_to_one"
    coverage: 0.77  # 77% of JEs have vendor_id

# Manual relationship hints
extraction:
  schema:
    relationship_hints:
      - from: "orders.customer_code"
        to: "customers.code"
      - from: "line_items.order_id"
        to: "orders.id"
```

### Column Type Mapping

| Detected Type | Fingerprint Type | Statistics Computed |
|---------------|------------------|---------------------|
| integer, bigint | `integer` | Distribution, percentiles |
| float, double, decimal | `numeric` | Distribution, percentiles, Benford |
| varchar, text | `categorical` | Frequencies, cardinality |
| date, timestamp | `temporal` | Range, patterns, seasonality |
| boolean | `boolean` | True/false ratio |
| uuid | `identifier` | Cardinality only |

---

## Statistical Extraction

### Numeric Columns

For each numeric column, the extractor computes:

```yaml
statistics:
  numeric_columns:
    - column: "journal_entries.amount"

      # Distribution fitting
      distribution:
        type: "log_normal"
        parameters:
          mu: 7.234
          sigma: 1.89
        goodness_of_fit:
          ks_statistic: 0.023
          p_value: 0.87

      # Percentiles (with DP noise)
      percentiles:
        p1: 12.50
        p5: 45.00
        p10: 78.30
        p25: 234.50
        p50: 1247.00
        p75: 5832.00
        p90: 23450.00
        p95: 47250.00
        p99: 187500.00

      # Benford's Law analysis
      benford:
        observed: [0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046]
        expected: [0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046]
        mad: 0.008
        compliant: true

      # Round number analysis
      round_numbers:
        rate: 0.15
        common_values:
          1000: 0.04
          5000: 0.03
          10000: 0.02

      # Null rate
      null_rate: 0.0
```

### Categorical Columns

For each categorical column:

```yaml
statistics:
  categorical_columns:
    - column: "journal_entries.business_process"

      # Cardinality
      cardinality: 12

      # Frequency distribution (anonymized)
      frequencies:
        category_1: 0.342  # Actual label not stored if sensitive
        category_2: 0.234
        category_3: 0.156
        category_4: 0.089
        category_5: 0.067
        category_6: 0.045
        other: 0.067  # Merged rare categories

      # Semantic hints (if detected)
      semantic:
        domain: "accounting_process"
        pattern: "business_process_enum"

      # Null rate
      null_rate: 0.02
```

### Temporal Columns

For each temporal column:

```yaml
statistics:
  temporal_columns:
    - column: "journal_entries.posting_date"

      # Range
      range:
        min: "2020-01-01"
        max: "2024-12-31"

      # Granularity
      granularity: "daily"

      # Weekday distribution
      weekday:
        monday: 0.18
        tuesday: 0.19
        wednesday: 0.20
        thursday: 0.19
        friday: 0.18
        saturday: 0.03
        sunday: 0.03

      # Period patterns
      patterns:
        month_end_spike: 2.3
        quarter_end_spike: 3.1
        year_end_spike: 4.7

      # Seasonality
      seasonality:
        detected: true
        period: 12  # months
        coefficients: [0.92, 0.95, 1.02, 0.98, 1.01, 0.97, 0.89, 0.91, 1.05, 1.08, 1.12, 1.10]

      # Trend
      trend:
        detected: true
        type: "linear"
        slope: 0.02  # 2% monthly growth
```

---

## Correlation Extraction

### Pairwise Correlations

```yaml
correlations:
  pairwise:
    # Numeric × Numeric (Pearson)
    - columns: ["amount", "line_count"]
      type: "pearson"
      value: 0.67
      p_value: 0.0001

    # Categorical × Categorical (Cramér's V)
    - columns: ["business_process", "account_type"]
      type: "cramers_v"
      value: 0.89

    # Numeric × Categorical (Point-biserial)
    - columns: ["amount", "is_month_end"]
      type: "point_biserial"
      value: 0.23
```

### Conditional Distributions

```yaml
correlations:
  conditionals:
    - condition:
        column: "business_process"
        value: "accounts_payable"
      then:
        - column: "account_type"
          distribution:
            expense: 0.65
            asset: 0.25
            liability: 0.10
        - column: "amount"
          shift:
            type: "multiplicative"
            factor: 1.2
```

### Copulas (Advanced)

For high-fidelity correlation preservation:

```yaml
correlations:
  copulas:
    - columns: ["amount", "line_count", "approval_level"]
      type: "gaussian"
      correlation_matrix:
        - [1.00, 0.67, 0.45]
        - [0.67, 1.00, 0.52]
        - [0.45, 0.52, 1.00]
```

---

## Business Rule Inference

### Balance Equations

```yaml
rules:
  balance_constraints:
    - name: "debits_equal_credits"
      type: "sum_equals"
      group_by: "document_id"
      columns:
        sum_1: "debit_amount"
        sum_2: "credit_amount"
      tolerance: 0.01
      satisfaction_rate: 1.0
```

### Approval Thresholds

```yaml
rules:
  approval_thresholds:
    - column: "amount"
      thresholds:
        - max: 1000
          level: 1
          rate: 0.35
        - max: 10000
          level: 2
          rate: 0.42
        - max: 100000
          level: 3
          rate: 0.18
        - max: null  # unlimited
          level: 4
          rate: 0.05
```

### Temporal Ordering

```yaml
rules:
  temporal_ordering:
    - name: "invoice_before_payment"
      before: "invoice_date"
      after: "payment_date"
      satisfaction_rate: 0.998
      lag_distribution:
        type: "gamma"
        shape: 2.5
        scale: 12.3  # days
```

---

## Anomaly Profiling

### Automatic Detection

```yaml
anomalies:
  detection:
    method: "isolation_forest"
    contamination: 0.02  # Expected anomaly rate

  profile:
    overall_rate: 0.023

    by_type:
      statistical:
        rate: 0.015
        subtypes:
          unusual_amount: 0.40
          benford_violation: 0.30
          outlier: 0.30

      process:
        rate: 0.005
        subtypes:
          late_posting: 0.50
          missing_approval: 0.30
          out_of_sequence: 0.20

      error:
        rate: 0.003
        subtypes:
          duplicate: 0.40
          wrong_account: 0.35
          reversal: 0.25
```

### From Labels

If your data has existing labels:

```yaml
extraction:
  anomalies:
    label_column: "is_fraud"
    type_column: "fraud_type"  # Optional

# Extracted profile based on labels
anomalies:
  labeled_profile:
    fraud_rate: 0.002
    fraud_types:
      fictitious_transaction: 0.30
      duplicate_payment: 0.25
      expense_manipulation: 0.20
      kickback: 0.15
      other: 0.10
```

---

## Sampling Strategies

For very large datasets, use sampling:

```yaml
extraction:
  sampling:
    enabled: true
    method: "stratified"  # random, stratified, systematic

    # For stratified sampling
    stratify_by: ["company_code", "fiscal_year"]

    # Sample size
    target_rows: 100000  # Or percentage
    # target_percentage: 10

    # Seed for reproducibility
    seed: 42
```

---

## Performance Optimization

### Large Datasets

```yaml
extraction:
  performance:
    # Parallel processing
    threads: 8

    # Memory limits
    max_memory_mb: 4096

    # Batch processing
    batch_size: 10000

    # Streaming mode (don't load all data at once)
    streaming: true
```

### Incremental Extraction

For datasets that grow over time:

```bash
# Initial extraction
datasynth-fingerprint extract \
    --input ./data/ \
    --output ./fingerprint_v1.dsf

# Incremental update (new data only)
datasynth-fingerprint extract \
    --input ./new_data/ \
    --base ./fingerprint_v1.dsf \
    --output ./fingerprint_v2.dsf \
    --incremental
```

---

## Validation and Quality Checks

After extraction, always validate:

```bash
# Validate privacy
datasynth-fingerprint validate ./fingerprint.dsf

# Check completeness
datasynth-fingerprint info ./fingerprint.dsf --check-completeness

# Compare to previous version
datasynth-fingerprint diff ./fingerprint_v1.dsf ./fingerprint_v2.dsf
```

---

## Troubleshooting

### "Column type detection failed"

```
Warning: Could not detect type for column 'mixed_data'. Treating as string.
```

**Solution**: Use type hints in configuration:

```yaml
input:
  type_hints:
    my_table:
      mixed_data: "numeric"
```

### "Insufficient data for distribution fitting"

```
Warning: Column 'rare_amount' has only 45 values. Cannot fit distribution.
```

**Solution**: Use empirical histogram instead:

```yaml
extraction:
  statistics:
    numeric:
      min_samples_for_distribution: 100
      fallback: "histogram"
```

### "Memory limit exceeded"

```
Error: Memory limit exceeded during correlation computation.
```

**Solution**: Enable streaming mode or increase memory:

```yaml
extraction:
  performance:
    streaming: true
    max_memory_mb: 8192
```

---

## Next Steps

- [Generation Guide](./03-generation-guide.md): Using fingerprints to generate data
- [Privacy Configuration](./04-privacy-configuration.md): Fine-tuning privacy settings
- [Fingerprint Specification](../reference/01-fingerprint-spec.md): Complete format reference
