"""Require the four hosted M1 bundles to agree semantically."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path
from typing import cast

type JsonValue = None | bool | int | float | str | list[JsonValue] | dict[str, JsonValue]

EXPECTED_PLATFORMS = {("windows", "x86_64"), ("linux", "x86_64"), ("macos", "arm64"), ("macos", "x86_64")}


def canonical(value: JsonValue) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode("ascii")


def load(path: Path) -> dict[str, JsonValue]:
    return cast(dict[str, JsonValue], json.loads(path.read_text(encoding="utf-8")))


def validate(root: Path) -> tuple[str, list[str]]:
    errors: list[str] = []
    summary_paths = sorted(root.rglob("semantic-summary.json"))
    if len(summary_paths) != 4:
        return "", [f"expected 4 semantic summaries, found {len(summary_paths)}"]
    summaries = [load(path) for path in summary_paths]
    semantic_hashes = {hashlib.sha256(canonical(summary)).hexdigest() for summary in summaries}
    if len(semantic_hashes) != 1:
        errors.append(f"semantic summaries disagree: {sorted(semantic_hashes)}")
    platform_rows: set[tuple[str, str]] = set()
    for path in summary_paths:
        bundle = path.parent
        for required in ("manifest.json", "report.json", "report.html", "platform-summary.json"):
            if not (bundle / required).is_file():
                errors.append(f"{bundle.name} missing {required}")
        platform = load(bundle / "platform-summary.json")
        family = platform.get("os_family")
        architecture = platform.get("architecture")
        if isinstance(family, str) and isinstance(architecture, str):
            platform_rows.add((family, architecture))
        else:
            errors.append(f"{bundle.name} has invalid platform row")
    if platform_rows != EXPECTED_PLATFORMS:
        errors.append(f"platform rows differ: {sorted(platform_rows)}")
    return next(iter(semantic_hashes), ""), errors


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check_m1_equivalence.py <DOWNLOADED_ARTIFACT_ROOT>", file=sys.stderr)
        return 2
    semantic_hash, errors = validate(Path(sys.argv[1]))
    if errors:
        print("M1 equivalence check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print(f"M1 equivalence passed: 4 hosted bundles semantic_sha256={semantic_hash}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
