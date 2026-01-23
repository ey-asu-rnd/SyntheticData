//! Business rules fingerprint models.

use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Rules fingerprint containing business rules and constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesFingerprint {
    /// Balance rules (e.g., debits = credits).
    pub balance_rules: Vec<BalanceRule>,

    /// Approval threshold rules.
    pub approval_thresholds: Vec<ApprovalThreshold>,

    /// Temporal ordering rules.
    pub temporal_rules: Vec<TemporalRule>,

    /// Value range constraints.
    pub range_constraints: Vec<RangeConstraint>,

    /// Custom business rules.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_rules: Vec<CustomRule>,

    /// Rule compliance statistics.
    pub compliance_stats: HashMap<String, RuleComplianceStats>,
}

impl RulesFingerprint {
    /// Create a new empty rules fingerprint.
    pub fn new() -> Self {
        Self {
            balance_rules: Vec::new(),
            approval_thresholds: Vec::new(),
            temporal_rules: Vec::new(),
            range_constraints: Vec::new(),
            custom_rules: Vec::new(),
            compliance_stats: HashMap::new(),
        }
    }

    /// Add a balance rule.
    pub fn add_balance_rule(&mut self, rule: BalanceRule) {
        self.balance_rules.push(rule);
    }

    /// Add an approval threshold.
    pub fn add_approval_threshold(&mut self, threshold: ApprovalThreshold) {
        self.approval_thresholds.push(threshold);
    }

    /// Add compliance statistics for a rule.
    pub fn add_compliance(&mut self, rule_name: impl Into<String>, stats: RuleComplianceStats) {
        self.compliance_stats.insert(rule_name.into(), stats);
    }
}

impl Default for RulesFingerprint {
    fn default() -> Self {
        Self::new()
    }
}

/// Balance rule (e.g., sum of debits = sum of credits).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceRule {
    /// Rule name.
    pub name: String,

    /// Description of the rule.
    pub description: String,

    /// Table this rule applies to.
    pub table: String,

    /// Grouping columns (e.g., document_id for JE balance).
    pub group_by: Vec<String>,

    /// Left side of the equation (column or expression).
    pub left_side: BalanceExpression,

    /// Right side of the equation.
    pub right_side: BalanceExpression,

    /// Tolerance for matching (absolute or relative).
    pub tolerance: BalanceTolerance,

    /// Compliance rate observed in data.
    pub compliance_rate: f64,
}

impl BalanceRule {
    /// Create a new balance rule.
    pub fn new(
        name: impl Into<String>,
        table: impl Into<String>,
        left_side: BalanceExpression,
        right_side: BalanceExpression,
    ) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            table: table.into(),
            group_by: Vec::new(),
            left_side,
            right_side,
            tolerance: BalanceTolerance::Absolute(Decimal::ZERO),
            compliance_rate: 1.0,
        }
    }

    /// Add grouping columns.
    pub fn with_group_by(mut self, columns: Vec<String>) -> Self {
        self.group_by = columns;
        self
    }
}

/// Expression for balance rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BalanceExpression {
    /// Sum of a column.
    Sum { column: String },

    /// Sum with filter condition.
    SumWhere {
        column: String,
        filter: FilterCondition,
    },

    /// Count of rows.
    Count,

    /// Count with filter.
    CountWhere { filter: FilterCondition },

    /// Constant value.
    Constant { value: Decimal },

    /// Difference between two expressions.
    Difference {
        left: Box<BalanceExpression>,
        right: Box<BalanceExpression>,
    },
}

/// Filter condition for expressions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCondition {
    /// Column to filter on.
    pub column: String,
    /// Operator.
    pub operator: FilterOperator,
    /// Value to compare against.
    pub value: String,
}

/// Filter operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    In,
    NotIn,
    IsNull,
    IsNotNull,
}

/// Tolerance for balance matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BalanceTolerance {
    /// Absolute tolerance (e.g., within $0.01).
    Absolute(Decimal),
    /// Relative tolerance (e.g., within 0.1%).
    Relative(f64),
}

/// Approval threshold rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalThreshold {
    /// Threshold name.
    pub name: String,

    /// Description.
    pub description: String,

    /// Amount thresholds in ascending order.
    pub thresholds: Vec<ThresholdLevel>,

    /// Observed compliance rate.
    pub compliance_rate: f64,

    /// Distribution of amounts across threshold levels.
    pub level_distribution: Vec<f64>,
}

