"""Emit CycloneDX-shaped SBOMs from committed lockfiles. No network."""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "evidence" / "release-candidate"
PACKAGE = re.compile(
    r"^\[\[package\]\]\nname = \"([^\"]+)\"\nversion = \"([^\"]+)\""
    r"(?:\nsource = (?:\"([^\"]+)\"|\{([^}\n]*)\}))?",
    re.MULTILINE,
)


def source_identity(quoted: str | None, table: str | None) -> str:
    if quoted:
        return quoted
    if table:
        registry = re.search(r'registry\s*=\s*"([^"]+)"', table)
        if registry:
            return registry.group(1)
        virtual = re.search(r'virtual\s*=\s*"([^"]+)"', table)
        if virtual:
            return f"virtual+{virtual.group(1)}"
        git = re.search(r'git\s*=\s*"([^"]+)"', table)
        if git:
            return git.group(1)
        path = re.search(r'path\s*=\s*"([^"]+)"', table)
        if path:
            return f"path+{path.group(1)}"
    return "path+workspace"


def packages_from_lock(text: str) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for match in PACKAGE.finditer(text):
        name, version, quoted, table = match.group(1), match.group(2), match.group(3), match.group(4)
        rows.append(
            {
                "type": "library",
                "name": name,
                "version": version,
                "purl": source_identity(quoted, table),
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
