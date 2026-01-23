//! Schema extractor.

use crate::error::{FingerprintError, FingerprintResult};
use crate::models::{DataType, FieldSchema, SchemaFingerprint, TableSchema};
use crate::privacy::PrivacyEngine;

use super::{DataSource, ExtractionConfig, ExtractedComponent, Extractor};

/// Extractor for schema information.
pub struct SchemaExtractor;

impl Extractor for SchemaExtractor {
    fn name(&self) -> &'static str {
        "schema"
    }

    fn extract(
        &self,
        data: &DataSource,
        config: &ExtractionConfig,
        privacy: &mut PrivacyEngine,
    ) -> FingerprintResult<ExtractedComponent> {
        let schema = match data {
            DataSource::Csv(csv) => extract_from_csv(csv, config, privacy)?,
            DataSource::Memory(mem) => extract_from_memory(mem, config, privacy)?,
        };

        Ok(ExtractedComponent::Schema(schema))
    }
}

/// Extract schema from CSV.
fn extract_from_csv(
    csv: &super::CsvDataSource,
    config: &ExtractionConfig,
    privacy: &mut PrivacyEngine,
) -> FingerprintResult<SchemaFingerprint> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(csv.has_headers)
        .delimiter(csv.delimiter)
        .from_path(&csv.path)?;

    let headers: Vec<String> = reader.headers()?.iter().map(|s| s.to_string()).collect();

    // Sample rows to infer types
    let mut sample_rows: Vec<Vec<String>> = Vec::new();
    let mut row_count = 0u64;

    for result in reader.records() {
        let record = result?;
        row_count += 1;

        if sample_rows.len() < 1000 {
            sample_rows.push(record.iter().map(|s| s.to_string()).collect());
        }
    }

    // Check minimum rows
    if row_count < config.min_rows as u64 {
        return Err(FingerprintError::InsufficientData {
            required: config.min_rows,
            actual: row_count as usize,
        });
    }

    // Infer column types
    let columns: Vec<FieldSchema> = headers
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let values: Vec<&str> = sample_rows
                .iter()
                .filter_map(|row| row.get(i).map(|s| s.as_str()))
                .collect();

            let data_type = infer_data_type(&values);
            let null_rate = values.iter().filter(|v| v.is_empty()).count() as f64 / values.len() as f64;
            let cardinality = estimate_cardinality(&values);

            FieldSchema::new(name.clone(), data_type)
                .with_nullable(null_rate)
                .with_cardinality(cardinality)
        })
        .collect();

    // Add noise to row count
    let noised_row_count = privacy.add_noise_to_count(row_count, "schema.row_count")?;

    let table_name = csv.path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("data")
        .to_string();

    let mut table = TableSchema::new(&table_name, noised_row_count);
    for col in columns {
        table.add_column(col);
    }

    let mut schema = SchemaFingerprint::new();
    schema.add_table(table_name, table);

    Ok(schema)
}

/// Extract schema from memory.
fn extract_from_memory(
    mem: &super::MemoryDataSource,
    config: &ExtractionConfig,
    privacy: &mut PrivacyEngine,
) -> FingerprintResult<SchemaFingerprint> {
    let row_count = mem.row_count() as u64;

    if row_count < config.min_rows as u64 {
        return Err(FingerprintError::InsufficientData {
            required: config.min_rows,
            actual: row_count as usize,
        });
    }

    let columns: Vec<FieldSchema> = mem.columns
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let values: Vec<&str> = mem.rows
                .iter()
                .filter_map(|row| row.get(i).map(|s| s.as_str()))
                .collect();

            let data_type = infer_data_type(&values);
            let null_rate = values.iter().filter(|v| v.is_empty()).count() as f64 / values.len().max(1) as f64;
            let cardinality = estimate_cardinality(&values);

            FieldSchema::new(name.clone(), data_type)
                .with_nullable(null_rate)
                .with_cardinality(cardinality)
        })
        .collect();

    let noised_row_count = privacy.add_noise_to_count(row_count, "schema.row_count")?;

    let mut table = TableSchema::new("memory", noised_row_count);
    for col in columns {
        table.add_column(col);
    }

    let mut schema = SchemaFingerprint::new();
    schema.add_table("memory", table);

    Ok(schema)
}

/// Infer data type from sample values.
fn infer_data_type(values: &[&str]) -> DataType {
    let non_empty: Vec<_> = values.iter().filter(|v| !v.is_empty()).collect();
    if non_empty.is_empty() {
        return DataType::String;
    }

    // Check for boolean
    let all_bool = non_empty.iter().all(|v| {
        let lower = v.to_lowercase();
        lower == "true" || lower == "false" || lower == "1" || lower == "0"
    });
    if all_bool {
        return DataType::Boolean;
    }

    // Check for integer
    let all_int = non_empty.iter().all(|v| v.parse::<i64>().is_ok());
    if all_int {
        return DataType::Int64;
    }

    // Check for decimal/float
    let all_float = non_empty.iter().all(|v| v.parse::<f64>().is_ok());
    if all_float {
        // Check if it looks like a decimal (has decimal point)
        let has_decimal = non_empty.iter().any(|v| v.contains('.'));
        return if has_decimal { DataType::Decimal } else { DataType::Float64 };
    }

    // Check for date patterns
    let date_patterns = [
        r"^\d{4}-\d{2}-\d{2}$",  // YYYY-MM-DD
        r"^\d{2}/\d{2}/\d{4}$",  // MM/DD/YYYY
        r"^\d{2}\.\d{2}\.\d{4}$", // DD.MM.YYYY
    ];
    let all_date = non_empty.iter().all(|v| {
        date_patterns.iter().any(|p| {
            regex_lite::Regex::new(p).map(|r| r.is_match(v)).unwrap_or(false)
        })
    });
    if all_date {
        return DataType::Date;
    }

    // Check for UUID
    let all_uuid = non_empty.iter().all(|v| {
        uuid::Uuid::parse_str(v).is_ok()
    });
    if all_uuid {
        return DataType::Uuid;
    }

    DataType::String
}

/// Estimate cardinality from sample.
fn estimate_cardinality(values: &[&str]) -> u64 {
    let unique: std::collections::HashSet<_> = values.iter().collect();
    unique.len() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_data_type() {
        assert_eq!(infer_data_type(&["1", "2", "3"]), DataType::Int64);
        assert_eq!(infer_data_type(&["1.5", "2.5", "3.5"]), DataType::Decimal);
        assert_eq!(infer_data_type(&["true", "false"]), DataType::Boolean);
        assert_eq!(infer_data_type(&["2024-01-15", "2024-02-20"]), DataType::Date);
        assert_eq!(infer_data_type(&["hello", "world"]), DataType::String);
    }
}
