"""S/C pilots and preregistration freeze (BE-16). A/L claim runs stay gated."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

Profile = Literal["S", "C", "A", "L"]


@dataclass(frozen=True, slots=True)
class PilotRecord:
    profile: Profile
    feasible: bool
    overhead_ppm: int
    exploratory: bool


@dataclass(frozen=True, slots=True)
class FreezeArtifact:
    schema: str
    exploratory_profiles: tuple[Profile, ...]
    confirmatory_profiles: tuple[Profile, ...]
    outcome: str
    seed_schedule: tuple[int, ...]
    horizon: int
    tolerance_ppm: int
    comparison_family: str
    stop_rule: str
    claim_runs_gated: tuple[Profile, ...]
    pspr_amendment_required: bool


def freeze_from_pilots(pilots: tuple[PilotRecord, ...]) -> FreezeArtifact:
    exploratory: tuple[Profile, ...] = tuple(pilot.profile for pilot in pilots if pilot.exploratory)
    confirmatory: tuple[Profile, ...] = ()
    if all(pilot.feasible and pilot.overhead_ppm <= 50_000 for pilot in pilots if pilot.profile in {"S", "C"}):
        confirmatory = ("S", "C")
    return FreezeArtifact(
        schema="bonsai.preregistration-freeze/v1",
        exploratory_profiles=exploratory,
        confirmatory_profiles=confirmatory,
        outcome="lifetime_reward_rate",
        seed_schedule=(1, 2, 3, 4, 5),
        horizon=1_000,
        tolerance_ppm=10_000,
        comparison_family="charter-11",
        stop_rule="preregistered_horizon",
        claim_runs_gated=("A", "L"),
        pspr_amendment_required=False,
    )


def freeze_summary(artifact: FreezeArtifact) -> dict[str, object]:
    return {
        "schema": artifact.schema,
        "exploratory_profiles": list(artifact.exploratory_profiles),
        "confirmatory_profiles": list(artifact.confirmatory_profiles),
        "claim_runs_gated": list(artifact.claim_runs_gated),
        "pilot_used_as_confirmatory": False,
        "pspr_amendment_required": artifact.pspr_amendment_required,
    }
