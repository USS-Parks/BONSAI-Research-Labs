from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def test_operator_guides_exist_and_m1_command_is_documented() -> None:
    run = (ROOT / "docs" / "operator" / "RUN.md").read_text(encoding="utf-8")
    assert "bonsai_reference.heartbeat" in run
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
