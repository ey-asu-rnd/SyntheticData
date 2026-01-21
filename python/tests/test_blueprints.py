import unittest

from datasynth_py.config import blueprints


class BlueprintTests(unittest.TestCase):
    def test_blueprint_registry(self) -> None:
        available = blueprints.list()
        self.assertIn("retail_small", available)
        self.assertIn("banking_medium", available)

    def test_retail_small_blueprint(self) -> None:
        config = blueprints.retail_small(companies=4, transactions=12000)
        payload = config.to_dict()
        self.assertEqual(payload["companies"]["count"], 4)
        self.assertEqual(payload["transactions"]["count"], 12000)


if __name__ == "__main__":
    unittest.main()
