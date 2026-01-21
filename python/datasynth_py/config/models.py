"""Typed configuration models for the DataSynth Python wrapper."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict, Optional

import importlib

from datasynth_py.config.validation import ConfigValidationError, validate_config


@dataclass(frozen=True)
class GlobalSettings:
    locale: Optional[str] = None
    fiscal_year_start: Optional[str] = None
    periods: Optional[int] = None
    seed: Optional[int] = None
    currency: Optional[str] = None


@dataclass(frozen=True)
class CompanySettings:
    count: Optional[int] = None
    industry: Optional[str] = None
    complexity: Optional[str] = None


@dataclass(frozen=True)
class TransactionSettings:
    count: Optional[int] = None
    currency: Optional[str] = None
    anomaly_rate: Optional[float] = None


@dataclass(frozen=True)
class OutputSettings:
    format: Optional[str] = None
    compression: Optional[str] = None
    path: Optional[str] = None


@dataclass(frozen=True)
class Config:
    global_settings: Optional[GlobalSettings] = None
    companies: Optional[CompanySettings] = None
    transactions: Optional[TransactionSettings] = None
    output: Optional[OutputSettings] = None
    extra: Dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        payload: Dict[str, Any] = {}
        if self.global_settings is not None:
            payload["global"] = _strip_none(self.global_settings.__dict__)
        if self.companies is not None:
            payload["companies"] = _strip_none(self.companies.__dict__)
        if self.transactions is not None:
            payload["transactions"] = _strip_none(self.transactions.__dict__)
        if self.output is not None:
            payload["output"] = _strip_none(self.output.__dict__)
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
        companies = _build_dataclass(CompanySettings, data.get("companies"))
        transactions = _build_dataclass(TransactionSettings, data.get("transactions"))
        output = _build_dataclass(OutputSettings, data.get("output"))
        known_keys = {"global", "companies", "transactions", "output"}
        extra = {key: value for key, value in data.items() if key not in known_keys}
        return Config(
            global_settings=global_settings,
            companies=companies,
            transactions=transactions,
            output=output,
            extra=extra,
        )


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
    return cls(**payload)


def _is_dataclass_instance(value: Any) -> bool:
    return hasattr(value, "__dataclass_fields__")


class MissingDependencyError(RuntimeError):
    """Raised when an optional dependency is required but unavailable."""
