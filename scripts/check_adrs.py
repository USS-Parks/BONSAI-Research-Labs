"""Check that every settled PSPR decision maps to one accepted ADR."""

from __future__ import annotations

import re
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ADR_DIR = ROOT / "docs" / "architecture" / "adr"
EXPECTED = {f"D-{number:02d}" for number in range(1, 22)}
INDEX_ROW = re.compile(r"^\| \[\d{4}]\([^)]+\.md\) \| accepted \| ([^|]+) \|", re.MULTILINE)
DECISION_HEADING = re.compile(r"^## (D-\d{2}) — ", re.MULTILINE)


def main() -> int:
    errors: list[str] = []
    index = (ADR_DIR / "README.md").read_text(encoding="utf-8")
    indexed = [decision.strip() for cell in INDEX_ROW.findall(index) for decision in cell.split(",")]
    counts = Counter(indexed)

    if set(indexed) != EXPECTED:
        errors.append(f"index coverage mismatch: missing={sorted(EXPECTED - set(indexed))}, extra={sorted(set(indexed) - EXPECTED)}")
    duplicates = sorted(decision for decision, count in counts.items() if count != 1)
    if duplicates:
        errors.append(f"index decisions not mapped exactly once: {duplicates}")

    body_locations: dict[str, list[str]] = {}
    for adr in sorted(ADR_DIR.glob("[0-9][0-9][0-9][0-9]-*.md")):
        text = adr.read_text(encoding="utf-8")
        if "- Status: accepted" not in text or "## Supersession" not in text:
            errors.append(f"{adr.name}: missing accepted status or supersession rule")
        for decision in DECISION_HEADING.findall(text):
            body_locations.setdefault(decision, []).append(adr.name)

    if set(body_locations) != EXPECTED:
        errors.append(f"ADR body coverage mismatch: missing={sorted(EXPECTED - set(body_locations))}, extra={sorted(set(body_locations) - EXPECTED)}")
    duplicates = sorted(decision for decision, locations in body_locations.items() if len(locations) != 1)
    if duplicates:
        errors.append(f"ADR body decisions not defined exactly once: {duplicates}")

    if errors:
        print("ADR check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print(f"ADR check passed: {len(EXPECTED)} accepted decisions mapped exactly once across {len(body_locations)} headings")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
