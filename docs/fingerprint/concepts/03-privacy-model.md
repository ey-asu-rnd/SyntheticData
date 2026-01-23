# Privacy Model: Guarantees and Mechanisms

## Overview

The fingerprint system is designed with **privacy by design** principles. This document explains the privacy guarantees, the mechanisms used to achieve them, and how to configure privacy settings.

---

## Privacy Threats

Before discussing protections, it's important to understand the threats we're defending against:

### 1. Direct Disclosure

**Threat**: Individual records or values appear in the fingerprint.

```
VULNERABLE:
  rare_values:
    - "John Smith"      # Direct disclosure!
    - "$1,234,567.89"   # Unique amount reveals individual
```

**Protection**: Fingerprints contain only aggregate statistics, never individual values.

### 2. Inference Attack

**Threat**: Attacker deduces sensitive information from aggregate statistics.

```
EXAMPLE:
  If fingerprint says:
    department_X_average_salary: $247,000
    department_X_count: 1

  Attacker knows: The single person in dept X earns $247,000
```

**Protection**: Minimum group sizes (k-anonymity) and noise injection (differential privacy).

### 3. Reconstruction Attack

**Threat**: Attacker reconstructs individual records from detailed statistics.

```
EXAMPLE:
  If fingerprint has very fine-grained histograms:
    amount_bin_1234567_to_1234568: count=1

  Attacker knows: Exactly one transaction of ~$1,234,567
```

**Protection**: Coarse histogram bins, noise injection, outlier suppression.

### 4. Membership Inference

**Threat**: Attacker determines whether a specific individual is in the dataset.

```
EXAMPLE:
  Attacker has prior knowledge: "Alice typically transacts $50,000/month"
  Fingerprint shows spike at $50,000
  Attacker infers: "Alice is likely in this dataset"
```

**Protection**: Differential privacy provides formal membership hiding guarantees.

### 5. Linkage Attack

**Threat**: Combining fingerprint with external data to identify individuals.

```
EXAMPLE:
  Fingerprint: 3 employees in (Engineering, 30-35, PhD)
  LinkedIn: Only 3 people match this profile at Company X
  Attack: Can now link fingerprint data to real identities
```

**Protection**: Suppress rare quasi-identifier combinations.

---

## Privacy Mechanisms

### 1. Differential Privacy

Differential privacy provides a mathematical guarantee that the fingerprint doesn't reveal too much about any individual.

#### Definition

A mechanism M satisfies ε-differential privacy if for any two datasets D and D' that differ in one record, and any output S:

```
P(M(D) ∈ S) ≤ e^ε × P(M(D') ∈ S)
```

This means: Adding or removing any single record changes the output probability by at most a factor of e^ε.

#### Epsilon (ε) Values

| ε Value | Privacy Level | Use Case |
|---------|---------------|----------|
| 0.1 | Very High | Highly sensitive data (medical, financial PII) |
| 0.5 | High | Sensitive business data |
| 1.0 | Moderate | General enterprise data (recommended default) |
| 2.0 | Lower | Less sensitive data, higher utility needed |
| 5.0+ | Minimal | Near-public data |

#### Noise Mechanisms

**Laplace Mechanism** (for numeric values):

```
noised_value = true_value + Laplace(0, sensitivity/ε)
```

Used for: Row counts, sums, averages

**Exponential Mechanism** (for selection):

```
P(select x) ∝ exp(ε × utility(x) / (2 × sensitivity))
```

Used for: Selecting distribution type, percentile values

**Randomized Response** (for categorical):

```
With probability p: report true value
With probability 1-p: report random value
```

Used for: Category frequencies

#### Implementation in Fingerprints

```yaml
privacy:
  differential_privacy:
    enabled: true
    epsilon: 1.0

    # Budget allocation
    budget_allocation:
      row_counts: 0.2      # 20% of budget
      distributions: 0.4    # 40% of budget
      percentiles: 0.2      # 20% of budget
      correlations: 0.2     # 20% of budget

    # Sensitivity bounds (used to calibrate noise)
    sensitivity:
      row_count: 1          # Adding one row changes count by 1
      sum_amount: 1000000   # Max single transaction amount
      average_amount: 10000 # Bounded contribution per record
```

### 2. K-Anonymity

Every combination of quasi-identifiers must appear at least k times.

#### Quasi-Identifiers

Fields that, when combined, could identify individuals:

```yaml
quasi_identifiers:
  - department
  - cost_center
  - posting_date
  - amount_range
```

#### Enforcement

```yaml
privacy:
  k_anonymity:
    enabled: true
    k: 5  # Minimum group size

    # Actions when k is violated
    on_violation:
      strategy: "suppress_or_generalize"

      suppression:
        # Remove entire group from fingerprint
        threshold: 3  # Suppress if count < 3

      generalization:
        # Merge into broader category
        department: "merge_to_other"
        amount: "widen_bin"
        date: "generalize_to_month"
```

#### Example

