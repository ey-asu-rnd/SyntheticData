import unittest

from datasynth_py.config import Config, GlobalSettings, CompanySettings, TransactionSettings
from datasynth_py.config.validation import ConfigValidationError


class ConfigTests(unittest.TestCase):
    def test_override_merges_nested_sections(self) -> None:
        base = Config(
            global_settings=GlobalSettings(locale="en_US"),
            companies=CompanySettings(count=2, industry="retail"),
            extra={"fraud": {"enabled": False}},
        )

        updated = base.override(
            companies={"count": 5},
            fraud={"enabled": True, "rate": 0.05},
        )

        payload = updated.to_dict()
        self.assertEqual(payload["companies"]["count"], 5)
        self.assertEqual(payload["fraud"]["enabled"], True)
        self.assertEqual(payload["fraud"]["rate"], 0.05)

    def test_validate_reports_schema_errors(self) -> None:
        config = Config(
            global_settings=GlobalSettings(periods=0),
            companies=CompanySettings(count=0),
            transactions=TransactionSettings(anomaly_rate=2.0),
        )
        with self.assertRaises(ConfigValidationError):
            config.validate()


if __name__ == "__main__":
    unittest.main()
