# synth-eval

Evaluation framework for synthetic financial data quality and coherence.

## Overview

`synth-eval` provides automated quality assessment for generated data:

- **Statistical Evaluation**: Benford's Law compliance, distribution analysis
- **Coherence Checking**: Balance verification, document chain integrity
- **Intercompany Validation**: IC matching and elimination verification
- **Uniqueness Analysis**: Duplicate detection across datasets

## Evaluation Categories

| Category | Description |
|----------|-------------|
| Statistical | Benford's Law, amount distributions, temporal patterns |
| Coherence | Trial balance, subledger reconciliation, FX consistency |
| Intercompany | IC matching rates, elimination completeness |
| Uniqueness | Document ID collisions, duplicate transaction detection |

## Key Types

### Evaluator

```rust
pub struct Evaluator {
    config: EvaluationConfig,
    checkers: Vec<Box<dyn Checker>>,
}

pub struct EvaluationConfig {
    pub benford_threshold: f64,      // Chi-square threshold
    pub balance_tolerance: Decimal,   // Allowed imbalance
    pub ic_match_threshold: f64,      // Required match rate
    pub duplicate_check: bool,
}
```

### Evaluation Report

```rust
pub struct EvaluationReport {
    pub overall_status: Status,
    pub categories: Vec<CategoryResult>,
    pub warnings: Vec<Warning>,
    pub details: Vec<Finding>,
    pub scores: Scores,
}

pub struct Scores {
    pub benford_score: f64,           // 0.0-1.0
    pub balance_coherence: f64,       // 0.0-1.0
    pub ic_matching_rate: f64,        // 0.0-1.0
    pub uniqueness_score: f64,        // 0.0-1.0
}

pub enum Status {
    Passed,
    PassedWithWarnings,
    Failed,
}
```

## Usage Examples

### Basic Evaluation

```rust
use synth_eval::{Evaluator, EvaluationConfig};

let evaluator = Evaluator::new(EvaluationConfig::default());
let report = evaluator.evaluate(&generated_data)?;

println!("Status: {:?}", report.overall_status);
println!("Benford compliance: {:.2}%", report.scores.benford_score * 100.0);
```

### Custom Configuration

```rust
let config = EvaluationConfig {
    benford_threshold: 0.05,          // 5% significance level
    balance_tolerance: dec!(0.01),    // 1 cent tolerance
    ic_match_threshold: 0.99,         // 99% required match
    duplicate_check: true,
};

let evaluator = Evaluator::new(config);
```

### Category-Specific Evaluation

```rust
use synth_eval::checkers::{BenfordChecker, BalanceChecker};

let benford = BenfordChecker::new(0.05);
let result = benford.check(&amounts)?;

let balance = BalanceChecker::new(dec!(0.01));
let result = balance.check(&trial_balance)?;
```

## Evaluation Checks

### Benford's Law

Verifies first-digit distribution follows Benford's Law:

```rust
// Expected: P(d) = log10(1 + 1/d)
// d=1: 30.1%, d=2: 17.6%, d=3: 12.5%, etc.

let benford_result = evaluator.check_benford(&amounts)?;

if benford_result.chi_square > critical_value {
    println!("Warning: Amounts don't follow Benford's Law");
}
```

### Balance Coherence

Verifies accounting equation:

```rust
// Assets = Liabilities + Equity
let balance_result = evaluator.check_balance(&trial_balance)?;

if !balance_result.passed {
    println!("Imbalance: {:?}", balance_result.difference);
}
```

### Document Chain Integrity

Verifies document references:

```rust
// PO → GR → Invoice → Payment chain
let chain_result = evaluator.check_document_chains(&documents)?;

for broken_chain in &chain_result.broken_chains {
    println!("Broken chain: {} → {}", broken_chain.from, broken_chain.to);
}
```

### IC Matching

Verifies intercompany transactions match:

```rust
let ic_result = evaluator.check_ic_matching(&ic_entries)?;

println!("Match rate: {:.2}%", ic_result.match_rate * 100.0);
println!("Unmatched: {}", ic_result.unmatched.len());
```

### Uniqueness

Detects duplicate document IDs:

```rust
let unique_result = evaluator.check_uniqueness(&entries)?;

if !unique_result.duplicates.is_empty() {
    for dup in &unique_result.duplicates {
        println!("Duplicate ID: {}", dup.document_id);
    }
}
```

## Report Output

### Console Report

```rust
evaluator.print_report(&report);
```

```
=== Evaluation Report ===
Status: PASSED

Scores:
  Benford Compliance:    98.5%
  Balance Coherence:    100.0%
  IC Matching Rate:      99.8%
  Uniqueness:           100.0%

Warnings:
  - 3 entries with unusual amounts detected

Categories:
  [✓] Statistical:   PASSED
  [✓] Coherence:     PASSED
  [✓] Intercompany:  PASSED
  [✓] Uniqueness:    PASSED
```

### JSON Report

```rust
let json = evaluator.to_json(&report)?;
std::fs::write("evaluation_report.json", json)?;
```

## Integration with Generation

```rust
use synth_runtime::GenerationOrchestrator;
use synth_eval::Evaluator;

let orchestrator = GenerationOrchestrator::new(config)?;
let data = orchestrator.run()?;

// Evaluate generated data
let evaluator = Evaluator::new(EvaluationConfig::default());
let report = evaluator.evaluate(&data)?;

if report.overall_status == Status::Failed {
    return Err("Generated data failed quality checks");
}
```

## See Also

- [synth-generators](synth-generators.md)
- [Data Quality](../advanced/data-quality.md)
- [Testing Guidelines](../contributing/testing.md)
