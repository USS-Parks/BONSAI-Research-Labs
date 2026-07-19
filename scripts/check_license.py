"""Validate BONSAI's approved dual-license and publication boundary."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    errors: list[str] = []
    apache = (ROOT / "LICENSE-APACHE").read_text(encoding="utf-8")
    mit = (ROOT / "LICENSE-MIT").read_text(encoding="utf-8")
    publication = (ROOT / "docs" / "governance" / "PUBLICATION-POLICY.md").read_text(encoding="utf-8")
    contributing = (ROOT / "CONTRIBUTING.md").read_text(encoding="utf-8")

    apache_markers = (
        "Apache License",
        "Version 2.0, January 2004",
        "TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION",
        "END OF TERMS AND CONDITIONS",
    )
    for section in range(1, 10):
        apache_markers += (f"   {section}.",)
    if not all(marker in apache for marker in apache_markers):
        errors.append("LICENSE-APACHE is not a complete Apache-2.0 text")

    mit_markers = (
        "MIT License",
        "Permission is hereby granted, free of charge",
        'THE SOFTWARE IS PROVIDED "AS IS"',
        "Copyright (c) 2026 USS-Parks",
    )
    if not all(marker in mit for marker in mit_markers):
        errors.append("LICENSE-MIT is not the approved MIT text")

    if "MIT OR Apache-2.0" not in contributing:
        errors.append("CONTRIBUTING.md lacks the approved SPDX expression")

    required_policy = (
        "explicit user authorization",
        "secret and privacy scan",
        "Redaction review",
        "public source-repository scope",
        "BONSAI-Research-Labs",
    )
    if not all(marker in publication for marker in required_policy):
        errors.append("publication policy lacks authorization or secret/redaction controls")

    expected_license_files = {"LICENSE-APACHE", "LICENSE-MIT"}
    unexpected = sorted(path.name for path in ROOT.glob("LICENSE*") if path.name not in expected_license_files)
    if unexpected:
        errors.append(f"unexpected license files: {unexpected}")

    if errors:
        print("license policy check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print(
        "license policy check passed: MIT OR Apache-2.0 only; "
        "publication authorization and secret/redaction review required"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
