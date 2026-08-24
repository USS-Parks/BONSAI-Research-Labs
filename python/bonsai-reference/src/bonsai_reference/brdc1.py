"""BRDC-1 Track A feature and subproblem stages.

Creation is cost, not utility. Semantic labels never enter Track A.
"""

from __future__ import annotations

from dataclasses import dataclass, replace
from typing import Literal

RewardMode = Literal["respecting"]


class CycleError(ValueError):
    """Stable BRDC-1 protocol failure."""

    def __init__(self, code: str) -> None:
        self.code = code
        super().__init__(code)


@dataclass(frozen=True, slots=True)
class FeatureRecord:
    feature_id: str
    representation: tuple[int, ...]
    birth_step: int
    retirement_step: int | None
    bytes: int
    work: int
    consumers: int
    utility: int | None
    parents: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class SubproblemRecord:
    subproblem_id: str
    feature_id: str
    reward_mode: RewardMode
    original_reward: int
    stopping_bonus: int
    attained: bool
    success: bool
    work: int


@dataclass(frozen=True, slots=True)
class CycleAccounting:
    feature_work: int
    subproblem_work: int
    replay_items_retained: int


class FeatureStage:
    """Produce, revise, and retire integer feature candidates."""

    def __init__(self) -> None:
        self._features: dict[str, FeatureRecord] = {}
        self._work = 0

    @property
    def work(self) -> int:
        return self._work

    def snapshot(self) -> tuple[FeatureRecord, ...]:
        return tuple(self._features[key] for key in sorted(self._features))

    def produce(
        self,
        feature_id: str,
        representation: tuple[int, ...],
        step: int,
        *,
        semantic_label: str | None = None,
        parents: tuple[str, ...] = (),
    ) -> FeatureRecord:
        if semantic_label is not None:
            raise CycleError("TRACK_A_LABEL_REJECTED")
        if not feature_id or feature_id in self._features or not representation:
            raise CycleError("FEATURE_IDENTITY_INVALID")
        for parent in parents:
            if parent not in self._features:
                raise CycleError("FEATURE_PARENT_UNKNOWN")
        record = FeatureRecord(
            feature_id=feature_id,
            representation=representation,
            birth_step=step,
            retirement_step=None,
            bytes=len(representation),
            work=2,
            consumers=0,
            utility=None,
            parents=parents,
        )
        self._features[feature_id] = record
        self._work += record.work
        return record

    def revise(self, feature_id: str, representation: tuple[int, ...], step: int) -> FeatureRecord:
        current = self._require_active(feature_id)
        if not representation:
            raise CycleError("FEATURE_IDENTITY_INVALID")
        record = replace(
            current,
            representation=representation,
            bytes=len(representation),
            work=current.work + 1,
            utility=None,
        )
        self._features[feature_id] = record
        self._work += 1
        return record

    def retire(self, feature_id: str, step: int) -> FeatureRecord:
        current = self._require_active(feature_id)
        record = replace(current, retirement_step=step)
        self._features[feature_id] = record
        self._work += 1
        return record

    def add_consumer(self, feature_id: str) -> FeatureRecord:
        current = self._require_active(feature_id)
        record = replace(current, consumers=current.consumers + 1)
        self._features[feature_id] = record
        return record

    def _require_active(self, feature_id: str) -> FeatureRecord:
        current = self._features.get(feature_id)
        if current is None or current.retirement_step is not None:
            raise CycleError("FEATURE_UNAVAILABLE")
        return current


class SubproblemStage:
    """Pose reward-respecting feature-attainment subproblems."""

    def __init__(self, features: FeatureStage) -> None:
        self._features = features
        self._subproblems: dict[str, SubproblemRecord] = {}
        self._work = 0

    @property
    def work(self) -> int:
        return self._work

    def snapshot(self) -> tuple[SubproblemRecord, ...]:
        return tuple(self._subproblems[key] for key in sorted(self._subproblems))

    def pose(
        self,
        subproblem_id: str,
        feature_id: str,
        original_reward: int,
        stopping_bonus: int,
        attained: bool,
        *,
        reward_mode: RewardMode = "respecting",
    ) -> SubproblemRecord:
        if reward_mode != "respecting":
            raise CycleError("TRACK_A_OBLIVIOUS_SUBPROBLEM")
        if not subproblem_id or subproblem_id in self._subproblems:
            raise CycleError("SUBPROBLEM_IDENTITY_INVALID")
        self._features.add_consumer(feature_id)
        success = attained and original_reward > 0
        record = SubproblemRecord(
            subproblem_id=subproblem_id,
            feature_id=feature_id,
            reward_mode="respecting",
            original_reward=original_reward,
            stopping_bonus=stopping_bonus,
            attained=attained,
            success=success,
            work=3,
        )
        self._subproblems[subproblem_id] = record
        self._work += record.work
        return record


class Brdc1Cycle:
    """Track A reference cycle through the feature and subproblem stages."""

    def __init__(self) -> None:
        self.features = FeatureStage()
        self.subproblems = SubproblemStage(self.features)

    @property
    def accounting(self) -> CycleAccounting:
        return CycleAccounting(
            feature_work=self.features.work,
            subproblem_work=self.subproblems.work,
            replay_items_retained=0,
        )

    def run_diagnostic(self) -> dict[str, object]:
        self.features.produce("f_old", (1, 0), 0)
        self.features.produce("f_new", (0, 1), 1, parents=("f_old",))
        self.features.revise("f_new", (0, 2), 2)
        self.features.retire("f_old", 3)
        self.subproblems.pose("sub_ok", "f_new", 1, 1, True)
        self.subproblems.pose("sub_fail", "f_new", 0, 1, True)
        return diagnostic_summary(self)


def diagnostic_summary(cycle: Brdc1Cycle) -> dict[str, object]:
    features = {
        record.feature_id: {
            "birth_step": record.birth_step,
            "retirement_step": record.retirement_step,
            "bytes": record.bytes,
            "work": record.work,
            "consumers": record.consumers,
            "utility": record.utility,
            "parents": list(record.parents),
        }
        for record in cycle.features.snapshot()
    }
    subproblems = {
        record.subproblem_id: {
            "feature_id": record.feature_id,
            "reward_mode": record.reward_mode,
            "original_reward": record.original_reward,
            "success": record.success,
            "work": record.work,
        }
        for record in cycle.subproblems.snapshot()
    }
    return {
        "schema": "bonsai.brdc1-feature-subproblem-outcomes/v1",
        "features": features,
        "subproblems": subproblems,
        "accounting": {
            "feature_work": cycle.accounting.feature_work,
            "subproblem_work": cycle.accounting.subproblem_work,
            "replay_items_retained": cycle.accounting.replay_items_retained,
        },
    }
