//! JSON/JSONL output sink.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use synth_core::error::{SynthError, SynthResult};
use synth_core::models::JournalEntry;
use synth_core::traits::Sink;

/// JSON Lines sink for journal entry output.
pub struct JsonLinesSink {
    writer: BufWriter<File>,
    items_written: u64,
}

impl JsonLinesSink {
    /// Create a new JSON Lines sink.
    pub fn new(path: PathBuf) -> SynthResult<Self> {
        let file = File::create(&path)?;
        Ok(Self {
            writer: BufWriter::new(file),
            items_written: 0,
        })
    }
}

impl Sink for JsonLinesSink {
    type Item = JournalEntry;

    fn write(&mut self, item: Self::Item) -> SynthResult<()> {
        let json = serde_json::to_string(&item)
            .map_err(|e| SynthError::SerializationError(e.to_string()))?;
        self.writer.write_all(json.as_bytes())?;
        self.writer.write_all(b"\n")?;
        self.items_written += 1;
        Ok(())
    }

    fn flush(&mut self) -> SynthResult<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn close(mut self) -> SynthResult<()> {
        self.flush()?;
        Ok(())
    }

    fn items_written(&self) -> u64 {
        self.items_written
    }
}