```
BEFORE k-anonymity (k=5):
  Engineering + Q1 + High: count=3  ❌ Too small
  Engineering + Q1 + Medium: count=47 ✓
  Engineering + Q2 + High: count=2  ❌ Too small

AFTER k-anonymity:
  Engineering + Q1 + High: SUPPRESSED
  Engineering + Q1 + Medium: count=47 ✓
  Engineering + Q2 + High: SUPPRESSED

  OR (with generalization):
  Engineering + Q1 + [High,Medium]: count=50 ✓  (merged)
```

### 3. Outlier Handling

Extreme values can reveal individuals even in aggregate statistics.

#### Strategies

**Winsorization** (Cap extreme values before computing stats):

```yaml
outlier_handling:
  strategy: "winsorize"
  lower_percentile: 1    # Cap at 1st percentile
  upper_percentile: 99   # Cap at 99th percentile
```

**Exclusion** (Remove outliers from statistics):

```yaml
outlier_handling:
  strategy: "exclude"
  method: "iqr"
  multiplier: 3.0  # Exclude if > Q3 + 3*IQR or < Q1 - 3*IQR
```

**Separate Reporting** (Aggregate outliers):

```yaml
outlier_handling:
  strategy: "separate"
  # Report: "X outliers excluded, range [min, max]"
  # But don't report individual outlier values
```

### 4. Suppression Rules

Certain data elements are always suppressed regardless of statistics.

```yaml
suppression:
  # Never include these fields in fingerprint
  always_suppress:
    - "employee.ssn"
    - "customer.email"
    - "vendor.contact_phone"
    - "*.password"
    - "*.api_key"

  # Suppress if appears in fewer than N records
  minimum_occurrence: 10

  # Suppress rare categories
  rare_category:
    threshold: 0.01  # < 1% of records
    action: "merge_to_other"
```

### 5. Aggregation Requirements

Certain statistics require minimum sample sizes.

```yaml
aggregation:
  minimum_samples:
    mean: 30        # Need 30+ records for mean
    std_dev: 30     # Need 30+ records for std dev
    percentiles: 100 # Need 100+ records for percentiles
    correlation: 50  # Need 50+ pairs for correlation

  # What to do if insufficient samples
  on_insufficient:
    strategy: "omit"  # Don't include statistic
    # OR
    strategy: "generalize"  # Use parent category
```

---

## Privacy Configuration

### Basic Configuration

```yaml
# privacy_config.yaml

privacy:
  # Overall privacy level (convenience preset)
  level: "standard"  # Options: minimal, standard, high, maximum

  # Detailed settings (override level defaults)
  differential_privacy:
    enabled: true
    epsilon: 1.0

  k_anonymity:
    enabled: true
    k: 5

  suppression:
    enabled: true
    min_group_size: 5

  outlier_handling:
    enabled: true
    strategy: "winsorize"
    percentiles: [1, 99]
```

### Privacy Level Presets

| Level | ε | k | Outlier | Use Case |
|-------|---|---|---------|----------|
| `minimal` | 5.0 | 3 | Exclude 0.1% | Near-public data |
| `standard` | 1.0 | 5 | Winsorize 1-99% | General enterprise (default) |
| `high` | 0.5 | 10 | Winsorize 2-98% | Sensitive business data |
| `maximum` | 0.1 | 20 | Winsorize 5-95% | Highly sensitive (PII, medical) |

### Advanced Configuration

```yaml
privacy:
  differential_privacy:
    enabled: true
    epsilon: 1.0

    # Per-component budget allocation
    budget:
      schema: 0.1
      row_counts: 0.15
      distributions: 0.35
      percentiles: 0.15
      correlations: 0.15
      rules: 0.1

    # Sensitivity bounds for numeric columns
    sensitivity_bounds:
      "transactions.amount":
        max_value: 10000000  # $10M cap
        contribution_limit: 1000000  # Max contribution per entity
      "line_items.quantity":
        max_value: 10000

  k_anonymity:
    enabled: true
    k: 5

    quasi_identifiers:
      - "company_code"
      - "department"
      - "cost_center"
      - "fiscal_period"

    generalization_hierarchy:
      fiscal_period:
        - period  # Most specific
        - quarter
        - year    # Most general
      department:
        - department
        - division
        - company   # Most general

  suppression:
    always_suppress:
      - "*.ssn"
      - "*.email"
      - "*.phone"
      - "*.address"
      - "*.name"

    field_patterns:
      - pattern: ".*_id$"
        action: "hash"  # Replace with hash indicator, not actual IDs
      - pattern: ".*_name$"
        action: "suppress"

  outlier_handling:
    global:
      strategy: "winsorize"
      lower: 1
      upper: 99

    per_column:
      "transactions.amount":
        strategy: "exclude"
        method: "iqr"
        multiplier: 3.0
```

---

## Privacy Audit

Every fingerprint includes a privacy audit trail documenting all privacy measures applied.

### Audit Contents

