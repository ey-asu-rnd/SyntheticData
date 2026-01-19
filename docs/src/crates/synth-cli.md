# synth-cli

Command-line interface for synthetic accounting data generation.

## Overview

`synth-cli` provides the `synth-data` binary for command-line usage:

- **generate**: Generate synthetic data from configuration
- **init**: Create configuration files with industry presets
- **validate**: Validate configuration files
- **info**: Display available presets and options

## Installation

```bash
cargo build --release
# Binary at: target/release/synth-data
```

## Commands

### generate

Generate synthetic financial data.

```bash
# From configuration file
synth-data generate --config config.yaml --output ./output

# Demo mode with defaults
synth-data generate --demo --output ./demo-output

# Override seed
synth-data generate --config config.yaml --output ./output --seed 12345

# Verbose output
synth-data generate --config config.yaml --output ./output -v
```

### init

Create a configuration file from presets.

```bash
# Industry preset with complexity
synth-data init --industry manufacturing --complexity medium -o config.yaml
```

**Available industries:**
- `manufacturing`
- `retail`
- `financial_services`
- `healthcare`
- `technology`
- `energy`
- `telecom`
- `transportation`
- `hospitality`

### validate

Validate configuration files.

```bash
synth-data validate --config config.yaml
```

### info

Display available options.

```bash
synth-data info
```

## Key Types

### CLI Arguments

```rust
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress non-error output
    #[arg(short, long)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Command {
    Generate(GenerateArgs),
    Init(InitArgs),
    Validate(ValidateArgs),
    Info,
}
```

### Generate Arguments

```rust
pub struct GenerateArgs {
    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Use demo preset
    #[arg(long)]
    pub demo: bool,

    /// Output directory (required)
    #[arg(short, long)]
    pub output: PathBuf,

    /// Override random seed
    #[arg(long)]
    pub seed: Option<u64>,

    /// Output format
    #[arg(long, default_value = "csv")]
    pub format: String,
}
```

## Signal Handling

On Unix systems, pause/resume generation with `SIGUSR1`:

```bash
# Start in background
synth-data generate --config config.yaml --output ./output &

# Toggle pause
kill -USR1 $(pgrep synth-data)
```

Progress bar shows pause state:
```
[████████░░░░░░░░░░░░] 40% - 40000/100000 entries (PAUSED)
```

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | Configuration error |
| 2 | Generation error |
| 3 | I/O error |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SYNTH_DATA_LOG` | Log level (error, warn, info, debug, trace) |
| `SYNTH_DATA_THREADS` | Worker thread count |
| `SYNTH_DATA_MEMORY_LIMIT` | Memory limit in bytes |

```bash
SYNTH_DATA_LOG=debug synth-data generate --demo --output ./output
```

## Progress Display

During generation, a progress bar shows:

```
Generating synthetic data...
[████████████████████] 100% - 100000/100000 entries
Phase: Transactions | 85,432 entries/sec | ETA: 0:00

Generation complete!
- Journal entries: 100,000
- Document flows: 15,000
- Output: ./output/
- Duration: 1.2s
```

## Usage Examples

### Basic Generation

```bash
synth-data init --industry manufacturing -o config.yaml
synth-data generate --config config.yaml --output ./output
```

### Scripting

```bash
#!/bin/bash
for industry in manufacturing retail healthcare; do
    synth-data init --industry $industry --complexity medium -o ${industry}.yaml
    synth-data generate --config ${industry}.yaml --output ./output/${industry}
done
```

### CI/CD

```yaml
# GitHub Actions
- name: Generate Test Data
  run: |
    cargo build --release
    ./target/release/synth-data generate --demo --output ./test-data
```

### Reproducible Generation

```bash
# Same seed = same output
synth-data generate --config config.yaml --output ./run1 --seed 42
synth-data generate --config config.yaml --output ./run2 --seed 42
diff -r run1 run2  # No differences
```

## See Also

- [CLI Reference](../user-guide/cli-reference.md)
- [Quick Start](../getting-started/quick-start.md)
- [synth-runtime](synth-runtime.md)
