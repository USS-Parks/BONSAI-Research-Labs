from __future__ import annotations

import json
from pathlib import Path

from bonsai_reference.comparators import COMPARATORS, comparator_summary, diagnostic_worlds
from bonsai_reference.scenario import semantic_stream, validate_spec

FIXTURE = Path(__file__).resolve().parents[3] / "fixtures" / "brdc1-comparators" / "v1" / "expected-outcomes.json"


def test_each_comparator_differs_in_one_mechanism() -> None:
    mechanisms = [spec.intended_mechanism for spec in COMPARATORS]
    assert len(mechanisms) == len(set(mechanisms))
    assert all(spec.difference_count == 1 for spec in COMPARATORS)
    assert all(spec.merge_with_track_a is False for spec in COMPARATORS)


def test_track_b_c_d_never_merge_with_track_a() -> None:
    foreign = [spec for spec in COMPARATORS if spec.track != "track_a"]
    assert foreign
    assert all(not spec.claim_eligible for spec in foreign)
    assert comparator_summary()["merged_tracks"] is False


def test_committed_comparators_and_worlds_match_fixture() -> None:
    expected = json.loads(FIXTURE.read_text(encoding="utf-8"))
    assert comparator_summary() == expected
    worlds = diagnostic_worlds()
    assert len({world.family for world in worlds}) >= 3
    for world in worlds:
        validate_spec(world.spec)
        semantic_stream(world.spec, tuple(0 for _ in range(world.spec.horizon)))
