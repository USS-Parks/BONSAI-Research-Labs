"""Bounded, admitted online subproblem and option learning.

The controller consumes each causal transition once.  It retains finite
sufficient statistics and learned parameters, but never retains transitions.
"""

from __future__ import annotations

import copy
import hashlib
import sys
import uuid
from dataclasses import dataclass, fields, is_dataclass
from typing import Literal, Protocol, cast

from bonsai_reference.brdc1 import CycleError, OptionStage, SubproblemStage
from bonsai_reference.control import WorkAccounting
from bonsai_reference.feature_discovery import OnlineFeatureStage, canonical

RewardMode = Literal["respecting", "oblivious"]
State = tuple[int, ...]
StateAction = tuple[State, int]

MAX_STATES = 64
MAX_OPTIONS = 4
MAX_DURATION = 16
MAX_UPDATE_BYTES = 32 * 1024
MAX_RETAINED_BYTES = 1024 * 1024
MAX_SERIALIZED_BYTES = 256 * 1024
MAX_PARAMETER_TOUCHES_PER_UPDATE = 1024
FEATURE_STATISTICS = 32
FEATURES = 4
STOPPING_BONUS = 1
Q_LIMIT = 2**31 - 1


class OptionAdmission(Protocol):
    """External authority for class-bound work and state reservations."""

    def work(self, work_class: str, request_id: str, amount: int) -> dict[str, object]: ...

    def allocation(
        self, request_id: str, allocated_bytes: int, serialized_bytes: int
    ) -> dict[str, object]: ...


@dataclass(slots=True)
class _Subproblem:
    subproblem_id: str
    feature_id: str
    reward_mode: RewardMode
    birth_update: int
    feature_birth_revision_id: str
    feature_latest_revision_id: str
    feature_revision_count: int
    stopping_bonus: int = STOPPING_BONUS
    updates: int = 0
    attainments: int = 0
    reward_aligned_successes: int = 0
    cumulative_environment_reward: int = 0
    cumulative_subproblem_return: int = 0


@dataclass(slots=True)
class _Option:
    option_id: str
    subproblem_id: str
    birth_update: int
    states: dict[State, None]
    q_values: dict[StateAction, int]
    action_visits: dict[StateAction, int]
    beta_counts: dict[State, tuple[int, int]]
    q_updates: int = 0
    beta_updates: int = 0
    executions: int = 0
    completed_executions: int = 0
    successful_executions: int = 0
    cumulative_duration: int = 0
    cumulative_environment_return: int = 0
    cumulative_subproblem_return: int = 0


@dataclass(slots=True)
class _Execution:
    invocation_id: str
    option_id: str
    start_update: int
    duration: int = 0
    environment_return: int = 0
    subproblem_return: int = 0


@dataclass(slots=True)
class _Pending:
    observation: State
    action: int
    events: list[dict[str, object]]
    acting_work: dict[str, object]
    state_allocation: dict[str, object]


class _OnlineSubproblemStage(SubproblemStage):
    """Online extension of the BRDC subproblem seam.

    The legacy ``pose`` method and its historical fixtures remain untouched.
    This stage constructs only from feature discovery events and updates only
    from observed transitions.
    """

    def __init__(self, features: OnlineFeatureStage, reward_mode: RewardMode) -> None:
        super().__init__(features)
        self._online_reward_mode: RewardMode = reward_mode
        self._online_records: dict[str, _Subproblem] = {}

    def construct(self, event: dict[str, object], update: int) -> _Subproblem | None:
        feature_id = _event_string(event, "artifact_id")
        revision_id = _event_string(event, "artifact_revision_id")
        subproblem_id = str(uuid.uuid5(uuid.UUID(feature_id), "online-subproblem-v1"))
        current = self._online_records.get(subproblem_id)
        if current is not None:
            if revision_id != current.feature_latest_revision_id:
                current.feature_latest_revision_id = revision_id
                current.feature_revision_count += 1
            return None
        record = _Subproblem(
            subproblem_id=subproblem_id,
            feature_id=feature_id,
            reward_mode=self._online_reward_mode,
            birth_update=update,
            feature_birth_revision_id=revision_id,
            feature_latest_revision_id=revision_id,
            feature_revision_count=1,
        )
        self._online_records[subproblem_id] = record
        return record

    def records(self) -> tuple[_Subproblem, ...]:
        return tuple(self._online_records[key] for key in sorted(self._online_records))

    def require(self, subproblem_id: str) -> _Subproblem:
        try:
            return self._online_records[subproblem_id]
        except KeyError as error:
            raise CycleError("ONLINE_SUBPROBLEM_UNKNOWN") from error


