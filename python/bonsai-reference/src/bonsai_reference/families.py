"""M3 scenario families. Privileged labels never enter Track A."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

from bonsai_reference.comparators import COMPARATORS
from bonsai_reference.scenario import ScenarioSpec

FamilyId = Literal[
    "stable_dynamics_changing_values",
    "observation_aliasing",
    "late_factor",
    "long_life_plasticity",
    "noisy_single_pass",
    "temporal_joints",
    "recursive_reuse",
    "distractor",
    "resource_shock",
    "model_mismatch",
]


@dataclass(frozen=True, slots=True)
class FamilySpec:
    family_id: FamilyId
    diagnostic: ScenarioSpec
    enlarged: ScenarioSpec
    localization: str
    track_a_labels: Literal[False]
    manifest_visible_shock: bool


def _spec(scenario_id: str, seed: int, horizon: int, width: int, changes: tuple[int, ...]) -> ScenarioSpec:
    return ScenarioSpec(
        scenario_id=scenario_id,
        version="1.0",
        seed=seed,
        horizon=horizon,
        action_count=3,
        observation_width=width,
        big_world_size=32,
        change_points=changes,
    )


FAMILIES: tuple[FamilySpec, ...] = (
    FamilySpec(
        "stable_dynamics_changing_values",
        _spec("revalue-diag", 11, 12, 2, (6,)),
        _spec("revalue-big", 11, 24, 2, (8, 16)),
        "reward_revaluation_with_fixed_dynamics",
        False,
        False,
    ),
    FamilySpec(
        "observation_aliasing",
        _spec("alias-diag", 12, 12, 1, (4,)),
        _spec("alias-big", 12, 24, 1, (8,)),
        "aliasing_not_representation_insufficiency",
        False,
        False,
    ),
    FamilySpec(
        "late_factor",
        _spec("late-diag", 13, 12, 3, (8,)),
        _spec("late-big", 13, 24, 3, (16,)),
        "new_useful_feature_after_latent_arrival",
        False,
        False,
    ),
    FamilySpec(
        "long_life_plasticity",
        _spec("plastic-diag", 14, 16, 2, (4, 8, 12)),
        _spec("plastic-big", 14, 32, 2, (8, 16, 24)),
        "plasticity_loss_not_forgetting",
        False,
        False,
    ),
    FamilySpec(
        "noisy_single_pass",
        _spec("noise-diag", 15, 12, 2, (6,)),
        _spec("noise-big", 15, 24, 2, (12,)),
        "single_authorized_encounter",
        False,
        False,
    ),
    FamilySpec(
        "temporal_joints",
        _spec("joint-diag", 16, 12, 2, (6,)),
        _spec("joint-big", 16, 24, 2, (12,)),
        "sustained_behavior_from_earlier_option",
        False,
        False,
    ),
    FamilySpec(
        "recursive_reuse",
        _spec("reuse-diag", 17, 12, 2, (6,)),
        _spec("reuse-big", 17, 24, 2, (12,)),
        "higher_abstraction_depends_on_earlier",
        False,
        False,
    ),
    FamilySpec(
        "distractor",
        _spec("distract-diag", 18, 12, 3, (6,)),
        _spec("distract-big", 18, 24, 3, (12,)),
        "utility_selects_against_known_distractor",
        False,
        True,
    ),
    FamilySpec(
        "resource_shock",
        _spec("shock-diag", 19, 12, 2, (6,)),
        _spec("shock-big", 19, 24, 2, (12,)),
        "governor_localizes_manifest_budget_change",
        False,
        True,
    ),
    FamilySpec(
        "model_mismatch",
        _spec("mismatch-diag", 20, 12, 2, (6,)),
        _spec("mismatch-big", 20, 24, 2, (12,)),
        "harmful_stale_model_detector",
        False,
        True,
    ),
)


def family_summary() -> dict[str, object]:
    rows = {
        spec.family_id: {
            "diagnostic": spec.diagnostic.scenario_id,
            "enlarged": spec.enlarged.scenario_id,
            "localization": spec.localization,
            "track_a_labels": spec.track_a_labels,
            "manifest_visible_shock": spec.manifest_visible_shock,
        }
        for spec in FAMILIES
    }
    return {
        "schema": "bonsai.scenario-families/v1",
        "families": rows,
        "family_count": len(FAMILIES),
        "comparators": [spec.comparator_id for spec in COMPARATORS],
        "privileged_labels_in_track_a": False,
    }
