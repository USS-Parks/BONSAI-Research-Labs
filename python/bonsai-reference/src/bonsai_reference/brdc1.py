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
class BackupRecord:
    backup_id: str
    state_id: str
    random_draw: int
    value_delta: int
    realized_action: str
    counterfactual_action: str
    consequential: bool
    work: int


@dataclass(frozen=True, slots=True)
class CreditRecord:
    artifact_id: str
    consumer_id: str
    construction_step: int
    credit_step: int
    latency: int
    utility: int
    tier: Literal["exact_leave_one_out"]
    work: int


@dataclass(frozen=True, slots=True)
class CurationRecord:
    artifact_id: str
    estimator: Literal["declared_utility"]
    disposition: Literal["retain", "deprioritize", "replace", "remove"]
    reason: str
    applied: bool
    lineage_intact: bool
    birth_step: int
    representation: tuple[int, ...]


@dataclass(frozen=True, slots=True)
class CycleAccounting:
    feature_work: int
    subproblem_work: int
    option_work: int
    model_work: int
    planning_work: int
    credit_work: int
    curation_work: int
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

    def set_utility(self, feature_id: str, utility: int) -> FeatureRecord:
        current = self._features.get(feature_id)
        if current is None:
            raise CycleError("FEATURE_UNAVAILABLE")
        record = replace(current, utility=utility)
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


class PlanningStage:
    """Plan with option models under an external backup budget."""

    def __init__(self, budget: int) -> None:
        if budget <= 0:
            raise CycleError("PLANNING_BUDGET_INVALID")
        self.budget = budget
        self._backups: dict[str, BackupRecord] = {}
        self._work = 0

    @property
    def work(self) -> int:
        return self._work

    def snapshot(self) -> tuple[BackupRecord, ...]:
        return tuple(self._backups[key] for key in sorted(self._backups))

    def backup(
        self,
        backup_id: str,
        state_id: str,
        random_draw: int,
        value_delta: int,
        realized_action: str,
        counterfactual_action: str,
    ) -> BackupRecord:
        if self._work >= self.budget:
            raise CycleError("PLANNING_BUDGET_EXHAUSTED")
        if not backup_id or backup_id in self._backups or not state_id:
            raise CycleError("PLANNING_IDENTITY_INVALID")
        record = BackupRecord(
            backup_id=backup_id,
            state_id=state_id,
            random_draw=random_draw,
            value_delta=value_delta,
            realized_action=realized_action,
            counterfactual_action=counterfactual_action,
            consequential=realized_action != counterfactual_action,
            work=1,
        )
        self._backups[backup_id] = record
        self._work += 1
        return record

    def enforce_compliance(self) -> None:
        raise CycleError("SCHEDULER_NOT_AUTHORITATIVE")


class CreditStage:
    """Return slower backward utility credit to upstream artifacts."""

    def __init__(self, features: FeatureStage) -> None:
        self._features = features
        self._credits: dict[str, CreditRecord] = {}
        self._work = 0

    @property
    def work(self) -> int:
        return self._work

    def snapshot(self) -> tuple[CreditRecord, ...]:
        return tuple(self._credits[key] for key in sorted(self._credits))

    def credit(
        self,
        artifact_id: str,
        consumer_id: str,
        construction_step: int,
        credit_step: int,
        utility: int,
    ) -> CreditRecord:
        if credit_step <= construction_step:
            raise CycleError("CREDIT_NOT_SLOWER")
        if not artifact_id or artifact_id in self._credits:
            raise CycleError("CREDIT_IDENTITY_INVALID")
        self._features.set_utility(artifact_id, utility)
        record = CreditRecord(
            artifact_id=artifact_id,
            consumer_id=consumer_id,
            construction_step=construction_step,
            credit_step=credit_step,
            latency=credit_step - construction_step,
            utility=utility,
            tier="exact_leave_one_out",
            work=1,
        )
        self._credits[artifact_id] = record
        self._work += 1
        return record