```json
{
  "privacy_audit": {
    "generated_at": "2024-12-15T10:30:00Z",
    "tool_version": "1.0.0",
    "configuration": {
      "epsilon": 1.0,
      "k": 5,
      "outlier_strategy": "winsorize"
    },

    "source_summary": {
      "total_tables": 15,
      "total_rows": 1247832,
      "date_range": ["2020-01-01", "2024-12-31"]
    },

    "privacy_actions": [
      {
        "action": "differential_privacy_noise",
        "target": "row_counts",
        "epsilon_spent": 0.15,
        "noise_scale": 6.67
      },
      {
        "action": "k_anonymity_suppression",
        "target": "department_X + cost_center_Y",
        "original_count": 3,
        "action_taken": "merged_to_other"
      },
      {
        "action": "outlier_winsorization",
        "target": "transactions.amount",
        "records_affected": 1247,
        "percentile_bounds": [1, 99]
      },
      {
        "action": "field_suppression",
        "target": "employee.ssn",
        "reason": "always_suppress_rule"
      }
    ],

    "epsilon_budget": {
      "total": 1.0,
      "spent": 0.87,
      "remaining": 0.13
    },

    "checks_passed": [
      {
        "check": "no_individual_values",
        "status": "passed"
      },
      {
        "check": "minimum_group_sizes",
        "status": "passed",
        "details": "All groups have n >= 5"
      },
      {
        "check": "epsilon_budget",
        "status": "passed",
        "details": "0.87 <= 1.0"
      },
      {
        "check": "no_quasi_identifier_uniqueness",
        "status": "passed"
      }
    ],

    "warnings": [
      {
        "level": "info",
        "message": "12 rare categories merged into 'Other'",
        "affected_columns": ["vendor_country", "expense_type"]
      }
    ],

    "certification": {
      "privacy_compliant": true,
      "epsilon_differential_privacy": 1.0,
      "k_anonymity_level": 5,
      "recommended_use": "General enterprise analytics"
    }
  }
}
```

### Validation Command

```bash
# Validate privacy compliance of a fingerprint
datasynth-fingerprint validate ./fingerprint.dsf

# Output:
# ✓ Privacy audit found
# ✓ No individual values detected
# ✓ All group sizes >= 5 (k-anonymity satisfied)
# ✓ Epsilon budget: 0.87 / 1.0 (within limit)
# ✓ No quasi-identifier uniqueness
# ✓ All required fields suppressed
#
# Privacy Status: COMPLIANT
# Recommended Use: General enterprise analytics
```

---

## Privacy vs. Fidelity Trade-offs

Higher privacy comes at the cost of statistical fidelity:

| Setting | Privacy | Fidelity | Trade-off |
|---------|---------|----------|-----------|
| ε = 0.1 | ★★★★★ | ★★☆☆☆ | High noise, distributions less accurate |
| ε = 1.0 | ★★★★☆ | ★★★★☆ | Balanced (recommended) |
| ε = 5.0 | ★★☆☆☆ | ★★★★★ | Low noise, near-exact statistics |
| k = 20 | ★★★★★ | ★★★☆☆ | Many categories suppressed |
| k = 5 | ★★★★☆ | ★★★★☆ | Balanced (recommended) |
| k = 3 | ★★★☆☆ | ★★★★★ | Few suppressions |

### Tuning Recommendations

```yaml
# High-sensitivity data (medical, financial PII)
privacy:
  level: "maximum"
  differential_privacy:
    epsilon: 0.1
  k_anonymity:
    k: 20

# Standard enterprise data
privacy:
  level: "standard"
  differential_privacy:
    epsilon: 1.0
  k_anonymity:
    k: 5

# Near-public or internal testing
privacy:
  level: "minimal"
  differential_privacy:
    epsilon: 5.0
  k_anonymity:
    k: 3
```

---

## Compliance Considerations

### GDPR

The fingerprint approach aligns with GDPR's **data minimization** principle:

- No personal data in fingerprints
- Aggregated statistics are not personal data under GDPR (Recital 26)
- Differential privacy provides quantifiable privacy guarantee

### HIPAA

For healthcare data:

- Use `privacy.level: maximum`
- Ensure epsilon ≤ 0.1
- Apply additional de-identification rules per HIPAA Safe Harbor

### SOX / Financial Regulations

For financial data:

- Maintain audit trail (`privacy_audit.json`)
- Document privacy configuration
- Retain mapping of suppressions

---

## Best Practices

1. **Start with `standard` privacy level** and adjust based on data sensitivity
2. **Review the privacy audit** after every fingerprint extraction
3. **Test fidelity** with a sample dataset before production use
4. **Document your privacy configuration** for compliance audits
5. **Version your fingerprints** if privacy requirements change

---

## Next Steps

- [Fidelity Model](./04-fidelity-model.md): Understanding quality metrics
- [Privacy Configuration Guide](../guides/04-privacy-configuration.md): Hands-on configuration
- [Fingerprint Specification](../reference/01-fingerprint-spec.md): Complete format details
