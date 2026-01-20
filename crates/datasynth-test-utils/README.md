# datasynth-test-utils

Test utilities and helpers for the SyntheticData workspace.

## Overview

`datasynth-test-utils` provides shared testing infrastructure:

- **Test Fixtures**: Pre-configured test data and scenarios
- **Assertion Helpers**: Domain-specific assertions for financial data
- **Mock Generators**: Simplified generators for unit testing
- **Snapshot Testing**: Helpers for snapshot-based testing

## Usage

```rust
use datasynth_test_utils::{fixtures, assertions};

#[test]
fn test_journal_entry_balance() {
    let entry = fixtures::balanced_journal_entry();
    assertions::assert_balanced(&entry);
}

#[test]
fn test_benford_compliance() {
    let amounts = fixtures::sample_amounts(1000);
    assertions::assert_benford_compliant(&amounts, 0.05);
}
```

## Fixtures

| Fixture | Description |
|---------|-------------|
| `balanced_journal_entry()` | Valid balanced JE |
| `sample_amounts(n)` | Random Benford-compliant amounts |
| `test_chart_of_accounts()` | Small COA for testing |
| `test_company_config()` | Minimal company configuration |

## Assertions

| Assertion | Description |
|-----------|-------------|
| `assert_balanced()` | Verify debits equal credits |
| `assert_benford_compliant()` | Check first-digit distribution |
| `assert_valid_document_chain()` | Verify document references |

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
