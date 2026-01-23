//! Integrity fingerprint models.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Integrity fingerprint containing referential integrity information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityFingerprint {
    /// Foreign key relationships.
    pub foreign_keys: Vec<ForeignKeyDef>,

    /// Cardinality statistics for relationships.
    pub cardinality_stats: HashMap<String, CardinalityStats>,

    /// Unique constraints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unique_constraints: Vec<UniqueConstraint>,

    /// Check constraints (expressed as rules).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub check_constraints: Vec<CheckConstraint>,
}

impl IntegrityFingerprint {
    /// Create a new empty integrity fingerprint.
    pub fn new() -> Self {
        Self {
            foreign_keys: Vec::new(),
            cardinality_stats: HashMap::new(),
            unique_constraints: Vec::new(),
            check_constraints: Vec::new(),
        }
    }

    /// Add a foreign key definition.
    pub fn add_foreign_key(&mut self, fk: ForeignKeyDef) {
        self.foreign_keys.push(fk);
    }

    /// Add cardinality statistics.
    pub fn add_cardinality(&mut self, key: impl Into<String>, stats: CardinalityStats) {
        self.cardinality_stats.insert(key.into(), stats);
    }
}

impl Default for IntegrityFingerprint {
    fn default() -> Self {
        Self::new()
    }
}

/// Foreign key relationship definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyDef {
    /// Name identifier for this relationship.
    pub name: String,

    /// Source (child) table.
    pub from_table: String,

    /// Source column(s).
    pub from_columns: Vec<String>,

    /// Target (parent) table.
    pub to_table: String,

    /// Target column(s).
    pub to_columns: Vec<String>,

    /// Whether this is an inferred (not declared) relationship.
    pub inferred: bool,

    /// Confidence score for inferred relationships (0.0 to 1.0).
    pub confidence: f64,

    /// Coverage: proportion of child values that have matching parent.
    pub coverage: f64,

    /// Whether orphans exist (child values without parent).
    pub has_orphans: bool,

    /// Orphan rate (proportion of child values without parent).
    pub orphan_rate: f64,
}

impl ForeignKeyDef {
    /// Create a new foreign key definition.
    pub fn new(
        name: impl Into<String>,
        from_table: impl Into<String>,
        from_columns: Vec<String>,
        to_table: impl Into<String>,
        to_columns: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            from_table: from_table.into(),
            from_columns,
            to_table: to_table.into(),
            to_columns,
            inferred: false,
            confidence: 1.0,
            coverage: 1.0,
            has_orphans: false,
            orphan_rate: 0.0,
        }
    }

    /// Mark as inferred with a confidence score.
    pub fn as_inferred(mut self, confidence: f64) -> Self {
        self.inferred = true;
        self.confidence = confidence;
        self
    }
}

/// Cardinality statistics for a relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardinalityStats {
    /// Minimum children per parent.
    pub min_children: u64,

    /// Maximum children per parent.
    pub max_children: u64,

    /// Mean children per parent.
    pub mean_children: f64,

    /// Median children per parent.
    pub median_children: f64,

    /// Standard deviation of children per parent.
    pub std_dev_children: f64,

    /// Distribution of child counts (bucket: count).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_count_distribution: Option<Vec<CardinalityBucket>>,

    /// Percentage of parents with exactly one child.
    pub one_to_one_rate: f64,
}

impl CardinalityStats {
    /// Create basic cardinality stats.
    pub fn new(min: u64, max: u64, mean: f64, median: f64) -> Self {
        Self {
            min_children: min,
            max_children: max,
            mean_children: mean,
            median_children: median,
            std_dev_children: 0.0,
            child_count_distribution: None,
            one_to_one_rate: 0.0,
        }
    }

    /// Infer relationship type from statistics.
    pub fn infer_relationship_type(&self) -> RelationshipType {
        if self.max_children == 1 {
            RelationshipType::OneToOne
        } else if self.min_children == 0 && self.one_to_one_rate > 0.8 {
            RelationshipType::ZeroOrOne
        } else {
            RelationshipType::OneToMany
        }
    }
}

/// Bucket for cardinality distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardinalityBucket {
    /// Lower bound (inclusive).
    pub lower: u64,
    /// Upper bound (exclusive, None for unbounded).
    pub upper: Option<u64>,
    /// Proportion of parents in this bucket.
    pub proportion: f64,
}

/// Inferred relationship type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    /// Exactly one child per parent.
    OneToOne,
    /// Zero or one child per parent.
    ZeroOrOne,
    /// One or more children per parent.
    OneToMany,
    /// Zero or more children per parent.
    ZeroToMany,
}

/// Unique constraint definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniqueConstraint {
    /// Table name.
    pub table: String,

    /// Column(s) that form the unique constraint.
    pub columns: Vec<String>,

    /// Whether this is actually unique in the data.
    pub is_satisfied: bool,

    /// Number of duplicate groups if not satisfied.
    pub duplicate_groups: u64,
}

impl UniqueConstraint {
    /// Create a new unique constraint.
    pub fn new(table: impl Into<String>, columns: Vec<String>) -> Self {
        Self {
            table: table.into(),
            columns,
            is_satisfied: true,
            duplicate_groups: 0,
        }
    }
}

/// Check constraint definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckConstraint {
    /// Table name.
    pub table: String,

    /// Constraint name.
    pub name: String,

    /// Constraint expression (human-readable).
    pub expression: String,

    /// Column(s) involved.
    pub columns: Vec<String>,

    /// Satisfaction rate in the data.
    pub satisfaction_rate: f64,
}

impl CheckConstraint {
    /// Create a new check constraint.
    pub fn new(
        table: impl Into<String>,
        name: impl Into<String>,
        expression: impl Into<String>,
    ) -> Self {
        Self {
            table: table.into(),
            name: name.into(),
            expression: expression.into(),
            columns: Vec::new(),
            satisfaction_rate: 1.0,
        }
    }
}
