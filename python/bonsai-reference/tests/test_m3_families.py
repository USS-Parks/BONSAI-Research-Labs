from __future__ import annotations

from bonsai_reference.families import FAMILIES, family_summary
from bonsai_reference.freeze import PilotRecord, freeze_from_pilots, freeze_summary
from bonsai_reference.orchestrate import ablation_pairs, orchestration_summary, undeclared_delta_blocks
from bonsai_reference.scenario import semantic_stream


def test_ten_families_keep_labels_off_track_a() -> None:
    summary = family_summary()
    assert summary["family_count"] == 10
    assert summary["privileged_labels_in_track_a"] is False
    assert all(spec.track_a_labels is False for spec in FAMILIES)
    shocks = [spec.family_id for spec in FAMILIES if spec.manifest_visible_shock]
    assert shocks == ["distractor", "resource_shock", "model_mismatch"]
    revalue = next(spec for spec in FAMILIES if spec.family_id == "stable_dynamics_changing_values")
    actions = tuple(0 for _ in range(revalue.diagnostic.horizon))
    first = semantic_stream(revalue.diagnostic, actions)
    assert semantic_stream(revalue.diagnostic, actions) == first


def test_eleven_charter_ablations_are_matched() -> None:
    pairs = ablation_pairs()
    assert len(pairs) == 11
    summary = orchestration_summary()
    assert summary["pair_count"] == 11
    first = pairs[0]
    assert first.stream_id == pairs[-1].stream_id
    assert first.seed_schedule == pairs[-1].seed_schedule
    assert undeclared_delta_blocks(first, "hidden-budget") is True
    assert undeclared_delta_blocks(first, first.declared_difference) is False


def test_freeze_separates_exploratory_from_gated_al() -> None:
    artifact = freeze_from_pilots(
        (
            PilotRecord("S", True, 12_000, True),
            PilotRecord("C", True, 20_000, True),
        )
    )
    summary = freeze_summary(artifact)
    assert summary["confirmatory_profiles"] == ["S", "C"]
    assert summary["claim_runs_gated"] == ["A", "L"]
    assert summary["pilot_used_as_confirmatory"] is False
    assert artifact.pspr_amendment_required is False
