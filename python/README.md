# datasynth-py

Python wrapper for the DataSynth synthetic data generator.

## Installation

### From PyPI

```bash
pip install datasynth-py[all]
```

Or install specific extras:

```bash
pip install datasynth-py           # Core only (no dependencies)
pip install datasynth-py[cli]      # CLI generation (PyYAML)
pip install datasynth-py[memory]   # In-memory tables (pandas)
pip install datasynth-py[streaming] # Streaming (websockets)
pip install datasynth-py[all]      # All optional dependencies
```

### From Source

```bash
cd python
pip install -e ".[all]"
```

## Quick Start

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
print(result.output_dir)
```

## Using Blueprints

```python
from datasynth_py import DataSynth
from datasynth_py.config import blueprints

config = blueprints.retail_small(companies=4, transactions=10000)
synth = DataSynth()
result = synth.generate(config=config, output={"format": "parquet", "sink": "path", "path": "./output"})
```

## Requirements

The wrapper shells out to the `datasynth-data` CLI binary. Build it with:

```bash
cargo build --release
export DATASYNTH_BINARY=target/release/datasynth-data
```

Or pass `binary_path` when creating the client:

```python
synth = DataSynth(binary_path="/path/to/datasynth-data")
```

## Documentation

See the [Python Wrapper Guide](../docs/src/user-guide/python-wrapper.md) for complete documentation.

## License

MIT License - see the main project LICENSE file.
