"""Blueprint registry for common DataSynth configurations."""

from __future__ import annotations

from typing import Callable, Dict, List

from datasynth_py.config.models import (
    CompanySettings,
    Config,
    GlobalSettings,
    TransactionSettings,
)

BlueprintFactory = Callable[..., Config]


def retail_small(companies: int = 3, transactions: int = 5000) -> Config:
    return Config(
        global_settings=GlobalSettings(locale="en_US", fiscal_year_start="2024-01-01"),
        companies=CompanySettings(count=companies, industry="retail", complexity="small"),
        transactions=TransactionSettings(count=transactions),
    )


def banking_medium(companies: int = 5, transactions: int = 20000) -> Config:
    return Config(
        global_settings=GlobalSettings(locale="en_US", fiscal_year_start="2024-01-01"),
        companies=CompanySettings(count=companies, industry="financial_services", complexity="medium"),
        transactions=TransactionSettings(count=transactions, anomaly_rate=0.01),
    )


_REGISTRY: Dict[str, BlueprintFactory] = {
    "retail_small": retail_small,
    "banking_medium": banking_medium,
}


def list() -> List[str]:
    return sorted(_REGISTRY.keys())


def get(name: str) -> BlueprintFactory:
    return _REGISTRY[name]