class _OnlineOptionStage(OptionStage):
    """Online extension that deliberately never calls the fixed ``solve`` policy."""

    def __init__(self, subproblems: _OnlineSubproblemStage) -> None:
        super().__init__(subproblems)
        self._online_records: dict[str, _Option] = {}

    def construct(self, subproblem: _Subproblem) -> _Option:
        option_id = str(uuid.uuid5(uuid.UUID(subproblem.subproblem_id), "online-option-v1"))
        if option_id in self._online_records:
            raise CycleError("ONLINE_OPTION_IDENTITY_INVALID")
        record = _Option(
            option_id=option_id,
            subproblem_id=subproblem.subproblem_id,
            birth_update=subproblem.birth_update,
            states={},
            q_values={},
            action_visits={},
            beta_counts={},
        )
        self._online_records[option_id] = record
        return record

    def records(self) -> tuple[_Option, ...]:
        return tuple(self._online_records[key] for key in sorted(self._online_records))

    def require(self, option_id: str) -> _Option:
        try:
            return self._online_records[option_id]
        except KeyError as error:
            raise CycleError("ONLINE_OPTION_UNKNOWN") from error


class _FeatureAdmission:
    """Translate the v2 class-bound grant into the BX-12 feature seam."""

    def __init__(self, admission: OptionAdmission) -> None:
        self._admission = admission

    def work(self, request_id: str, amount: int) -> dict[str, object]:
        return _request_work(self._admission, "feature_generation", request_id, amount)

    def allocation(
        self, request_id: str, allocated_bytes: int, serialized_bytes: int
    ) -> dict[str, object]:
        return _request_allocation(self._admission, request_id, allocated_bytes, serialized_bytes)


