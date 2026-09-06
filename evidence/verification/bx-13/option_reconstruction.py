"""Independent finite-state reconstruction for BX-13 online option evidence.

This module deliberately uses only the Python standard library.  It does not
import the learner, feature stage, adapter, or their historical fixtures.
"""

from __future__ import annotations

import copy
import hashlib
import json
import uuid
from dataclasses import dataclass
from typing import Any, cast

MAX_STATES = 64
MAX_OPTIONS = 4
MAX_DURATION = 16
MAX_UPDATE_BYTES = 32 * 1024
MAX_RETAINED_BYTES = 1024 * 1024
MAX_SERIALIZED_BYTES = 256 * 1024
MAX_PARAMETER_TOUCHES = 1024
FEATURE_STATISTICS = 32
FEATURES = 4
STOPPING_BONUS = 1
Q_LIMIT = 2**31 - 1


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def digest(value: object) -> str:
    return hashlib.sha256(canonical(value)).hexdigest()


def feedback_event_id(step: int) -> str:
    value = bytearray([1] * 16)
    value[:8] = step.to_bytes(8, "little")
    return str(uuid.UUID(bytes=bytes(value)))


def _checked_add(left: int, right: int) -> int:
    result = left + right
    assert -(2**63) <= result < 2**63
    return result


def _checked_q(value: int) -> int:
    assert -Q_LIMIT <= value <= Q_LIMIT
    return value


@dataclass
class Execution:
    invocation_id: str
    option_id: str
    start_update: int
    duration: int = 0
    environment_return: int = 0
    subproblem_return: int = 0


