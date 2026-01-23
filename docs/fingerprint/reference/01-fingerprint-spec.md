# Fingerprint Specification Reference

This document provides the complete specification for the DataSynth Fingerprint (.dsf) file format.

---

## File Format Overview

A `.dsf` file is a ZIP archive containing YAML/JSON files organized as follows:

```
fingerprint.dsf (ZIP archive)
├── manifest.json           # Version, metadata, checksums
├── schema.yaml             # Schema fingerprint
├── statistics.yaml         # Statistical fingerprint
├── correlations.yaml       # Correlation fingerprint
├── integrity.yaml          # Referential integrity fingerprint
├── rules.yaml              # Business rules fingerprint
├── anomalies.yaml          # Anomaly profile fingerprint
├── privacy_audit.json      # Privacy compliance documentation
└── signature.sig           # Optional: Ed25519 signature
```

---

## Manifest (`manifest.json`)

The manifest contains metadata about the fingerprint.

```json
{
  "version": "1.0.0",
  "format": "datasynth_fingerprint",
  "created_at": "2024-12-15T10:30:00Z",
  "created_by": {
    "tool": "datasynth-fingerprint",
    "version": "0.1.0"
  },
  "source": {
    "description": "ERP General Ledger - Production",
    "hash": "sha256:abc123...",
    "tables": ["journal_entries", "vendors", "customers"],
    "total_rows": 1247832,
    "date_range": {
      "start": "2020-01-01",
      "end": "2024-12-31"
    }
  },
  "privacy": {
    "level": "standard",
    "differential_privacy": {
      "enabled": true,
      "epsilon": 1.0
    },
    "k_anonymity": {
      "enabled": true,
      "k": 5
    }
  },
  "checksums": {
    "schema.yaml": "sha256:def456...",
    "statistics.yaml": "sha256:ghi789...",
    "correlations.yaml": "sha256:jkl012...",
    "integrity.yaml": "sha256:mno345...",
    "rules.yaml": "sha256:pqr678...",
    "anomalies.yaml": "sha256:stu901...",
    "privacy_audit.json": "sha256:vwx234..."
  }
}
```

### Manifest Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | string | Yes | Fingerprint format version (semver) |
| `format` | string | Yes | Always "datasynth_fingerprint" |
| `created_at` | ISO 8601 | Yes | Creation timestamp |
| `created_by.tool` | string | Yes | Tool name |
| `created_by.version` | string | Yes | Tool version |
| `source.description` | string | No | Human-readable description |
| `source.hash` | string | Yes | Hash of source file paths (not content) |
| `source.tables` | string[] | Yes | List of source tables |
| `source.total_rows` | integer | Yes | Total row count (may be noised) |
| `source.date_range` | object | No | Date range of data |
| `privacy.level` | string | Yes | Privacy level preset used |
| `privacy.differential_privacy` | object | No | DP settings |
| `privacy.k_anonymity` | object | No | k-anonymity settings |
| `checksums` | object | Yes | SHA-256 checksums of all files |

---

## Schema Fingerprint (`schema.yaml`)

Captures the structural blueprint of the data.