class OnlineOptionControl:
    """Finite one-pass primitive control plus learned options.

    The capacity arguments are explicit for configuration hashing, but v2
    freezes them to one interoperable profile.
    """

    def __init__(
        self,
        actions: int,
        width: int,
        seed: int,
        *,
        options_enabled: bool = True,
        reward_mode: RewardMode = "respecting",
        max_states: int = MAX_STATES,
        max_options: int = MAX_OPTIONS,
        max_duration: int = MAX_DURATION,
    ) -> None:
        if (
            type(actions) is not int
            or not 2 <= actions <= 8
            or type(width) is not int
            or not 1 <= width <= 8
            or type(seed) is not int
            or not 0 <= seed < 2**64
            or type(options_enabled) is not bool
            or reward_mode not in ("respecting", "oblivious")
            or type(max_states) is not int
            or max_states != MAX_STATES
            or type(max_options) is not int
            or max_options != MAX_OPTIONS
            or type(max_duration) is not int
            or max_duration != MAX_DURATION
        ):
            raise CycleError("ONLINE_OPTION_CONFIGURATION_INVALID")
        self._actions = actions
        self._width = width
        self._seed = seed
        self._options_enabled = options_enabled
        self._reward_mode: RewardMode = reward_mode
        self._max_states = max_states
        self._max_options = max_options
        self._max_duration = max_duration
        self._features = OnlineFeatureStage(
            width, seed, max_statistics=FEATURE_STATISTICS, max_features=FEATURES
        )
        self._subproblems = _OnlineSubproblemStage(self._features, reward_mode)
        self._options = _OnlineOptionStage(self._subproblems)
        self._primitive_states: dict[State, None] = {}
        self._primitive_counts: dict[StateAction, int] = {}
        self._primitive_returns: dict[StateAction, int] = {}
        self._active: _Execution | None = None
        self._pending: _Pending | None = None
        self._steps = 0
        self._updates = 0
        self._parameter_touches = 0
        self._work_by_class = {
            "acting": 0,
            "learning": 0,
            "feature_generation": 0,
            "option_learning": 0,
        }
        self._parameter_update = b""

    @property
    def actions(self) -> int:
        return self._actions

    @property
    def width(self) -> int:
        return self._width

    @property
    def seed(self) -> int:
        return self._seed

    @property
    def options_enabled(self) -> bool:
        return self._options_enabled

    @property
    def reward_mode(self) -> RewardMode:
        return self._reward_mode

    @property
    def max_states(self) -> int:
        return self._max_states

    @property
    def max_options(self) -> int:
        return self._max_options

    @property
    def max_duration(self) -> int:
        return self._max_duration

    @property
    def acting_tariff(self) -> int:
        return self.actions + MAX_OPTIONS

    @property
    def feature_generation_tariff(self) -> int:
        return 168 + 4 * self.width

    @property
    def option_learning_tariff(self) -> int:
        return 16 + MAX_OPTIONS * (8 + 2 * self.actions + 2 * self.width)

    @property
    def accounting(self) -> WorkAccounting:
        return WorkAccounting(
            environment_steps=self._steps,
            updates=self._updates,
            parameter_touches=self._parameter_touches,
            work_items=sum(self._work_by_class.values()),
            replay_items_retained=0,
        )

    @property
    def parameter_update(self) -> bytes:
        return self._parameter_update

    def reset_episode(self) -> None:
        if self._pending is not None:
            raise CycleError("ONLINE_OPTION_UPDATE_REQUIRED")
        if self._active is not None:
            raise CycleError("ONLINE_OPTION_EXECUTION_ACTIVE")

    def act(self, observation: State, admission: OptionAdmission) -> int:
        self._validate_observation(observation)
        if self._pending is not None:
            raise CycleError("ONLINE_OPTION_UPDATE_REQUIRED")
        if self._steps >= 2**32:
            raise CycleError("ONLINE_OPTION_STEP_LIMIT")

        request_id = f"option-acting-{self._steps}"
        work = _request_work(admission, "acting", request_id, self.acting_tariff)
        if work["outcome"] != "admit":
            raise CycleError(_reason(work, "ONLINE_OPTION_ACTING_DENIED"))

        prospective = copy.deepcopy(self)
        action, events = prospective._choose_action(observation)
        prospective._work_by_class["acting"] += prospective.acting_tariff
        prospective._pending = _Pending(
            observation=observation,
            action=action,
            events=events,
            acting_work=work,
            state_allocation={},
        )
        encoded = canonical(prospective.online_snapshot())
        allocated = prospective.allocated_bytes()
        prospective._enforce_state_limits(allocated, len(encoded))
        allocation = _request_allocation(
            admission, f"option-act-state-{self._steps}", allocated, len(encoded)
        )
        if allocation["outcome"] != "admit":
            raise CycleError(_reason(allocation, "ONLINE_OPTION_ALLOCATION_DENIED"))
        prospective._pending.state_allocation = allocation
        self.__dict__ = prospective.__dict__
        return action

    def observe(
        self,
        reward: int,
        next_observation: State,
        terminated: bool,
        truncated: bool,
        source_event_id: str,
        admission: OptionAdmission,
    ) -> dict[str, object]:
        self._validate_feedback(reward, next_observation, terminated, truncated)
        if self._pending is None:
            raise CycleError("ONLINE_OPTION_ACTION_REQUIRED")

        before = hashlib.sha256(canonical(self.online_snapshot())).hexdigest()
        learning = _request_work(admission, "learning", f"learning-{self._steps}", 1)
        if learning["outcome"] != "admit":
            raise CycleError(_reason(learning, "ONLINE_OPTION_LEARNING_DENIED"))

        prospective = copy.deepcopy(self)
        prior_option_ids = {record.option_id for record in prospective._options.records()}
        feature_before = prospective._features.online_snapshot()
        feature_result = prospective._features.observe(
            self._steps,
            next_observation,
            reward,
            source_event_id,
            _FeatureAdmission(admission),
        )
        if feature_result.get("accepted") is not True:
            raise CycleError(_reason(feature_result, "ONLINE_OPTION_FEATURE_DENIED"))

        option_work = _request_work(
            admission,
            "option_learning",
            f"option-learning-{self._steps}",
            self.option_learning_tariff,
        )
        if option_work["outcome"] != "admit":
            raise CycleError(_reason(option_work, "ONLINE_OPTION_LEARNING_DENIED"))

        pending = prospective._pending
        if pending is None:  # pragma: no cover - structural invariant
            raise CycleError("ONLINE_OPTION_INTERNAL_STATE_INVALID")
        primitive_delta, touches = prospective._update_primitive(pending, reward)
        feature_events = _feature_events(feature_result)
        feature_touches = _feature_parameter_touches(
            feature_before, prospective._features.online_snapshot(), feature_events
        )
        touches += feature_touches
        construction_events = prospective._apply_feature_events(feature_events)
        q_deltas: list[dict[str, object]] = []
        beta_deltas: list[dict[str, object]] = []
        subproblem_returns: list[dict[str, object]] = []
        for option in prospective._options.records():
            if option.option_id not in prior_option_ids:
                continue
            q_delta, beta_delta, subproblem_return = prospective._learn_option(
                option,
                pending,
                reward,
                next_observation,
                terminated,
                truncated,
            )
            if q_delta is not None:
                q_deltas.append(q_delta)
                beta_deltas.append(cast(dict[str, object], beta_delta))
                subproblem_returns.append(cast(dict[str, object], subproblem_return))
                touches += 4

        termination_event = prospective._finish_or_continue(
            reward, next_observation, terminated, truncated
        )
        execution_events = list(pending.events)
        if termination_event is not None:
            execution_events.append(termination_event)
        prospective._pending = None
        prospective._steps += 1
        prospective._updates += 1
        prospective._parameter_touches += touches
        prospective._work_by_class["learning"] += 1
        prospective._work_by_class["feature_generation"] += self.feature_generation_tariff
        prospective._work_by_class["option_learning"] += self.option_learning_tariff
        if touches > MAX_PARAMETER_TOUCHES_PER_UPDATE:
            raise CycleError("ONLINE_OPTION_TOUCH_LIMIT")

        encoded = canonical(prospective.online_snapshot())
        allocated = prospective.allocated_bytes()
        prospective._enforce_state_limits(allocated, len(encoded))
        allocation = _request_allocation(
            admission, f"option-state-{self._steps}", allocated, len(encoded)
        )
        if allocation["outcome"] != "admit":
            raise CycleError(_reason(allocation, "ONLINE_OPTION_ALLOCATION_DENIED"))
        after = hashlib.sha256(encoded).hexdigest()

        audit: dict[str, object] = {
            "schema": "bonsai.online-update/v2",
            "update": prospective._updates,
            "action": pending.action,
            "reward": reward,
            "work_by_class": dict(prospective._work_by_class),
            "parameter_touches": prospective._parameter_touches,
            "state_before": before,
            "state_after": after,
            "allocated_bytes": allocated,
            "serialized_bytes": len(encoded),
            "details": {
                "track": "track_a",
                "reward_mode": self.reward_mode,
                "reward_respecting_claim_eligible": self.reward_mode == "respecting",
                "options_enabled": self.options_enabled,
                "replay_items_retained": 0,
                "causal_transition": {
                    "observation": list(pending.observation),
                    "action": pending.action,
                    "reward": reward,
                    "next_observation": list(next_observation),
                    "terminated": terminated,
                    "truncated": truncated,
                    "source_event_id": source_event_id,
                },
                "primitive_delta": primitive_delta,
                "feature_parameter_touches": feature_touches,
                "feature_proposals": feature_events,
                "option_construction": construction_events,
                "option_q_deltas": q_deltas,
                "option_beta_deltas": beta_deltas,
                "option_execution_events": execution_events,
                "rewards": {
                    "environment": reward,
                    "subproblem_returns": subproblem_returns,
                },
                "admission": {
                    "acting": pending.acting_work,
                    "act_state": pending.state_allocation,
                    "learning": learning,
                    "feature_generation": cast(dict[str, object], feature_result["work"]),
                    "feature_state": cast(dict[str, object], feature_result["allocation"]),
                    "option_learning": option_work,
                    "option_state": allocation,
                },
                "capacity": {
                    "max_states": self.max_states,
                    "max_options": self.max_options,
                    "max_duration": self.max_duration,
                    "max_update_bytes": MAX_UPDATE_BYTES,
                },
            },
        }
        update_bytes = canonical(audit)
        if len(update_bytes) > MAX_UPDATE_BYTES:
            raise CycleError("ONLINE_OPTION_UPDATE_TOO_LARGE")
        prospective._parameter_update = update_bytes
        self.__dict__ = prospective.__dict__
        return audit

    def allocated_bytes(self) -> int:
        return _owned_learner_bytes(self)

    def online_snapshot(self) -> dict[str, object]:
        return {
            "schema": "bonsai.online-option-state/v2",
            "actions": self.actions,
            "width": self.width,
            "seed": self.seed,
            "options_enabled": self.options_enabled,
            "reward_mode": self.reward_mode,
            "max_states": self.max_states,
            "max_options": self.max_options,
            "max_duration": self.max_duration,
            "next_update": self._updates,
            "environment_steps": self._steps,
            "work_by_class": dict(self._work_by_class),
            "parameter_touches": self._parameter_touches,
            "features": self._features.online_snapshot(),
            "subproblems": [self._subproblem_snapshot(record) for record in self._subproblems.records()],
            "options": [self._option_snapshot(record) for record in self._options.records()],
            "primitive": [
                {
                    "observation": list(key[0]),
                    "action": key[1],
                    "count": self._primitive_counts[key],
                    "return_sum": self._primitive_returns[key],
                }
                for key in sorted(self._primitive_counts)
            ],
            "active_execution": None if self._active is None else self._execution_snapshot(self._active),
            "pending": None if self._pending is None else self._pending_snapshot(self._pending),
            "replay_items_retained": 0,
        }

    def _choose_action(self, observation: State) -> tuple[int, list[dict[str, object]]]:
        events: list[dict[str, object]] = []
        if self._active is None and self.options_enabled:
            eligible = [record for record in self._options.records() if self._executable(record)]
            for option in eligible:
                if self._learned_beta(option, observation):
                    continue
                invocation_id = str(
                    uuid.uuid5(
                        uuid.NAMESPACE_URL,
                        f"bonsai-option-invocation:{self.seed}:{option.option_id}:{option.executions + 1}",
                    )
                )
                option.executions += 1
                self._active = _Execution(invocation_id, option.option_id, self._steps)
                events.append(
                    {
                        "kind": "initiate",
                        "invocation_id": invocation_id,
                        "option_id": option.option_id,
                        "update": self._steps + 1,
                        "reason": "learned_subproblem_ready",
                    }
                )
                break

        if self._active is None:
            action = self._primitive_action(observation)
            events.append(
                {
                    "kind": "action",
                    "invocation_id": None,
                    "option_id": None,
                    "action": action,
                    "duration": 1,
                    "reason": "primitive_control",
                }
            )
            return action, events

        option = self._options.require(self._active.option_id)
        action = self._option_action(option, observation)
        self._active.duration += 1
        events.append(
            {
                "kind": "action",
                "invocation_id": self._active.invocation_id,
                "option_id": option.option_id,
                "action": action,
                "duration": self._active.duration,
                "reason": "learned_option_policy",
            }
        )
        return action, events

    def _update_primitive(self, pending: _Pending, reward: int) -> tuple[dict[str, object], int]:
        if pending.observation not in self._primitive_states:
            if len(self._primitive_states) >= self.max_states:
                return (
                    {
                        "observation": list(pending.observation),
                        "action": pending.action,
                        "applied": False,
                        "reason": "state_capacity",
                        "rule": "integer-sample-average-sufficient-statistics",
                    },
                    0,
                )
            self._primitive_states[pending.observation] = None
        key = (pending.observation, pending.action)
        before = [self._primitive_counts.get(key, 0), self._primitive_returns.get(key, 0)]
        after = [before[0] + 1, _checked_add(before[1], reward)]
        self._primitive_counts[key] = after[0]
        self._primitive_returns[key] = after[1]
        return (
            {
                "observation": list(pending.observation),
                "action": pending.action,
                "applied": True,
                "before": before,
                "after": after,
                "rule": "integer-sample-average-sufficient-statistics",
            },
            2,
        )

    def _apply_feature_events(self, events: list[dict[str, object]]) -> list[dict[str, object]]:
        changes: list[dict[str, object]] = []
        for event in events:
            feature_id = _event_string(event, "artifact_id")
            revision_id = _event_string(event, "artifact_revision_id")
            existing = next(
                (
                    record
                    for record in self._subproblems.records()
                    if record.feature_id == feature_id
                ),
                None,
            )
            created = self._subproblems.construct(event, self._updates)
            if created is not None and len(self._options.records()) < self.max_options:
                option = self._options.construct(created)
                changes.append(
                    {
                        "kind": "construction",
                        "feature_id": feature_id,
                        "feature_revision_id": revision_id,
                        "subproblem_id": created.subproblem_id,
                        "option_id": option.option_id,
                        "parents": [revision_id],
                    }
                )
            elif existing is not None:
                changes.append(
                    {
                        "kind": "maintenance",
                        "feature_id": feature_id,
                        "feature_revision_id": revision_id,
                        "subproblem_id": existing.subproblem_id,
                        "option_id": next(
                            (
                                option.option_id
                                for option in self._options.records()
                                if option.subproblem_id == existing.subproblem_id
                            ),
                            None,
                        ),
                        "parents": [existing.feature_latest_revision_id],
                    }
                )
        return changes

    def _learn_option(
        self,
        option: _Option,
        pending: _Pending,
        reward: int,
        next_observation: State,
        terminated: bool,
        truncated: bool,
    ) -> tuple[dict[str, object] | None, dict[str, object] | None, dict[str, object] | None]:
        if not self._admit_states(option, pending.observation, next_observation):
            return None, None, None
        subproblem = self._subproblems.require(option.subproblem_id)
        attained = self._feature_attained(subproblem.feature_id, next_observation)
        original_term = reward if self.reward_mode == "respecting" else 0
        stopping_term = subproblem.stopping_bonus if attained else 0
        subproblem_return = _checked_add(original_term, stopping_term)
        key = (pending.observation, pending.action)
        q_before = option.q_values.get(key, 0)
        future = 0
        if not attained and not terminated and not truncated:
            future = max(option.q_values.get((next_observation, action), 0) for action in range(self.actions))
        q_after = _checked_q(_checked_add(subproblem_return, future))
        visits_before = option.action_visits.get(key, 0)
        option.q_values[key] = q_after
        option.action_visits[key] = visits_before + 1
        option.q_updates += 1

        beta_before = option.beta_counts.get(next_observation, (0, 0))
        beta_after = (beta_before[0] + int(attained), beta_before[1] + 1)
        option.beta_counts[next_observation] = beta_after
        option.beta_updates += 1

        subproblem.updates += 1
        subproblem.attainments += int(attained)
        subproblem.reward_aligned_successes += int(attained and reward > 0)
        subproblem.cumulative_environment_reward = _checked_add(
            subproblem.cumulative_environment_reward, reward
        )
        subproblem.cumulative_subproblem_return = _checked_add(
            subproblem.cumulative_subproblem_return, subproblem_return
        )
        option.cumulative_subproblem_return = _checked_add(
            option.cumulative_subproblem_return, subproblem_return
        )
        return (
            {
                "option_id": option.option_id,
                "observation": list(pending.observation),
                "action": pending.action,
                "before": q_before,
                "after": q_after,
                "visits_before": visits_before,
                "visits_after": visits_before + 1,
                "future": future,
                "rule": "integer-q-learning-alpha-one-gamma-one",
            },
            {
                "option_id": option.option_id,
                "observation": list(next_observation),
                "before": list(beta_before),
                "after": list(beta_after),
                "minimum_support": 2,
                "threshold": "positive_count*2>=total_count",
                "terminates": self._learned_beta(option, next_observation),
            },
            {
                "option_id": option.option_id,
                "environment_reward": reward,
                "original_reward_term": original_term,
                "stopping_bonus_term": stopping_term,
                "return": subproblem_return,
                "attained": attained,
            },
        )

    def _finish_or_continue(
        self, reward: int, next_observation: State, terminated: bool, truncated: bool
    ) -> dict[str, object] | None:
        if self._active is None:
            return None
        execution = self._active
        option = self._options.require(execution.option_id)
        subproblem = self._subproblems.require(option.subproblem_id)
        attained = self._feature_attained(subproblem.feature_id, next_observation)
        original_term = reward if self.reward_mode == "respecting" else 0
        subproblem_return = _checked_add(
            original_term, subproblem.stopping_bonus if attained else 0
        )
        execution.environment_return = _checked_add(execution.environment_return, reward)
        execution.subproblem_return = _checked_add(execution.subproblem_return, subproblem_return)
        reason: str | None = None
        if terminated:
            reason = "environment_terminated"
        elif truncated:
            reason = "environment_truncated"
        elif self._learned_beta(option, next_observation):
            reason = "learned_beta"
        elif execution.duration >= self.max_duration:
            reason = "duration_cap"
        if reason is None:
            return {
                "kind": "continue",
                "invocation_id": execution.invocation_id,
                "option_id": option.option_id,
                "duration": execution.duration,
                "reason": "learned_beta_continues",
            }
        option.completed_executions += 1
        option.successful_executions += int(reason == "learned_beta")
        option.cumulative_duration += execution.duration
        option.cumulative_environment_return = _checked_add(
            option.cumulative_environment_return, execution.environment_return
        )
        self._active = None
        return {
            "kind": "terminate",
            "invocation_id": execution.invocation_id,
            "option_id": option.option_id,
            "duration": execution.duration,
            "reason": reason,
            "environment_return": execution.environment_return,
            "subproblem_return": execution.subproblem_return,
            "attained": attained,
        }

    def _primitive_action(self, observation: State) -> int:
        if observation not in self._primitive_states and len(self._primitive_states) >= self.max_states:
            return hashlib.sha256(canonical([self.seed, observation])).digest()[0] % self.actions
        unseen = [
            action for action in range(self.actions) if (observation, action) not in self._primitive_counts
        ]
        if unseen:
            return unseen[0]
        total = sum(self._primitive_counts[(observation, action)] for action in range(self.actions))
        if total % (self.actions * 4) == 0:
            return min(
                range(self.actions),
                key=lambda action: (self._primitive_counts[(observation, action)], action),
            )
        return self._best_average_action(observation)

    def _best_average_action(self, observation: State) -> int:
        best = 0
        for action in range(1, self.actions):
            left = (observation, action)
            right = (observation, best)
            if (
                self._primitive_returns[left] * self._primitive_counts[right]
                > self._primitive_returns[right] * self._primitive_counts[left]
            ):
                best = action
        return best

    def _option_action(self, option: _Option, observation: State) -> int:
        unseen = [
            action for action in range(self.actions) if (observation, action) not in option.action_visits
        ]
        if unseen:
            return unseen[0]
        total = sum(option.action_visits[(observation, action)] for action in range(self.actions))
        if total % (self.actions * 4) == 0:
            return min(
                range(self.actions),
                key=lambda action: (option.action_visits[(observation, action)], action),
            )
        return min(
            range(self.actions),
            key=lambda action: (-option.q_values.get((observation, action), 0), action),
        )

    def _executable(self, option: _Option) -> bool:
        subproblem = self._subproblems.require(option.subproblem_id)
        return (
            subproblem.attainments > 0
            and option.q_updates >= self.actions
            and option.beta_updates >= 1
        )

    @staticmethod
    def _learned_beta(option: _Option, observation: State) -> bool:
        positive, total = option.beta_counts.get(observation, (0, 0))
        return total >= 2 and positive * 2 >= total

    def _feature_attained(self, feature_id: str, observation: State) -> bool:
        feature = next(
            (record for record in self._features.snapshot() if record.feature_id == feature_id),
            None,
        )
        if feature is None or len(feature.representation) < 3:
            raise CycleError("ONLINE_OPTION_FEATURE_UNKNOWN")
        coordinate = feature.representation[1]
        return observation[coordinate] == feature.representation[2]

    def _admit_states(self, option: _Option, *states: State) -> bool:
        missing = [state for state in states if state not in option.states]
        if len(option.states) + len(dict.fromkeys(missing)) > self.max_states:
            return False
        for state in missing:
            option.states[state] = None
        return True

    def _validate_observation(self, observation: State) -> None:
        if (
            type(observation) is not tuple
            or len(observation) != self.width
            or any(type(value) is not int or not 0 <= value < 2**31 for value in observation)
        ):
            raise CycleError("ONLINE_OPTION_OBSERVATION_INVALID")

    def _validate_feedback(
        self, reward: int, observation: State, terminated: bool, truncated: bool
    ) -> None:
        self._validate_observation(observation)
        if (
            type(reward) is not int
            or not -(2**31) <= reward < 2**31
            or type(terminated) is not bool
            or type(truncated) is not bool
            or (terminated and truncated)
        ):
            raise CycleError("ONLINE_OPTION_FEEDBACK_INVALID")

    @staticmethod
    def _subproblem_snapshot(record: _Subproblem) -> dict[str, object]:
        return {
            "subproblem_id": record.subproblem_id,
            "feature_id": record.feature_id,
            "reward_mode": record.reward_mode,
            "birth_update": record.birth_update,
            "stopping_bonus": record.stopping_bonus,
            "feature_birth_revision_id": record.feature_birth_revision_id,
            "feature_latest_revision_id": record.feature_latest_revision_id,
            "feature_revision_count": record.feature_revision_count,
            "updates": record.updates,
            "attainments": record.attainments,
            "reward_aligned_successes": record.reward_aligned_successes,
            "cumulative_environment_reward": record.cumulative_environment_reward,
            "cumulative_subproblem_return": record.cumulative_subproblem_return,
        }

    @staticmethod
    def _option_snapshot(record: _Option) -> dict[str, object]:
        return {
            "option_id": record.option_id,
            "subproblem_id": record.subproblem_id,
            "birth_update": record.birth_update,
            "states": [list(state) for state in sorted(record.states)],
            "q_values": [
                {
                    "observation": list(key[0]),
                    "action": key[1],
                    "value": record.q_values[key],
                    "visits": record.action_visits[key],
                }
                for key in sorted(record.q_values)
            ],
            "beta": [
                {
                    "observation": list(state),
                    "positive": record.beta_counts[state][0],
                    "total": record.beta_counts[state][1],
                }
                for state in sorted(record.beta_counts)
            ],
            "q_updates": record.q_updates,
            "beta_updates": record.beta_updates,
            "executions": record.executions,
            "completed_executions": record.completed_executions,
            "successful_executions": record.successful_executions,
            "cumulative_duration": record.cumulative_duration,
            "cumulative_environment_return": record.cumulative_environment_return,
            "cumulative_subproblem_return": record.cumulative_subproblem_return,
        }

    @staticmethod
    def _execution_snapshot(execution: _Execution) -> dict[str, object]:
        return {
            "invocation_id": execution.invocation_id,
            "option_id": execution.option_id,
            "start_update": execution.start_update,
            "duration": execution.duration,
            "environment_return": execution.environment_return,
            "subproblem_return": execution.subproblem_return,
        }

    @staticmethod
    def _pending_snapshot(pending: _Pending) -> dict[str, object]:
        return {
            "observation": list(pending.observation),
            "action": pending.action,
            "events": copy.deepcopy(pending.events),
        }

    @staticmethod
    def _enforce_state_limits(allocated: int, serialized: int) -> None:
        if allocated > MAX_RETAINED_BYTES or serialized > MAX_SERIALIZED_BYTES:
            raise CycleError("ONLINE_OPTION_STATE_LIMIT")


