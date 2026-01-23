# Fidelity Model: Quality Metrics and Validation

## Overview

**Fidelity** measures how well synthetic data matches the statistical properties captured in the fingerprint. High fidelity means the synthetic data is a faithful representation of the original data's characteristics—without containing any of the original records.

---

## Fidelity Dimensions

Fidelity is measured across five dimensions:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         FIDELITY DIMENSIONS                                 │
│                                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │ Statistical │  │ Structural  │  │ Correlation │  │   Rule      │       │
│  │  Fidelity   │  │  Fidelity   │  │  Fidelity   │  │ Compliance  │       │
│  │             │  │             │  │             │  │             │       │
│  │ Distributions│ │ Schema      │  │ Pairwise    │  │ Balance     │       │
│  │ Percentiles │  │ Cardinality │  │ Conditional │  │ Thresholds  │       │
│  │ Patterns    │  │ Null rates  │  │ Copulas     │  │ Constraints │       │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘       │
│                                                                             │
│                          ┌─────────────┐                                   │
│                          │  Anomaly    │                                   │
│                          │  Fidelity   │                                   │
│                          │             │                                   │
│                          │ Rates       │                                   │
│                          │ Types       │                                   │
│                          │ Patterns    │                                   │
│                          └─────────────┘                                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 1. Statistical Fidelity

Measures how well synthetic data distributions match the fingerprint.

### Metrics

#### Kolmogorov-Smirnov (KS) Statistic

Measures the maximum difference between cumulative distribution functions.

```
KS = max|F_synthetic(x) - F_fingerprint(x)|
```

| KS Value | Interpretation |
|----------|----------------|
| < 0.05 | Excellent match |
| 0.05 - 0.10 | Good match |
| 0.10 - 0.20 | Acceptable |
| > 0.20 | Poor match |

```
Example:
                    CDF Comparison (Amount)
    1.0 ┤                                    ▄▄▄▄▄▄
        │                              ▄▄▄▄▄▀░░░░░░
        │                        ▄▄▄▄▀▀░░░░░░░░░░░░
    0.5 ┤                  ▄▄▄▄▀▀░░░░░░░░░░░░░░░░░░
        │            ▄▄▄▄▀▀░░░░░░░░░░░░░░░░░░░░░░░░
        │      ▄▄▄▄▀▀░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
    0.0 ┼──────────────────────────────────────────
        0       1K      10K     100K    1M
                        Amount ($)

        ▀ Fingerprint   ░ Synthetic   ↕ KS = 0.03
```

#### Wasserstein Distance (Earth Mover's Distance)

Measures the "cost" of transforming one distribution into another.

```
W = ∫|F_synthetic(x) - F_fingerprint(x)| dx
```

Lower values indicate better match. Scale-dependent, so normalize by data range.

#### Jensen-Shannon Divergence (for categorical)

Symmetric version of KL divergence, bounded between 0 and 1.

```
JSD = 0.5 × KL(P||M) + 0.5 × KL(Q||M)
where M = 0.5 × (P + Q)
```

| JSD Value | Interpretation |
|-----------|----------------|
| < 0.05 | Excellent match |
| 0.05 - 0.15 | Good match |
| 0.15 - 0.30 | Acceptable |
| > 0.30 | Poor match |

#### Benford's Law Mean Absolute Deviation

For financial amounts, measures adherence to Benford's Law.

```
MAD = (1/9) × Σ|observed_d - expected_d|
where expected_d = log10(1 + 1/d)
```

| MAD Value | Interpretation |
|-----------|----------------|
| < 0.006 | Close conformity |
| 0.006 - 0.012 | Acceptable conformity |
| 0.012 - 0.015 | Marginally acceptable |
| > 0.015 | Non-conforming |

### Statistical Fidelity Score

```
Statistical_Fidelity = (
    0.30 × (1 - avg_KS_numeric) +
    0.25 × (1 - avg_JSD_categorical) +
    0.20 × (1 - normalized_wasserstein) +
    0.15 × temporal_pattern_correlation +
    0.10 × benford_compliance
) × 100
```

---

## 2. Structural Fidelity

Measures how well the synthetic data preserves schema and structure.

### Metrics

#### Schema Match

All tables, columns, and types must match exactly.

```
Schema_Match = (matched_elements / total_elements) × 100

Requirements:
- All table names present
- All column names present
- All data types match
- All constraints preserved
```

#### Row Count Ratio

Compares synthetic row counts to fingerprint targets.

```
Row_Ratio = synthetic_count / fingerprint_count

Target: 0.95 - 1.05 (within 5%)
```

#### Null Rate Difference

Compares null rates per column.

```
Null_Diff = |synthetic_null_rate - fingerprint_null_rate|

Target: < 0.02 (within 2 percentage points)
```

#### Cardinality Ratio

Compares unique value counts.

```
Cardinality_Ratio = synthetic_unique / fingerprint_unique

Target: 0.9 - 1.1 (within 10%)
```

#### Foreign Key Coverage

Measures FK relationship preservation.

```
FK_Coverage = (valid_FK_references / total_FK_references) × 100

Target: > 99%
```