impl ApprovalThreshold {
    /// Create a new approval threshold.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            thresholds: Vec::new(),
            compliance_rate: 1.0,
            level_distribution: Vec::new(),
        }
    }

    /// Add a threshold level.
    pub fn add_level(&mut self, level: ThresholdLevel) {
        self.thresholds.push(level);
        // Keep sorted by amount
        self.thresholds.sort_by(|a, b| a.amount.cmp(&b.amount));
    }
}

/// A single threshold level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdLevel {
    /// Threshold amount.
    pub amount: Decimal,
    /// Required approval level.
    pub approval_level: String,
    /// Proportion of transactions at this level.
    pub proportion: f64,
}

/// Temporal ordering rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalRule {
    /// Rule name.
    pub name: String,

    /// Description.
    pub description: String,

    /// First event/date column.
    pub before_column: String,

    /// Second event/date column (must be >= before_column).
    pub after_column: String,

    /// Table(s) involved.
    pub tables: Vec<String>,

    /// Join condition if multiple tables.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join_condition: Option<String>,

    /// Observed compliance rate.
    pub compliance_rate: f64,

    /// Typical gap statistics.
    pub gap_stats: Option<GapStatistics>,
}

impl TemporalRule {
    /// Create a new temporal rule.
    pub fn new(
        name: impl Into<String>,
        before_column: impl Into<String>,
        after_column: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            before_column: before_column.into(),
            after_column: after_column.into(),
            tables: Vec::new(),
            join_condition: None,
            compliance_rate: 1.0,
            gap_stats: None,
        }
    }
}

/// Statistics about time gaps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapStatistics {
    /// Minimum gap in days.
    pub min_days: f64,
    /// Maximum gap in days.
    pub max_days: f64,
    /// Mean gap in days.
    pub mean_days: f64,
    /// Median gap in days.
    pub median_days: f64,
    /// Standard deviation of gap.
    pub std_dev_days: f64,
}

/// Value range constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeConstraint {
    /// Constraint name.
    pub name: String,

    /// Table name.
    pub table: String,

    /// Column name.
    pub column: String,

    /// Minimum value (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<f64>,

    /// Maximum value (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<f64>,

    /// Compliance rate.
    pub compliance_rate: f64,
}

impl RangeConstraint {
    /// Create a new range constraint.
    pub fn new(
        name: impl Into<String>,
        table: impl Into<String>,
        column: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            table: table.into(),
            column: column.into(),
            min_value: None,
            max_value: None,
            compliance_rate: 1.0,
        }
    }

    /// Set minimum value.
    pub fn with_min(mut self, min: f64) -> Self {
        self.min_value = Some(min);
        self
    }

    /// Set maximum value.
    pub fn with_max(mut self, max: f64) -> Self {
        self.max_value = Some(max);
        self
    }
}

/// Custom business rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    /// Rule name.
    pub name: String,

    /// Description.
    pub description: String,

    /// Rule category.
    pub category: String,

    /// Tables involved.
    pub tables: Vec<String>,

    /// Columns involved.
    pub columns: Vec<String>,

    /// Rule expression (human-readable).
    pub expression: String,

    /// Compliance rate.
    pub compliance_rate: f64,

    /// Additional parameters.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub parameters: HashMap<String, String>,
}

/// Rule compliance statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleComplianceStats {
    /// Total rows checked.
    pub total_checked: u64,

    /// Rows that passed the rule.
    pub passed: u64,

    /// Rows that failed the rule.
    pub failed: u64,

    /// Compliance rate (passed / total).
    pub compliance_rate: f64,

    /// Common failure patterns.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failure_patterns: Vec<FailurePattern>,
}

impl RuleComplianceStats {
    /// Create from counts.
    pub fn from_counts(total: u64, passed: u64) -> Self {
        let failed = total.saturating_sub(passed);
        let compliance_rate = if total > 0 {
            passed as f64 / total as f64
        } else {
            1.0
        };

        Self {
            total_checked: total,
            passed,
            failed,
            compliance_rate,
            failure_patterns: Vec::new(),
        }
    }
}

/// Pattern of rule failures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    /// Pattern description.
    pub description: String,
    /// Count of this pattern.
    pub count: u64,
    /// Proportion of failures.
    pub proportion: f64,
}