def _feature_events(result: dict[str, object]) -> list[dict[str, object]]:
    value: object = result.get("events")
    if not isinstance(value, list):
        raise CycleError("ONLINE_OPTION_FEATURE_RESULT_INVALID")
    raw_events = cast(list[object], value)
    if any(not isinstance(item, dict) for item in raw_events):
        raise CycleError("ONLINE_OPTION_FEATURE_RESULT_INVALID")
    return [cast(dict[str, object], item) for item in raw_events]


def _event_string(event: dict[str, object], key: str) -> str:
    value = event.get(key)
    if not isinstance(value, str) or not value:
        raise CycleError("ONLINE_OPTION_FEATURE_RESULT_INVALID")
    return value


def _feature_parameter_touches(
    before: dict[str, object],
    after: dict[str, object],
    events: list[dict[str, object]],
) -> int:
    before_statistics = _feature_statistics(before)
    after_statistics = _feature_statistics(after)
    exposure_touches = 2 * sum(
        before_statistics.get(key) != value for key, value in after_statistics.items()
    )
    artifact_touches = 0
    for event in events:
        parameter_count = event.get("parameter_count")
        if type(parameter_count) is not int or not 0 <= parameter_count <= 16:
            raise CycleError("ONLINE_OPTION_FEATURE_RESULT_INVALID")
        artifact_touches += parameter_count
    return exposure_touches + artifact_touches


