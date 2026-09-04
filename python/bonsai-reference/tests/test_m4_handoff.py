from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _generate_sbom():
    path = ROOT / "scripts" / "generate_sbom.py"
    spec = importlib.util.spec_from_file_location("generate_sbom", path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_operator_guides_exist_and_m1_command_is_documented() -> None:
    run = (ROOT / "docs" / "operator" / "RUN.md").read_text(encoding="utf-8")
    assert "bonsai_reference.heartbeat" in run
    assert "PYTHONPATH=python/bonsai-reference/src" in run
    limitations = (ROOT / "docs" / "operator" / "LIMITATIONS.md").read_text(encoding="utf-8")
    assert "not a hostile-native-code sandbox" in limitations.lower()
    for name in ("INSTALL.md", "ANALYZE.md", "ADAPTER.md", "METRIC.md", "CLAIM.md", "PLATFORM.md"):
        assert (ROOT / "docs" / "operator" / name).is_file()


def test_l_attestations_are_not_run() -> None:
    for name in ("windows-not-run.json", "macos-not-run.json", "linux-not-run.json"):
        text = (ROOT / "fixtures" / "host-attestation" / "v1" / name).read_text(encoding="utf-8")
        assert '"status": "not-run"' in text
        assert '"physical_acceptance": false' in text
        assert '"long_duration_claim": false' in text


def test_uv_lock_sbom_keeps_registry_packages_off_workspace_path() -> None:
    module = _generate_sbom()
    lock = (ROOT / "uv.lock").read_text(encoding="utf-8")
    rows = {row["name"]: row["purl"] for row in module.packages_from_lock(lock)}
    assert rows["bonsai-reference"] == "virtual+."
    assert rows["protobuf"] == "https://pypi.org/simple"
    assert rows["pytest"] == "https://pypi.org/simple"
    assert "path+workspace" not in {rows["protobuf"], rows["pytest"], rows["colorama"]}


def test_cargo_lock_sbom_still_reads_quoted_registry_sources() -> None:
    module = _generate_sbom()
    rows = module.packages_from_lock((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
    by_name = {row["name"]: row["purl"] for row in rows}
    assert by_name["serde"].startswith("registry+")
    assert any(row["purl"] == "path+workspace" for row in rows)
