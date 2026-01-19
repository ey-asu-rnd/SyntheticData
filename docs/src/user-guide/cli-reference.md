# CLI Reference

The `synth-data` command-line tool provides commands for generating synthetic financial data.

## Installation

After building the project, the binary is at `target/release/synth-data`.

```bash
cargo build --release
./target/release/synth-data --help
```

## Global Options

| Option | Description |
|--------|-------------|
| `-h, --help` | Show help information |
| `-V, --version` | Show version number |
| `-v, --verbose` | Enable verbose output |
| `-q, --quiet` | Suppress non-error output |

## Commands

### generate

Generate synthetic financial data.

```bash
synth-data generate [OPTIONS]
```

**Options:**

| Option | Type | Description |
|--------|------|-------------|
| `--config <PATH>` | Path | Configuration YAML file |
| `--demo` | Flag | Use demo preset instead of config |
| `--output <DIR>` | Path | Output directory (required) |
| `--format <FMT>` | String | Output format: csv, json |
| `--seed <NUM>` | u64 | Override random seed |

**Examples:**

```bash
# Generate with configuration file
synth-data generate --config config.yaml --output ./output

# Use demo mode
synth-data generate --demo --output ./demo-output

# Override seed for reproducibility
synth-data generate --config config.yaml --output ./output --seed 12345

# JSON output format
synth-data generate --config config.yaml --output ./output --format json
```

### init

Create a new configuration file from industry presets.

```bash
synth-data init [OPTIONS]
```

**Options:**

| Option | Type | Description |
|--------|------|-------------|
| `--industry <NAME>` | String | Industry preset |
| `--complexity <LEVEL>` | String | small, medium, large |
| `-o, --output <PATH>` | Path | Output file path |

**Available Industries:**
- `manufacturing`
- `retail`
- `financial_services`
- `healthcare`
- `technology`

**Examples:**

```bash
# Create manufacturing config
synth-data init --industry manufacturing --complexity medium -o config.yaml

# Create large retail config
synth-data init --industry retail --complexity large -o retail.yaml
```

### validate

Validate a configuration file.

```bash
synth-data validate --config <PATH>
```

**Options:**

| Option | Type | Description |
|--------|------|-------------|
| `--config <PATH>` | Path | Configuration file to validate |

**Example:**

```bash
synth-data validate --config config.yaml
```

**Validation Checks:**
- Required fields present
- Value ranges (period_months: 1-120)
- Distribution weights sum to 1.0
- Date consistency
- Company code uniqueness

### info

Display available presets and configuration options.

```bash
synth-data info
```

**Output includes:**
- Available industry presets
- Complexity levels
- Supported output formats
- Feature capabilities

## Signal Handling (Unix)

On Unix systems, you can pause and resume generation:

```bash
# Start generation in background
synth-data generate --config config.yaml --output ./output &

# Pause generation
kill -USR1 $(pgrep synth-data)

# Resume generation
kill -USR1 $(pgrep synth-data)
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | I/O error |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SYNTH_DATA_LOG` | Log level (error, warn, info, debug, trace) |
| `SYNTH_DATA_THREADS` | Number of worker threads |

**Example:**

```bash
SYNTH_DATA_LOG=debug synth-data generate --config config.yaml --output ./output
```

## Configuration File Location

The tool looks for configuration files in this order:
1. Path specified with `--config`
2. `./synth-data.yaml` in current directory
3. `~/.config/synth-data/config.yaml`

## Output Directory Structure

Generation creates this structure:

```
output/
├── master_data/
├── transactions/
├── subledgers/
├── period_close/
├── consolidation/
├── fx/
├── graphs/          # If graph_export enabled
├── labels/          # If anomaly/fraud enabled
└── controls/
```

## Scripting Examples

### Batch Generation

```bash
#!/bin/bash
for industry in manufacturing retail healthcare; do
    synth-data init --industry $industry --complexity medium -o ${industry}.yaml
    synth-data generate --config ${industry}.yaml --output ./output/${industry}
done
```

### CI/CD Pipeline

```yaml
# GitHub Actions example
- name: Generate Test Data
  run: |
    cargo build --release
    ./target/release/synth-data generate --demo --output ./test-data
```

### Reproducible Generation

```bash
# Same seed produces identical output
synth-data generate --config config.yaml --output ./run1 --seed 42
synth-data generate --config config.yaml --output ./run2 --seed 42
diff -r run1 run2  # No differences
```

## Troubleshooting

### Common Issues

**"Configuration file not found"**
```bash
# Check file path
ls -la config.yaml
# Use absolute path
synth-data generate --config /full/path/to/config.yaml --output ./output
```

**"Invalid configuration"**
```bash
# Validate first
synth-data validate --config config.yaml
```

**"Permission denied"**
```bash
# Check output directory permissions
mkdir -p ./output
chmod 755 ./output
```

**"Out of memory"**
Reduce transaction count or enable streaming in configuration.

## See Also

- [Quick Start](../getting-started/quick-start.md)
- [Configuration Reference](../configuration/README.md)
- [Output Formats](output-formats.md)
