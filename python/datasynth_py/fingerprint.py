"""Fingerprint operations for DataSynth.

This module provides a Python interface to fingerprint extraction, validation,
and fidelity evaluation using the datasynth-data CLI.
"""

from __future__ import annotations

import json
import os
import subprocess
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Tuple


@dataclass(frozen=True)
class FingerprintInfo:
    """Information about a fingerprint file."""

    version: str
    format: str
    privacy_level: str
    epsilon: float
    k_anonymity: int
    tables: Dict[str, int] = field(default_factory=dict)
    epsilon_spent: float = 0.0
    created_at: Optional[str] = None
    description: Optional[str] = None


@dataclass(frozen=True)
class FidelityReport:
    """Fidelity evaluation results comparing synthetic data to a fingerprint."""

    overall_score: float
    statistical_fidelity: float
    correlation_fidelity: float
    schema_fidelity: float
    rule_compliance: float
    anomaly_fidelity: float
    passes: bool
    threshold: float = 0.8


class FingerprintClient:
    """Client for fingerprint operations.

    Wraps the datasynth-data fingerprint CLI commands.

    Example:
        >>> from datasynth_py import DataSynth
        >>> synth = DataSynth()
        >>> synth.fingerprint.extract("./data/", "./fp.dsf", privacy_level="standard")
        >>> info = synth.fingerprint.info("./fp.dsf")
        >>> print(f"Tables: {info.tables}")
    """

    def __init__(self, binary_path: str = "datasynth-data") -> None:
        """Initialize the fingerprint client.

        Args:
            binary_path: Path to the datasynth-data binary.
        """
        self._binary_path = binary_path

    def extract(
        self,
        input_path: str,
        output_path: str,
        privacy_level: str = "standard",
        epsilon: Optional[float] = None,
        k_anonymity: Optional[int] = None,
        sign: bool = False,
    ) -> None:
        """Extract a fingerprint from data.

        Args:
            input_path: Path to input data (CSV file or directory).
            output_path: Path for output .dsf fingerprint file.
            privacy_level: Privacy level (minimal, standard, high, maximum).
            epsilon: Optional custom epsilon for differential privacy.
            k_anonymity: Optional custom k-anonymity threshold.
            sign: Whether to sign the fingerprint.

        Raises:
            RuntimeError: If extraction fails.
        """
        command = [
            self._binary_path,
            "fingerprint",
            "extract",
            "--input",
            input_path,
            "--output",
            output_path,
            "--privacy-level",
            privacy_level,
        ]

        if epsilon is not None:
            command.extend(["--privacy-epsilon", str(epsilon)])
        if k_anonymity is not None:
            command.extend(["--privacy-k", str(k_anonymity)])
        if sign:
            command.append("--sign")

        self._run_command(command)

    def validate(self, path: str) -> Tuple[bool, List[str]]:
        """Validate a fingerprint file.

        Args:
            path: Path to the .dsf fingerprint file.

        Returns:
            Tuple of (is_valid, list of warnings/errors).
        """
        command = [
            self._binary_path,
            "fingerprint",
            "validate",
            path,
        ]

        try:
            result = subprocess.run(
                command,
                check=True,
                capture_output=True,
                text=True,
            )
            # Parse output for warnings
            warnings = []
            for line in result.stdout.split("\n"):
                if "Warning" in line or "warning" in line:
                    warnings.append(line.strip())
            return True, warnings
        except subprocess.CalledProcessError as exc:
            # Validation failed
            errors = []
            output = exc.stdout or exc.stderr or ""
            for line in output.split("\n"):
                if line.strip() and not line.startswith("Fingerprint"):
                    errors.append(line.strip())
            return False, errors

    def info(self, path: str, detailed: bool = False) -> FingerprintInfo:
        """Get information about a fingerprint file.

        Args:
            path: Path to the .dsf fingerprint file.
            detailed: Whether to include detailed statistics.

        Returns:
            FingerprintInfo with fingerprint metadata.
        """
        command = [
            self._binary_path,
            "fingerprint",
            "info",
            path,
        ]
        if detailed:
            command.append("--detailed")

        result = subprocess.run(
            command,
            check=True,
            capture_output=True,
            text=True,
        )

        return self._parse_info_output(result.stdout)

    def evaluate(
        self,
        fingerprint_path: str,
        synthetic_path: str,
        threshold: float = 0.8,
        output_path: Optional[str] = None,
    ) -> FidelityReport:
        """Evaluate fidelity of synthetic data against a fingerprint.

        Args:
            fingerprint_path: Path to the original .dsf fingerprint.
            synthetic_path: Path to synthetic data directory.
            threshold: Fidelity threshold (0.0-1.0) for pass/fail.
            output_path: Optional path to write JSON report.

        Returns:
            FidelityReport with evaluation results.

        Raises:
            RuntimeError: If evaluation fails.
        """
        command = [
            self._binary_path,
            "fingerprint",
            "evaluate",
            "--fingerprint",
            fingerprint_path,
            "--synthetic",
            synthetic_path,
            "--threshold",
            str(threshold),
        ]
        if output_path:
            command.extend(["--output", output_path])

        result = subprocess.run(
            command,
            capture_output=True,
            text=True,
        )

        # Parse the output to extract fidelity scores
        return self._parse_evaluate_output(result.stdout, threshold, result.returncode == 0)

    def diff(self, file1: str, file2: str) -> Dict[str, Any]:
        """Compare two fingerprint files.

        Args:
            file1: Path to first .dsf fingerprint.
            file2: Path to second .dsf fingerprint.

        Returns:
            Dictionary with comparison results.
        """
        command = [
            self._binary_path,
            "fingerprint",
            "diff",
            file1,
            file2,
        ]

        result = subprocess.run(
            command,
            check=True,
            capture_output=True,
            text=True,
        )

        return {"output": result.stdout, "differences": self._parse_diff_output(result.stdout)}

    def _run_command(self, command: List[str]) -> str:
        """Run a CLI command and return output."""
        try:
            result = subprocess.run(
                command,
                check=True,
                capture_output=True,
                text=True,
            )
            return result.stdout
        except FileNotFoundError as exc:
            raise RuntimeError(
                f"datasynth-data binary not found at '{self._binary_path}'. "
                "Build it with `cargo build --release` or set the correct path."
            ) from exc
        except subprocess.CalledProcessError as exc:
            raise RuntimeError(
                f"Command failed: {exc.stderr or exc.stdout}"
            ) from exc

    def _parse_info_output(self, output: str) -> FingerprintInfo:
        """Parse info command output into FingerprintInfo."""
        lines = output.strip().split("\n")
        info_dict: Dict[str, Any] = {
            "version": "1.0",
            "format": "dsf",
            "privacy_level": "standard",
            "epsilon": 1.0,
            "k_anonymity": 5,
            "tables": {},
            "epsilon_spent": 0.0,
        }

        current_section = None
        for line in lines:
            line = line.strip()
            if not line or line.startswith("="):
                continue

            # Section headers
            if line.endswith(":") and not line.startswith(" "):
                current_section = line[:-1].lower().replace(" ", "_")
                continue

            # Key-value pairs
            if ":" in line:
                key, _, value = line.partition(":")
                key = key.strip().lower().replace(" ", "_")
                value = value.strip()

                if current_section == "manifest":
                    if key == "version":
                        info_dict["version"] = value
                    elif key == "format":
                        info_dict["format"] = value
                    elif key == "created":
                        info_dict["created_at"] = value

                elif current_section == "source":
                    if key == "description":
                        info_dict["description"] = value
                    elif key == "tables":
                        try:
                            info_dict["table_count"] = int(value)
                        except ValueError:
                            pass

                elif current_section == "privacy":
                    if key == "level":
                        info_dict["privacy_level"] = value
                    elif key == "epsilon":
                        try:
                            info_dict["epsilon"] = float(value)
                        except ValueError:
                            pass
                    elif key == "k-anonymity" or key == "k_anonymity":
                        try:
                            info_dict["k_anonymity"] = int(value)
                        except ValueError:
                            pass

                elif current_section == "schema":
                    if key == "tables":
                        try:
                            info_dict["table_count"] = int(value)
                        except ValueError:
                            pass

                elif current_section == "privacy_audit":
                    if key == "epsilon_spent":
                        try:
                            info_dict["epsilon_spent"] = float(value)
                        except ValueError:
                            pass

            # Table entries (indented with -)
            if line.startswith("-"):
                table_info = line[1:].strip()
                if "(" in table_info:
                    name = table_info.split("(")[0].strip()
                    try:
                        cols = int(table_info.split("(")[1].split()[0])
                        info_dict["tables"][name] = cols
                    except (ValueError, IndexError):
                        info_dict["tables"][name] = 0

        return FingerprintInfo(
            version=info_dict.get("version", "1.0"),
            format=info_dict.get("format", "dsf"),
            privacy_level=info_dict.get("privacy_level", "standard"),
            epsilon=info_dict.get("epsilon", 1.0),
            k_anonymity=info_dict.get("k_anonymity", 5),
            tables=info_dict.get("tables", {}),
            epsilon_spent=info_dict.get("epsilon_spent", 0.0),
            created_at=info_dict.get("created_at"),
            description=info_dict.get("description"),
        )

    def _parse_evaluate_output(
        self, output: str, threshold: float, success: bool
    ) -> FidelityReport:
        """Parse evaluate command output into FidelityReport."""
        scores = {
            "overall_score": 0.0,
            "statistical_fidelity": 0.0,
            "correlation_fidelity": 0.0,
            "schema_fidelity": 0.0,
            "rule_compliance": 0.0,
            "anomaly_fidelity": 0.0,
        }

        for line in output.split("\n"):
            line = line.strip()
            if "Overall Score:" in line:
                try:
                    scores["overall_score"] = float(line.split(":")[1].strip().rstrip("%")) / 100
                except (ValueError, IndexError):
                    pass
            elif "Statistical Fidelity:" in line:
                try:
                    scores["statistical_fidelity"] = float(line.split(":")[1].strip().rstrip("%")) / 100
                except (ValueError, IndexError):
                    pass
            elif "Correlation Fidelity:" in line:
                try:
                    scores["correlation_fidelity"] = float(line.split(":")[1].strip().rstrip("%")) / 100
                except (ValueError, IndexError):
                    pass
            elif "Schema Fidelity:" in line:
                try:
                    scores["schema_fidelity"] = float(line.split(":")[1].strip().rstrip("%")) / 100
                except (ValueError, IndexError):
                    pass
            elif "Rule Compliance:" in line:
                try:
                    scores["rule_compliance"] = float(line.split(":")[1].strip().rstrip("%")) / 100
                except (ValueError, IndexError):
                    pass
            elif "Anomaly Fidelity:" in line:
                try:
                    scores["anomaly_fidelity"] = float(line.split(":")[1].strip().rstrip("%")) / 100
                except (ValueError, IndexError):
                    pass

        return FidelityReport(
            overall_score=scores["overall_score"],
            statistical_fidelity=scores["statistical_fidelity"],
            correlation_fidelity=scores["correlation_fidelity"],
            schema_fidelity=scores["schema_fidelity"],
            rule_compliance=scores["rule_compliance"],
            anomaly_fidelity=scores["anomaly_fidelity"],
            passes=success and scores["overall_score"] >= threshold,
            threshold=threshold,
        )

    def _parse_diff_output(self, output: str) -> Dict[str, Any]:
        """Parse diff command output into structured differences."""
        differences: Dict[str, Any] = {
            "manifest": {},
            "schema": {"only_in_first": [], "only_in_second": [], "common": 0},
            "statistics": {"numeric_diff": 0, "categorical_diff": 0},
        }

        current_section = None
        for line in output.split("\n"):
            line = line.strip()
            if line.startswith("Manifests:"):
                current_section = "manifest"
            elif line.startswith("Schema:"):
                current_section = "schema"
            elif line.startswith("Statistics:"):
                current_section = "statistics"

            if current_section == "manifest" and "vs" in line:
                key = line.split(":")[0].strip()
                differences["manifest"][key] = line

            if current_section == "schema":
                if "Only in" in line and "file1" in line.lower():
                    differences["schema"]["only_in_first"].append(line)
                elif "Only in" in line and "file2" in line.lower():
                    differences["schema"]["only_in_second"].append(line)
                elif "Common tables:" in line:
                    try:
                        differences["schema"]["common"] = int(line.split(":")[1].strip())
                    except (ValueError, IndexError):
                        pass

        return differences
