# Python Wrapper Guide

This guide explains how to use the DataSynth Python wrapper for in-memory configuration, local CLI generation, and streaming generation through the server API.

## Installation

The wrapper lives in the repository under `python/`. Install it in development mode:

```bash
cd python
pip install -e ".[all]"
```

Or install just the core with specific extras:

```bash
pip install -e ".[cli]"      # For CLI generation (requires PyYAML)
pip install -e ".[memory]"   # For in-memory tables (requires pandas)
pip install -e ".[streaming]" # For streaming (requires websockets)
```

## Quick start (CLI generation)

```python
from datasynth_py import DataSynth, CompanyConfig, Config, GlobalSettings, ChartOfAccountsSettings

config = Config(
    global_settings=GlobalSettings(
        industry="retail",
        start_date="2024-01-01",
        period_months=12,
    ),
    companies=[
        CompanyConfig(code="C001", name="Retail Corp", currency="USD", country="US"),
    ],
    chart_of_accounts=ChartOfAccountsSettings(complexity="small"),
)

synth = DataSynth()
result = synth.generate(config=config, output={"format": "csv", "sink": "temp_dir"})

print(result.output_dir)  # Path to generated files
```

## Using blueprints

Blueprints provide preconfigured templates for common scenarios:

```python
from datasynth_py import DataSynth
from datasynth_py.config import blueprints

# List available blueprints
print(blueprints.list())  # ['banking_medium', 'manufacturing_large', 'retail_small']

# Create a retail configuration with 4 companies
config = blueprints.retail_small(companies=4, transactions=10000)

synth = DataSynth()
result = synth.generate(config=config, output={"format": "parquet", "sink": "path", "path": "./output"})
```

## Configuration model

The configuration model matches the CLI schema:

```python
from datasynth_py import (
    ChartOfAccountsSettings,
    CompanyConfig,
    Config,
    FraudSettings,
    GlobalSettings,
)

config = Config(
    global_settings=GlobalSettings(
        industry="manufacturing",      # Industry sector
        start_date="2024-01-01",       # Simulation start date
        period_months=12,              # Number of months to simulate
        seed=42,                       # Random seed for reproducibility
        group_currency="USD",          # Base currency
    ),
    companies=[
        CompanyConfig(
            code="M001",
            name="Manufacturing Co",
            currency="USD",
            country="US",
            annual_transaction_volume="ten_k",  # Volume preset
        ),
        CompanyConfig(
            code="M002",
            name="Manufacturing EU",
            currency="EUR",
            country="DE",
            annual_transaction_volume="hundred_k",
        ),
    ],
    chart_of_accounts=ChartOfAccountsSettings(
        complexity="medium",           # small, medium, or large
    ),
    fraud=FraudSettings(
        enabled=True,
        rate=0.01,                     # 1% fraud rate
    ),
)
```

### Valid industry values

- `manufacturing`
- `retail`
- `financial_services`
- `healthcare`
- `technology`
- `professional_services`
- `energy`
- `transportation`
- `real_estate`
- `telecommunications`

### Transaction volume presets

- `ten_k` - 10,000 transactions/year
- `hundred_k` - 100,000 transactions/year
- `one_m` - 1,000,000 transactions/year
- `ten_m` - 10,000,000 transactions/year
- `hundred_m` - 100,000,000 transactions/year

## Configuration layering

Override configuration values:

```python
from datasynth_py import Config, GlobalSettings

base = Config(global_settings=GlobalSettings(industry="retail", start_date="2024-01-01"))
custom = base.override(
    fraud={"enabled": True, "rate": 0.02},
)
```

## Validation

Validation raises `ConfigValidationError` with structured error details:

```python
from datasynth_py import Config, GlobalSettings
from datasynth_py.config.validation import ConfigValidationError

try:
    Config(global_settings=GlobalSettings(period_months=0)).validate()
except ConfigValidationError as exc:
    for error in exc.errors:
        print(error.path, error.message, error.value)
```

## Output options

Control where and how data is generated:

```python
from datasynth_py import DataSynth, OutputSpec

synth = DataSynth()

# Write to a specific path
result = synth.generate(
    config=config,
    output=OutputSpec(format="csv", sink="path", path="./output"),
)

# Write to a temporary directory
result = synth.generate(
    config=config,
    output=OutputSpec(format="parquet", sink="temp_dir"),
)
print(result.output_dir)  # Temp directory path

# Load into memory (requires pandas)
result = synth.generate(
    config=config,
    output=OutputSpec(format="csv", sink="memory"),
)
print(result.tables["journal_entries"].head())
```

## Streaming generation

Streaming uses the DataSynth server for real-time event generation. Start the server first:

```bash
cargo run -p datasynth-server -- --port 3000
```

Then stream events:

```python
import asyncio

from datasynth_py import DataSynth
from datasynth_py.config import blueprints


async def main() -> None:
    synth = DataSynth(server_url="http://localhost:3000")
    config = blueprints.retail_small(companies=2)
    session = synth.stream(config=config, events_per_second=100)

    async for event in session.events():
        print(event)
        break


asyncio.run(main())
```

### Stream controls

```python
session.pause()
session.resume()
session.stop()
```

## Runtime requirements

The wrapper shells out to the `datasynth-data` CLI for batch generation. Ensure the binary is available:

```bash
cargo build --release
export DATASYNTH_BINARY=target/release/datasynth-data
```

Alternatively, pass `binary_path` when creating the client:

```python
synth = DataSynth(binary_path="/path/to/datasynth-data")
```

## Troubleshooting

- **MissingDependencyError**: Install the required optional dependency (`PyYAML`, `pandas`, or `websockets`).
- **CLI not found**: Build the `datasynth-data` binary and set `DATASYNTH_BINARY` or pass `binary_path`.
- **ConfigValidationError**: Check the error details for invalid configuration values.
- **Streaming errors**: Verify the server is running and reachable at the configured URL.
