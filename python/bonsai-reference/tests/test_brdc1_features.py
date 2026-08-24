from __future__ import annotations

import json
from pathlib import Path

from bonsai_reference.brdc1 import Brdc1Cycle, CycleError

FIXTURE = Path(__file__).resolve().parents[3] / "fixtures" / "brdc1-features" / "v1" / "expected-outcomes.json"


def test_diagnostic_lifecycle_and_cost_match_fixture() -> None:
    expected = json.loads(FIXTURE.read_text(encoding="utf-8"))
    assert Brdc1Cycle().run_diagnostic() == expected


def test_creation_is_not_utility() -> None:
    cycle = Brdc1Cycle()
    record = cycle.features.produce("f0", (1,), 0)
    assert record.utility is None
    assert record.work == 2


def test_track_a_rejects_semantic_labels() -> None:
    cycle = Brdc1Cycle()
    try:
        cycle.features.produce("f0", (1,), 0, semantic_label="door")
    except CycleError as error:
        assert error.code == "TRACK_A_LABEL_REJECTED"
    else:
        raise AssertionError("expected label rejection")
