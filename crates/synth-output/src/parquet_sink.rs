//! Parquet output sink.
//!
//! Placeholder for Parquet output - full implementation requires Arrow/Parquet integration.

use std::path::PathBuf;

use synth_core::error::SynthResult;
use synth_core::models::JournalEntry;
use synth_core::traits::Sink;

/// Parquet sink for journal entry output.
pub struct ParquetSink {
    #[allow(dead_code)]
    path: PathBuf,
    items_written: u64,
    buffer: Vec<JournalEntry>,
    batch_size: usize,
}

impl ParquetSink {
    /// Create a new Parquet sink.
    pub fn new(path: PathBuf, batch_size: usize) -> SynthResult<Self> {
        Ok(Self {
            path,
            items_written: 0,
            buffer: Vec::with_capacity(batch_size),
            batch_size,
        })
    }

    fn flush_buffer(&mut self) -> SynthResult<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        // TODO: Implement actual Parquet writing with Arrow
        // For now, just clear the buffer
        self.buffer.clear();
        Ok(())
    }
}

impl Sink for ParquetSink {
    type Item = JournalEntry;

    fn write(&mut self, item: Self::Item) -> SynthResult<()> {
        self.buffer.push(item);
        self.items_written += 1;

        if self.buffer.len() >= self.batch_size {
            self.flush_buffer()?;
        }

        Ok(())
    }

    fn flush(&mut self) -> SynthResult<()> {
        self.flush_buffer()
    }

    fn close(mut self) -> SynthResult<()> {
        self.flush()
    }

    fn items_written(&self) -> u64 {
        self.items_written
    }
}
