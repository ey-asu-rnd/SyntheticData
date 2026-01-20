//! Parquet output sink.
//!
//! NOTE: Parquet output is not yet fully implemented. Use CSV or JSON format instead.
//! This sink will return an error when used to prevent silent data loss.

use std::path::PathBuf;

use datasynth_core::error::{SynthError, SynthResult};
use datasynth_core::models::JournalEntry;
use datasynth_core::traits::Sink;

/// Parquet sink for journal entry output.
///
/// # Status: Not Implemented
///
/// This sink is a placeholder. Parquet format requires full Arrow schema
/// integration which is not yet complete. Using this sink will return
/// an error rather than silently discarding data.
///
/// Use `CsvSink` or `JsonSink` instead for production workloads.
#[derive(Debug)]
pub struct ParquetSink {
    path: PathBuf,
    items_written: u64,
}

impl ParquetSink {
    /// Create a new Parquet sink.
    ///
    /// # Errors
    ///
    /// Returns an error immediately indicating Parquet format is not supported.
    /// Use CSV or JSON format instead.
    pub fn new(path: PathBuf, _batch_size: usize) -> SynthResult<Self> {
        // Return error immediately on construction to fail fast
        Err(SynthError::config(format!(
            "Parquet output format is not yet implemented. \
             Requested output path: {}. \
             Please use 'csv' or 'json' format instead. \
             See https://github.com/your-repo/issues for tracking.",
            path.display()
        )))
    }
}

impl Sink for ParquetSink {
    type Item = JournalEntry;

    fn write(&mut self, _item: Self::Item) -> SynthResult<()> {
        // This should never be reached since new() returns an error,
        // but if somehow called, return an error
        Err(SynthError::config(format!(
            "Parquet output is not implemented. Cannot write to {}. Use CSV or JSON format.",
            self.path.display()
        )))
    }

    fn flush(&mut self) -> SynthResult<()> {
        // No-op since nothing is buffered
        Ok(())
    }

    fn close(self) -> SynthResult<()> {
        // No-op since nothing to close
        Ok(())
    }

    fn items_written(&self) -> u64 {
        self.items_written
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.parquet");

        let result = ParquetSink::new(path, 1000);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not yet implemented"));
        assert!(err.to_string().contains("csv"));
        assert!(err.to_string().contains("json"));
    }
}
