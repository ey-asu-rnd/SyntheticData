# Synthetic Data Generator Enhancement Plan

This document outlines enhancements identified through comprehensive evaluation of the generated synthetic data. The evaluation was performed using the `synth-eval` crate on 10,000 generated journal entries with 111,689 line items.

## Executive Summary

| Metric | Status | Score |
|--------|--------|-------|
| Balance Coherence | ❌ CRITICAL | 99.73% |
| Benford's Law Compliance | ⚠️ HIGH | MAD: 0.0045 (Close) |
| Line Item Distribution | ⚠️ MEDIUM | Chi-sq fails |
| Temporal Patterns | ✅ PASS | Corr: 0.74 |
| Day-of-Week Patterns | ✅ PASS | Corr: 0.99 |
| Weekend Activity | ✅ PASS | 3.69% |

**Overall Evaluation Score: 55/100**

---

## 1. CRITICAL: Balance Equation Violations

### Finding
27 of 10,000 entries (0.27%) have imbalanced debits and credits.

### Impact
- Fundamental accounting violation
- Data cannot be used for training ML models that assume balanced entries
- Would cause audit failures in real scenarios

### Root Cause (CONFIRMED)

The bug is in `crates/synth-generators/src/je_generator.rs` lines 921-1031, in the `inject_human_error` function.

**Problem:** The function modifies individual line amounts without adjusting contra-entries:
- Error type 0 (transposition): Swaps digits in one line's amount
- Error type 1 (decimal shift): Multiplies one line by 10
- Error type 3 (rounding): Rounds one line to nearest 100

These modifications only affect one side of the entry, breaking the balance.

**Evidence:**
- 26 of 27 imbalanced entries are from "manual" source (persona errors only apply to manual)
- Imbalances are large (not rounding errors): $177.03, $720.00, $4829.13, $40364.82
- Header text likely contains "[HUMAN_ERROR:...]" markers

### Recommended Fix

**File:** `crates/synth-generators/src/je_generator.rs`

1. Modify `inject_human_error` to also adjust contra-entries:
```rust
fn generate_entry(...) -> JournalEntry {
    let entry = /* ... generate entry ... */;
    
    // Validate balance
    let total_debit: Decimal = entry.lines.iter().map(|l| l.debit_amount).sum();
    let total_credit: Decimal = entry.lines.iter().map(|l| l.credit_amount).sum();
    
    if (total_debit - total_credit).abs() > Decimal::new(1, 2) {
        // Force balance by adjusting last line
        self.force_balance(&mut entry);
    }
    
    entry
}
```

2. Add `force_balance()` helper that adjusts the last line item to ensure balance

**Priority:** P0 - Must fix before next release

---

## 2. HIGH: Benford's Law Chi-Squared Test Failure

### Finding
- **MAD (Mean Absolute Deviation):** 0.0045 → **Close Conformity** ✅
- **Chi-squared p-value:** 0.000000 → **Test Fails** ❌

The actual first-digit distribution is very close to Benford's Law:
| Digit | Expected | Observed | Deviation |
|-------|----------|----------|-----------|
| 1 | 30.1% | 31.2% | +1.1% |
| 2 | 17.6% | 16.2% | -1.4% |
| 3 | 12.5% | 12.0% | -0.5% |
| 4 | 9.7% | 10.0% | +0.3% |
| 5 | 7.9% | 8.2% | +0.3% |
| 6 | 6.7% | 6.6% | -0.1% |
| 7 | 5.8% | 5.9% | +0.1% |
| 8 | 5.1% | 5.2% | +0.1% |
| 9 | 4.6% | 4.8% | +0.2% |

### Analysis
The chi-squared test failure is expected with large sample sizes (n=111,689). With such large samples, even tiny deviations become statistically significant. The **MAD of 0.0045 indicates excellent conformity** (threshold for "Close" conformity is < 0.006).

### Recommendation
1. **No immediate action required** - distribution is excellent
2. **Documentation update:** Note that chi-squared tests become overly sensitive at large sample sizes
3. **Optional tuning:** Slight adjustments to `lognormal_mu` could reduce digit 1 by ~1%:
   - Current: μ=7.0, σ=2.5
   - Suggested: μ=6.8, σ=2.6

**Priority:** P2 - Low priority, distribution quality is good

---

## 3. MEDIUM: Line Item Distribution Deviation

### Finding
Chi-squared test fails but observed values are close to expected:

| Line Count | Expected | Observed | Status |
|------------|----------|----------|--------|
| 2 | 60.68% | 63.98% | ✅ Close |
| 3 | 5.77% | 8.42% | ⚠️ +2.65% |
| 4 | 16.63% | 14.46% | ⚠️ -2.17% |
| 5 | 3.06% | 0.73% | ⚠️ -2.33% |
| 6 | 3.32% | 3.79% | ✅ Close |

