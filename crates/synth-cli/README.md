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

### Generate Data

```bash
# From configuration file
synth-data generate --config config.yaml --output ./output

# Demo mode with defaults
synth-data generate --demo --output ./demo-output

# With verbose logging
synth-data generate --config config.yaml --output ./output -v
```

### Create Configuration

```bash
# Industry preset with complexity level
synth-data init --industry manufacturing --complexity medium -o config.yaml

# Available industries:
#   manufacturing, retail, financial_services, healthcare,
#   technology, energy, telecom, transportation, hospitality
```

### Validate Configuration

```bash
synth-data validate --config config.yaml
```

### Show Options

```bash
synth-data info
```

## Signal Handling (Unix)

Toggle pause during generation:

```bash
kill -USR1 $(pgrep synth-data)
```

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | Configuration error |
| 2 | Generation error |
| 3 | I/O error |

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
