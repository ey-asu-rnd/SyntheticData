//! Semantic coherence evaluation module.
//!
//! Validates that generated data maintains accounting coherence including
//! balance sheet equations, subledger reconciliation, and document chain integrity.

mod balance;
mod document_chain;
mod intercompany;
mod referential;
mod subledger;

pub use balance::{BalanceSheetEvaluation, BalanceSheetEvaluator};
pub use document_chain::{DocumentChainEvaluation, DocumentChainEvaluator};
pub use intercompany::{ICMatchingEvaluation, ICMatchingEvaluator};
pub use referential::{ReferentialIntegrityEvaluation, ReferentialIntegrityEvaluator};
pub use subledger::{SubledgerEvaluator, SubledgerReconciliationEvaluation};

use serde::{Deserialize, Serialize};

/// Combined coherence evaluation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceEvaluation {
    /// Balance sheet validation results.
    pub balance: Option<BalanceSheetEvaluation>,
    /// Subledger reconciliation results.
    pub subledger: Option<SubledgerReconciliationEvaluation>,
    /// Document chain completeness results.
    pub document_chain: Option<DocumentChainEvaluation>,
    /// Intercompany matching results.
    pub intercompany: Option<ICMatchingEvaluation>,
    /// Referential integrity results.
    pub referential: Option<ReferentialIntegrityEvaluation>,
    /// Overall pass/fail status.
    pub passes: bool,
    /// Summary of failed checks.
    pub failures: Vec<String>,
}

impl CoherenceEvaluation {
    /// Create a new empty evaluation.
    pub fn new() -> Self {
        Self {
            balance: None,
            subledger: None,
            document_chain: None,
            intercompany: None,
            referential: None,
            passes: true,
            failures: Vec::new(),
        }
    }

    /// Check all results against thresholds and update pass status.
    pub fn check_thresholds(&mut self, thresholds: &crate::config::EvaluationThresholds) {
        self.failures.clear();

        if let Some(ref balance) = self.balance {
            if !balance.equation_balanced {
                self.failures.push(format!(
                    "Balance sheet equation not balanced (max imbalance: {})",
                    balance.max_imbalance
                ));
            }
        }

        if let Some(ref subledger) = self.subledger {
            if subledger.completeness_score < thresholds.subledger_reconciliation_rate_min {
                self.failures.push(format!(
                    "Subledger reconciliation {} < {} (threshold)",
                    subledger.completeness_score, thresholds.subledger_reconciliation_rate_min
                ));
            }
        }

        if let Some(ref doc_chain) = self.document_chain {
            let min_rate = thresholds.document_chain_completion_min;
            if doc_chain.p2p_completion_rate < min_rate {
                self.failures.push(format!(
                    "P2P chain completion {} < {} (threshold)",
                    doc_chain.p2p_completion_rate, min_rate
                ));
            }
            if doc_chain.o2c_completion_rate < min_rate {
                self.failures.push(format!(
                    "O2C chain completion {} < {} (threshold)",
                    doc_chain.o2c_completion_rate, min_rate
                ));
            }
        }

        if let Some(ref ic) = self.intercompany {
            if ic.match_rate < thresholds.ic_match_rate_min {
                self.failures.push(format!(
                    "IC match rate {} < {} (threshold)",
                    ic.match_rate, thresholds.ic_match_rate_min
                ));
            }
        }

        if let Some(ref referential) = self.referential {
            if referential.overall_integrity_score < thresholds.referential_integrity_min {
                self.failures.push(format!(
                    "Referential integrity {} < {} (threshold)",
                    referential.overall_integrity_score, thresholds.referential_integrity_min
                ));
            }
        }

        self.passes = self.failures.is_empty();
    }
}

impl Default for CoherenceEvaluation {
    fn default() -> Self {
        Self::new()
    }
}
