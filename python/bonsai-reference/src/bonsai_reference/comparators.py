"""BRDC-1 control and comparator adapters.

Each comparator differs from Track A BRDC-1 in exactly one intended mechanism.
Track B/C/D results never merge with Track A.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

from bonsai_reference.scenario import ScenarioSpec

Track = Literal["track_a", "track_b", "track_c", "track_d"]


@dataclass(frozen=True, slots=True)
class ComparatorSpec:
    comparator_id: str
    intended_mechanism: str
    track: Track
    claim_eligible: bool
    merge_with_track_a: Literal[False]
    difference_count: Literal[1]


@dataclass(frozen=True, slots=True)
class DiagnosticWorld:
    world_id: str
    family: str
    spec: ScenarioSpec
    track: Literal["track_a"]


COMPARATORS: tuple[ComparatorSpec, ...] = (
    ComparatorSpec("fixed_feature", "feature_discovery_disabled", "track_b", False, False, 1),
    ComparatorSpec("primitive_only", "options_disabled", "track_a", True, False, 1),
    ComparatorSpec("no_planning", "planning_disabled", "track_a", True, False, 1),
    ComparatorSpec("local_only_utility", "backward_credit_disabled", "track_a", True, False, 1),
    ComparatorSpec("random_age_retention", "curation_estimator", "track_a", True, False, 1),
    ComparatorSpec("frozen_representation", "feature_revision_disabled", "track_a", True, False, 1),
    ComparatorSpec("reward_oblivious_subtask", "reward_mode", "track_b", False, False, 1),
    ComparatorSpec("primitive_model_only", "option_models_disabled", "track_a", True, False, 1),
    ComparatorSpec("bounded_replay", "replay_capacity", "track_b", False, False, 1),
    ComparatorSpec("dense_update", "batch_size", "track_b", False, False, 1),
    ComparatorSpec("oracle", "privileged_labels", "track_d", False, False, 1),
    ComparatorSpec("unconstrained_growth", "governor_disabled", "track_c", False, False, 1),
)


def diagnostic_worlds() -> tuple[DiagnosticWorld, ...]:
    return (
        DiagnosticWorld(
            world_id="bandit_changepoint",
            family="nonstationary_bandit",
            spec=ScenarioSpec(
                scenario_id="bandit_changepoint",
                version="1.0",
                seed=1,
                horizon=8,
                action_count=3,
                observation_width=2,
                big_world_size=8,
                change_points=(4,),
            ),
            track="track_a",
        ),
        DiagnosticWorld(
            world_id="feature_attainment",
            family="feature_attainment",
            spec=ScenarioSpec(
                scenario_id="feature_attainment",
                version="1.0",
                seed=2,
                horizon=8,
                action_count=3,
                observation_width=3,
                big_world_size=8,
                change_points=(3,),
            ),
            track="track_a",
        ),
        DiagnosticWorld(
            world_id="option_horizon",
            family="option_horizon",
            spec=ScenarioSpec(
                scenario_id="option_horizon",
                version="1.0",
                seed=3,
                horizon=10,
                action_count=4,
                observation_width=2,
                big_world_size=8,
                change_points=(5,),
            ),
            track="track_a",
        ),
    )


def comparator_summary() -> dict[str, object]:
    rows = {
        spec.comparator_id: {
            "intended_mechanism": spec.intended_mechanism,
            "track": spec.track,
            "claim_eligible": spec.claim_eligible,
            "merge_with_track_a": spec.merge_with_track_a,
            "difference_count": spec.difference_count,
        }
        for spec in COMPARATORS
    }
    worlds = {
        world.world_id: {"family": world.family, "track": world.track}
        for world in diagnostic_worlds()
    }
    return {
        "schema": "bonsai.brdc1-comparator-outcomes/v1",
        "comparators": rows,
        "worlds": worlds,
        "merged_tracks": False,
    }