```yaml
schema:
  version: "1.0"

  tables:
    - name: "journal_entries"
      row_count: 1247832  # May be noised
      row_count_noise: 100  # Noise added (for transparency)

      columns:
        - name: "document_id"
          data_type: "uuid"
          nullable: false
          unique: true
          role: "primary_key"
          semantic_type: "identifier"

        - name: "company_code"
          data_type: "string"
          nullable: false
          max_length: 10
          role: "dimension"
          semantic_type: "company_identifier"

        - name: "posting_date"
          data_type: "date"
          nullable: false
          format: "ISO8601"
          role: "temporal"
          semantic_type: "posting_date"

        - name: "amount"
          data_type: "decimal"
          nullable: false
          precision: 18
          scale: 2
          role: "measure"
          semantic_type: "monetary_amount"
          currency_column: "currency"

        - name: "currency"
          data_type: "string"
          nullable: false
          max_length: 3
          role: "attribute"
          semantic_type: "currency_code"

        - name: "vendor_id"
          data_type: "string"
          nullable: true
          null_rate: 0.23
          role: "foreign_key"
          references:
            table: "vendors"
            column: "vendor_id"
          semantic_type: "vendor_identifier"

        - name: "business_process"
          data_type: "string"
          nullable: false
          role: "dimension"
          semantic_type: "business_process"
          cardinality: 12

        - name: "is_fraud"
          data_type: "boolean"
          nullable: false
          role: "label"
          semantic_type: "fraud_indicator"

      primary_key: ["document_id"]

      indexes:
        - columns: ["posting_date"]
          type: "btree"
        - columns: ["company_code", "fiscal_period"]
          type: "btree"

    - name: "vendors"
      row_count: 4521
      columns:
        - name: "vendor_id"
          data_type: "string"
          nullable: false
          unique: true
          role: "primary_key"
        # ... additional columns

  relationships:
    - name: "je_to_vendor"
      from:
        table: "journal_entries"
        column: "vendor_id"
      to:
        table: "vendors"
        column: "vendor_id"
      type: "many_to_one"
      cardinality:
        avg_children: 47.3
        std_children: 12.8
        min_children: 1
        max_children: 523
        distribution: "negative_binomial"
        parameters:
          r: 3.2
          p: 0.063

    - name: "je_to_customer"
      from:
        table: "journal_entries"
        column: "customer_id"
      to:
        table: "customers"
        column: "customer_id"
      type: "many_to_one"
      # ... cardinality details
```

### Schema Field Definitions

#### Table Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Table name |
| `row_count` | integer | Row count (may include DP noise) |
| `row_count_noise` | integer | Noise added to row count |
| `columns` | array | Column definitions |
| `primary_key` | string[] | Primary key columns |
| `indexes` | array | Index definitions |

#### Column Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Column name |
| `data_type` | string | Data type (see Data Types) |
| `nullable` | boolean | Whether nulls are allowed |
| `null_rate` | float | Rate of null values (0.0-1.0) |
| `unique` | boolean | Whether values are unique |
| `role` | string | Column role (see Roles) |
| `semantic_type` | string | Semantic meaning (see Semantic Types) |
| `max_length` | integer | Maximum string length |
| `precision` | integer | Decimal precision |
| `scale` | integer | Decimal scale |
| `cardinality` | integer | Distinct value count |
| `references` | object | Foreign key reference |

#### Data Types

| Type | Description |
|------|-------------|
| `string` | Variable-length text |
| `integer` | Whole numbers |
| `decimal` | Fixed-precision decimals |
| `float` | Floating-point numbers |
| `boolean` | True/false |
| `date` | Date without time |
| `datetime` | Date with time |
| `timestamp` | Unix timestamp |
| `uuid` | UUID/GUID |

#### Column Roles

| Role | Description |
|------|-------------|
| `primary_key` | Primary key column |
| `foreign_key` | Foreign key column |
| `dimension` | Categorical/grouping column |
| `measure` | Numeric measure |
| `temporal` | Date/time column |
| `attribute` | Descriptive attribute |
| `label` | ML label column |
| `identifier` | Unique identifier |

#### Semantic Types

| Type | Description |
|------|-------------|
| `identifier` | Generic identifier |
| `company_identifier` | Company/entity code |
| `vendor_identifier` | Vendor ID |
| `customer_identifier` | Customer ID |
| `posting_date` | Transaction posting date |
| `document_date` | Document creation date |
| `monetary_amount` | Currency amount |
| `currency_code` | ISO currency code |
| `business_process` | Business process type |
| `account_number` | GL account number |
| `cost_center` | Cost center code |
| `fraud_indicator` | Fraud label |

