"""Validation helpers for DataSynth wrapper configs."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict, List


@dataclass(frozen=True)
class ValidationErrorDetail:
    path: str
    message: str
    value: Any


class ConfigValidationError(ValueError):
    """Raised when configuration validation fails."""

    def __init__(self, errors: List[ValidationErrorDetail]):
        self.errors = errors
        message = "Configuration validation failed: " + "; ".join(
            f"{error.path}: {error.message}" for error in errors
        )
        super().__init__(message)


def validate_config(config: Any) -> List[ValidationErrorDetail]:
    errors: List[ValidationErrorDetail] = []
    payload = config.to_dict()

    def add_error(path: str, message: str, value: Any) -> None:
        errors.append(ValidationErrorDetail(path=path, message=message, value=value))

    global_settings = payload.get("global", {})
    periods = global_settings.get("periods")
    if periods is not None and (periods < 1 or periods > 120):
        add_error("global.periods", "must be between 1 and 120", periods)

    companies = payload.get("companies", {})
    company_count = companies.get("count")
    if company_count is not None and company_count < 1:
        add_error("companies.count", "must be >= 1", company_count)

    transactions = payload.get("transactions", {})
    transaction_count = transactions.get("count")
    if transaction_count is not None and transaction_count < 1:
        add_error("transactions.count", "must be >= 1", transaction_count)

    anomaly_rate = transactions.get("anomaly_rate")
    if anomaly_rate is not None and (anomaly_rate < 0 or anomaly_rate > 1):
        add_error("transactions.anomaly_rate", "must be between 0 and 1", anomaly_rate)

    return errors
