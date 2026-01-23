# Getting Started with Fingerprinting

This guide walks you through the complete fingerprinting workflow: extracting a fingerprint from real data, generating synthetic data, and validating the results.

---

## Prerequisites

- DataSynth installed (`cargo build --release`)
- Sample data to fingerprint (CSV, Parquet, or database connection)
- Basic familiarity with DataSynth configuration

---

## Quick Start (5 Minutes)

### Step 1: Extract a Fingerprint

```bash
# From CSV files
datasynth-fingerprint extract \
    --input ./real_data/ \
    --output ./my_fingerprint.dsf

# From a database
datasynth-fingerprint extract \
    --connection "postgresql://user:pass@localhost/erp" \
    --tables "journal_entries,vendors,customers" \
    --output ./my_fingerprint.dsf
```

### Step 2: Validate Privacy

```bash
datasynth-fingerprint validate ./my_fingerprint.dsf
```

Expected output:
```
✓ Privacy audit present
✓ No individual values detected
✓ All group sizes >= 5
✓ Epsilon budget: 0.87 / 1.0
✓ No quasi-identifier uniqueness

Status: PRIVACY COMPLIANT
```

### Step 3: Generate Synthetic Data

```bash
datasynth-data generate \
    --fingerprint ./my_fingerprint.dsf \
    --output ./synthetic_data/
```

### Step 4: Evaluate Fidelity

```bash
datasynth-data evaluate \
    --fingerprint ./my_fingerprint.dsf \
    --synthetic ./synthetic_data/ \
    --output ./fidelity_report.html
```

---

## Detailed Walkthrough

### Understanding Your Data

Before extracting a fingerprint, understand what you're working with:

```bash
# Preview the data structure
datasynth-fingerprint inspect ./real_data/

# Output:
# Detected files:
#   journal_entries.csv - 1,247,832 rows, 23 columns
#   vendors.csv - 4,521 rows, 15 columns
#   customers.csv - 12,847 rows, 18 columns
#
# Detected relationships:
#   journal_entries.vendor_id → vendors.vendor_id
#   journal_entries.customer_id → customers.customer_id
#
# Suggested configuration:
#   Privacy level: standard (recommended for enterprise data)
```

### Configuring Extraction

Create an extraction configuration file for fine-grained control:

```yaml
# extraction_config.yaml

input:
  type: "files"
  path: "./real_data/"
  format: "csv"

  # Specific files (optional, default: all)
  include:
    - "journal_entries.csv"
    - "vendors.csv"
    - "customers.csv"

  # Column mappings (if names differ from expected)
  column_mappings:
    journal_entries:
      "doc_id": "document_id"
      "post_date": "posting_date"

output:
  path: "./my_fingerprint.dsf"

privacy:
  level: "standard"  # minimal, standard, high, maximum

  # Override specific settings
  differential_privacy:
    epsilon: 1.0

  k_anonymity:
    k: 5

  suppression:
    # Fields to always exclude
    always_suppress:
      - "employee_ssn"
      - "customer_email"
      - "vendor_contact"

extraction:
  # Statistical extraction settings
  statistics:
    numeric:
      percentiles: [1, 5, 10, 25, 50, 75, 90, 95, 99]
      detect_distribution: true
      benford_analysis: true

    categorical:
      max_categories: 100  # Merge beyond this
      rare_threshold: 0.01  # < 1% → merge to "Other"

    temporal:
      detect_seasonality: true
      detect_trends: true
      granularity: "daily"

  # Correlation extraction
  correlations:
    compute_pairwise: true
    compute_conditionals: true
    min_samples: 50  # Need 50+ for correlation

  # Business rule inference
  rules:
    infer_balance_equations: true
    infer_thresholds: true
    infer_temporal_ordering: true
```

Run with configuration:

```bash
datasynth-fingerprint extract --config extraction_config.yaml
```

### Examining the Fingerprint

After extraction, examine what was captured:

```bash
# Summary view
datasynth-fingerprint info ./my_fingerprint.dsf

# Output:
# ═══════════════════════════════════════════════════════════════
#                     FINGERPRINT SUMMARY
# ═══════════════════════════════════════════════════════════════
#
# Created: 2024-12-15T10:30:00Z
# Source: ./real_data/ (3 files)
# Privacy: ε=1.0, k=5
#
# Schema:
#   Tables: 3
#   Columns: 56
#   Relationships: 2
#
# Statistics:
#   Numeric columns: 12 (distributions fitted)
#   Categorical columns: 28 (frequencies captured)
#   Temporal columns: 5 (patterns detected)
#
# Correlations:
#   Pairwise: 66 pairs
#   Conditionals: 8 rules
#
# Business Rules:
#   Balance equations: 1
#   Approval thresholds: 4 levels
#   Temporal orderings: 2
#
# Privacy Audit:
#   Status: COMPLIANT
#   Epsilon spent: 0.87 / 1.0
#   Suppressions: 3 fields
#   Warnings: 2 (minor)
```

```bash
# Detailed view of a specific component
datasynth-fingerprint info ./my_fingerprint.dsf --component statistics

# Output:
# ═══════════════════════════════════════════════════════════════
#                   STATISTICS FINGERPRINT
# ═══════════════════════════════════════════════════════════════
#
# Numeric Columns:
#
#   journal_entries.amount:
#     Distribution: LogNormal(μ=7.234, σ=1.89)
#     Percentiles:
#       P1: $12.50 | P25: $234.50 | P50: $1,247.00
#       P75: $5,832.00 | P99: $187,500.00
#     Benford: Compliant (MAD=0.008)
#     Round number rate: 15%
#
#   journal_entries.line_count:
#     Distribution: Empirical (60.68% two-line)
#     Range: 2-12 (88% even counts)
#
# Categorical Columns:
#
#   journal_entries.business_process:
#     Cardinality: 12
#     Top categories:
#       accounts_payable: 34.2%
#       accounts_receivable: 23.4%
#       payroll: 15.6%
#       ...
#
# Temporal Columns:
#
#   journal_entries.posting_date:
#     Range: 2020-01-01 to 2024-12-31
#     Weekday pattern: [0.18, 0.19, 0.20, 0.19, 0.18, 0.03, 0.03]
#     Month-end spike: 2.3x
#     Quarter-end spike: 3.1x
#     Year-end spike: 4.7x
#     Seasonality: 12-month cycle detected
```

### Generating Synthetic Data

#### Basic Generation

```bash
datasynth-data generate \
    --fingerprint ./my_fingerprint.dsf \
    --output ./synthetic_data/
```

#### With Options

```bash
# Generate 2x the original volume
datasynth-data generate \
    --fingerprint ./my_fingerprint.dsf \
    --output ./synthetic_data/ \
    --scale 2.0

# Generate for a different date range
datasynth-data generate \
    --fingerprint ./my_fingerprint.dsf \
    --output ./synthetic_data/ \
    --start-date 2025-01-01 \
    --end-date 2025-12-31

# Increase anomaly rate for ML training
datasynth-data generate \
    --fingerprint ./my_fingerprint.dsf \
    --output ./synthetic_data/ \
    --anomaly-rate 0.05  # 5% anomalies
```

#### With Configuration Overrides

```yaml
# generation_config.yaml

fingerprint:
  path: "./my_fingerprint.dsf"

output:
  path: "./synthetic_data/"
  format: "csv"

overrides:
  # Scale the data
  scale: 1.5

  # Shift date range
  date_range:
    start: "2025-01-01"
    end: "2025-12-31"

  # Modify anomaly injection
  anomaly_injection:
    enabled: true
    fraud_rate: 0.01  # 1% fraud (higher for ML training)

  # Add additional companies
  companies:
    - code: "C100"
      name: "Synthetic Corp A"
      currency: "USD"
      volume_weight: 0.5
    - code: "C200"
      name: "Synthetic Corp B"
      currency: "EUR"
      volume_weight: 0.5
```