### Analysis
- 2-line entries: Slightly over-represented (+3.3%)
- 5-line entries: Under-represented (-2.33%)
- Even ratio: 89.75% vs expected 88% ✅
- Equal split ratio: 87.36% vs expected 82% (higher is acceptable)

### Root Cause
The deviation in 5-line entries suggests the sampling distribution weights need tuning.

### Recommended Fix

**File:** `crates/synth-core/src/distributions/line_item.rs`

Adjust the sampling weights:
```yaml
line_item_distribution:
  two_items: 0.58    # Reduce from 0.6068
  three_items: 0.055  # Slight reduction
  four_items: 0.17    # Slight increase
  five_items: 0.04    # Increase from 0.0306
  # ... rest unchanged
```

**Priority:** P2 - Quality improvement

---

## 4. MEDIUM: Source Distribution Mismatch

### Finding
Configuration specified:
- Manual: 20%
- Automated: 70%

Observed:
- Manual: 39.1%
- Automated: 60.9%

### Analysis
The generator is over-generating manual entries. This could be due to:
1. Document flow entries defaulting to manual
2. Certain business processes preferring manual sources

### Recommended Investigation

**Files to review:**
- `crates/synth-generators/src/je_generator.rs` - Check source assignment logic
- `crates/synth-generators/src/document_flow/p2p_generator.rs`
- `crates/synth-generators/src/document_flow/o2c_generator.rs`

**Priority:** P2 - Config compliance issue

---

## 5. MEDIUM: Fraud Rate Discrepancy

### Finding
Configuration: `fraud_rate: 0.005` (0.5%)
Observed: 0% fraud entries

### Root Cause
Fraud injection is likely happening but fraud entries may not be marked with `is_fraud: true` in the sample output.

### Recommended Fix
Verify fraud marking in:
- `crates/synth-generators/src/anomaly/injector.rs`
- `crates/synth-runtime/src/orchestrator.rs`

**Priority:** P2 - Feature verification

---

## 6. LOW: High Amount Skewness

### Finding
- Skewness: 75.56 (very high)
- Kurtosis: 6398.39 (extreme)
- Max amount: $9.49M vs Mean: $4,142

### Analysis
This is expected for log-normal distributions with high sigma. The distribution has a long right tail with some very large transactions.

### Recommendation
If more realistic distribution is desired:
1. Cap maximum amounts at 99th percentile
2. Reduce `max_amount` from 100M to 10M
3. Reduce `lognormal_sigma` from 2.5 to 2.0

**Priority:** P3 - Optional tuning

---

## 7. PASS: Temporal Patterns ✅

### Findings
- Pattern correlation: 0.74 (good)
- Weekend activity: 3.69% (expected <10%) ✅
- Day-of-week correlation: 0.99 (excellent)
- Month-end spike: 2.17x (expected ~2.5x) - slightly low

### Day-of-Week Distribution
| Day | Volume |
|-----|--------|
| Monday | 23.2% |
| Tuesday | 18.2% |
| Wednesday | 19.5% |
| Thursday | 18.8% |
| Friday | 16.7% |
| Saturday | 1.9% |
| Sunday | 1.8% |

This closely matches expected business activity patterns.

---

## Implementation Roadmap

### Phase 1 - Critical Fixes (Week 1)
1. [ ] Fix balance equation violations in JE generator
2. [ ] Add balance validation before output
3. [ ] Add integration tests for balance coherence

### Phase 2 - Quality Improvements (Week 2)
4. [ ] Investigate source distribution mismatch
5. [ ] Verify fraud marking in output
6. [ ] Tune line item distribution weights

### Phase 3 - Enhancements (Week 3)
7. [ ] Add evaluation metrics to CI pipeline
8. [ ] Create automated regression tests
9. [ ] Document tuning parameters

---

## Appendix: Evaluation Metrics

### Statistical Tests Used
1. **Benford's Law Chi-Squared Test** - Tests first-digit distribution
2. **Mean Absolute Deviation (MAD)** - Measures average deviation from expected
3. **Kolmogorov-Smirnov Test** - Tests log-normal fit
4. **Pearson Correlation** - Measures temporal pattern correlation

### Threshold Definitions
| Metric | Close | Acceptable | Marginal | Non-conforming |
|--------|-------|------------|----------|----------------|
| MAD | <0.006 | <0.012 | <0.015 | ≥0.015 |
| Chi-sq p-value | >0.05 | >0.01 | >0.001 | ≤0.001 |

---

*Generated by synth-eval on 2026-01-17*
