//! Schema fingerprint models.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Schema fingerprint containing table and column definitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaFingerprint {
    /// Table schemas indexed by table name.
    pub tables: HashMap<String, TableSchema>,

    /// Detected relationships between tables.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<TableRelationship>,
}

impl SchemaFingerprint {
    /// Create a new empty schema fingerprint.
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            relationships: Vec::new(),
        }
    }

    /// Add a table schema.
    pub fn add_table(&mut self, name: impl Into<String>, schema: TableSchema) {
        self.tables.insert(name.into(), schema);
    }

    /// Get the total number of columns across all tables.
    pub fn total_columns(&self) -> usize {
        self.tables.values().map(|t| t.columns.len()).sum()
    }

    /// Get a table schema by name.
    pub fn get_table(&self, name: &str) -> Option<&TableSchema> {
        self.tables.get(name)
    }
}

impl Default for SchemaFingerprint {
    fn default() -> Self {
        Self::new()
    }
}

/// Schema for a single table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    /// Table name.
    pub name: String,

    /// Row count (with optional noise for privacy).
    pub row_count: u64,

    /// Column schemas in order.
    pub columns: Vec<FieldSchema>,

    /// Primary key column names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub primary_key: Vec<String>,

    /// Additional table metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

impl TableSchema {
    /// Create a new table schema.
    pub fn new(name: impl Into<String>, row_count: u64) -> Self {
        Self {
            name: name.into(),
            row_count,
            columns: Vec::new(),
            primary_key: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a column schema.
    pub fn add_column(&mut self, column: FieldSchema) {
        self.columns.push(column);
    }

    /// Get a column by name.
    pub fn get_column(&self, name: &str) -> Option<&FieldSchema> {
        self.columns.iter().find(|c| c.name == name)
    }
}

/// Schema for a single field/column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    /// Column name.
    pub name: String,

    /// Data type.
    pub data_type: DataType,

    /// Whether the column allows nulls.
    pub nullable: bool,

    /// Null rate (0.0 to 1.0).
    pub null_rate: f64,

    /// Cardinality (unique value count, may be noised).
    pub cardinality: u64,

    /// Whether this is a primary key column.
    #[serde(default)]
    pub is_primary_key: bool,

    /// Whether this is a foreign key column.
    #[serde(default)]
    pub is_foreign_key: bool,

    /// Foreign key reference if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreign_key_ref: Option<ForeignKeyRef>,

    /// Semantic type hint (e.g., "currency", "date", "email").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_type: Option<String>,

    /// Additional field metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

impl FieldSchema {
    /// Create a new field schema.
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable: false,
            null_rate: 0.0,
            cardinality: 0,
            is_primary_key: false,
            is_foreign_key: false,
            foreign_key_ref: None,
            semantic_type: None,
            metadata: HashMap::new(),
        }
    }

    /// Set nullable flag and null rate.
    pub fn with_nullable(mut self, null_rate: f64) -> Self {
        self.nullable = null_rate > 0.0;
        self.null_rate = null_rate;
        self
    }

    /// Set cardinality.
    pub fn with_cardinality(mut self, cardinality: u64) -> Self {
        self.cardinality = cardinality;
        self
    }

    /// Mark as primary key.
    pub fn as_primary_key(mut self) -> Self {
        self.is_primary_key = true;
        self
    }

    /// Mark as foreign key with reference.
    pub fn as_foreign_key(mut self, reference: ForeignKeyRef) -> Self {
        self.is_foreign_key = true;
        self.foreign_key_ref = Some(reference);
        self
    }

    /// Set semantic type.
    pub fn with_semantic_type(mut self, semantic_type: impl Into<String>) -> Self {
        self.semantic_type = Some(semantic_type.into());
        self
    }
}

/// Supported data types for fingerprinting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    /// Boolean type.
    Boolean,
    /// 32-bit integer.
    Int32,
    /// 64-bit integer.
    Int64,
    /// 32-bit floating point.
    Float32,
    /// 64-bit floating point.
    Float64,
    /// Fixed-precision decimal.
    Decimal,
    /// Variable-length string.
    String,
    /// Date without time.
    Date,
    /// Timestamp with timezone.
    Timestamp,
    /// Time without date.
    Time,
    /// UUID.
    Uuid,
    /// Binary data.
    Binary,
    /// JSON object.
    Json,
    /// Unknown/unsupported type.
    Unknown,
}

impl DataType {
    /// Check if this is a numeric type.
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::Int32 | Self::Int64 | Self::Float32 | Self::Float64 | Self::Decimal
        )
    }

    /// Check if this is a temporal type.
    pub fn is_temporal(&self) -> bool {
        matches!(self, Self::Date | Self::Timestamp | Self::Time)
    }

    /// Check if this is a string-like type.
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String | Self::Uuid)
    }

    /// Check if this is categorical (low cardinality expected).
    pub fn is_categorical(&self) -> bool {
        matches!(self, Self::Boolean | Self::String)
    }
}

/// Foreign key reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyRef {
    /// Referenced table name.
    pub table: String,
    /// Referenced column name.
    pub column: String,
}

impl ForeignKeyRef {
    /// Create a new foreign key reference.
    pub fn new(table: impl Into<String>, column: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            column: column.into(),
        }
    }
}

/// Relationship between tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRelationship {
    /// Source table.
    pub from_table: String,
    /// Source column.
    pub from_column: String,
    /// Target table.
    pub to_table: String,
    /// Target column.
    pub to_column: String,
    /// Relationship cardinality.
    pub cardinality: RelationshipCardinality,
    /// Confidence score (0.0 to 1.0) for detected relationships.
    pub confidence: f64,
}

/// Relationship cardinality types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipCardinality {
    /// One-to-one relationship.
    OneToOne,
    /// One-to-many relationship.
    OneToMany,
    /// Many-to-one relationship.
    ManyToOne,
    /// Many-to-many relationship.
    ManyToMany,
}