def _feature_statistics(snapshot: dict[str, object]) -> dict[tuple[int, int], tuple[int, int]]:
    raw = snapshot.get("statistics")
    if not isinstance(raw, list):
        raise CycleError("ONLINE_OPTION_FEATURE_RESULT_INVALID")
    result: dict[tuple[int, int], tuple[int, int]] = {}
    for value in cast(list[object], raw):
        if not isinstance(value, dict):
            raise CycleError("ONLINE_OPTION_FEATURE_RESULT_INVALID")
        statistic = cast(dict[str, object], value)
        coordinate = statistic.get("coordinate")
        observed = statistic.get("value")
        count = statistic.get("count")
        reward_sum = statistic.get("reward_sum")
        if any(type(item) is not int for item in (coordinate, observed, count, reward_sum)):
            raise CycleError("ONLINE_OPTION_FEATURE_RESULT_INVALID")
        result[(cast(int, coordinate), cast(int, observed))] = (
            cast(int, count),
            cast(int, reward_sum),
        )
    return result


def _checked_add(left: int, right: int) -> int:
    result = left + right
    if not -(2**63) <= result < 2**63:
        raise CycleError("ONLINE_OPTION_INTEGER_OVERFLOW")
    return result


def _checked_q(value: int) -> int:
    if not -Q_LIMIT <= value <= Q_LIMIT:
        raise CycleError("ONLINE_OPTION_Q_LIMIT")
    return value


