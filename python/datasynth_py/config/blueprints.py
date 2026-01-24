"""Blueprint registry for common DataSynth configurations."""

from __future__ import annotations

from typing import Callable, Dict, List, Optional

from datasynth_py.config.models import (
    AuditSettings,
    BankingSettings,
    ChartOfAccountsSettings,
    CompanyConfig,
    Config,
    DataQualitySettings,
    FraudSettings,
    GlobalSettings,
    GraphExportSettings,
    ScenarioSettings,
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


def banking_aml(customers: int = 1000, typologies: bool = True) -> Config:
    """Create a banking KYC/AML focused configuration.

    Enables banking transaction generation with AML typology injection
    for training fraud detection and compliance models.

    Args:
        customers: Number of banking customers to generate.
        typologies: Whether to inject AML typologies (structuring, layering, etc.).
    """
    return Config(
        global_settings=GlobalSettings(
            industry="financial_services",
            start_date="2024-01-01",
            period_months=12,
        ),
        companies=[
            CompanyConfig(
                code="BANK001",
                name="DataSynth Bank",
                currency="USD",
                country="US",
                annual_transaction_volume="hundred_k",
            ),
        ],
        chart_of_accounts=ChartOfAccountsSettings(complexity="medium"),
        banking=BankingSettings(
            enabled=True,
            retail_customers=int(customers * 0.7),
            business_customers=int(customers * 0.25),
            trusts=int(customers * 0.05),
            typologies_enabled=typologies,
        ),
        scenario=ScenarioSettings(
            tags=["banking", "aml", "compliance"],
            ml_training=True,
        ),
    )


def ml_training(
    industry: str = "manufacturing",
    anomaly_ratio: float = 0.05,
    with_data_quality: bool = True,
) -> Config:
    """Create a configuration optimized for ML training datasets.

    Generates balanced datasets with labeled anomalies and optional
    data quality issues for robust model training.

    Args:
        industry: Industry sector for the data.
        anomaly_ratio: Target ratio of anomalous transactions (0.0-1.0).
        with_data_quality: Whether to inject data quality variations.
    """
    return Config(
        global_settings=GlobalSettings(
            industry=industry,
            start_date="2024-01-01",
            period_months=12,
        ),
        companies=[
            CompanyConfig(
                code="ML001",
                name="ML Training Corp",
                currency="USD",
                country="US",
                annual_transaction_volume="hundred_k",
            ),
        ],
        chart_of_accounts=ChartOfAccountsSettings(complexity="medium"),
        fraud=FraudSettings(
            enabled=True,
            rate=anomaly_ratio,
        ),
        data_quality=DataQualitySettings(
            enabled=with_data_quality,
            missing_rate=0.03,
            typo_rate=0.02,
        ) if with_data_quality else None,
        scenario=ScenarioSettings(
            tags=["ml_training", "labeled_data"],
            ml_training=True,
            target_anomaly_ratio=anomaly_ratio,
        ),
        graph_export=GraphExportSettings(
            enabled=True,
            formats=["pytorch_geometric"],
        ),
    )


def with_graph_export(base_config: Config, formats: Optional[List[str]] = None) -> Config:
    """Add graph export to an existing configuration.

    Args:
        base_config: Base configuration to extend.
        formats: Export formats (pytorch_geometric, neo4j, dgl). Defaults to pytorch_geometric.

    Returns:
        New Config with graph export enabled.
    """
    if formats is None:
        formats = ["pytorch_geometric"]

    graph_settings = GraphExportSettings(
        enabled=True,
        formats=formats,
    )

    # Use override to merge in graph export
    return base_config.override(graph_export=graph_settings.__dict__)


def audit_engagement(
    engagements: int = 5,
    with_evidence: bool = True,
) -> Config:
    """Create a configuration for audit data generation.

    Generates audit engagements, workpapers, evidence, and findings
    following ISA standards.

    Args:
        engagements: Number of audit engagements to generate.
        with_evidence: Whether to generate evidence items.
    """
    return Config(
        global_settings=GlobalSettings(
            industry="financial_services",
            start_date="2024-01-01",
            period_months=12,
        ),
        companies=[
            CompanyConfig(
                code="AUDITEE001",
                name="Auditee Corporation",
                currency="USD",
                country="US",
                annual_transaction_volume="hundred_k",
            ),
        ],
        chart_of_accounts=ChartOfAccountsSettings(complexity="medium"),
        audit=AuditSettings(
            enabled=True,
            engagements=engagements,
            workpapers_per_engagement=20,
            evidence_per_workpaper=5 if with_evidence else 0,
            risks_per_engagement=15,
            findings_per_engagement=8,
        ),
        scenario=ScenarioSettings(
            tags=["audit", "isa"],
        ),
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
    "banking_aml": banking_aml,
    "ml_training": ml_training,
    "audit_engagement": audit_engagement,
}


def list() -> List[str]:
    """List available blueprint names."""
    return sorted(_REGISTRY.keys())


def get(name: str) -> BlueprintFactory:
    """Get a blueprint factory by name."""
    return _REGISTRY[name]
