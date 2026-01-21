import unittest

from datasynth_py.config import blueprints


class BlueprintTests(unittest.TestCase):
    def test_blueprint_registry(self) -> None:
        available = blueprints.list()
        self.assertIn("retail_small", available)
        self.assertIn("banking_medium", available)
        self.assertIn("manufacturing_large", available)

    def test_retail_small_blueprint(self) -> None:
        config = blueprints.retail_small(companies=4, transactions=12000)
        payload = config.to_dict()

        # Check global settings
        self.assertEqual(payload["global"]["industry"], "retail")
        self.assertEqual(payload["global"]["start_date"], "2024-01-01")
        self.assertEqual(payload["global"]["period_months"], 12)

        # Check companies is a list with correct count
        self.assertEqual(len(payload["companies"]), 4)
        self.assertEqual(payload["companies"][0]["code"], "R001")

        # Check chart_of_accounts
        self.assertEqual(payload["chart_of_accounts"]["complexity"], "small")

    def test_banking_medium_blueprint(self) -> None:
        config = blueprints.banking_medium(companies=3)
        payload = config.to_dict()

        self.assertEqual(payload["global"]["industry"], "financial_services")
        self.assertEqual(len(payload["companies"]), 3)
        self.assertEqual(payload["chart_of_accounts"]["complexity"], "medium")
        self.assertEqual(payload["fraud"]["enabled"], True)
        self.assertEqual(payload["fraud"]["rate"], 0.01)

    def test_blueprints_validate(self) -> None:
        # All blueprints should produce valid configs
        for name in blueprints.list():
            factory = blueprints.get(name)
            config = factory()
            config.validate()  # Should not raise


if __name__ == "__main__":
    unittest.main()
