from __future__ import annotations

import json
from pathlib import Path

from bonsai_reference.stats import (
    OutcomeFamily,
    SeedPair,
    StatError,
    aggregate,
    diagnostic_families,
    diagnostic_plan,
)

FIXTURE = (
    Path(__file__).resolve().parents[3]
    / "fixtures"
    / "statistical-aggregation"
    / "v1"
    / "expected-outcomes.json"
)


def test_golden_corpus_matches_committed_reference() -> None:
    expected = json.loads(FIXTURE.read_text(encoding="utf-8"))
    table = aggregate(diagnostic_plan(), diagnostic_families())
    rows = {
        row.outcome_id: {
            "outcome_id": row.outcome_id,
            "complete_pairs": row.complete_pairs,
            "missing_runs": row.missing_runs,
            "failed_runs": row.failed_runs,
            "excluded_runs": row.excluded_runs,
            "effect_numerator": row.effect_numerator,
            "effect_denominator": row.effect_denominator,
            "ci_low_numerator": row.ci_low_numerator,
            "ci_high_numerator": row.ci_high_numerator,
            "ci_denominator": row.ci_denominator,
            "p_numerator": row.p_numerator,
            "p_denominator": row.p_denominator,
            "holm_rejected": row.holm_rejected,
            "sensitivity_min_numerator": row.sensitivity_min_numerator,
            "sensitivity_max_numerator": row.sensitivity_max_numerator,
            "sensitivity_denominator": row.sensitivity_denominator,
            "detail_code": row.detail_code,
        }
        for row in table
    }
    assert {"schema": "bonsai.statistical-outcomes/v1", "rows": rows} == expected


def test_undeclared_exclusion_is_rejected() -> None:
    families = list(diagnostic_families())
    sneak = OutcomeFamily(
        outcome_id=families[0].outcome_id,
        pairs=(*families[0].pairs, SeedPair("sneak", 1, 0, False, False)),
    )
    try:
        aggregate(diagnostic_plan(), (sneak, families[1]))
    except StatError as error:
        assert error.code == "STAT_UNDECLARED_EXCLUSION"
    else:
        raise AssertionError("expected undeclared exclusion")


def test_undeclared_metric_is_rejected() -> None:
    extra = OutcomeFamily(outcome_id="posthoc", pairs=(SeedPair("p1", 1, 0, False, False),))
    try:
        aggregate(diagnostic_plan(), (*diagnostic_families(), extra))
    except StatError as error:
        assert error.code == "STAT_UNDECLARED_METRIC"
    else:
        raise AssertionError("expected undeclared metric")
