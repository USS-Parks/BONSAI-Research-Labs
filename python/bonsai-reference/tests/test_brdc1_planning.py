from __future__ import annotations

import json
from pathlib import Path

from bonsai_reference.brdc1 import Brdc1Cycle, CycleError

FIXTURE = Path(__file__).resolve().parents[3] / "fixtures" / "brdc1-planning" / "v1" / "expected-outcomes.json"


def test_forward_construction_and_slower_credit_match_fixture() -> None:
    expected = json.loads(FIXTURE.read_text(encoding="utf-8"))
    assert Brdc1Cycle().run_planning_credit_diagnostic() == expected


def test_scheduler_cannot_self_enforce() -> None:
    try:
        Brdc1Cycle().planning.enforce_compliance()
    except CycleError as error:
        assert error.code == "SCHEDULER_NOT_AUTHORITATIVE"
    else:
        raise AssertionError("expected scheduler denial")


def test_credit_must_be_slower_than_construction() -> None:
    cycle = Brdc1Cycle()
    cycle.run_option_model_diagnostic()
    try:
        cycle.credit.credit("f_new", "opt_ok", 1, 1, 5)
    except CycleError as error:
        assert error.code == "CREDIT_NOT_SLOWER"
    else:
        raise AssertionError("expected slower-credit rejection")