---

## Statistics Fingerprint (`statistics.yaml`)

Captures distributional properties of columns.

```yaml
statistics:
  version: "1.0"

  numeric_columns:
    - column: "journal_entries.amount"
      count: 1247832
      null_rate: 0.0

      # Fitted distribution
      distribution:
        type: "mixture"
        components:
          - type: "log_normal"
            weight: 0.85
            parameters:
              mu: 7.234
              sigma: 1.89
            goodness_of_fit:
              ks_statistic: 0.023
              p_value: 0.87

          - type: "point_mass"
            weight: 0.15
            values: [1000, 5000, 10000, 50000, 100000]
            probabilities: [0.35, 0.28, 0.20, 0.12, 0.05]

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
        chi_squared: 4.23
        compliant: true

      # Round number analysis
      round_numbers:
        rate: 0.15
        by_magnitude:
          "10": 0.02
          "100": 0.04
          "1000": 0.05
          "10000": 0.03
          "100000": 0.01

    - column: "journal_entries.line_count"
      count: 1247832
      null_rate: 0.0

      distribution:
        type: "empirical"
        histogram:
          - bin: [2, 3]
            density: 0.6068
          - bin: [3, 4]
            density: 0.0
          - bin: [4, 5]
            density: 0.2245
          - bin: [5, 6]
            density: 0.0
          - bin: [6, 7]
            density: 0.1032
          # ...

      percentiles:
        p25: 2
        p50: 2
        p75: 4
        p90: 6
        p99: 10

  categorical_columns:
    - column: "journal_entries.business_process"
      count: 1247832
      null_rate: 0.02
      cardinality: 12

      # Frequency distribution
      frequencies:
        # Using anonymous category IDs for privacy
        # Actual labels stored separately if safe
        category_1: 0.342
        category_2: 0.234
        category_3: 0.156
        category_4: 0.089
        category_5: 0.067
        category_6: 0.045
        category_7: 0.032
        category_8: 0.018
        category_9: 0.010
        other: 0.007  # Merged rare categories

      # Label mapping (only if safe to include)
      labels:
        category_1: "accounts_payable"
        category_2: "accounts_receivable"
        category_3: "payroll"
        # ...

      # Semantic hints
      semantic:
        domain: "accounting_process"
        enum_like: true

    - column: "journal_entries.company_code"
      count: 1247832
      null_rate: 0.0
      cardinality: 5

      frequencies:
        "C001": 0.42
        "C002": 0.28
        "C003": 0.18
        "C004": 0.08
        "C005": 0.04

  temporal_columns:
    - column: "journal_entries.posting_date"
      count: 1247832
      null_rate: 0.0

      # Date range
      range:
        min: "2020-01-01"
        max: "2024-12-31"
        span_days: 1826

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

      # Day of month distribution
      day_of_month:
        # Higher density at month-end
        distribution_type: "empirical"
        histogram:
          - bin: [1, 5]
            density: 0.12
          - bin: [6, 10]
            density: 0.10
          - bin: [11, 15]
            density: 0.10
          - bin: [16, 20]
            density: 0.10
          - bin: [21, 25]
            density: 0.12
          - bin: [26, 31]
            density: 0.46  # Month-end spike

      # Period patterns
      patterns:
        month_end:
          window_days: 3
          spike_multiplier: 2.3
        quarter_end:
          window_days: 5
          spike_multiplier: 3.1
        year_end:
          window_days: 10
          spike_multiplier: 4.7

      # Seasonality
      seasonality:
        detected: true
        type: "multiplicative"
        period: 12  # months
        coefficients:
          january: 0.92
          february: 0.95
          march: 1.02
          april: 0.98
          may: 1.01
          june: 0.97
          july: 0.89
          august: 0.91
          september: 1.05
          october: 1.08
          november: 1.12
          december: 1.10

      # Trend
      trend:
        detected: true
        type: "linear"
        slope: 0.02  # 2% monthly growth
        intercept: 1000  # Base daily volume

      # Holidays (aggregated, not specific dates)
      holiday_effect:
        reduction_rate: 0.7  # 30% less on holidays
        affected_days_per_year: 10
```

