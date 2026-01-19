# Audit Analytics

Test audit procedures and analytical tools with realistic data.

## Overview

SyntheticData generates comprehensive datasets for audit analytics:

- Complete document trails
- Known control exceptions
- Benford's Law compliant amounts
- Realistic temporal patterns

## Configuration

```yaml
global:
  seed: 42
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 12

transactions:
  target_count: 100000

  benford:
    enabled: true                    # Realistic first-digit distribution

  temporal:
    month_end_spike: 2.5
    quarter_end_spike: 3.0
    year_end_spike: 4.0

document_flows:
  p2p:
    enabled: true
    flow_rate: 0.35
    three_way_match:
      quantity_tolerance: 0.02
      price_tolerance: 0.01
  o2c:
    enabled: true
    flow_rate: 0.35

master_data:
  vendors:
    count: 200
  customers:
    count: 500

internal_controls:
  enabled: true

anomaly_injection:
  enabled: true
  total_rate: 0.03
  generate_labels: true

  categories:
    fraud: 0.20
    error: 0.50
    process_issue: 0.30

output:
  format: csv
```

## Audit Procedures

### 1. Benford's Law Analysis

Test first-digit distribution of amounts:

```python
import pandas as pd
import numpy as np
from scipy import stats

# Load data
entries = pd.read_csv('output/transactions/journal_entries.csv')

# Extract first digits
amounts = entries['debit_amount'] + entries['credit_amount']
amounts = amounts[amounts > 0]
first_digits = amounts.apply(lambda x: int(str(x)[0]))

# Calculate observed distribution
observed = first_digits.value_counts().sort_index()
observed_freq = observed / observed.sum()

# Expected Benford distribution
benford = {d: np.log10(1 + 1/d) for d in range(1, 10)}

# Chi-square test
chi_stat, p_value = stats.chisquare(
    observed.values,
    [benford[d] * observed.sum() for d in range(1, 10)]
)

print(f"Chi-square: {chi_stat:.2f}, p-value: {p_value:.4f}")
```

### 2. Three-Way Match Testing

Verify PO, GR, and Invoice alignment:

```python
# Load documents
po = pd.read_csv('output/documents/purchase_orders.csv')
gr = pd.read_csv('output/documents/goods_receipts.csv')
inv = pd.read_csv('output/documents/vendor_invoices.csv')

# Join on references
matched = po.merge(gr, left_on='po_number', right_on='po_reference')
matched = matched.merge(inv, left_on='po_number', right_on='po_reference')

# Calculate variances
matched['qty_variance'] = abs(matched['gr_quantity'] - matched['po_quantity']) / matched['po_quantity']
matched['price_variance'] = abs(matched['inv_unit_price'] - matched['po_unit_price']) / matched['po_unit_price']

# Identify exceptions
qty_exceptions = matched[matched['qty_variance'] > 0.02]
price_exceptions = matched[matched['price_variance'] > 0.01]

print(f"Quantity exceptions: {len(qty_exceptions)}")
print(f"Price exceptions: {len(price_exceptions)}")
```

### 3. Duplicate Payment Detection

Find potential duplicate payments:

```python
# Load payments and invoices
payments = pd.read_csv('output/documents/payments.csv')
invoices = pd.read_csv('output/documents/vendor_invoices.csv')

# Group by vendor and amount
potential_dups = invoices.groupby(['vendor_id', 'total_amount']).filter(
    lambda x: len(x) > 1
)

# Check payment dates
duplicates = []
for (vendor, amount), group in potential_dups.groupby(['vendor_id', 'total_amount']):
    if len(group) > 1:
        duplicates.append({
            'vendor': vendor,
            'amount': amount,
            'count': len(group),
            'invoices': group['invoice_number'].tolist()
        })

print(f"Potential duplicate payments: {len(duplicates)}")
```

### 4. Journal Entry Testing

Analyze manual journal entries:

```python
# Load entries
entries = pd.read_csv('output/transactions/journal_entries.csv')

# Filter manual entries
manual = entries[entries['source'] == 'Manual']

# Analyze characteristics
print(f"Manual entries: {len(manual)}")
print(f"Weekend entries: {manual['is_weekend'].sum()}")
print(f"Month-end entries: {manual['is_month_end'].sum()}")

# Top accounts with manual entries
top_accounts = manual.groupby('account_number').size().sort_values(ascending=False).head(10)
```

### 5. Cutoff Testing

Verify transactions recorded in correct period:

```python
# Identify late postings
entries['posting_date'] = pd.to_datetime(entries['posting_date'])
entries['document_date'] = pd.to_datetime(entries['document_date'])
entries['posting_lag'] = (entries['posting_date'] - entries['document_date']).dt.days

# Find entries posted after period end
late_postings = entries[entries['posting_lag'] > 5]
print(f"Late postings: {len(late_postings)}")

# Check year-end cutoff
year_end = entries['posting_date'].dt.year.max()
cutoff_issues = entries[
    (entries['document_date'].dt.year < year_end) &
    (entries['posting_date'].dt.year == year_end + 1)
]
```

### 6. Segregation of Duties

Check for SoD violations:

```python
# Load controls data
sod_rules = pd.read_csv('output/controls/sod_rules.csv')
entries = pd.read_csv('output/transactions/journal_entries.csv')

# Find entries with SoD violations
violations = entries[entries['sod_violation'] == True]
print(f"SoD violations: {len(violations)}")

# Analyze by conflict type
violation_types = violations.groupby('sod_conflict_type').size()
```

## Audit Analytics Dashboard

### Key Metrics

| Metric | Query | Expected |
|--------|-------|----------|
| Benford Chi-square | First-digit test | < 15.51 (p > 0.05) |
| Match exceptions | Three-way match | < 2% |
| Duplicate indicators | Amount/vendor matching | < 0.5% |
| Late postings | Document vs posting date | < 1% |
| SoD violations | Control violations | Known from labels |

### Population Statistics

```python
# Summary statistics
print("=== Audit Population Summary ===")
print(f"Total transactions: {len(entries):,}")
print(f"Total amount: ${entries['debit_amount'].sum():,.2f}")
print(f"Unique vendors: {entries['vendor_id'].nunique()}")
print(f"Unique customers: {entries['customer_id'].nunique()}")
print(f"Date range: {entries['posting_date'].min()} to {entries['posting_date'].max()}")
```

## Sampling

### Statistical Sampling

```python
from scipy import stats

# Calculate sample size for attribute testing
population_size = len(entries)
confidence_level = 0.95
tolerable_error_rate = 0.05
expected_error_rate = 0.01

# Sample size formula
z_score = stats.norm.ppf(1 - (1 - confidence_level) / 2)
sample_size = int(
    (z_score ** 2 * expected_error_rate * (1 - expected_error_rate)) /
    (tolerable_error_rate ** 2)
)

print(f"Recommended sample size: {sample_size}")

# Random sample
sample = entries.sample(n=sample_size, random_state=42)
```

### Stratified Sampling

```python
# Stratify by amount
entries['amount_stratum'] = pd.qcut(
    entries['debit_amount'] + entries['credit_amount'],
    q=5,
    labels=['Very Low', 'Low', 'Medium', 'High', 'Very High']
)

# Sample from each stratum
stratified_sample = entries.groupby('amount_stratum').apply(
    lambda x: x.sample(n=min(100, len(x)), random_state=42)
)
```

## See Also

- [Anomaly Injection](../advanced/anomaly-injection.md)
- [Document Flows](../configuration/document-flows.md)
- [SOX Compliance](sox-compliance.md)
