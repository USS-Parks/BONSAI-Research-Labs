"""Validate the M0 CI topology and evidence-class boundary."""

from __future__ import annotations

import subprocess
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
        "cargo clippy --workspace --all-targets --all-features",
        "cargo test --workspace --all-features",
        "cargo xtask schema-check",
        "uv run --frozen pytest",
        "check_m4.py",
        "cargo xtask bundle-check",
    )
    for marker in required_workflow_markers:
        if marker not in workflow:
            errors.append(f"workflow missing required marker {marker}")
    for marker in ("physical_acceptance=false", "energy_claim=false", "long_duration_claim=false", "not-run"):
        if marker not in matrix:
            errors.append(f"test matrix missing evidence boundary {marker}")
    return errors


def validate_capture_attributes() -> list[str]:
    sample = "evidence/verification/artifacts/BC-01-1784427540937208400.stdout.txt"
    result = subprocess.run(
        ["git", "check-attr", "text", "--", sample],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    if "text: unset" not in result.stdout:
        return [f"{sample} must remain -text so captured bytes stay intact"]
    return []


def self_test(workflow: str, matrix: str) -> list[str]:
    errors: list[str] = []
    broken_runner = workflow.replace("runner: windows-2025", "runner: windows-removed")
    if not any("windows-2025" in error for error in validate_text(broken_runner, matrix)):
        errors.append("missing-runner negative fixture was accepted")
    broken_gate = workflow.replace("cargo test --workspace --all-features", "cargo test --lib")
    if not any("cargo test --workspace --all-features" in error for error in validate_text(broken_gate, matrix)):
        errors.append("dropped workspace cargo test was accepted")
    return errors


def main() -> int:
    workflow = WORKFLOW.read_text(encoding="utf-8")
    matrix = MATRIX.read_text(encoding="utf-8")
    errors = validate_text(workflow, matrix) + self_test(workflow, matrix) + validate_capture_attributes()
    if errors:
        print("CI topology check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("CI topology check passed: Windows, macOS arm64/Intel, and Linux hosted jobs; physical claims remain false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