### Structural Fidelity Score

```
Structural_Fidelity = (
    0.30 × schema_match +
    0.25 × (1 - |row_ratio - 1|) +
    0.20 × (1 - avg_null_diff) +
    0.15 × (1 - |cardinality_ratio - 1|) +
    0.10 × fk_coverage
) × 100
```

---

## 3. Correlation Fidelity

Measures preservation of inter-column dependencies.

### Metrics

#### Correlation Matrix RMSE

Root mean squared error between correlation matrices.

```
RMSE = √(Σ(r_synthetic - r_fingerprint)² / n_pairs)
```

| RMSE | Interpretation |
|------|----------------|
| < 0.05 | Excellent |
| 0.05 - 0.10 | Good |
| 0.10 - 0.20 | Acceptable |
| > 0.20 | Poor |

#### Individual Correlation Differences

Track significant correlation deviations.

```
For each pair (i, j):
  diff = |r_synthetic(i,j) - r_fingerprint(i,j)|

Flag if diff > 0.15
```

#### Conditional Distribution Match

Measure preservation of conditional relationships.

```
Example: P(account_type | business_process)

For each condition:
  JSD(P_synthetic | condition) vs (P_fingerprint | condition)
```

### Correlation Fidelity Score

```
Correlation_Fidelity = (
    0.50 × (1 - correlation_rmse) +
    0.30 × (1 - pct_significant_deviations) +
    0.20 × (1 - avg_conditional_jsd)
) × 100
```

---

## 4. Rule Compliance Fidelity

Measures adherence to business rules captured in the fingerprint.

### Metrics

#### Balance Equation Compliance

For financial data, debits must equal credits.

```
Balance_Compliance = (balanced_documents / total_documents) × 100

Target: 100%
```

#### Approval Threshold Distribution

Compares approval level distributions.

```
For each threshold level:
  diff = |synthetic_pct - fingerprint_pct|

Threshold_Match = 1 - Σdiff
```

#### Temporal Ordering Compliance

For ordered events (e.g., invoice_date <= payment_date).

```
Ordering_Compliance = (valid_orderings / total_orderings) × 100

Target: Match fingerprint rate (e.g., 99.8%)
```

#### Value Constraint Compliance

Custom business rules (e.g., amount > 0).

```
Constraint_Compliance = (compliant_records / total_records) × 100

Target: Match fingerprint rate
```

### Rule Compliance Score

```
Rule_Fidelity = (
    0.35 × balance_compliance +
    0.25 × threshold_match +
    0.25 × ordering_compliance +
    0.15 × constraint_compliance
) × 100
```

---

## 5. Anomaly Fidelity

Measures reproduction of anomaly patterns.

### Metrics

#### Overall Anomaly Rate

```
Rate_Diff = |synthetic_anomaly_rate - fingerprint_anomaly_rate|

Target: < 0.005 (within 0.5 percentage points)
```

#### Anomaly Type Distribution

```
For each anomaly type:
  JSD(synthetic_type_dist, fingerprint_type_dist)

Target: JSD < 0.10
```

#### Temporal Pattern Correlation

Compares anomaly occurrence patterns over time.

```
Pattern_Correlation = corr(synthetic_anomaly_time_series, fingerprint_pattern)

Target: > 0.85
```

### Anomaly Fidelity Score

```
Anomaly_Fidelity = (
    0.40 × (1 - rate_diff × 100) +
    0.35 × (1 - type_jsd) +
    0.25 × pattern_correlation
) × 100
```

---

## Overall Fidelity Score

The overall fidelity score combines all dimensions:

```
Overall_Fidelity = (
    0.30 × Statistical_Fidelity +
    0.20 × Structural_Fidelity +
    0.20 × Correlation_Fidelity +
    0.20 × Rule_Fidelity +
    0.10 × Anomaly_Fidelity
)
```

### Interpretation

| Score | Grade | Interpretation |
|-------|-------|----------------|
| 95-100 | A | Excellent - Synthetic data is highly representative |
| 90-95 | B | Good - Minor deviations, suitable for most uses |
| 80-90 | C | Acceptable - Some deviations, review recommendations |
| 70-80 | D | Poor - Significant deviations, may not be suitable |
| < 70 | F | Failing - Major issues, requires investigation |

---

## Fidelity Report

