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

## Usage

```rust
use synth_eval::{Evaluator, EvaluationConfig};

let evaluator = Evaluator::new(EvaluationConfig::default());
let report = evaluator.evaluate(&generated_data)?;

println!("Benford compliance: {:.2}%", report.benford_score * 100.0);
println!("Balance coherence: {}", report.balance_check.passed);
```

## Evaluation Report

The evaluation produces a comprehensive report including:

- **Pass/Fail Status**: Overall and per-category
- **Scores**: Numerical scores for statistical measures
- **Warnings**: Potential issues that don't fail validation
- **Details**: Specific findings and recommendations

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
