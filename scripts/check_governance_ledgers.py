"""Validate BONSAI governance ledger schemas and seeded identifiers."""

from __future__ import annotations

import sys
from collections.abc import Iterable
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GOVERNANCE = ROOT / "docs" / "governance"


def parse_table(text: str, required: tuple[str, ...]) -> list[dict[str, str]]:
    lines = [line.strip() for line in text.splitlines() if line.strip().startswith("|")]
    for index, line in enumerate(lines):
        headers = tuple(cell.strip() for cell in line.strip("|").split("|"))
        if headers != required:
            continue
        if index + 1 >= len(lines):
            return []
        rows: list[dict[str, str]] = []
        for row in lines[index + 2 :]:
            cells = tuple(cell.strip() for cell in row.strip("|").split("|"))
            if len(cells) != len(headers):
                break
            rows.append(dict(zip(headers, cells, strict=True)))
        return rows
    return []


def missing_fields(rows: Iterable[dict[str, str]], fields: tuple[str, ...]) -> list[str]:
    errors: list[str] = []
    for row_number, row in enumerate(rows, start=1):
        identity = row.get("ID", f"row-{row_number}")
        for field in fields:
            if not row.get(field) or row[field] in {"—", "none"}:
                errors.append(f"{identity}: missing {field}")
    return errors


def validate() -> list[str]:
    errors: list[str] = []
    risk_headers = (
        "ID",
        "Type",
        "Risk or blocker",
        "Owner",
        "Trigger",
        "Impact",
        "Mitigation",
        "Status",
        "Affected prompts",
        "Review cadence",
    )
    parked_headers = (
        "ID",
        "Parked item",
        "Owner",
        "Rationale",
        "Status",
        "Revival criteria",
        "Required authorization",
        "Affected prompts",
        "Review cadence",
    )
    risk_rows = parse_table((GOVERNANCE / "RISK-AND-BLOCKER-REGISTER.md").read_text(encoding="utf-8"), risk_headers)
    parked_rows = parse_table((GOVERNANCE / "PARKED-SCOPE-LEDGER.md").read_text(encoding="utf-8"), parked_headers)
    if {row.get("ID") for row in risk_rows} != {f"R-{number:02d}" for number in range(1, 17)}:
        errors.append("risk register must contain exactly R-01 through R-16")
    if {row.get("ID") for row in parked_rows} != {f"P-{number:02d}" for number in range(1, 10)}:
        errors.append("parked ledger must contain exactly P-01 through P-09")
    errors.extend(missing_fields(risk_rows, ("Owner", "Status", "Affected prompts", "Review cadence")))
    errors.extend(
        missing_fields(
            parked_rows,
            ("Owner", "Status", "Revival criteria", "Required authorization", "Affected prompts", "Review cadence"),
        )
    )
    return errors


def self_test() -> list[str]:
    errors: list[str] = []
    broken_risk = [{"ID": "R-X", "Owner": "", "Status": "active"}]
    if missing_fields(broken_risk, ("Owner", "Status")) != ["R-X: missing Owner"]:
        errors.append("negative owner fixture was not rejected")
    broken_parked = [{"ID": "P-X", "Owner": "owner", "Status": "parked", "Revival criteria": ""}]
    if missing_fields(broken_parked, ("Owner", "Status", "Revival criteria")) != [
        "P-X: missing Revival criteria"
    ]:
        errors.append("negative revival fixture was not rejected")
    return errors


def main() -> int:
    errors = validate() + self_test()
    if errors:
        print("governance ledger check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("governance ledger check passed: R-01..R-16 and P-01..P-09 complete; negative fixtures rejected")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
