# synth-output

Output sinks for CSV, JSON, and streaming formats.

## Overview

`synth-output` provides the output layer for SyntheticData:

- **CSV Sink**: High-performance CSV writing with optional compression
- **JSON Sink**: JSON and JSONL (newline-delimited) output
- **Streaming**: Async streaming output for real-time generation
- **Control Export**: Internal control and SoD rule export

## Supported Formats

| Format | Description |
|--------|-------------|
| CSV | Standard comma-separated values |
| JSON | Pretty-printed JSON arrays |
| JSONL | Newline-delimited JSON (streaming-friendly) |

## Features

- Configurable compression (gzip, zstd)
- Streaming writes for memory efficiency
- Decimal values serialized as strings (IEEE 754 safe)
- Configurable field ordering and headers

## Usage

```rust
use synth_output::{CsvSink, JsonSink, OutputConfig};

// CSV output
let sink = CsvSink::new("output/journal_entries.csv", config)?;
sink.write_batch(&entries)?;

// JSON streaming
let sink = JsonSink::new("output/entries.jsonl", OutputConfig::jsonl())?;
for entry in entries {
    sink.write(&entry)?;
}
```

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