```bash
datasynth-data generate --config generation_config.yaml
```

### Evaluating Fidelity

After generation, validate the synthetic data matches the fingerprint:

```bash
datasynth-data evaluate \
    --fingerprint ./my_fingerprint.dsf \
    --synthetic ./synthetic_data/ \
    --output ./fidelity_report.html \
    --detailed
```

Output:
```
═══════════════════════════════════════════════════════════════
                    FIDELITY EVALUATION
═══════════════════════════════════════════════════════════════

Overall Score: 94.7% (Grade: A)

Dimension Scores:
  Statistical:  96.2%
  Structural:  100.0%
  Correlation:  91.5%
  Rules:        98.3%
  Anomalies:    87.4%

Key Findings:
  ✓ 35/35 metrics passed
  ⚠ 3 warnings (non-critical)

Recommendations:
  1. [MEDIUM] Consider enabling copula-based generation for
     amount × line_count correlation (current: 0.52, target: 0.67)

Full report: ./fidelity_report.html
```

### Iterating on Quality

If fidelity is not satisfactory, iterate:

```bash
# 1. Review recommendations
open ./fidelity_report.html

# 2. Adjust generation config based on recommendations
vim generation_config.yaml

# 3. Regenerate
datasynth-data generate --config generation_config.yaml

# 4. Re-evaluate
datasynth-data evaluate \
    --fingerprint ./my_fingerprint.dsf \
    --synthetic ./synthetic_data/ \
    --output ./fidelity_report_v2.html
```

---

## Example: Complete Workflow

### Scenario

You have an ERP database with sensitive financial data. You need to share representative data with a software vendor for testing, but cannot share real data due to GDPR.

### Step-by-Step

```bash
# 1. Connect to the database and extract fingerprint
datasynth-fingerprint extract \
    --connection "postgresql://readonly:pass@erp-db.internal/production" \
    --tables "gl_journal_entries,ap_vendors,ar_customers,gl_accounts" \
    --output ./erp_fingerprint.dsf \
    --privacy-level high  # Sensitive data

# 2. Validate privacy before sharing
datasynth-fingerprint validate ./erp_fingerprint.dsf
# ✓ Privacy compliant - safe to share

# 3. Share fingerprint with vendor (email, secure transfer, etc.)
# The .dsf file contains NO sensitive data

# --- At vendor site ---

# 4. Vendor generates synthetic test data
datasynth-data generate \
    --fingerprint ./erp_fingerprint.dsf \
    --output ./test_data/ \
    --scale 0.1  # 10% of original volume for testing

# 5. Vendor validates the synthetic data
datasynth-data evaluate \
    --fingerprint ./erp_fingerprint.dsf \
    --synthetic ./test_data/

# 6. Vendor uses synthetic data for testing
# Data has same structure, distributions, patterns
# But contains zero real records
```

---

## Common Issues

### "Insufficient data for statistics"

```
Warning: Column 'rare_category' has only 15 distinct values with
         count < 5. Statistics suppressed for privacy.
```

**Solution**: This is expected behavior for rare categories. The fingerprint is protecting privacy by not revealing information about small groups.

### "Privacy budget exhausted"

```
Error: Epsilon budget exhausted. Requested 0.3, remaining 0.1.
```

**Solution**: Either increase the total epsilon budget or reduce the number of statistics being extracted.

### "Schema mismatch during generation"

```
Error: Fingerprint expects column 'vendor_id' in table 'journal_entries'
       but generation config does not include vendors.
```

**Solution**: Ensure your generation config includes all tables and relationships present in the fingerprint.

---

## Next Steps

- [Extraction Guide](./02-extraction-guide.md): Deep dive into extraction options
- [Generation Guide](./03-generation-guide.md): Advanced generation configuration
- [Privacy Configuration](./04-privacy-configuration.md): Tuning privacy settings
- [Fidelity Tuning](./05-fidelity-tuning.md): Improving synthetic data quality
