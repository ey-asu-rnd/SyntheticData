//! Custom assertion macros for testing accounting invariants.

use rust_decimal::Decimal;
use synth_core::models::JournalEntry;

/// Assert that a journal entry is balanced (debits equal credits).
#[macro_export]
macro_rules! assert_balanced {
    ($entry:expr) => {{
        let entry = &$entry;
        let total_debits: rust_decimal::Decimal =
            entry.lines.iter().map(|l| l.debit_amount).sum();
        let total_credits: rust_decimal::Decimal =
            entry.lines.iter().map(|l| l.credit_amount).sum();
        assert_eq!(
            total_debits, total_credits,
            "Journal entry is not balanced: debits={}, credits={}",
            total_debits, total_credits
        );
    }};
}

/// Assert that all journal entries in a collection are balanced.
#[macro_export]
macro_rules! assert_all_balanced {
    ($entries:expr) => {{
        for (i, entry) in $entries.iter().enumerate() {
            let total_debits: rust_decimal::Decimal =
                entry.lines.iter().map(|l| l.debit_amount).sum();
            let total_credits: rust_decimal::Decimal =
                entry.lines.iter().map(|l| l.credit_amount).sum();
            assert_eq!(
                total_debits, total_credits,
                "Journal entry {} is not balanced: debits={}, credits={}",
                i, total_debits, total_credits
            );
        }
    }};
}

/// Assert that an amount follows Benford's Law distribution within tolerance.
/// This checks if the first digit distribution matches expected frequencies.
#[macro_export]
macro_rules! assert_benford_compliant {
    ($amounts:expr, $tolerance:expr) => {{
        let amounts = &$amounts;
        let expected = [0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046];
        let mut counts = [0u64; 9];
        let mut total = 0u64;

        for amount in amounts.iter() {
            if *amount > rust_decimal::Decimal::ZERO {
                let first_digit = amount
                    .to_string()
                    .chars()
                    .find(|c| c.is_ascii_digit() && *c != '0')
                    .map(|c| c.to_digit(10).unwrap() as usize);

                if let Some(d) = first_digit {
                    if d >= 1 && d <= 9 {
                        counts[d - 1] += 1;
                        total += 1;
                    }
                }
            }
        }

        if total > 0 {
            for (i, (count, exp)) in counts.iter().zip(expected.iter()).enumerate() {
                let observed = *count as f64 / total as f64;
                let diff = (observed - exp).abs();
                assert!(
                    diff < $tolerance,
                    "Benford's Law violation for digit {}: observed={:.4}, expected={:.4}, diff={:.4}",
                    i + 1,
                    observed,
                    exp,
                    diff
                );
            }
        }
    }};
}

/// Check if a journal entry is balanced.
pub fn is_balanced(entry: &JournalEntry) -> bool {
    let total_debits: Decimal = entry.lines.iter().map(|l| l.debit_amount).sum();
    let total_credits: Decimal = entry.lines.iter().map(|l| l.credit_amount).sum();
    total_debits == total_credits
}

/// Calculate the imbalance of a journal entry.
pub fn calculate_imbalance(entry: &JournalEntry) -> Decimal {
    let total_debits: Decimal = entry.lines.iter().map(|l| l.debit_amount).sum();
    let total_credits: Decimal = entry.lines.iter().map(|l| l.credit_amount).sum();
    total_debits - total_credits
}

/// Check if amounts follow Benford's Law distribution.
/// Returns the chi-squared statistic and whether it passes the test at p < 0.05.
pub fn check_benford_distribution(amounts: &[Decimal]) -> (f64, bool) {
    let expected = [0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046];
    let mut counts = [0u64; 9];
    let mut total = 0u64;

    for amount in amounts.iter() {
        if *amount > Decimal::ZERO {
            let first_digit = amount
                .to_string()
                .chars()
                .find(|c| c.is_ascii_digit() && *c != '0')
                .map(|c| c.to_digit(10).unwrap() as usize);

            if let Some(d) = first_digit {
                if d >= 1 && d <= 9 {
                    counts[d - 1] += 1;
                    total += 1;
                }
            }
        }
    }

    if total == 0 {
        return (0.0, true);
    }

    // Calculate chi-squared statistic
    let mut chi_squared = 0.0;
    for (count, exp) in counts.iter().zip(expected.iter()) {
        let expected_count = exp * total as f64;
        if expected_count > 0.0 {
            let diff = *count as f64 - expected_count;
            chi_squared += diff * diff / expected_count;
        }
    }

    // Critical value for chi-squared with 8 degrees of freedom at p < 0.05 is 15.507
    // At p < 0.01 is 20.090
    let passes = chi_squared < 20.090;

    (chi_squared, passes)
}

