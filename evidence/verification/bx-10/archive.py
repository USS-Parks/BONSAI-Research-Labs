"""Archive the final paired bundles, common certifications and exact source bytes."""
from __future__ import annotations

import hashlib
import json
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / "evidence/verification/bx-10"


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def main():
    batch = ROOT / sys.argv[1]
    analysis = ROOT / sys.argv[2]
    summary = json.loads((batch / "summary.json").read_text())
    replay = json.loads((analysis / "replay.json").read_text())
    assert not summary["smoke"] and len(summary["runs"]) == len(replay["runs"]) == 20
    payloads = {}
    identities = []
    certifications = []
    for row in summary["runs"]:
        case = ROOT / row["case"]
        assert row["runner_exit"] == row["verifier_exit"] == 0
        identities.append(json.loads((case / "run/observer/source-identity.json").read_text()))
        report = json.loads((analysis / case.name / "certification.json").read_text())
        assert report["verdict"] == "certified" and len(report["checks"]) == 7
        assert all(check["verdict"] == "pass" for check in report["checks"])
        assert not report["scientific_quality_certified"]
        certifications.append(report)
    assert all(identity == identities[0] for identity in identities)
    for path in sorted(batch.rglob("*")):
        if path.is_file():
            payloads[path.relative_to(ROOT).as_posix()] = path.read_bytes()
    for path, expected in identities[0]["source_files"].items():
        raw = (ROOT / path).read_bytes()
        assert sha(raw) == expected, path
        payloads["source/" + path] = raw
    for path in BASE.iterdir():
        if path.suffix in {".py", ".sh", ".cmd", ".json"}:
            payloads["harness/" + path.name] = path.read_bytes()
    payloads["harness/bx05-segment-reader.py"] = (ROOT / "evidence/verification/bx-05/reconcile.py").read_bytes()
    for path in sorted((ROOT / "target/bx10-audit-proto").rglob("*_pb2.py")):
        payloads["audit-bindings/" + path.relative_to(ROOT / "target/bx10-audit-proto").as_posix()] = path.read_bytes()
    manifest = {name: {"bytes": len(raw), "sha256": sha(raw)} for name, raw in sorted(payloads.items())}
    archive = BASE / "paired-learners.zip"
    with zipfile.ZipFile(archive, "x", zipfile.ZIP_DEFLATED) as output:
        for name, raw in sorted(payloads.items()):
            output.writestr(name, raw)
        output.writestr("manifest.json", json.dumps(manifest, indent=2) + "\n")
    with zipfile.ZipFile(archive) as check:
        assert check.testzip() is None
        for name, item in manifest.items():
            raw = check.read(name)
            assert len(raw) == item["bytes"] and sha(raw) == item["sha256"]
    result = {
        "archive": archive.relative_to(ROOT).as_posix(), "bytes": archive.stat().st_size,
        "sha256": sha(archive.read_bytes()), "payloads": len(payloads),
        "source_files": len(identities[0]["source_files"]), "live_runs": 20,
        "live_certifications": len(certifications), "conformance_checks_passed": 140,
        "paired_seeds": [7, 19, 42, 73, 91], "steps_per_run": 200,
        "operator_sha256": summary["operator_sha256"],
        "numeric_updates_replayed": sum(row["steps"] for row in replay["runs"]),
        "scientific_quality_claim": False,
    }
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
