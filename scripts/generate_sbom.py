"""Emit CycloneDX-shaped SBOMs from committed lockfiles. No network."""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "evidence" / "release-candidate"
PACKAGE = re.compile(
    r"^\[\[package\]\]\nname = \"([^\"]+)\"\nversion = \"([^\"]+)\"(?:\nsource = \"([^\"]+)\")?",
    re.MULTILINE,
)


def packages_from_lock(text: str) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for match in PACKAGE.finditer(text):
        name, version, source = match.group(1), match.group(2), match.group(3)
        rows.append(
            {
                "type": "library",
                "name": name,
                "version": version,
                "purl": source or "path+workspace",
            }
        )
    return rows


def cyclonedx(name: str, components: list[dict[str, str]]) -> dict[str, object]:
    return {
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "version": 1,
        "metadata": {
            "component": {"type": "application", "name": name, "version": "0.0.0"},
            "licenses": [{"license": {"id": "MIT"}}, {"license": {"id": "Apache-2.0"}}],
        },
        "components": components,
    }


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    cargo = packages_from_lock((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
    python = packages_from_lock((ROOT / "uv.lock").read_text(encoding="utf-8"))
    if not cargo or not python:
        print("generate_sbom: lockfile produced no components", flush=True)
        return 1
    cargo_bom = json.dumps(cyclonedx("bonsai-workspace", cargo), indent=2) + "\n"
    python_bom = json.dumps(cyclonedx("bonsai-reference", python), indent=2) + "\n"
    (OUT / "sbom-cargo.json").write_text(cargo_bom, encoding="utf-8")
    (OUT / "sbom-python.json").write_text(python_bom, encoding="utf-8")
    print(f"generate_sbom: cargo={len(cargo)} python={len(python)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