### Distribution Types

| Type | Parameters | Description |
|------|------------|-------------|
| `normal` | `mean`, `std` | Normal/Gaussian distribution |
| `log_normal` | `mu`, `sigma` | Log-normal distribution |
| `gamma` | `shape`, `scale` | Gamma distribution |
| `exponential` | `lambda` | Exponential distribution |
| `pareto` | `alpha`, `x_min` | Pareto/power-law distribution |
| `uniform` | `min`, `max` | Uniform distribution |
| `point_mass` | `values`, `probabilities` | Discrete point masses |
| `mixture` | `components` | Mixture of distributions |
| `empirical` | `histogram` | Empirical histogram |

---

## Correlations Fingerprint (`correlations.yaml`)

Captures inter-column dependencies.

```yaml
correlations:
  version: "1.0"

  # Pairwise correlations
  pairwise:
    # Numeric × Numeric (Pearson correlation)
    numeric_numeric:
      - columns: ["journal_entries.amount", "journal_entries.line_count"]
        correlation: 0.67
        p_value: 0.0001
        sample_size: 1247832

      - columns: ["journal_entries.amount", "journal_entries.approval_level"]
        correlation: 0.45
        p_value: 0.0001
        sample_size: 1247832

    # Categorical × Categorical (Cramér's V)
    categorical_categorical:
      - columns: ["journal_entries.business_process", "journal_entries.account_type"]
        cramers_v: 0.89
        chi_squared: 45678.23
        p_value: 0.0001
        degrees_of_freedom: 55

    # Numeric × Categorical (Point-biserial / ANOVA)
    numeric_categorical:
      - columns: ["journal_entries.amount", "journal_entries.is_month_end"]
        point_biserial: 0.23
        p_value: 0.0001

      - columns: ["journal_entries.amount", "journal_entries.business_process"]
        eta_squared: 0.34
        f_statistic: 1234.56
        p_value: 0.0001

  # Conditional distributions
  conditionals:
    - condition:
        column: "journal_entries.business_process"
        value: "accounts_payable"
      then:
        - column: "journal_entries.account_type"
          distribution:
            expense: 0.65
            asset: 0.25
            liability: 0.10

        - column: "journal_entries.amount"
          shift:
            type: "multiplicative"
            factor: 1.2
            # AP amounts are 20% higher than average

    - condition:
        column: "journal_entries.company_code"
        value: "C001"
      then:
        - column: "journal_entries.currency"
          distribution:
            USD: 0.95
            EUR: 0.03
            GBP: 0.02

  # Copula specification (for high-fidelity generation)
  copulas:
    - name: "amount_line_approval"
      columns:
        - "journal_entries.amount"
        - "journal_entries.line_count"
        - "journal_entries.approval_level"
      type: "gaussian"
      correlation_matrix:
        - [1.00, 0.67, 0.45]
        - [0.67, 1.00, 0.52]
        - [0.45, 0.52, 1.00]

  # Temporal correlations
  temporal:
    - column: "journal_entries.amount"
      autocorrelation:
        lag_1: 0.23
        lag_7: 0.45  # Weekly pattern
        lag_30: 0.12  # Monthly pattern

    - columns: ["journal_entries.amount", "journal_entries.posting_date"]
      trend_correlation: 0.15
      seasonality_correlation: 0.34
```

---

## Integrity Fingerprint (`integrity.yaml`)

Captures referential integrity patterns.

