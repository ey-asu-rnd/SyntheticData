# datasynth-eval

Evaluation framework for synthetic financial data quality and coherence.

## Overview

`datasynth-eval` provides automated quality assessment for generated data:

- **Statistical Evaluation**: Benford's Law compliance, distribution analysis
- **Coherence Checking**: Balance verification, document chain integrity
- **Intercompany Validation**: IC matching and elimination verification
- **Data Quality Analysis**: Completeness, consistency, format validation
- **ML Readiness**: Feature distributions, label quality, graph structure
- **Enhancement Derivation**: Auto-tuning with configuration recommendations

## Evaluation Categories

| Category | Description |
|----------|-------------|
| Statistical | Benford's Law, amount distributions, temporal patterns, line items |
| Coherence | Trial balance, subledger reconciliation, FX consistency, document chains |
| Intercompany | IC matching rates, elimination completeness |
| Quality | Completeness, consistency, duplicates, format validation, uniqueness |
| ML Readiness | Feature distributions, label quality, graph structure, train/val/test splits |
| Enhancement | Auto-tuning, configuration recommendations, root cause analysis |

## Module Structure

| Module | Description |
|--------|-------------|
| `statistical/` | Benford's Law, amount distributions, temporal patterns |
| `coherence/` | Balance sheet, IC matching, document chains, subledger reconciliation |
| `quality/` | Completeness, consistency, duplicates, formats, uniqueness |
| `ml/` | Feature analysis, label quality, graph structure, splits |
| `report/` | HTML and JSON report generation with baseline comparisons |
| `tuning/` | Configuration optimization recommendations |
| `enhancement/` | Auto-tuning engine with config patch generation |

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

## Enhancement Module

The `enhancement` module provides automatic configuration tuning based on evaluation results.

### Pipeline Flow

```
Evaluation Results → Threshold Check → Gap Analysis → Root Cause → Config Suggestion
```

### Auto-Tuning

```rust
use synth_eval::enhancement::{AutoTuner, AutoTuneResult};

let tuner = AutoTuner::new();
let result: AutoTuneResult = tuner.analyze(&evaluation);

for patch in result.patches_by_confidence() {
    println!("{}: {} → {} (confidence: {:.0}%)",
        patch.path,
        patch.current_value.as_deref().unwrap_or("?"),
        patch.suggested_value,
        patch.confidence * 100.0
    );
}
```

### Key Types

```rust
pub struct ConfigPatch {
    pub path: String,              // e.g., "transactions.amount.benford_compliance"
    pub current_value: Option<String>,
    pub suggested_value: String,
    pub confidence: f64,           // 0.0-1.0
    pub expected_impact: String,
}

pub struct AutoTuneResult {
    pub patches: Vec<ConfigPatch>,
    pub expected_improvement: f64,
    pub addressed_metrics: Vec<String>,
    pub unaddressable_metrics: Vec<String>,
    pub summary: String,
}
```

### Recommendation Engine

```rust
use synth_eval::enhancement::{RecommendationEngine, RecommendationPriority};

let engine = RecommendationEngine::new();
let recommendations = engine.generate(&evaluation);

for rec in recommendations.iter().filter(|r| r.priority == RecommendationPriority::Critical) {
    println!("CRITICAL: {} - {}", rec.title, rec.root_cause.description);
}
```

### Metric-to-Config Mappings

| Metric | Config Path | Strategy |
|--------|-------------|----------|
| `benford_p_value` | `transactions.amount.benford_compliance` | Enable boolean |
| `round_number_ratio` | `transactions.amount.round_number_bias` | Set to target |
| `temporal_correlation` | `transactions.temporal.seasonality_strength` | Increase by gap |
| `anomaly_rate` | `anomaly_injection.base_rate` | Set to target |
| `ic_match_rate` | `intercompany.match_precision` | Increase by gap |
| `completeness_rate` | `data_quality.missing_values.overall_rate` | Decrease by gap |

## See Also

- [datasynth-generators](datasynth-generators.md)
- [Data Quality](../advanced/data-quality.md)
- [Testing Guidelines](../contributing/testing.md)