class Reconstructor:
    """Rebuild the bounded v2 state from original public transitions."""

    def __init__(self, config: dict[str, Any]) -> None:
        assert set(config) == {
            "action_count",
            "observation_width",
            "seed",
            "options_enabled",
            "reward_mode",
            "retained_state_limit_bytes",
            "serialized_state_limit_bytes",
        }
        self.actions = config["action_count"]
        self.width = config["observation_width"]
        self.seed = config["seed"]
        self.options_enabled = config["options_enabled"]
        self.reward_mode = config["reward_mode"]
        self.retained_limit = config["retained_state_limit_bytes"]
        self.serialized_limit = config["serialized_state_limit_bytes"]
        assert type(self.actions) is int and 2 <= self.actions <= 8
        assert type(self.width) is int and 1 <= self.width <= 8
        assert type(self.seed) is int and 0 <= self.seed < 2**64
        assert type(self.options_enabled) is bool
        assert self.reward_mode in {"respecting", "oblivious"}
        assert type(self.retained_limit) is int and 0 < self.retained_limit <= 16 * 1024 * 1024
        assert type(self.serialized_limit) is int and 0 < self.serialized_limit <= 1024 * 1024

        self.feature_step = 0
        self.feature_work = 0
        self.statistics: dict[tuple[int, int], dict[str, Any]] = {}
        self.features: dict[str, dict[str, Any]] = {}
        self.versions: dict[str, list[Any]] = {}
        self.subproblems: dict[str, dict[str, Any]] = {}
        self.options: dict[str, dict[str, Any]] = {}
        self.primitive_states: set[tuple[int, ...]] = set()
        self.primitive_counts: dict[tuple[tuple[int, ...], int], int] = {}
        self.primitive_returns: dict[tuple[tuple[int, ...], int], int] = {}
        self.active: Execution | None = None
        self.pending: dict[str, Any] | None = None
        self.steps = 0
        self.updates = 0
        self.parameter_touches = 0
        self.work_by_class = {key: 0 for key in self.tariffs()}
        self.execution_events: list[dict[str, Any]] = []
        self.feature_events: list[dict[str, Any]] = []
        self.act_serialized_bytes = 0
        self.curve_environment_reward = 0
        self.learning_curve: list[dict[str, Any]] = []

    def tariffs(self) -> dict[str, int]:
        return {
            "acting": self.actions + MAX_OPTIONS,
            "learning": 1,
            "feature_generation": 168 + 4 * self.width,
            "option_learning": 16 + MAX_OPTIONS * (8 + 2 * self.actions + 2 * self.width),
        }

    def feature_snapshot(self) -> dict[str, Any]:
        return {
            "schema": "bonsai.online-feature-state/v1",
            "width": self.width,
            "seed": self.seed,
            "enabled": True,
            "max_statistics": FEATURE_STATISTICS,
            "max_features": FEATURES,
            "next_step": self.feature_step,
            "feature_work": self.feature_work,
            "features": [copy.deepcopy(self.features[key]) for key in sorted(self.features)],
            "statistics": [
                {"coordinate": key[0], "value": key[1], **copy.deepcopy(self.statistics[key])}
                for key in sorted(self.statistics)
            ],
            "versions": copy.deepcopy(self.versions),
            "replay_items_retained": 0,
        }

    def snapshot(self) -> dict[str, Any]:
        return {
            "schema": "bonsai.online-option-state/v2",
            "actions": self.actions,
            "width": self.width,
            "seed": self.seed,
            "options_enabled": self.options_enabled,
            "reward_mode": self.reward_mode,
            "max_states": MAX_STATES,
            "max_options": MAX_OPTIONS,
            "max_duration": MAX_DURATION,
            "next_update": self.updates,
            "environment_steps": self.steps,
            "work_by_class": dict(self.work_by_class),
            "parameter_touches": self.parameter_touches,
            "features": self.feature_snapshot(),
            "subproblems": [copy.deepcopy(self.subproblems[key]) for key in sorted(self.subproblems)],
            "options": [self._option_snapshot(self.options[key]) for key in sorted(self.options)],
            "primitive": [
                {
                    "observation": list(key[0]),
                    "action": key[1],
                    "count": self.primitive_counts[key],
                    "return_sum": self.primitive_returns[key],
                }
                for key in sorted(self.primitive_counts)
            ],
            "active_execution": self.execution_snapshot(self.active),
            "pending": copy.deepcopy(self.pending),
            "replay_items_retained": 0,
        }

    @staticmethod
    def execution_snapshot(execution: Execution | None) -> dict[str, Any] | None:
        if execution is None:
            return None
        return {
            "invocation_id": execution.invocation_id,
            "option_id": execution.option_id,
            "start_update": execution.start_update,
            "duration": execution.duration,
            "environment_return": execution.environment_return,
            "subproblem_return": execution.subproblem_return,
        }

    @staticmethod
    def _option_snapshot(option: dict[str, Any]) -> dict[str, Any]:
        q_values = option["q_values"]
        visits = option["action_visits"]
        beta = option["beta_counts"]
        return {
            "option_id": option["option_id"],
            "subproblem_id": option["subproblem_id"],
            "birth_update": option["birth_update"],
            "states": [list(state) for state in sorted(option["states"])],
            "q_values": [
                {
                    "observation": list(key[0]),
                    "action": key[1],
                    "value": q_values[key],
                    "visits": visits[key],
                }
                for key in sorted(q_values)
            ],
            "beta": [
                {
                    "observation": list(state),
                    "positive": beta[state][0],
                    "total": beta[state][1],
                }
                for state in sorted(beta)
            ],
            "q_updates": option["q_updates"],
            "beta_updates": option["beta_updates"],
            "executions": option["executions"],
            "completed_executions": option["completed_executions"],
            "successful_executions": option["successful_executions"],
            "cumulative_duration": option["cumulative_duration"],
            "cumulative_environment_return": option["cumulative_environment_return"],
            "cumulative_subproblem_return": option["cumulative_subproblem_return"],
        }

    def _feature_attained(self, feature_id: str, observation: tuple[int, ...]) -> bool:
        representation = cast(list[int], self.features[feature_id]["representation"])
        return observation[representation[1]] == representation[2]

    @staticmethod
    def _learned_beta(option: dict[str, Any], observation: tuple[int, ...]) -> bool:
        positive, total = option["beta_counts"].get(observation, (0, 0))
        return total >= 2 and positive * 2 >= total

    def _executable(self, option: dict[str, Any]) -> bool:
        subproblem = self.subproblems[option["subproblem_id"]]
        return (
            subproblem["attainments"] > 0
            and option["q_updates"] >= self.actions
            and option["beta_updates"] >= 1
        )

    def _primitive_action(self, observation: tuple[int, ...]) -> int:
        if observation not in self.primitive_states and len(self.primitive_states) >= MAX_STATES:
            return hashlib.sha256(canonical([self.seed, observation])).digest()[0] % self.actions
        unseen = [
            action
            for action in range(self.actions)
            if (observation, action) not in self.primitive_counts
        ]
        if unseen:
            return unseen[0]
        total = sum(self.primitive_counts[(observation, action)] for action in range(self.actions))
        if total % (self.actions * 4) == 0:
            return min(
                range(self.actions),
                key=lambda action: (self.primitive_counts[(observation, action)], action),
            )
        best = 0
        for action in range(1, self.actions):
            left, right = (observation, action), (observation, best)
            if (
                self.primitive_returns[left] * self.primitive_counts[right]
                > self.primitive_returns[right] * self.primitive_counts[left]
            ):
                best = action
        return best

    def _option_action(self, option: dict[str, Any], observation: tuple[int, ...]) -> int:
        unseen = [
            action
            for action in range(self.actions)
            if (observation, action) not in option["action_visits"]
        ]
        if unseen:
            return unseen[0]
        total = sum(option["action_visits"][(observation, action)] for action in range(self.actions))
        if total % (self.actions * 4) == 0:
            return min(
                range(self.actions),
                key=lambda action: (option["action_visits"][(observation, action)], action),
            )
        return min(
            range(self.actions),
            key=lambda action: (-option["q_values"].get((observation, action), 0), action),
        )

    def begin(self, observation: tuple[int, ...], action: int) -> str:
        assert len(observation) == self.width
        assert all(type(value) is int and 0 <= value < 2**31 for value in observation)
        assert self.pending is None and self.steps < 2**32
        events: list[dict[str, Any]] = []
        if self.active is None and self.options_enabled:
            for option_id in sorted(self.options):
                option = self.options[option_id]
                if not self._executable(option) or self._learned_beta(option, observation):
                    continue
                invocation_id = str(
                    uuid.uuid5(
                        uuid.NAMESPACE_URL,
                        f"bonsai-option-invocation:{self.seed}:{option_id}:{option['executions'] + 1}",
                    )
                )
                option["executions"] += 1
                self.active = Execution(invocation_id, option_id, self.steps)
                events.append(
                    {
                        "kind": "initiate",
                        "invocation_id": invocation_id,
                        "option_id": option_id,
                        "update": self.steps + 1,
                        "reason": "learned_subproblem_ready",
                    }
                )
                break
        if self.active is None:
            selected = self._primitive_action(observation)
            events.append(
                {
                    "kind": "action",
                    "invocation_id": None,
                    "option_id": None,
                    "action": selected,
                    "duration": 1,
                    "reason": "primitive_control",
                }
            )
        else:
            option = self.options[self.active.option_id]
            selected = self._option_action(option, observation)
            self.active.duration += 1
            events.append(
                {
                    "kind": "action",
                    "invocation_id": self.active.invocation_id,
                    "option_id": self.active.option_id,
                    "action": selected,
                    "duration": self.active.duration,
                    "reason": "learned_option_policy",
                }
            )
        assert selected == action
        self.work_by_class["acting"] += self.tariffs()["acting"]
        self.pending = {"observation": list(observation), "action": action, "events": events}
        snapshot = self.snapshot()
        self.act_serialized_bytes = len(canonical(snapshot))
        return digest(snapshot)

    def _update_feature(
        self, observation: tuple[int, ...], reward: int, source_event_id: str
    ) -> list[dict[str, Any]]:
        eligible: list[tuple[int, int]] = []
        for coordinate, value in enumerate(observation):
            key = (coordinate, value)
            prior = self.statistics.get(key)
            if prior is None:
                if len(self.statistics) >= FEATURE_STATISTICS:
                    continue
                current: dict[str, Any] = {
                    "count": 1,
                    "reward_sum": reward,
                    "first_step": self.feature_step,
                    "last_step": self.feature_step,
                    "first_event_id": source_event_id,
                    "last_event_id": source_event_id,
                }
            else:
                current = {
                    **prior,
                    "count": prior["count"] + 1,
                    "reward_sum": prior["reward_sum"] + reward,
                    "last_step": self.feature_step,
                    "last_event_id": source_event_id,
                }
            self.statistics[key] = current
            count, reward_sum = current["count"], current["reward_sum"]
            assert type(count) is int and type(reward_sum) is int
            if reward_sum > 0 and (count == 2 or count % 4 == 0):
                eligible.append(key)
        eligible.sort(key=lambda key: hashlib.sha256(canonical([self.seed, self.feature_step, key])).digest())
        events: list[dict[str, Any]] = []
        for key in eligible:
            feature_id = str(
                uuid.uuid5(uuid.NAMESPACE_URL, f"bonsai-feature:{self.seed}:{key[0]}:{key[1]}")
            )
            if feature_id not in self.versions and len(self.features) >= FEATURES:
                continue
            events = [self._feature_candidate(feature_id, key)]
            break
        self.feature_step += 1
        self.feature_events.extend(copy.deepcopy(events))
        return events

    def _feature_candidate(self, feature_id: str, key: tuple[int, int]) -> dict[str, Any]:
        exposure = self.statistics[key]
        representation = [0, key[0], key[1], exposure["reward_sum"], exposure["count"]]
        old = self.versions.get(feature_id)
        version = 1 if old is None else old[0] + 1
        revision_id = str(uuid.uuid5(uuid.UUID(feature_id), str(version)))
        sequence = 1 if old is None else old[2] + 1
        if old is None:
            self.features[feature_id] = {
                "feature_id": feature_id,
                "representation": representation,
                "birth_step": self.feature_step,
                "retirement_step": None,
                "bytes": len(canonical(representation)),
                "work": 2,
                "consumers": 0,
                "utility": None,
                "parents": [],
            }
            self.feature_work += 2
        else:
            record = self.features[feature_id]
            record["representation"] = representation
            record["bytes"] = len(canonical(representation))
            record["work"] += 1
            record["utility"] = None
            self.feature_work += 1
        self.versions[feature_id] = [version, revision_id, sequence + 1]
        return {
            "artifact_id": feature_id,
            "artifact_revision_id": revision_id,
            "lifecycle_sequence": sequence,
            "kind": "birth" if old is None else "revision",
            "previous_revision_id": None if old is None else old[1],
            "representation": representation,
            "representation_sha256": digest(representation),
            "parents": [],
            "exposure": copy.deepcopy(exposure),
            "provenance": {
                "producer_id": "bonsai-online-features",
                "producer_version": "1.0.0",
                "source_event_ids": sorted(
                    {exposure["first_event_id"], exposure["last_event_id"]}
                ),
                "method_ids": ["observed-equality-positive-support-v1"],
            },
            "utility": None,
            "parameter_count": 2,
            "representation_serialized_bytes": len(canonical(representation)),
        }

    def _apply_feature_events(self, events: list[dict[str, Any]]) -> list[dict[str, Any]]:
        changes: list[dict[str, Any]] = []
        for event in events:
            feature_id = event["artifact_id"]
            revision_id = event["artifact_revision_id"]
            subproblem_id = str(uuid.uuid5(uuid.UUID(feature_id), "online-subproblem-v1"))
            existing = self.subproblems.get(subproblem_id)
            if existing is None:
                existing = {
                    "subproblem_id": subproblem_id,
                    "feature_id": feature_id,
                    "reward_mode": self.reward_mode,
                    "birth_update": self.updates,
                    "stopping_bonus": STOPPING_BONUS,
                    "feature_birth_revision_id": revision_id,
                    "feature_latest_revision_id": revision_id,
                    "feature_revision_count": 1,
                    "updates": 0,
                    "attainments": 0,
                    "reward_aligned_successes": 0,
                    "cumulative_environment_reward": 0,
                    "cumulative_subproblem_return": 0,
                }
                self.subproblems[subproblem_id] = existing
                if len(self.options) < MAX_OPTIONS:
                    option_id = str(uuid.uuid5(uuid.UUID(subproblem_id), "online-option-v1"))
                    self.options[option_id] = {
                        "option_id": option_id,
                        "subproblem_id": subproblem_id,
                        "birth_update": self.updates,
                        "states": set(),
                        "q_values": {},
                        "action_visits": {},
                        "beta_counts": {},
                        "q_updates": 0,
                        "beta_updates": 0,
                        "executions": 0,
                        "completed_executions": 0,
                        "successful_executions": 0,
                        "cumulative_duration": 0,
                        "cumulative_environment_return": 0,
                        "cumulative_subproblem_return": 0,
                    }
                    changes.append(
                        {
                            "kind": "construction",
                            "feature_id": feature_id,
                            "feature_revision_id": revision_id,
                            "subproblem_id": subproblem_id,
                            "option_id": option_id,
                            "parents": [revision_id],
                        }
                    )
                continue
            if revision_id != existing["feature_latest_revision_id"]:
                existing["feature_latest_revision_id"] = revision_id
                existing["feature_revision_count"] += 1
            option_id = next(
                (key for key in sorted(self.options) if self.options[key]["subproblem_id"] == subproblem_id),
                None,
            )
            changes.append(
                {
                    "kind": "maintenance",
                    "feature_id": feature_id,
                    "feature_revision_id": revision_id,
                    "subproblem_id": subproblem_id,
                    "option_id": option_id,
                    "parents": [revision_id],
                }
            )
        return changes

    def _update_primitive(self, observation: tuple[int, ...], action: int, reward: int) -> tuple[dict[str, Any], int]:
        if observation not in self.primitive_states:
            if len(self.primitive_states) >= MAX_STATES:
                return {
                    "observation": list(observation),
                    "action": action,
                    "applied": False,
                    "reason": "state_capacity",
                    "rule": "integer-sample-average-sufficient-statistics",
                }, 0
            self.primitive_states.add(observation)
        key = (observation, action)
        before = [self.primitive_counts.get(key, 0), self.primitive_returns.get(key, 0)]
        after = [before[0] + 1, _checked_add(before[1], reward)]
        self.primitive_counts[key], self.primitive_returns[key] = after
        return {
            "observation": list(observation),
            "action": action,
            "applied": True,
            "before": before,
            "after": after,
            "rule": "integer-sample-average-sufficient-statistics",
        }, 2

    def _learn_option(
        self,
        option: dict[str, Any],
        observation: tuple[int, ...],
        action: int,
        reward: int,
        next_observation: tuple[int, ...],
        terminated: bool,
        truncated: bool,
    ) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
        missing = {state for state in (observation, next_observation) if state not in option["states"]}
        assert len(option["states"]) + len(missing) <= MAX_STATES
        option["states"].update(missing)
        subproblem = self.subproblems[option["subproblem_id"]]
        attained = self._feature_attained(subproblem["feature_id"], next_observation)
        original_term = reward if self.reward_mode == "respecting" else 0
        stopping_term = STOPPING_BONUS if attained else 0
        subproblem_return = _checked_add(original_term, stopping_term)
        key = (observation, action)
        q_before = option["q_values"].get(key, 0)
        future = 0
        if not attained and not terminated and not truncated:
            future = max(
                option["q_values"].get((next_observation, candidate), 0)
                for candidate in range(self.actions)
            )
        q_after = _checked_q(_checked_add(subproblem_return, future))
        visits_before = option["action_visits"].get(key, 0)
        option["q_values"][key] = q_after
        option["action_visits"][key] = visits_before + 1
        option["q_updates"] += 1
        beta_before = option["beta_counts"].get(next_observation, (0, 0))
        beta_after = (beta_before[0] + int(attained), beta_before[1] + 1)
        option["beta_counts"][next_observation] = beta_after
        option["beta_updates"] += 1
        subproblem["updates"] += 1
        subproblem["attainments"] += int(attained)
        subproblem["reward_aligned_successes"] += int(attained and reward > 0)
        subproblem["cumulative_environment_reward"] = _checked_add(
            subproblem["cumulative_environment_reward"], reward
        )
        subproblem["cumulative_subproblem_return"] = _checked_add(
            subproblem["cumulative_subproblem_return"], subproblem_return
        )
        option["cumulative_subproblem_return"] = _checked_add(
            option["cumulative_subproblem_return"], subproblem_return
        )
        return (
            {
                "option_id": option["option_id"],
                "observation": list(observation),
                "action": action,
                "before": q_before,
                "after": q_after,
                "visits_before": visits_before,
                "visits_after": visits_before + 1,
                "future": future,
                "rule": "integer-q-learning-alpha-one-gamma-one",
            },
            {
                "option_id": option["option_id"],
                "observation": list(next_observation),
                "before": list(beta_before),
                "after": list(beta_after),
                "minimum_support": 2,
                "threshold": "positive_count*2>=total_count",
                "terminates": self._learned_beta(option, next_observation),
            },
            {
                "option_id": option["option_id"],
                "environment_reward": reward,
                "original_reward_term": original_term,
                "stopping_bonus_term": stopping_term,
                "return": subproblem_return,
                "attained": attained,
            },
        )

    def _finish_execution(
        self,
        reward: int,
        next_observation: tuple[int, ...],
        terminated: bool,
        truncated: bool,
    ) -> dict[str, Any] | None:
        if self.active is None:
            return None
        execution = self.active
        option = self.options[execution.option_id]
        subproblem = self.subproblems[option["subproblem_id"]]
        attained = self._feature_attained(subproblem["feature_id"], next_observation)
        original_term = reward if self.reward_mode == "respecting" else 0
        subproblem_return = _checked_add(original_term, STOPPING_BONUS if attained else 0)
        execution.environment_return = _checked_add(execution.environment_return, reward)
        execution.subproblem_return = _checked_add(execution.subproblem_return, subproblem_return)
        reason = None
        if terminated:
            reason = "environment_terminated"
        elif truncated:
            reason = "environment_truncated"
        elif self._learned_beta(option, next_observation):
            reason = "learned_beta"
        elif execution.duration >= MAX_DURATION:
            reason = "duration_cap"
        if reason is None:
            return {
                "kind": "continue",
                "invocation_id": execution.invocation_id,
                "option_id": execution.option_id,
                "duration": execution.duration,
                "reason": "learned_beta_continues",
            }
        option["completed_executions"] += 1
        option["successful_executions"] += int(reason == "learned_beta")
        option["cumulative_duration"] += execution.duration
        option["cumulative_environment_return"] = _checked_add(
            option["cumulative_environment_return"], execution.environment_return
        )
        self.active = None
        return {
            "kind": "terminate",
            "invocation_id": execution.invocation_id,
            "option_id": execution.option_id,
            "duration": execution.duration,
            "reason": reason,
            "environment_return": execution.environment_return,
            "subproblem_return": execution.subproblem_return,
            "attained": attained,
        }

    @staticmethod
    def _work_report(value: object, work_class: str, request_id: str, requested: int) -> dict[str, Any]:
        assert isinstance(value, dict)
        report = cast(dict[str, Any], value)
        expected: dict[str, Any] = {
            "work_class": work_class,
            "request_id": request_id,
            "requested": requested,
            "accepted": True,
            "outcome": "admit",
            "reason_code": "SUPERVISOR_WORK_GRANTED",
        }
        assert report == expected
        return expected

    def _allocation_report(
        self, value: object, request_id: str, serialized_bytes: int
    ) -> dict[str, Any]:
        assert isinstance(value, dict)
        report = cast(dict[str, Any], value)
        assert set(report) == {
            "request_id",
            "allocated_bytes",
            "serialized_bytes",
            "accepted",
            "outcome",
            "reason_code",
        }
        assert report["request_id"] == request_id
        assert (
            type(report["allocated_bytes"]) is int
            and 0 < report["allocated_bytes"] <= self.retained_limit
        )
        assert report["serialized_bytes"] == serialized_bytes <= self.serialized_limit
        assert report["accepted"] is True and report["outcome"] == "admit"
        assert report["reason_code"] == "SUPERVISOR_STATE_GRANTED"
        return copy.deepcopy(report)

    def finish(self, transition: dict[str, Any], audit: dict[str, Any], accounting: dict[str, Any]) -> None:
        assert self.pending is not None
        observation = tuple(transition["observation"])
        next_observation = tuple(transition["next_observation"])
        action, reward = transition["action"], transition["reward"]
        terminated, truncated = transition["terminated"], transition["truncated"]
        source_event_id = transition["source_event_id"]
        assert self.pending["observation"] == list(observation)
        assert self.pending["action"] == action
        assert type(reward) is int and -(2**31) <= reward < 2**31
        assert type(terminated) is bool and type(truncated) is bool and not (terminated and truncated)

        expected_before = digest(self.snapshot())
        primitive_delta, touches = self._update_primitive(observation, action, reward)
        prior_option_ids = set(self.options)
        feature_statistics_before = {
            key: (value["count"], value["reward_sum"])
            for key, value in self.statistics.items()
        }
        feature_proposals = self._update_feature(next_observation, reward, source_event_id)
        feature_statistics_after = {
            key: (value["count"], value["reward_sum"])
            for key, value in self.statistics.items()
        }
        feature_touches = 2 * sum(
            feature_statistics_before.get(key) != value
            for key, value in feature_statistics_after.items()
        ) + sum(proposal["parameter_count"] for proposal in feature_proposals)
        touches += feature_touches
        construction = self._apply_feature_events(feature_proposals)
        q_deltas: list[dict[str, Any]] = []
        beta_deltas: list[dict[str, Any]] = []
        subproblem_returns: list[dict[str, Any]] = []
        for option_id in sorted(self.options):
            if option_id not in prior_option_ids:
                continue
            q_delta, beta_delta, subproblem_return = self._learn_option(
                self.options[option_id],
                observation,
                action,
                reward,
                next_observation,
                terminated,
                truncated,
            )
            q_deltas.append(q_delta)
            beta_deltas.append(beta_delta)
            subproblem_returns.append(subproblem_return)
            touches += 4
        termination_event = self._finish_execution(reward, next_observation, terminated, truncated)
        execution_events = copy.deepcopy(self.pending["events"])
        if termination_event is not None:
            execution_events.append(termination_event)
        self.execution_events.extend(copy.deepcopy(execution_events))
        self.pending = None
        self.steps += 1
        self.updates += 1
        self.parameter_touches += touches
        tariffs = self.tariffs()
        for work_class in ("learning", "feature_generation", "option_learning"):
            self.work_by_class[work_class] += tariffs[work_class]
        assert touches <= MAX_PARAMETER_TOUCHES
        expected_after_snapshot = self.snapshot()
        expected_after = digest(expected_after_snapshot)
        expected_serialized = len(canonical(expected_after_snapshot))
        assert expected_serialized <= min(self.serialized_limit, MAX_SERIALIZED_BYTES)

        assert set(audit) == {
            "schema",
            "update",
            "action",
            "reward",
            "work_by_class",
            "parameter_touches",
            "state_before",
            "state_after",
            "allocated_bytes",
            "serialized_bytes",
            "details",
        }
        details_value = audit["details"]
        assert isinstance(details_value, dict)
        details = cast(dict[str, Any], details_value)
        expected_detail_keys = {
            "track",
            "reward_mode",
            "reward_respecting_claim_eligible",
            "options_enabled",
            "replay_items_retained",
            "causal_transition",
            "primitive_delta",
            "feature_parameter_touches",
            "feature_proposals",
            "option_construction",
            "option_q_deltas",
            "option_beta_deltas",
            "option_execution_events",
            "rewards",
            "admission",
            "capacity",
        }
        assert set(details) == expected_detail_keys, (
            sorted(set(details) - expected_detail_keys),
            sorted(expected_detail_keys - set(details)),
        )
        admission_value = details["admission"]
        assert isinstance(admission_value, dict)
        admission = cast(dict[str, Any], admission_value)
        assert set(admission) == {
            "acting",
            "act_state",
            "learning",
            "feature_generation",
            "feature_state",
            "option_learning",
            "option_state",
        }
        step = self.steps - 1
        self._work_report(admission["acting"], "acting", f"option-acting-{step}", tariffs["acting"])
        self._work_report(admission["learning"], "learning", f"learning-{step}", tariffs["learning"])
        self._work_report(
            admission["feature_generation"],
            "feature_generation",
            f"feature-work-{step}",
            tariffs["feature_generation"],
        )
        self._work_report(
            admission["option_learning"],
            "option_learning",
            f"option-learning-{step}",
            tariffs["option_learning"],
        )
        act_state = self._allocation_report(
            admission["act_state"],
            f"option-act-state-{step}",
            self.act_serialized_bytes,
        )
        feature_state = self._allocation_report(
            admission["feature_state"],
            f"feature-state-{step}",
            len(canonical(self.feature_snapshot())),
        )
        option_state = self._allocation_report(
            admission["option_state"], f"option-state-{step}", expected_serialized
        )
        expected_details: dict[str, Any] = {
            "track": "track_a",
            "reward_mode": self.reward_mode,
            "reward_respecting_claim_eligible": self.reward_mode == "respecting",
            "options_enabled": self.options_enabled,
            "replay_items_retained": 0,
            "causal_transition": {
                "observation": list(observation),
                "action": action,
                "reward": reward,
                "next_observation": list(next_observation),
                "terminated": terminated,
                "truncated": truncated,
                "source_event_id": source_event_id,
            },
            "primitive_delta": primitive_delta,
            "feature_parameter_touches": feature_touches,
            "feature_proposals": feature_proposals,
            "option_construction": construction,
            "option_q_deltas": q_deltas,
            "option_beta_deltas": beta_deltas,
            "option_execution_events": execution_events,
            "rewards": {
                "environment": reward,
                "subproblem_returns": subproblem_returns,
            },
            "admission": {
                "acting": admission["acting"],
                "act_state": act_state,
                "learning": admission["learning"],
                "feature_generation": admission["feature_generation"],
                "feature_state": feature_state,
                "option_learning": admission["option_learning"],
                "option_state": option_state,
            },
            "capacity": {
                "max_states": MAX_STATES,
                "max_options": MAX_OPTIONS,
                "max_duration": MAX_DURATION,
                "max_update_bytes": MAX_UPDATE_BYTES,
            },
        }
        assert details == expected_details
        assert audit == {
            "schema": "bonsai.online-update/v2",
            "update": self.updates,
            "action": action,
            "reward": reward,
            "work_by_class": dict(self.work_by_class),
            "parameter_touches": self.parameter_touches,
            "state_before": expected_before,
            "state_after": expected_after,
            "allocated_bytes": option_state["allocated_bytes"],
            "serialized_bytes": expected_serialized,
            "details": expected_details,
        }
        assert len(canonical(audit)) <= MAX_UPDATE_BYTES
        assert accounting == {
            "environment_steps": self.steps,
            "updates": self.updates,
            "parameter_touches": self.parameter_touches,
            "work_items": sum(self.work_by_class.values()),
            "replay_items_retained": 0,
        }
        self.curve_environment_reward = _checked_add(self.curve_environment_reward, reward)
        self.learning_curve.append(
            {
                "step": self.steps,
                "environment_reward": reward,
                "cumulative_environment_reward": self.curve_environment_reward,
                "feature_count": len(self.features),
                "option_count": len(self.options),
                "q_updates": sum(option["q_updates"] for option in self.options.values()),
                "beta_updates": sum(option["beta_updates"] for option in self.options.values()),
                "option_invocations": sum(option["executions"] for option in self.options.values()),
                "option_completions": sum(
                    option["completed_executions"] for option in self.options.values()
                ),
                "work_by_class": dict(self.work_by_class),
                "parameter_touches": self.parameter_touches,
                "retained_object_bytes": audit["allocated_bytes"],
                "serialized_state_bytes": audit["serialized_bytes"],
            }
        )


