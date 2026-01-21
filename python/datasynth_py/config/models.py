"""Typed configuration models for the DataSynth Python wrapper."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional

import importlib.util

from datasynth_py.config.validation import ConfigValidationError, validate_config


@dataclass(frozen=True)
class GlobalSettings:
    """Global configuration settings matching the CLI schema."""

    industry: Optional[str] = None
    start_date: Optional[str] = None
    period_months: Optional[int] = None
    seed: Optional[int] = None
    group_currency: Optional[str] = None
    parallel: Optional[bool] = None
    worker_threads: Optional[int] = None
    memory_limit_mb: Optional[int] = None


@dataclass(frozen=True)
class CompanyConfig:
    """Single company configuration matching the CLI schema."""

    code: str
    name: str
    currency: str = "USD"
    country: str = "US"
    annual_transaction_volume: str = "ten_k"
    volume_weight: float = 1.0
    fiscal_year_variant: str = "K4"


@dataclass(frozen=True)
class ChartOfAccountsSettings:
    """Chart of Accounts configuration matching the CLI schema."""

    complexity: Optional[str] = None
    industry_specific: Optional[bool] = None


@dataclass(frozen=True)
class TransactionSettings:
    """Transaction generation settings."""

    # These are higher-level settings that map to the CLI schema
    count: Optional[int] = None
    currency: Optional[str] = None
    anomaly_rate: Optional[float] = None


@dataclass(frozen=True)
class OutputSettings:
    """Output configuration matching the CLI schema."""

    output_directory: Optional[str] = None
    formats: Optional[List[str]] = None
    compression_enabled: Optional[bool] = None
    compression_level: Optional[int] = None


@dataclass(frozen=True)
class FraudSettings:
    """Fraud simulation settings."""

    enabled: Optional[bool] = None
    rate: Optional[float] = None


@dataclass(frozen=True)
class Config:
    """Root configuration container.

    This model maps to the datasynth-cli GeneratorConfig schema.
    """

    global_settings: Optional[GlobalSettings] = None
    companies: Optional[List[CompanyConfig]] = None
    chart_of_accounts: Optional[ChartOfAccountsSettings] = None
    transactions: Optional[TransactionSettings] = None
    output: Optional[OutputSettings] = None
    fraud: Optional[FraudSettings] = None
    extra: Dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary matching CLI schema."""
        payload: Dict[str, Any] = {}

        if self.global_settings is not None:
            payload["global"] = _strip_none(self.global_settings.__dict__)

        if self.companies is not None:
            payload["companies"] = [
                _strip_none(c.__dict__) for c in self.companies
            ]

        if self.chart_of_accounts is not None:
            payload["chart_of_accounts"] = _strip_none(self.chart_of_accounts.__dict__)

        if self.transactions is not None:
            tx_dict = _strip_none(self.transactions.__dict__)
            # Map higher-level settings to CLI schema structure
            cli_transactions: Dict[str, Any] = {}
            if "count" in tx_dict:
                # The CLI doesn't have a direct 'count' field in transactions;
                # transaction count is derived from company volume settings
                pass
            if "currency" in tx_dict:
                # Currency is per-company in CLI schema
                pass
            if cli_transactions:
                payload["transactions"] = cli_transactions

        if self.output is not None:
            out_dict = _strip_none(self.output.__dict__)
            cli_output: Dict[str, Any] = {}
            if "output_directory" in out_dict:
                cli_output["output_directory"] = out_dict["output_directory"]
            if "formats" in out_dict:
                cli_output["formats"] = out_dict["formats"]
            if "compression_enabled" in out_dict or "compression_level" in out_dict:
                compression: Dict[str, Any] = {}
                if "compression_enabled" in out_dict:
                    compression["enabled"] = out_dict["compression_enabled"]
                if "compression_level" in out_dict:
                    compression["level"] = out_dict["compression_level"]
                cli_output["compression"] = compression
            if cli_output:
                payload["output"] = cli_output

        if self.fraud is not None:
            fraud_dict = _strip_none(self.fraud.__dict__)
            if fraud_dict:
                payload["fraud"] = fraud_dict

        # Merge extra fields
        payload.update(self.extra)
        return payload

    def to_json(self, **kwargs: Any) -> str:
        import json

        return json.dumps(self.to_dict(), **kwargs)

    def to_yaml(self) -> str:
        yaml_spec = importlib.util.find_spec("yaml")
        if yaml_spec is None:
            raise MissingDependencyError(
                "PyYAML is required for Config.to_yaml(). Install with `pip install PyYAML`."
            )
        import yaml  # type: ignore

        return yaml.safe_dump(self.to_dict(), sort_keys=False)

    def override(self, **overrides: Any) -> "Config":
        current = self.to_dict()
        merged = _deep_merge(current, overrides)
        return Config.from_dict(merged)

    def validate(self) -> None:
        errors = validate_config(self)
        if errors:
            raise ConfigValidationError(errors)

    @staticmethod
    def from_dict(data: Dict[str, Any]) -> "Config":
        global_settings = _build_dataclass(GlobalSettings, data.get("global"))
        companies_data = data.get("companies")
        companies = None
        if companies_data is not None:
            if isinstance(companies_data, list):
                companies = [CompanyConfig(**c) for c in companies_data]
            elif isinstance(companies_data, dict):
                # Handle legacy format where companies was a dict with count
                # Generate default companies based on count
                count = companies_data.get("count", 1)
                industry = companies_data.get("industry", "retail")
                companies = [
                    CompanyConfig(
                        code=f"C{i + 1:03d}",
                        name=f"Company {i + 1}",
                    )
                    for i in range(count)
                ]
                # Set industry in global if not already set
                if global_settings is None:
                    global_settings = GlobalSettings(industry=industry)
                elif global_settings.industry is None:
                    global_settings = GlobalSettings(
                        industry=industry,
                        start_date=global_settings.start_date,
                        period_months=global_settings.period_months,
                        seed=global_settings.seed,
                        group_currency=global_settings.group_currency,
                        parallel=global_settings.parallel,
                        worker_threads=global_settings.worker_threads,
                        memory_limit_mb=global_settings.memory_limit_mb,
                    )

        chart_of_accounts_data = data.get("chart_of_accounts")
        chart_of_accounts = _build_dataclass(ChartOfAccountsSettings, chart_of_accounts_data)
        # Handle legacy format where complexity was in companies
        if chart_of_accounts is None and isinstance(data.get("companies"), dict):
            complexity = data.get("companies", {}).get("complexity")
            if complexity:
                chart_of_accounts = ChartOfAccountsSettings(complexity=complexity)

        transactions = _build_dataclass(TransactionSettings, data.get("transactions"))
        output = _build_dataclass(OutputSettings, data.get("output"))
        fraud = _build_dataclass(FraudSettings, data.get("fraud"))

        known_keys = {"global", "companies", "chart_of_accounts", "transactions", "output", "fraud"}
        extra = {key: value for key, value in data.items() if key not in known_keys}

        return Config(
            global_settings=global_settings,
            companies=companies,
            chart_of_accounts=chart_of_accounts,
            transactions=transactions,
            output=output,
            fraud=fraud,
            extra=extra,
        )


# Legacy aliases for backwards compatibility
CompanySettings = CompanyConfig


def _strip_none(values: Dict[str, Any]) -> Dict[str, Any]:
    return {key: value for key, value in values.items() if value is not None}


def _deep_merge(base: Dict[str, Any], overrides: Dict[str, Any]) -> Dict[str, Any]:
    merged = dict(base)
    for key, value in overrides.items():
        if isinstance(value, dict) and isinstance(merged.get(key), dict):
            merged[key] = _deep_merge(merged[key], value)
        elif _is_dataclass_instance(value):
            merged[key] = _strip_none(value.__dict__)
        else:
            merged[key] = value
    return merged


def _build_dataclass(cls: Any, payload: Optional[Dict[str, Any]]) -> Optional[Any]:
    if payload is None:
        return None
    # Filter payload to only include fields that exist in the dataclass
    import dataclasses
    valid_fields = {f.name for f in dataclasses.fields(cls)}
    filtered_payload = {k: v for k, v in payload.items() if k in valid_fields}
    return cls(**filtered_payload)


def _is_dataclass_instance(value: Any) -> bool:
    return hasattr(value, "__dataclass_fields__")


class MissingDependencyError(RuntimeError):
    """Raised when an optional dependency is required but unavailable."""
