import unittest

from datasynth_py.config import (
    ChartOfAccountsSettings,
    CompanyConfig,
    Config,
    FraudSettings,
    GlobalSettings,
)
from datasynth_py.config.validation import ConfigValidationError


class ConfigTests(unittest.TestCase):
    def test_override_merges_nested_sections(self) -> None:
        base = Config(
            global_settings=GlobalSettings(industry="retail", start_date="2024-01-01"),
            companies=[
                CompanyConfig(code="C001", name="Test Company"),
            ],
            extra={"fraud": {"enabled": False}},
        )

        updated = base.override(
            fraud={"enabled": True, "rate": 0.05},
        )

        payload = updated.to_dict()
        self.assertEqual(payload["fraud"]["enabled"], True)
        self.assertEqual(payload["fraud"]["rate"], 0.05)

    def test_validate_reports_schema_errors(self) -> None:
        config = Config(
            global_settings=GlobalSettings(period_months=0, industry="invalid"),
            companies=[],  # Empty companies list
            fraud=FraudSettings(rate=2.0),
        )
        with self.assertRaises(ConfigValidationError):
            config.validate()

    def test_to_dict_matches_cli_schema(self) -> None:
        config = Config(
            global_settings=GlobalSettings(
                industry="manufacturing",
                start_date="2024-01-01",
                period_months=12,
                seed=42,
            ),
            companies=[
                CompanyConfig(
                    code="M001",
                    name="Manufacturing Co",
                    currency="USD",
                    country="US",
                    annual_transaction_volume="ten_k",
                ),
            ],
            chart_of_accounts=ChartOfAccountsSettings(complexity="small"),
        )

        payload = config.to_dict()

        # Check global section has correct fields
        self.assertEqual(payload["global"]["industry"], "manufacturing")
        self.assertEqual(payload["global"]["start_date"], "2024-01-01")
        self.assertEqual(payload["global"]["period_months"], 12)
        self.assertEqual(payload["global"]["seed"], 42)

        # Check companies is a list of objects
        self.assertIsInstance(payload["companies"], list)
        self.assertEqual(len(payload["companies"]), 1)
        self.assertEqual(payload["companies"][0]["code"], "M001")
        self.assertEqual(payload["companies"][0]["name"], "Manufacturing Co")

        # Check chart_of_accounts
        self.assertEqual(payload["chart_of_accounts"]["complexity"], "small")

    def test_valid_config_passes_validation(self) -> None:
        config = Config(
            global_settings=GlobalSettings(
                industry="retail",
                start_date="2024-01-01",
                period_months=12,
            ),
            companies=[
                CompanyConfig(code="C001", name="Test Company"),
            ],
            chart_of_accounts=ChartOfAccountsSettings(complexity="small"),
        )
        # Should not raise
        config.validate()


if __name__ == "__main__":
    unittest.main()
