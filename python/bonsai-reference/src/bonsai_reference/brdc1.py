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
class OptionRecord:
    option_id: str
    subproblem_id: str
    policy: tuple[int, ...]
    termination: tuple[int, ...]
    executions: int
    successes: int
    work: int


@dataclass(frozen=True, slots=True)
class ModelRecord:
    option_id: str
    target_id: str
    predicted_reward: int
    predicted_duration: int
    reward_error: int
    duration_error: int
    updates: int
    replay_items_retained: int
    within_tolerance: bool
    detail_code: str | None


@dataclass(frozen=True, slots=True)
class CycleAccounting:
    feature_work: int
    subproblem_work: int
    option_work: int
    model_work: int
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


class OptionStage:
    """Solve successful subproblems into options. Creation is not benefit."""

    def __init__(self, subproblems: SubproblemStage) -> None:
        self._subproblems = subproblems
        self._options: dict[str, OptionRecord] = {}
        self._work = 0

    @property
    def work(self) -> int:
        return self._work

    def snapshot(self) -> tuple[OptionRecord, ...]:
        return tuple(self._options[key] for key in sorted(self._options))

    def solve(self, option_id: str, subproblem_id: str) -> OptionRecord:
        if not option_id or option_id in self._options:
            raise CycleError("OPTION_IDENTITY_INVALID")
        subproblem = next(
            (record for record in self._subproblems.snapshot() if record.subproblem_id == subproblem_id),
            None,
        )
        if subproblem is None:
            raise CycleError("OPTION_SUBPROBLEM_UNKNOWN")
        if not subproblem.success:
            raise CycleError("OPTION_SUBPROBLEM_UNSOLVED")
        policy = (1, 0) if subproblem.original_reward > 0 else (0, 1)
        record = OptionRecord(
            option_id=option_id,
            subproblem_id=subproblem_id,
            policy=policy,
            termination=(0, 1),
            executions=0,
            successes=0,
            work=4,
        )
        self._options[option_id] = record
        self._work += record.work
        return record

    def execute(self, option_id: str, state_index: int) -> tuple[int, bool]:
        current = self._options.get(option_id)
        if current is None:
            raise CycleError("OPTION_UNAVAILABLE")
        action = current.policy[state_index % len(current.policy)]
        terminated = current.termination[state_index % len(current.termination)] == 1
        record = replace(
            current,
            executions=current.executions + 1,
            successes=current.successes + int(terminated),
            work=current.work + 1,
        )
        self._options[option_id] = record
        self._work += 1
        return action, terminated


class ModelStage:
    """Learn option consequences online, batch one, without replay."""

    def __init__(self) -> None:
        self._models: dict[str, ModelRecord] = {}
        self._work = 0

    @property
    def work(self) -> int:
        return self._work

    def snapshot(self) -> tuple[ModelRecord, ...]:
        return tuple(self._models[key] for key in sorted(self._models))

    def start(self, option_id: str, target_id: str) -> ModelRecord:
        if not option_id or option_id in self._models or not target_id:
            raise CycleError("MODEL_IDENTITY_INVALID")
        record = ModelRecord(
            option_id=option_id,
            target_id=target_id,
            predicted_reward=0,
            predicted_duration=0,
            reward_error=0,
            duration_error=0,
            updates=0,
            replay_items_retained=0,
            within_tolerance=False,
            detail_code="MODEL_NOT_UPDATED",
        )
        self._models[option_id] = record
        return record

    def update(
        self,
        option_id: str,
        actual_reward: int,
        actual_duration: int,
        tolerance: int,
    ) -> ModelRecord:
        current = self._models.get(option_id)
        if current is None:
            raise CycleError("MODEL_UNAVAILABLE")
        reward_error = abs(current.predicted_reward - actual_reward)
        duration_error = abs(current.predicted_duration - actual_duration)
        within = reward_error <= tolerance and duration_error <= tolerance
        record = replace(
            current,
            predicted_reward=actual_reward,
            predicted_duration=actual_duration,
            reward_error=reward_error,
            duration_error=duration_error,
            updates=current.updates + 1,
            replay_items_retained=0,
            within_tolerance=within,
            detail_code=None if within else "MODEL_TOLERANCE_FAILED",
        )
        self._models[option_id] = record
        self._work += 1
        return record

    def learn_one_pass(
        self,
        option_id: str,
        target_id: str,
        transitions: tuple[tuple[int, int], ...],
        tolerance: int,
    ) -> ModelRecord:
        self.start(option_id, target_id)
        record = self._models[option_id]
        for reward, duration in transitions:
            record = self.update(option_id, reward, duration, tolerance)
        return record


class Brdc1Cycle:
    """Track A reference cycle through option and model stages."""

    def __init__(self) -> None:
        self.features = FeatureStage()
        self.subproblems = SubproblemStage(self.features)
        self.options = OptionStage(self.subproblems)
        self.models = ModelStage()

    @property
    def accounting(self) -> CycleAccounting:
        return CycleAccounting(
            feature_work=self.features.work,
            subproblem_work=self.subproblems.work,
            option_work=self.options.work,
            model_work=self.models.work,
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

    def run_option_model_diagnostic(self) -> dict[str, object]:
        self.run_diagnostic()
        self.options.solve("opt_ok", "sub_ok")
        self.options.execute("opt_ok", 1)
        passed = self.models.learn_one_pass("opt_ok", "reward_duration", ((2, 1), (2, 1)), 1)
        failed = self.models.learn_one_pass("opt_fail", "reward_duration", ((1, 1), (9, 9)), 1)
        return option_model_summary(self, passed, failed)


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


def option_model_summary(
    cycle: Brdc1Cycle,
    passed: ModelRecord,
    failed: ModelRecord,
) -> dict[str, object]:
    options = {
        record.option_id: {
            "subproblem_id": record.subproblem_id,
            "executions": record.executions,
            "successes": record.successes,
            "work": record.work,
        }
        for record in cycle.options.snapshot()
    }
    return {
        "schema": "bonsai.brdc1-option-model-outcomes/v1",
        "options": options,
        "models": {
            passed.option_id: {
                "updates": passed.updates,
                "reward_error": passed.reward_error,
                "duration_error": passed.duration_error,
                "within_tolerance": passed.within_tolerance,
                "detail_code": passed.detail_code,
                "replay_items_retained": passed.replay_items_retained,
            },
            failed.option_id: {
                "updates": failed.updates,
                "reward_error": failed.reward_error,
                "duration_error": failed.duration_error,
                "within_tolerance": failed.within_tolerance,
                "detail_code": failed.detail_code,
                "replay_items_retained": failed.replay_items_retained,
            },
        },
        "accounting": {
            "option_work": cycle.accounting.option_work,
            "model_work": cycle.accounting.model_work,
            "replay_items_retained": cycle.accounting.replay_items_retained,
        },
    }
