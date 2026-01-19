# Anomaly Injection

Generate labeled anomalies for machine learning training.

## Overview

Anomaly injection adds realistic irregularities to generated data with full labeling for supervised learning:

- 20+ fraud types
- Error patterns
- Process issues
- Statistical outliers
- Relational anomalies

## Configuration

```yaml
anomaly_injection:
  enabled: true
  total_rate: 0.02                   # 2% anomaly rate
  generate_labels: true              # Output ML labels

  categories:                        # Category distribution
    fraud: 0.25
    error: 0.40
    process_issue: 0.20
    statistical: 0.10
    relational: 0.05

  temporal_pattern:
    year_end_spike: 1.5              # More anomalies at year-end

  clustering:
    enabled: true
    cluster_probability: 0.2         # 20% appear in clusters
```

## Anomaly Categories

### Fraud Types

| Type | Description | Detection Difficulty |
|------|-------------|----------------------|
| `fictitious_transaction` | Fabricated entries | Medium |
| `revenue_manipulation` | Premature recognition | Hard |
| `expense_capitalization` | Improper capitalization | Medium |
| `split_transaction` | Split to avoid threshold | Easy |
| `round_tripping` | Circular transactions | Hard |
| `kickback_scheme` | Vendor kickbacks | Hard |
| `ghost_employee` | Non-existent payee | Medium |
| `duplicate_payment` | Same invoice twice | Easy |
| `unauthorized_discount` | Unapproved discounts | Medium |
| `suspense_abuse` | Hide in suspense | Hard |

```yaml
fraud:
  types:
    fictitious_transaction: 0.15
    split_transaction: 0.20
    duplicate_payment: 0.15
    ghost_employee: 0.10
    kickback_scheme: 0.10
    revenue_manipulation: 0.10
    expense_capitalization: 0.10
    unauthorized_discount: 0.10
```

### Error Types

| Type | Description |
|------|-------------|
| `duplicate_entry` | Same entry posted twice |
| `reversed_amount` | Debit/credit swapped |
| `wrong_period` | Posted to wrong period |
| `wrong_account` | Incorrect GL account |
| `missing_reference` | Missing document reference |
| `incorrect_tax_code` | Wrong tax calculation |
| `misclassification` | Wrong account category |

### Process Issues

| Type | Description |
|------|-------------|
| `late_posting` | Posted after cutoff |
| `skipped_approval` | Missing required approval |
| `threshold_manipulation` | Amount just below threshold |
| `missing_documentation` | No supporting document |
| `out_of_sequence` | Documents out of order |

### Statistical Anomalies

| Type | Description |
|------|-------------|
| `unusual_amount` | Significant deviation from mean |
| `trend_break` | Sudden pattern change |
| `benford_violation` | Doesn't follow Benford's Law |
| `outlier_value` | Extreme value |

### Relational Anomalies

| Type | Description |
|------|-------------|
| `circular_transaction` | A → B → A flow |
| `dormant_account_activity` | Inactive account used |
| `unusual_counterparty` | Unexpected entity pairing |

## Injection Strategies

### Amount Manipulation

```yaml
anomaly_injection:
  strategies:
    amount:
      enabled: true
      threshold_adjacent: 0.3        # Just below approval limit
      round_number_bias: 0.4         # Suspicious round amounts
```

**Threshold-adjacent:** Amounts like $9,999 when limit is $10,000.

### Date Manipulation

```yaml
anomaly_injection:
  strategies:
    date:
      enabled: true
      weekend_bias: 0.2              # Weekend postings
      after_hours_bias: 0.15         # After business hours
```

### Duplication

```yaml
anomaly_injection:
  strategies:
    duplication:
      enabled: true
      exact_duplicate: 0.5           # Exact copy
      near_duplicate: 0.3            # Slight variations
      delayed_duplicate: 0.2         # Same entry later
```

## Temporal Patterns

Anomalies can follow realistic patterns:

```yaml
anomaly_injection:
  temporal_pattern:
    month_end_spike: 1.2             # 20% more at month-end
    quarter_end_spike: 1.5           # 50% more at quarter-end
    year_end_spike: 2.0              # Double at year-end
    seasonality: true                # Follow industry patterns
```

## Entity Targeting

Control which entities receive anomalies:

```yaml
anomaly_injection:
  entity_targeting:
    strategy: weighted               # random, repeat_offender, weighted

    repeat_offender:
      enabled: true
      rate: 0.4                      # 40% from same users

    high_volume_bias: 0.3            # Target high-volume entities
```

## Clustering

Real anomalies often cluster:

```yaml
anomaly_injection:
  clustering:
    enabled: true
    cluster_probability: 0.2         # 20% in clusters
    cluster_size:
      min: 3
      max: 10
    cluster_timespan_days: 30        # Within 30-day window
```

## Output Labels

### anomaly_labels.csv

| Field | Description |
|-------|-------------|
| `document_id` | Entry reference |
| `anomaly_id` | Unique anomaly ID |
| `anomaly_type` | Specific type |
| `anomaly_category` | Fraud, Error, etc. |
| `severity` | Low, Medium, High |
| `detection_difficulty` | Easy, Medium, Hard |
| `description` | Human-readable description |

### fraud_labels.csv

Subset with fraud-specific fields:

| Field | Description |
|-------|-------------|
| `document_id` | Entry reference |
| `fraud_type` | Specific fraud pattern |
| `perpetrator_id` | Employee ID |
| `scheme_id` | Related anomaly group |
| `amount_manipulated` | Fraud amount |

## ML Integration

### Loading Labels

```python
import pandas as pd

labels = pd.read_csv('output/labels/anomaly_labels.csv')
entries = pd.read_csv('output/transactions/journal_entries.csv')

# Merge
data = entries.merge(labels, on='document_id', how='left')
data['is_anomaly'] = data['anomaly_id'].notna()
```

### Feature Engineering

```python
# Create features
features = [
    'amount', 'line_count', 'is_round_number',
    'is_weekend', 'is_month_end', 'hour_of_day'
]

X = data[features]
y = data['is_anomaly']
```

### Train/Test Split

Labels include suggested splits:

```python
from sklearn.model_selection import train_test_split

X_train, X_test, y_train, y_test = train_test_split(
    X, y,
    test_size=0.2,
    stratify=y,  # Maintain anomaly ratio
    random_state=42
)
```

## Example Configuration

### Fraud Detection Training

```yaml
anomaly_injection:
  enabled: true
  total_rate: 0.02
  generate_labels: true

  categories:
    fraud: 1.0                       # Only fraud for focused training

  clustering:
    enabled: true
    cluster_probability: 0.3

fraud:
  enabled: true
  fraud_rate: 0.02
  types:
    split_transaction: 0.25
    duplicate_payment: 0.25
    kickback_scheme: 0.20
    ghost_employee: 0.15
    fictitious_transaction: 0.15
```

### General Anomaly Detection

```yaml
anomaly_injection:
  enabled: true
  total_rate: 0.05
  generate_labels: true

  categories:
    fraud: 0.15
    error: 0.45
    process_issue: 0.25
    statistical: 0.10
    relational: 0.05
```

## See Also

- [Configuration - Compliance](../configuration/compliance.md)
- [Fraud Detection Use Case](../use-cases/fraud-detection.md)
- [Graph Export](graph-export.md)