/// Check that the accounting equation holds: Assets = Liabilities + Equity
pub fn check_accounting_equation(
    total_assets: Decimal,
    total_liabilities: Decimal,
    total_equity: Decimal,
) -> bool {
    total_assets == total_liabilities + total_equity
}

/// Verify trial balance is balanced (total debits = total credits).
pub fn check_trial_balance(debit_balances: &[Decimal], credit_balances: &[Decimal]) -> bool {
    let total_debits: Decimal = debit_balances.iter().copied().sum();
    let total_credits: Decimal = credit_balances.iter().copied().sum();
    total_debits == total_credits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::*;

    #[test]
    fn test_is_balanced() {
        let entry = balanced_journal_entry(Decimal::new(10000, 2));
        assert!(is_balanced(&entry));
    }

    #[test]
    fn test_is_not_balanced() {
        let entry = unbalanced_journal_entry();
        assert!(!is_balanced(&entry));
    }

    #[test]
    fn test_calculate_imbalance_balanced() {
        let entry = balanced_journal_entry(Decimal::new(10000, 2));
        assert_eq!(calculate_imbalance(&entry), Decimal::ZERO);
    }

    #[test]
    fn test_calculate_imbalance_unbalanced() {
        let entry = unbalanced_journal_entry();
        let imbalance = calculate_imbalance(&entry);
        assert_ne!(imbalance, Decimal::ZERO);
    }

    #[test]
    fn test_check_accounting_equation() {
        // Assets = 1000, Liabilities = 600, Equity = 400
        assert!(check_accounting_equation(
            Decimal::new(1000, 0),
            Decimal::new(600, 0),
            Decimal::new(400, 0)
        ));

        // Unbalanced: Assets = 1000, Liabilities = 600, Equity = 300
        assert!(!check_accounting_equation(
            Decimal::new(1000, 0),
            Decimal::new(600, 0),
            Decimal::new(300, 0)
        ));
    }

    #[test]
    fn test_check_trial_balance() {
        let debits = vec![Decimal::new(1000, 0), Decimal::new(500, 0)];
        let credits = vec![Decimal::new(1500, 0)];
        assert!(check_trial_balance(&debits, &credits));

        let unbalanced_credits = vec![Decimal::new(1000, 0)];
        assert!(!check_trial_balance(&debits, &unbalanced_credits));
    }

    #[test]
    fn test_benford_distribution_perfect() {
        // Create a distribution that follows Benford's Law
        let mut amounts = Vec::new();
        let expected_counts = [301, 176, 125, 97, 79, 67, 58, 51, 46]; // Per 1000

        for (digit, count) in expected_counts.iter().enumerate() {
            let base = Decimal::new((digit + 1) as i64, 0);
            for _ in 0..*count {
                amounts.push(base);
            }
        }

        let (chi_squared, passes) = check_benford_distribution(&amounts);
        assert!(passes, "Chi-squared: {}", chi_squared);
    }

    #[test]
    fn test_assert_balanced_macro() {
        let entry = balanced_journal_entry(Decimal::new(10000, 2));
        assert_balanced!(entry); // Should not panic
    }

    #[test]
    fn test_assert_all_balanced_macro() {
        let entries = vec![
            balanced_journal_entry(Decimal::new(10000, 2)),
            balanced_journal_entry(Decimal::new(20000, 2)),
            balanced_journal_entry(Decimal::new(30000, 2)),
        ];
        assert_all_balanced!(entries); // Should not panic
    }
}
