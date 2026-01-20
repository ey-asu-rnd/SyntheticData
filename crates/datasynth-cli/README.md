# datasynth-cli

Command-line interface for synthetic accounting data generation.

## Overview

`datasynth-cli` provides the `datasynth-data` binary for command-line usage:

- **generate**: Generate synthetic data from configuration
- **init**: Create configuration files with industry presets
- **validate**: Validate configuration files
- **info**: Display available presets and options

## Installation

```bash
cargo build --release
# Binary at: target/release/datasynth-data
```

## Commands

### Generate Data

```bash
# From configuration file
datasynth-data generate --config config.yaml --output ./output

# Demo mode with defaults
datasynth-data generate --demo --output ./demo-output

# With verbose logging
datasynth-data generate --config config.yaml --output ./output -v
```

### Create Configuration

```bash
# Industry preset with complexity level
datasynth-data init --industry manufacturing --complexity medium -o config.yaml

# Available industries:
#   manufacturing, retail, financial_services, healthcare,
#   technology, energy, telecom, transportation, hospitality
```

### Validate Configuration

```bash
datasynth-data validate --config config.yaml
```

### Show Options

```bash
datasynth-data info
```

## Signal Handling (Unix)

Toggle pause during generation:

```bash
kill -USR1 $(pgrep datasynth-data)
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
