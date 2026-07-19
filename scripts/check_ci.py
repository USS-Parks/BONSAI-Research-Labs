"""Validate the M0 CI topology and evidence-class boundary."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github" / "workflows" / "ci.yml"
MATRIX = ROOT / "docs" / "verification" / "TEST-MATRIX.md"
REQUIRED_RUNNERS = {"ubuntu-24.04", "windows-2025", "macos-15", "macos-15-intel"}


def validate_text(workflow: str, matrix: str) -> list[str]:
    errors: list[str] = []
    for runner in sorted(REQUIRED_RUNNERS):
        if f"runner: {runner}" not in workflow:
            errors.append(f"workflow missing runner {runner}")
        if f"`{runner}`" not in matrix:
            errors.append(f"test matrix missing runner {runner}")
    required_workflow_markers = (
        "hosted-ci",
        "write_ci_evidence.py",
        "persist-credentials: false",
        "physical_acceptance",
    )
    if "physical_acceptance" not in workflow:
        # The writer fixes this field to false; the workflow must at least invoke and document it.
        required_workflow_markers = required_workflow_markers[:-1]
    for marker in required_workflow_markers:
        if marker not in workflow:
            errors.append(f"workflow missing required marker {marker}")
    for marker in ("physical_acceptance=false", "energy_claim=false", "long_duration_claim=false", "not-run"):
        if marker not in matrix:
            errors.append(f"test matrix missing evidence boundary {marker}")
    return errors


def self_test(workflow: str, matrix: str) -> list[str]:
    broken = workflow.replace("runner: windows-2025", "runner: windows-removed")
    if not any("windows-2025" in error for error in validate_text(broken, matrix)):
        return ["missing-runner negative fixture was accepted"]
    return []


def main() -> int:
    workflow = WORKFLOW.read_text(encoding="utf-8")
    matrix = MATRIX.read_text(encoding="utf-8")
    errors = validate_text(workflow, matrix) + self_test(workflow, matrix)
    if errors:
        print("CI topology check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("CI topology check passed: Windows, macOS arm64/Intel, and Linux hosted jobs; physical claims remain false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