```yaml
integrity:
  version: "1.0"

  foreign_keys:
    - relationship: "je_to_vendor"
      from:
        table: "journal_entries"
        column: "vendor_id"
      to:
        table: "vendors"
        column: "vendor_id"

      # Coverage statistics
      coverage: 0.77  # 77% of JEs have vendor_id
      orphan_rate: 0.0  # No orphan references

      # Cardinality patterns
      cardinality:
        # How many JEs per vendor?
        distribution:
          type: "pareto"
          parameters:
            alpha: 1.8
            x_min: 1
        percentiles:
          p50: 12
          p75: 45
          p90: 89
          p95: 187
          p99: 1247
        concentration:
          top_10_pct_share: 0.67  # Top 10% of vendors have 67% of JEs

    - relationship: "je_to_customer"
      from:
        table: "journal_entries"
        column: "customer_id"
      to:
        table: "customers"
        column: "customer_id"

      coverage: 0.45
      orphan_rate: 0.001

      cardinality:
        distribution:
          type: "negative_binomial"
          parameters:
            r: 5
            p: 0.1
        percentiles:
          p50: 8
          p90: 45
          p99: 234

  # Temporal ordering constraints
  temporal_ordering:
    - name: "invoice_before_payment"
      before:
        table: "invoices"
        column: "invoice_date"
      after:
        table: "payments"
        column: "payment_date"
      link:
        column: "invoice_id"

      satisfaction_rate: 0.998
      violation_rate: 0.002

      lag_distribution:
        type: "gamma"
        parameters:
          shape: 2.5
          scale: 12.3  # Mean ~30 days
        percentiles:
          p50: 28
          p75: 42
          p90: 67
          p99: 120

    - name: "po_before_gr"
      before:
        table: "purchase_orders"
        column: "order_date"
      after:
        table: "goods_receipts"
        column: "receipt_date"
      link:
        column: "po_number"

      satisfaction_rate: 0.999
      lag_distribution:
        type: "exponential"
        parameters:
          lambda: 0.05  # Mean 20 days
```

---

## Rules Fingerprint (`rules.yaml`)

Captures business rules and constraints.

```yaml
rules:
  version: "1.0"

  # Balance constraints (accounting equations)
  balance_constraints:
    - name: "debits_equal_credits"
      type: "sum_equals"
      scope: "per_document"
      group_by: ["document_id"]
      columns:
        sum_1: "debit_amount"
        sum_2: "credit_amount"
      tolerance: 0.01
      satisfaction_rate: 1.0  # Must be 100%

    - name: "trial_balance_balanced"
      type: "sum_equals"
      scope: "per_period"
      group_by: ["company_code", "fiscal_period"]
      columns:
        sum_1: "total_debits"
        sum_2: "total_credits"
      tolerance: 0.01
      satisfaction_rate: 1.0

  # Value constraints
  value_constraints:
    - name: "amount_positive"
      column: "journal_entries.amount"
      constraint: "value > 0"
      satisfaction_rate: 0.9997
      violation_context: "Rare reversals/adjustments"

    - name: "valid_currency"
      column: "journal_entries.currency"
      constraint: "value IN ('USD', 'EUR', 'GBP', 'JPY', 'CNY')"
      satisfaction_rate: 1.0

  # Approval thresholds
  approval_patterns:
    column: "journal_entries.amount"
    approval_column: "journal_entries.approval_level"

    thresholds:
      - level: 1
        max_amount: 1000
        rate: 0.35
        description: "Supervisor approval"

      - level: 2
        max_amount: 10000
        rate: 0.42
        description: "Manager approval"

      - level: 3
        max_amount: 100000
        rate: 0.18
        description: "Director approval"

      - level: 4
        max_amount: null  # Unlimited
        rate: 0.05
        description: "VP/CFO approval"

  # Segregation of duties
  segregation_of_duties:
    violation_rate: 0.008

    conflict_types:
      - type: "creator_approver"
        description: "Same person creates and approves"
        rate: 0.006

      - type: "preparer_reviewer"
        description: "Same person prepares and reviews"
        rate: 0.002

  # Document numbering
  document_numbering:
    - column: "journal_entries.document_id"
      pattern: "sequential_with_gaps"
      gap_rate: 0.02
      format: "JE-{YYYY}-{NNNNNN}"

  # Period constraints
  period_constraints:
    - name: "posting_in_open_period"
      posting_date: "journal_entries.posting_date"
      fiscal_period: "journal_entries.fiscal_period"
      satisfaction_rate: 0.998
      # 0.2% are late postings to prior periods
```

