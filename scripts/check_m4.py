"""M4 honesty gate: threat model, L not-run, operator docs, RC non-publication."""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    errors: list[str] = []
    threat = (ROOT / "docs" / "security" / "THREAT-MODEL.md").read_text(encoding="utf-8")
    if "not a hostile-native-code sandbox" not in threat.lower():
        errors.append("THREAT-MODEL.md missing prominent D-21 disclaimer")
    model = json.loads((ROOT / "fixtures" / "threat-model" / "v1" / "boundaries.json").read_text(encoding="utf-8"))
    ids = [row["id"] for row in model["boundaries"]]
    if ids != [f"TB-{n:02d}" for n in range(1, 11)]:
        errors.append(f"threat boundaries must be TB-01..TB-10, observed {ids}")

    for name in ("windows-not-run.json", "macos-not-run.json", "linux-not-run.json"):
        row = json.loads((ROOT / "fixtures" / "host-attestation" / "v1" / name).read_text(encoding="utf-8"))
        if row.get("status") != "not-run" or row.get("physical_acceptance") or row.get("long_duration_claim"):
            errors.append(f"{name} is not an honest L not-run attestation")

    readme = (ROOT / "README.md").read_text(encoding="utf-8").lower()
    if "instrument is complete" in readme or "instrument completion criteria pass" in readme:
        errors.append("README must keep instrument completion honest")
    if "no instrument-completion claim" not in readme and "no instrument completion claim" not in readme:
        errors.append("README must explicitly refuse an instrument-completion claim")

    notes = (ROOT / "docs" / "releases" / "RC-0.0.0-NOTES.md").read_text(encoding="utf-8")
    forbid = (ROOT / "evidence" / "release-candidate" / "OD-03-NO-UPLOAD.txt").read_text(encoding="utf-8")
    combined = notes + "\n" + forbid
    for marker in ("NO git tag", "NO crates.io", "NO pypi", "NO marketing claim"):
        if marker not in combined:
            errors.append(f"RC artifacts missing {marker}")

    waivers = json.loads((ROOT / "fixtures" / "supply-chain" / "v1" / "waivers.json").read_text(encoding="utf-8"))
    for waiver in waivers.get("accepted", []):
        if waiver.get("severity") in {"critical", "high"} and not waiver.get("accepted_by"):
            errors.append(f"silent {waiver.get('severity')} waiver {waiver.get('id')}")

    if errors:
        print("M4 honesty check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("M4 honesty check passed: D-21 prominent; L attestations not-run; OD-03 markers present")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