### Report Structure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        FIDELITY EVALUATION REPORT                           │
│                                                                             │
│  Fingerprint: erp_gl_2024.dsf                                              │
│  Synthetic Data: ./synthetic_output/                                        │
│  Generated: 2024-12-15T14:30:00Z                                           │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  OVERALL FIDELITY SCORE: 94.7%                                             │
│  ████████████████████████████████████████████████░░░░  Grade: A            │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  DIMENSION BREAKDOWN                                                        │
│                                                                             │
│  Statistical:   96.2%  ████████████████████████████████████████████████░░  │
│  Structural:   100.0%  ██████████████████████████████████████████████████  │
│  Correlation:   91.5%  █████████████████████████████████████████████░░░░░  │
│  Rules:         98.3%  ████████████████████████████████████████████████░░  │
│  Anomalies:     87.4%  ██████████████████████████████████████████░░░░░░░░  │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  DETAILED METRICS                                                           │
│                                                                             │
│  Statistical:                                                               │
│    ✓ Amount distribution (KS=0.032)                                        │
│    ✓ Business process distribution (JSD=0.021)                             │
│    ✓ Benford's Law (MAD=0.008)                                             │
│    ⚠ Weekday distribution (KS=0.087) - slightly off                        │
│                                                                             │
│  Structural:                                                                │
│    ✓ Schema match: 100%                                                    │
│    ✓ Row count: 1,247,832 (target: 1,247,832)                             │
│    ✓ FK coverage: 99.97%                                                   │
│                                                                             │
│  Correlation:                                                               │
│    ✓ Matrix RMSE: 0.067                                                    │
│    ⚠ amount × line_count: 0.52 (target: 0.67, diff: 0.15)                 │
│    ✓ business_process × account: 0.88 (target: 0.89)                       │
│                                                                             │
│  Rules:                                                                     │
│    ✓ Balance equations: 100%                                               │
│    ✓ Approval thresholds: 98.7%                                            │
│    ✓ Temporal ordering: 99.82%                                             │
│                                                                             │
│  Anomalies:                                                                 │
│    ✓ Overall rate: 2.28% (target: 2.30%)                                   │
│    ⚠ Fraud subtype distribution slightly off (JSD=0.12)                    │
│    ⚠ Year-end spike: 1.9x (target: 2.1x)                                   │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  RECOMMENDATIONS                                                            │
│                                                                             │
│  1. [MEDIUM] Increase correlation enforcement for amount × line_count      │
│     Suggested: Enable copula-based generation for these columns            │
│                                                                             │
│  2. [LOW] Adjust anomaly year-end spike multiplier                         │
│     Suggested: Increase temporal.year_end_spike from 1.9 to 2.1            │
│                                                                             │
│  3. [LOW] Review weekday distribution                                       │
│     Note: May be due to fingerprint noise; consider re-extraction          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### CLI Output

```bash
$ datasynth-data evaluate \
    --fingerprint ./erp_gl.dsf \
    --synthetic ./synthetic_output/ \
    --output ./report.html

Evaluating synthetic data against fingerprint...

Loading fingerprint: erp_gl.dsf
Loading synthetic data: ./synthetic_output/

Running evaluations:
  [████████████████████] Statistical metrics (12/12)
  [████████████████████] Structural metrics (8/8)
  [████████████████████] Correlation metrics (6/6)
  [████████████████████] Rule compliance (5/5)
  [████████████████████] Anomaly metrics (4/4)

═══════════════════════════════════════════════════════════
                    FIDELITY RESULTS
═══════════════════════════════════════════════════════════

Overall Score: 94.7% (Grade: A)

Dimension Scores:
  Statistical:  96.2% ████████████████████░░
  Structural:  100.0% ██████████████████████
  Correlation:  91.5% ██████████████████░░░░
  Rules:        98.3% ███████████████████░░░
  Anomalies:    87.4% █████████████████░░░░░

Issues Found: 3 (0 critical, 1 medium, 2 low)

Full report written to: ./report.html
```

---

## Improving Fidelity

### Common Issues and Solutions

| Issue | Likely Cause | Solution |
|-------|--------------|----------|
| Low statistical fidelity | Privacy noise too high | Reduce epsilon (accept lower privacy) |
| Correlation mismatch | Default generators | Enable copula-based generation |
| Rule violations | Config mismatch | Verify fingerprint rules are in config |
| Anomaly rate off | Default anomaly settings | Override anomaly config from fingerprint |
| Structural mismatch | Scale factor | Adjust `--scale` parameter |

### Fidelity Tuning Workflow

```
┌─────────────────┐
│  Generate       │
│  Synthetic Data │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Evaluate       │
│  Fidelity       │
└────────┬────────┘
         │
         ▼
    ┌────────────┐     Yes
    │ Score OK?  │─────────────▶ Done
    └────────┬───┘
             │ No
             ▼
┌─────────────────┐
│  Review         │
│  Recommendations│
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Adjust         │
│  Configuration  │
└────────┬────────┘
         │
         └─────────────────────▶ (repeat)
```

---

## Use Case Fidelity Requirements

Different use cases have different fidelity requirements:

| Use Case | Required Score | Critical Dimensions |
|----------|----------------|---------------------|
| ML Training | 90%+ | Statistical, Correlation, Anomaly |
| Software Testing | 85%+ | Structural, Rules |
| Analytics Development | 90%+ | Statistical, Structural |
| Performance Testing | 80%+ | Structural (volume) |
| Demo/Presentation | 85%+ | Statistical, Rules |

---

## Next Steps

- [Getting Started Guide](../guides/01-getting-started.md): Hands-on tutorial
- [Fidelity Tuning Guide](../guides/05-fidelity-tuning.md): Improving synthetic data quality
- [Metrics Reference](../reference/05-metrics-reference.md): Complete metrics documentation