---

## Anomalies Fingerprint (`anomalies.yaml`)

Captures the anomaly profile for ML training.

```yaml
anomalies:
  version: "1.0"

  # Overall anomaly rate
  overall:
    rate: 0.023
    count: 28700  # Approximate, noised

  # By anomaly category
  by_category:
    fraud:
      rate: 0.002
      count: 2496

      subtypes:
        fictitious_transaction:
          rate: 0.30
          characteristics:
            amount_range: [10000, 100000]
            time_pattern: "year_end_concentration"

        duplicate_payment:
          rate: 0.25
          characteristics:
            similarity_threshold: 0.95

        expense_manipulation:
          rate: 0.20

        kickback_scheme:
          rate: 0.15

        other:
          rate: 0.10

    errors:
      rate: 0.015
      count: 18717

      subtypes:
        wrong_account:
          rate: 0.40
        wrong_period:
          rate: 0.25
        duplicate_entry:
          rate: 0.20
        reversal_error:
          rate: 0.15

    process_issues:
      rate: 0.006
      count: 7487

      subtypes:
        late_posting:
          rate: 0.50
        missing_approval:
          rate: 0.30
        threshold_manipulation:
          rate: 0.20

  # Temporal patterns
  temporal_patterns:
    year_end_spike:
      multiplier: 2.1
      window: "last_2_weeks_of_year"

    quarter_end_correlation: 0.34

    weekday_pattern:
      friday_spike: 1.3
      monday_dip: 0.8

  # Entity patterns
  entity_patterns:
    concentration:
      top_1_pct_entities: 0.23  # 1% of entities have 23% of anomalies
      repeat_offender_rate: 0.15

    vendor_correlation:
      new_vendors_elevated: true
      elevation_factor: 2.5
      new_vendor_window_days: 90

  # Amount patterns
  amount_patterns:
    threshold_adjacent:
      rate: 0.12  # 12% of fraud near approval thresholds
      tolerance: 0.05  # Within 5% of threshold

    round_number_correlation: 0.23
```

---

## Privacy Audit (`privacy_audit.json`)

Documents all privacy measures applied.

