from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import cast

from bonsai_reference.heartbeat import HeartbeatArtifacts, JsonValue, build_heartbeat, canonical, emit

ROOT = Path(__file__).resolve().parents[3]
SOURCE_REVISION = "a" * 40


def load_json(path: Path) -> JsonValue:
    return cast(JsonValue, json.loads(path.read_text(encoding="utf-8")))


def test_all_hosted_platforms_share_the_frozen_semantic_summary() -> None:
    platforms = [("windows", "x86_64"), ("linux", "x86_64"), ("macos", "arm64"), ("macos", "x86_64")]
    artifacts = [build_heartbeat(ROOT, SOURCE_REVISION, family, arch) for family, arch in platforms]
    expected = load_json(ROOT / "fixtures/m1-heartbeat/expected-summary.json")
    assert all(item.semantic_summary == expected for item in artifacts)
    assert {(item.platform_summary["os_family"], item.platform_summary["architecture"]) for item in artifacts} == set(
        platforms
    )
    assert len({canonical(item.semantic_summary) for item in artifacts}) == 1


def test_bundle_hashes_budget_and_report_inputs_reconcile(tmp_path: Path) -> None:
    artifacts: HeartbeatArtifacts = build_heartbeat(ROOT, SOURCE_REVISION, "windows", "x86_64")
    emit(artifacts, tmp_path)
    manifest = cast(dict[str, JsonValue], load_json(tmp_path / "manifest.json"))
    entries = cast(list[JsonValue], manifest["files"])
    for raw_entry in entries:
        entry = cast(dict[str, JsonValue], raw_entry)
        path = cast(str, entry["path"])
        expected_hash = cast(str, entry["sha256"])
        assert hashlib.sha256((tmp_path / path).read_bytes()).hexdigest() == expected_hash
    report = cast(dict[str, JsonValue], load_json(tmp_path / "report-input.json"))
    resources = cast(dict[str, JsonValue], report["resources"])
    claims = cast(dict[str, JsonValue], report["claims"])
    assert resources == {
        "decision": "admit",
        "energy_tier": "E0",
        "headroom": 32,
        "work_budget": 160,
        "work_items": 128,
    }
    assert claims["verdict"] == "not_adjudicated"
    assert claims["C0"] == "reportable_evidence_present"
    assert claims["C1"] == "reportable_budget_evidence_present"
