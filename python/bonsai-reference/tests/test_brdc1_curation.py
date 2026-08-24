from __future__ import annotations

import json
from pathlib import Path

from bonsai_reference.brdc1 import Brdc1Cycle, CycleError

FIXTURE = Path(__file__).resolve().parents[3] / "fixtures" / "brdc1-curation" / "v1" / "expected-outcomes.json"


def test_useful_redundant_and_stale_dispositions_match_fixture() -> None:
    expected = json.loads(FIXTURE.read_text(encoding="utf-8"))
    assert Brdc1Cycle().run_curation_diagnostic() == expected


def test_governor_remains_authoritative() -> None:
    cycle = Brdc1Cycle()
    cycle.run_planning_credit_diagnostic()
    cycle.curation.propose("f_new")
    try:
        cycle.curation.apply("f_new", governor_admitted=False)
    except CycleError as error:
        assert error.code == "GOVERNOR_AUTHORITATIVE"
    else:
        raise AssertionError("expected governor authority")


def test_lineage_survives_curation() -> None:
    cycle = Brdc1Cycle()
    cycle.run_curation_diagnostic()
    feature = next(record for record in cycle.features.snapshot() if record.feature_id == "f_new")
    assert feature.birth_step == 1
    assert feature.representation == (0, 2)
    assert feature.utility == 5