def reconstruct_records(
    config: dict[str, Any],
    transitions: list[dict[str, Any]],
    audits: list[dict[str, Any]],
    accountings: list[dict[str, Any]],
    expected_steps: int,
) -> tuple[Reconstructor, dict[str, Any]]:
    assert len(transitions) == len(audits) == len(accountings) == expected_steps
    state = Reconstructor(config)
    prior_episode = -1
    prior_step = -1
    prior_done = True
    environment_reward = 0
    for total, (transition, audit, accounting) in enumerate(
        zip(transitions, audits, accountings, strict=True)
    ):
        assert transition["total_step"] == total
        episode, episode_step = transition["episode"], transition["episode_step"]
        if prior_done:
            assert episode == prior_episode + 1 and episode_step == 0
        else:
            assert episode == prior_episode and episode_step == prior_step + 1
        assert transition["source_event_id"] == feedback_event_id(total)
        state.begin(tuple(transition["observation"]), transition["action"])
        state.finish(transition, audit, accounting)
        prior_episode, prior_step = episode, episode_step
        prior_done = transition["terminated"] or transition["truncated"]
        environment_reward = _checked_add(environment_reward, transition["reward"])
    final_snapshot = state.snapshot()
    terminations = [event for event in state.execution_events if event["kind"] == "terminate"]
    return state, {
        "steps_reconstructed": expected_steps,
        "environment_reward": environment_reward,
        "features": len(state.features),
        "feature_births": sum(event["kind"] == "birth" for event in state.feature_events),
        "feature_revisions": sum(event["kind"] == "revision" for event in state.feature_events),
        "options": len(state.options),
        "q_updates": sum(option["q_updates"] for option in state.options.values()),
        "beta_updates": sum(option["beta_updates"] for option in state.options.values()),
        "initiations": sum(event["kind"] == "initiate" for event in state.execution_events),
        "termination_reasons": {
            reason: sum(event["reason"] == reason for event in terminations)
            for reason in sorted({event["reason"] for event in terminations})
        },
        "completed_durations": [event["duration"] for event in terminations],
        "active_at_run_end": state.execution_snapshot(state.active),
        "run_limit_censored_invocations": int(state.active is not None),
        "trajectory_sha256": digest(transitions),
        "state_sha256": digest(final_snapshot),
        "execution_sha256": digest(state.execution_events),
        "learning_curve": copy.deepcopy(state.learning_curve),
    }
