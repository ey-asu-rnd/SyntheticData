# Python Wrapper Guide

This guide explains how to use the DataSynth Python wrapper for in-memory configuration, local CLI generation, and streaming generation through the server API.

## Installation

The wrapper lives in the repository under `python/`. Use it directly in a virtual environment:

```bash
python -m venv .venv
source .venv/bin/activate
pip install -U pip
pip install pandas PyYAML websockets
```

Optional dependencies:

- **pandas**: required for in-memory tables (`sink="memory"`).
- **PyYAML**: required to serialize configs to YAML for CLI execution.
- **websockets**: required for streaming event consumption.

## Quick start (in-memory tables)

```python
from datasynth_py import DataSynth
from datasynth_py.config import Config, GlobalSettings, CompanySettings

config = Config(
    global_settings=GlobalSettings(locale="en_US", fiscal_year_start="2024-01-01"),
    companies=CompanySettings(count=2, industry="retail"),
)

synth = DataSynth()
result = synth.generate(config=config, output={"format": "csv", "sink": "memory"})

print(result.tables["journal_entries"].head())
```

## Writing output to disk

```python
from datasynth_py import DataSynth
from datasynth_py.config import blueprints

synth = DataSynth()
config = blueprints.retail_small(companies=3, transactions=15000)

result = synth.generate(
    config=config,
    output={"format": "parquet", "sink": "path", "path": "./output"},
    seed=42,
)

print(result.output_dir)
```

## Configuration layering

```python
from datasynth_py.config import Config, GlobalSettings, TransactionSettings

base = Config(global_settings=GlobalSettings(locale="en_GB"))
custom = base.override(
    transactions=TransactionSettings(count=25000, anomaly_rate=0.02),
)
```

## Validation errors

Validation raises `ConfigValidationError` with structured error details:

```python
from datasynth_py.config import Config, GlobalSettings
from datasynth_py.config.validation import ConfigValidationError

try:
    Config(global_settings=GlobalSettings(periods=0)).validate()
except ConfigValidationError as exc:
    for error in exc.errors:
        print(error.path, error.message, error.value)
```

## Streaming generation

Streaming uses the DataSynth server for event generation. Start the server first:

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
    config = blueprints.retail_small(companies=2, transactions=5000)
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

## Configuration blueprints

Blueprints are preconfigured templates:

```python
from datasynth_py.config import blueprints

config = blueprints.retail_small(companies=4, transactions=12000)
print(blueprints.list())
```

## Troubleshooting

- **Missing dependencies**: Install `pandas`, `PyYAML`, or `websockets` depending on the feature in use.
- **CLI not found**: Build the `datasynth-data` binary and set `DATASYNTH_BINARY`.
- **Streaming errors**: Verify the server is running and reachable at the configured URL.