class CurationStage:
    """Propose dispositions. The external governor remains authoritative."""

    def __init__(self, features: FeatureStage) -> None:
        self._features = features
        self._records: dict[str, CurationRecord] = {}
        self._work = 0

    @property
    def work(self) -> int:
        return self._work

    def snapshot(self) -> tuple[CurationRecord, ...]:
        return tuple(self._records[key] for key in sorted(self._records))

    def propose(self, artifact_id: str) -> CurationRecord:
        feature = next(
            (record for record in self._features.snapshot() if record.feature_id == artifact_id),
            None,
        )
        if feature is None:
            raise CycleError("CURATION_ARTIFACT_UNKNOWN")
        disposition, reason = self._classify(feature)
        record = CurationRecord(
            artifact_id=artifact_id,
            estimator="declared_utility",
            disposition=disposition,
            reason=reason,
            applied=False,
            lineage_intact=True,
            birth_step=feature.birth_step,
            representation=feature.representation,
        )
        self._records[artifact_id] = record
        self._work += 1
        return record

    def apply(self, artifact_id: str, *, governor_admitted: bool) -> CurationRecord:
        if not governor_admitted:
            raise CycleError("GOVERNOR_AUTHORITATIVE")
        current = self._records.get(artifact_id)
        if current is None:
            raise CycleError("CURATION_PROPOSAL_UNKNOWN")
        feature = next(
            record
            for record in self._features.snapshot()
            if record.feature_id == artifact_id
        )
        record = replace(
            current,
            applied=True,
            lineage_intact=feature.birth_step == current.birth_step
            and feature.representation == current.representation,
        )
        self._records[artifact_id] = record
        return record

    def _classify(
        self, feature: FeatureRecord
    ) -> tuple[Literal["retain", "deprioritize", "replace", "remove"], str]:
        if feature.utility is not None and feature.utility > 0 and feature.consumers > 0:
            return "retain", "useful"
        siblings = [
            record
            for record in self._features.snapshot()
            if record.feature_id != feature.feature_id
            and record.representation == feature.representation
        ]
        if siblings:
            return "deprioritize", "redundant"
        if feature.utility is None and feature.consumers == 0:
            return "remove", "stale"
        return "replace", "low_utility"


class Brdc1Cycle:
    """Track A reference cycle through planning and backward credit."""

    def __init__(self, planning_budget: int = 4) -> None:
        self.features = FeatureStage()
        self.subproblems = SubproblemStage(self.features)
        self.options = OptionStage(self.subproblems)
        self.models = ModelStage()
        self.planning = PlanningStage(planning_budget)
        self.credit = CreditStage(self.features)
        self.curation = CurationStage(self.features)

    @property
    def accounting(self) -> CycleAccounting:
        return CycleAccounting(
            feature_work=self.features.work,
            subproblem_work=self.subproblems.work,
            option_work=self.options.work,
            model_work=self.models.work,
            planning_work=self.planning.work,
            credit_work=self.credit.work,
            curation_work=self.curation.work,
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

    def run_planning_credit_diagnostic(self) -> dict[str, object]:
        self.run_option_model_diagnostic()
        self.planning.backup("value_only", "s0", 7, 8, "left", "left")
        self.planning.backup("action_change", "s0", 7, 0, "left", "right")
        self.credit.credit("f_new", "opt_ok", 1, 14, 5)
        return planning_credit_summary(self)

    def run_curation_diagnostic(self) -> dict[str, object]:
        self.run_planning_credit_diagnostic()
        self.features.produce("f_dup", (0, 2), 15)
        self.features.produce("f_stale", (9, 9), 16)
        for feature_id in ("f_new", "f_dup", "f_stale"):
            self.curation.propose(feature_id)
            self.curation.apply(feature_id, governor_admitted=True)
        return curation_summary(self)


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


def planning_credit_summary(cycle: Brdc1Cycle) -> dict[str, object]:
    backups = {
        record.backup_id: {
            "value_delta": record.value_delta,
            "consequential": record.consequential,
            "work": record.work,
        }
        for record in cycle.planning.snapshot()
    }
    credits = {
        record.artifact_id: {
            "consumer_id": record.consumer_id,
            "construction_step": record.construction_step,
            "credit_step": record.credit_step,
            "latency": record.latency,
            "utility": record.utility,
            "tier": record.tier,
        }
        for record in cycle.credit.snapshot()
    }
    return {
        "schema": "bonsai.brdc1-planning-credit-outcomes/v1",
        "forward_before_credit": True,
        "backups": backups,
        "credits": credits,
        "reconcile": {
            "planning_work": cycle.accounting.planning_work,
            "credit_work": cycle.accounting.credit_work,
            "consequential_backups": sum(
                1 for record in cycle.planning.snapshot() if record.consequential
            ),
            "budget": cycle.planning.budget,
        },
    }


def curation_summary(cycle: Brdc1Cycle) -> dict[str, object]:
    records = {
        record.artifact_id: {
            "estimator": record.estimator,
            "disposition": record.disposition,
            "reason": record.reason,
            "applied": record.applied,
            "lineage_intact": record.lineage_intact,
            "birth_step": record.birth_step,
        }
        for record in cycle.curation.snapshot()
    }
    return {
        "schema": "bonsai.brdc1-curation-outcomes/v1",
        "records": records,
        "oak_solution_claimed": False,
        "accounting": {"curation_work": cycle.accounting.curation_work},
    }