```json
{
  "privacy_audit": {
    "version": "1.0",
    "generated_at": "2024-12-15T10:30:00Z",

    "configuration": {
      "privacy_level": "standard",
      "differential_privacy": {
        "enabled": true,
        "epsilon": 1.0,
        "mechanism": "laplace"
      },
      "k_anonymity": {
        "enabled": true,
        "k": 5
      },
      "outlier_handling": {
        "strategy": "winsorize",
        "lower_percentile": 1,
        "upper_percentile": 99
      }
    },

    "source_summary": {
      "tables": 3,
      "columns": 56,
      "total_rows": 1265000,
      "date_range": ["2020-01-01", "2024-12-31"]
    },

    "actions_taken": [
      {
        "action_id": "dp_001",
        "type": "differential_privacy_noise",
        "target": "row_counts",
        "epsilon_spent": 0.15,
        "noise_mechanism": "laplace",
        "noise_scale": 6.67
      },
      {
        "action_id": "dp_002",
        "type": "differential_privacy_noise",
        "target": "percentiles",
        "epsilon_spent": 0.25,
        "noise_mechanism": "exponential"
      },
      {
        "action_id": "ka_001",
        "type": "k_anonymity_suppression",
        "target": "journal_entries.department",
        "original_categories": 47,
        "suppressed_categories": 12,
        "merged_to": "Other",
        "reason": "categories with count < 5"
      },
      {
        "action_id": "sup_001",
        "type": "field_suppression",
        "target": "vendors.contact_email",
        "reason": "always_suppress_rule (PII)"
      },
      {
        "action_id": "out_001",
        "type": "outlier_winsorization",
        "target": "journal_entries.amount",
        "records_affected": 12478,
        "lower_bound": 0.50,
        "upper_bound": 500000.00
      }
    ],

    "epsilon_budget": {
      "total": 1.0,
      "allocations": {
        "schema": 0.10,
        "row_counts": 0.15,
        "distributions": 0.35,
        "percentiles": 0.20,
        "correlations": 0.15,
        "rules": 0.05
      },
      "spent": 0.87,
      "remaining": 0.13
    },

    "checks_performed": [
      {
        "check_id": "chk_001",
        "check": "no_individual_values",
        "status": "passed",
        "details": "No raw values stored in fingerprint"
      },
      {
        "check_id": "chk_002",
        "check": "minimum_group_sizes",
        "status": "passed",
        "details": "All categorical groups have n >= 5"
      },
      {
        "check_id": "chk_003",
        "check": "epsilon_budget",
        "status": "passed",
        "details": "Total epsilon 0.87 <= budget 1.0"
      },
      {
        "check_id": "chk_004",
        "check": "no_quasi_identifier_uniqueness",
        "status": "passed",
        "details": "No unique QI combinations found"
      },
      {
        "check_id": "chk_005",
        "check": "outlier_suppression",
        "status": "passed",
        "details": "Extreme values winsorized at 1-99 percentiles"
      }
    ],

    "warnings": [
      {
        "warning_id": "warn_001",
        "level": "info",
        "message": "12 rare categories merged into 'Other'",
        "affected": ["department", "cost_center"]
      },
      {
        "warning_id": "warn_002",
        "level": "info",
        "message": "Temporal granularity reduced to weekly for small-count periods",
        "affected": ["posting_date"]
      }
    ],

    "certification": {
      "privacy_compliant": true,
      "epsilon_differential_privacy": 1.0,
      "k_anonymity_level": 5,
      "pii_fields_removed": 8,
      "recommended_use": "General enterprise analytics, ML training",
      "not_recommended_for": "Individual record reconstruction"
    }
  }
}
```

---

## Signature (`signature.sig`)

Optional Ed25519 signature over all fingerprint files.

```
-----BEGIN DATASYNTH SIGNATURE-----
Signer: CN=EU Data Team, O=Acme Corp
Algorithm: Ed25519
Signed-At: 2024-12-15T10:30:00Z
Files-Signed: manifest.json, schema.yaml, statistics.yaml, correlations.yaml,
              integrity.yaml, rules.yaml, anomalies.yaml, privacy_audit.json

Signature:
mQINBGN...base64-encoded-signature...==
-----END DATASYNTH SIGNATURE-----
```

---

## Versioning

The fingerprint format follows semantic versioning:

- **Major version**: Breaking changes to structure
- **Minor version**: New optional fields
- **Patch version**: Bug fixes, clarifications

Current version: **1.0.0**

### Version Compatibility

| Reader Version | Fingerprint 1.0.x | Fingerprint 1.1.x | Fingerprint 2.0.x |
|----------------|-------------------|-------------------|-------------------|
| 1.0.x | ✓ Full | ✓ Ignore new | ✗ Incompatible |
| 1.1.x | ✓ Full | ✓ Full | ✗ Incompatible |
| 2.0.x | ✓ Migrate | ✓ Migrate | ✓ Full |

---

## Next Steps

- [CLI Reference](./02-cli-reference.md): Command-line interface
- [API Reference](./03-api-reference.md): Rust API documentation
- [Examples](../examples/): Practical examples