def _request_work(
    admission: OptionAdmission, work_class: str, request_id: str, amount: int
) -> dict[str, object]:
    raw = admission.work(work_class, request_id, amount)
    admitted = _admission_outcome(raw)
    return {
        "work_class": work_class,
        "request_id": request_id,
        "requested": amount,
        "accepted": admitted,
        "outcome": "admit" if admitted else "reject",
        "reason_code": raw.get("reason_code"),
    }


def _request_allocation(
    admission: OptionAdmission, request_id: str, allocated_bytes: int, serialized_bytes: int
) -> dict[str, object]:
    raw = admission.allocation(request_id, allocated_bytes, serialized_bytes)
    admitted = _admission_outcome(raw)
    return {
        "request_id": request_id,
        "allocated_bytes": allocated_bytes,
        "serialized_bytes": serialized_bytes,
        "accepted": admitted,
        "outcome": "admit" if admitted else "reject",
        "reason_code": raw.get("reason_code"),
    }


def _admission_outcome(report: dict[str, object]) -> bool:
    accepted = report.get("accepted")
    outcome = report.get("outcome")
    admitted_by_outcome = isinstance(outcome, str) and outcome.lower() in {
        "admit",
        "admitted",
        "measured",
    }
    rejected_by_outcome = isinstance(outcome, str) and outcome.lower() in {
        "reject",
        "rejected",
        "deny",
        "denied",
    }
    if accepted is not None and type(accepted) is not bool:
        raise CycleError("ONLINE_OPTION_ADMISSION_REPORT_INVALID")
    if accepted is True and rejected_by_outcome:
        raise CycleError("ONLINE_OPTION_ADMISSION_REPORT_INVALID")
    if accepted is False and admitted_by_outcome:
        raise CycleError("ONLINE_OPTION_ADMISSION_REPORT_INVALID")
    if accepted is True or admitted_by_outcome:
        return True
    if accepted is False or rejected_by_outcome:
        return False
    raise CycleError("ONLINE_OPTION_ADMISSION_REPORT_INVALID")


