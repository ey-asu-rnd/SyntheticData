"""Blueprint registry for common DataSynth configurations."""

from __future__ import annotations

from typing import Callable, Dict, List

from datasynth_py.config.models import (
    ChartOfAccountsSettings,
    CompanyConfig,
    Config,
    FraudSettings,
    GlobalSettings,
)

BlueprintFactory = Callable[..., Config]


def retail_small(companies: int = 3, transactions: int = 5000) -> Config:
    """Create a small retail configuration.

    Args:
        companies: Number of companies to generate.
        transactions: Transaction volume hint (maps to volume preset).
    """
    volume = _transactions_to_volume(transactions)
    return Config(
        global_settings=GlobalSettings(
            industry="retail",
            start_date="2024-01-01",
            period_months=12,
        ),
        companies=[
            CompanyConfig(
                code=f"R{i + 1:03d}",
                name=f"Retail Company {i + 1}",
                currency="USD",
                country="US",
                annual_transaction_volume=volume,
            )
            for i in range(companies)
        ],
        chart_of_accounts=ChartOfAccountsSettings(complexity="small"),
    )


def banking_medium(companies: int = 5, transactions: int = 20000) -> Config:
    """Create a medium financial services configuration.

    Args:
        companies: Number of companies to generate.
        transactions: Transaction volume hint (maps to volume preset).
    """
    volume = _transactions_to_volume(transactions)
    return Config(
        global_settings=GlobalSettings(
            industry="financial_services",
            start_date="2024-01-01",
            period_months=12,
        ),
        companies=[
            CompanyConfig(
                code=f"F{i + 1:03d}",
                name=f"Financial Services Company {i + 1}",
                currency="USD",
                country="US",
                annual_transaction_volume=volume,
            )
            for i in range(companies)
        ],
        chart_of_accounts=ChartOfAccountsSettings(complexity="medium"),
        fraud=FraudSettings(enabled=True, rate=0.01),
    )


def manufacturing_large(companies: int = 10, transactions: int = 100000) -> Config:
    """Create a large manufacturing configuration.

    Args:
        companies: Number of companies to generate.
        transactions: Transaction volume hint (maps to volume preset).
    """
    volume = _transactions_to_volume(transactions)
    return Config(
        global_settings=GlobalSettings(
            industry="manufacturing",
            start_date="2024-01-01",
            period_months=12,
        ),
        companies=[
            CompanyConfig(
                code=f"M{i + 1:03d}",
                name=f"Manufacturing Company {i + 1}",
                currency="USD",
                country="US",
                annual_transaction_volume=volume,
            )
            for i in range(companies)
        ],
        chart_of_accounts=ChartOfAccountsSettings(complexity="large"),
    )


def _transactions_to_volume(count: int) -> str:
    """Map transaction count to volume preset."""
    if count <= 10_000:
        return "ten_k"
    elif count <= 100_000:
        return "hundred_k"
    elif count <= 1_000_000:
        return "one_m"
    elif count <= 10_000_000:
        return "ten_m"
    else:
        return "hundred_m"


_REGISTRY: Dict[str, BlueprintFactory] = {
    "retail_small": retail_small,
    "banking_medium": banking_medium,
    "manufacturing_large": manufacturing_large,
}


def list() -> List[str]:
    """List available blueprint names."""
    return sorted(_REGISTRY.keys())


def get(name: str) -> BlueprintFactory:
    """Get a blueprint factory by name."""
    return _REGISTRY[name]
