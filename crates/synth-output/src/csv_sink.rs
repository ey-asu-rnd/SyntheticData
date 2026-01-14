//! CSV output sink.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use synth_core::error::SynthResult;
use synth_core::models::JournalEntry;
use synth_core::traits::Sink;

/// CSV sink for journal entry output.
pub struct CsvSink {
    writer: BufWriter<File>,
    items_written: u64,
    header_written: bool,
}

impl CsvSink {
    /// Create a new CSV sink.
    pub fn new(path: PathBuf) -> SynthResult<Self> {
        let file = File::create(&path)?;
        Ok(Self {
            writer: BufWriter::new(file),
            items_written: 0,
            header_written: false,
        })
    }

    fn write_header(&mut self) -> SynthResult<()> {
        if self.header_written {
            return Ok(());
        }

        let header = "document_id,company_code,fiscal_year,fiscal_period,posting_date,\
            document_type,currency,source,line_number,gl_account,debit_amount,credit_amount\n";
        self.writer.write_all(header.as_bytes())?;
        self.header_written = true;
        Ok(())
    }
}

impl Sink for CsvSink {
    type Item = JournalEntry;

    fn write(&mut self, item: Self::Item) -> SynthResult<()> {
        self.write_header()?;

        for line in &item.lines {
            let row = format!(
                "{},{},{},{},{},{},{},{},{},{},{},{}\n",
                item.header.document_id,
                item.header.company_code,
                item.header.fiscal_year,
                item.header.fiscal_period,
                item.header.posting_date,
                item.header.document_type,
                item.header.currency,
                format!("{:?}", item.header.source),
                line.line_number,
                line.gl_account,
                line.debit_amount,
                line.credit_amount,
            );
            self.writer.write_all(row.as_bytes())?;
        }

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
