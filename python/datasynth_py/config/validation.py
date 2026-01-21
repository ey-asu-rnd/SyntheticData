"""Validation helpers for DataSynth wrapper configs."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, List


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
    """Validate a Config object against CLI schema requirements."""
    errors: List[ValidationErrorDetail] = []
    payload = config.to_dict()

    def add_error(path: str, message: str, value: Any) -> None:
        errors.append(ValidationErrorDetail(path=path, message=message, value=value))

    # Validate global settings
    global_settings = payload.get("global", {})

    period_months = global_settings.get("period_months")
    if period_months is not None and (period_months < 1 or period_months > 120):
        add_error("global.period_months", "must be between 1 and 120", period_months)

    industry = global_settings.get("industry")
    valid_industries = {
        "manufacturing", "retail", "financial_services", "healthcare",
        "technology", "professional_services", "energy", "transportation",
        "real_estate", "telecommunications",
    }
    if industry is not None and industry.lower() not in valid_industries:
        add_error("global.industry", f"must be one of {sorted(valid_industries)}", industry)

    # Validate companies
    companies = payload.get("companies", [])
    if isinstance(companies, list):
        if len(companies) < 1:
            add_error("companies", "must have at least one company", len(companies))
        for i, company in enumerate(companies):
            if not company.get("code"):
                add_error(f"companies[{i}].code", "is required", company.get("code"))
            if not company.get("name"):
                add_error(f"companies[{i}].name", "is required", company.get("name"))

    # Validate chart_of_accounts
    coa = payload.get("chart_of_accounts", {})
    complexity = coa.get("complexity")
    if complexity is not None and complexity.lower() not in {"small", "medium", "large"}:
        add_error("chart_of_accounts.complexity", "must be small, medium, or large", complexity)

    # Validate fraud settings
    fraud = payload.get("fraud", {})
    fraud_rate = fraud.get("rate")
    if fraud_rate is not None and (fraud_rate < 0 or fraud_rate > 1):
        add_error("fraud.rate", "must be between 0 and 1", fraud_rate)

    # Also check anomaly_rate in transactions for backwards compatibility
    transactions = payload.get("transactions", {})
    anomaly_rate = transactions.get("anomaly_rate")
    if anomaly_rate is not None and (anomaly_rate < 0 or anomaly_rate > 1):
        add_error("transactions.anomaly_rate", "must be between 0 and 1", anomaly_rate)

    return errors