def _reason(report: dict[str, object], fallback: str) -> str:
    reason = report.get("reason_code")
    return reason if isinstance(reason, str) and reason else fallback


def _owned_learner_bytes(control: OnlineOptionControl) -> int:
    """Measure the unique Python object graph owned by the learner.

    The last parameter-update response and the two admitted act reports are
    bounded protocol caches, not learned state. Their parent slots and mapping
    capacity remain counted, while their child graphs are accounted by their
    separate wire-size limits.
    """

    seen: set[int] = set()

    def visit(item: object, *, control_root: bool = False) -> int:
        identity = id(item)
        if identity in seen:
            return 0
        seen.add(identity)
        size = sys.getsizeof(item)
        if isinstance(item, dict):
            values = cast(dict[object, object], item)
            return size + sum(visit(key) + visit(value) for key, value in values.items())
        if isinstance(item, (tuple, list, set, frozenset)):
            values = cast(tuple[object, ...] | list[object] | set[object] | frozenset[object], item)
            return size + sum(visit(value) for value in values)
        if is_dataclass(item) and not isinstance(item, type):
            total = size
            for record_field in fields(item):
                if isinstance(item, _Pending) and record_field.name in {
                    "acting_work",
                    "state_allocation",
                }:
                    continue
                total += visit(getattr(item, record_field.name))
            return total
        if hasattr(item, "__dict__"):
            values = cast(dict[str, object], vars(item))
            total = size + sys.getsizeof(values)
            for key, value in values.items():
                if control_root and key == "_parameter_update":
                    continue
                total += visit(key) + visit(value)
            return total
        return size

    return visit(control, control_root=True)
