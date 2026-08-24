from __future__ import annotations

import json
from pathlib import Path

from bonsai_reference.brdc1 import Brdc1Cycle, CycleError

FIXTURE = Path(__file__).resolve().parents[3] / "fixtures" / "brdc1-options" / "v1" / "expected-outcomes.json"


def test_one_pass_model_tolerance_and_honest_failure() -> None:
    expected = json.loads(FIXTURE.read_text(encoding="utf-8"))
    assert Brdc1Cycle().run_option_model_diagnostic() == expected


def test_unsolved_subproblem_cannot_become_an_option() -> None:
    cycle = Brdc1Cycle()
    cycle.run_diagnostic()
    try:
        cycle.options.solve("opt_bad", "sub_fail")
    except CycleError as error:
        assert error.code == "OPTION_SUBPROBLEM_UNSOLVED"
    else:
        raise AssertionError("expected unsolved rejection")


def test_model_updates_are_batch_one_without_replay() -> None:
    cycle = Brdc1Cycle()
    cycle.run_option_model_diagnostic()
    for model in cycle.models.snapshot():
        assert model.replay_items_retained == 0
        assert model.updates >= 1
