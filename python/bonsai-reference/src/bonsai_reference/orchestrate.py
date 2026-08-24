"""Matched-budget ablation orchestration for the eleven charter pairs (BE-15)."""

from __future__ import annotations

from dataclasses import dataclass

CHARTER_ABLATIONS: tuple[tuple[str, str, str], ...] = (
    ("learned_features", "fixed_features", "feature_discovery_disabled"),
    ("feature_utility", "random_age_retention", "curation_estimator"),
    ("reward_respecting", "reward_oblivious", "reward_mode"),
    ("learned_options", "primitive_only", "options_disabled"),
    ("option_models", "primitive_models", "option_models_disabled"),
    ("planning", "reactive", "planning_disabled"),
    ("backward_credit", "local_only_utility", "backward_credit_disabled"),
    ("event_driven", "dense_update", "batch_size"),
    ("no_replay", "bounded_replay", "replay_capacity"),
    ("continual_replacement", "frozen_representation", "feature_revision_disabled"),
    ("resource_governance", "unconstrained_growth", "governor_disabled"),
)


@dataclass(frozen=True, slots=True)
class AblationPair:
    ablation_id: str
    treatment: str
    control: str
    stream_id: str
    seed_schedule: tuple[int, ...]
    resource_profile: str
    instrumentation: str
    declared_difference: str


def ablation_pairs(stream_id: str = "family-matched", seeds: tuple[int, ...] = (1, 2, 3)) -> tuple[AblationPair, ...]:
    return tuple(
        AblationPair(
            ablation_id=f"{treatment}_vs_{control}",
            treatment=treatment,
            control=control,
            stream_id=stream_id,
            seed_schedule=seeds,
            resource_profile="S",
            instrumentation="full",
            declared_difference=difference,
        )
        for treatment, control, difference in CHARTER_ABLATIONS
    )


def undeclared_delta_blocks(pair: AblationPair, extra_difference: str | None) -> bool:
    return extra_difference is not None and extra_difference != pair.declared_difference


def orchestration_summary() -> dict[str, object]:
    pairs = ablation_pairs()
    return {
        "schema": "bonsai.ablation-orchestration/v1",
        "pair_count": len(pairs),
        "pairs": {
            pair.ablation_id: {
                "stream_id": pair.stream_id,
                "seed_schedule": list(pair.seed_schedule),
                "resource_profile": pair.resource_profile,
                "instrumentation": pair.instrumentation,
                "declared_difference": pair.declared_difference,
            }
            for pair in pairs
        },
        "undeclared_delta_blocks": True,
    }
