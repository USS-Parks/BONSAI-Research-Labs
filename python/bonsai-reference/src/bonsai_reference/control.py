"""Minimal single-pass primitive tabular control for the M1 heartbeat."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

Track = Literal["track_a"]


class ControlError(ValueError):
    """Stable primitive-control protocol failure."""

    def __init__(self, code: str) -> None:
        self.code = code
        super().__init__(code)


@dataclass(frozen=True, slots=True)
class ControlCertification:
    adapter_version: str
    track: Track
    primitive_actions_only: bool
    batch_size: int
    replay_capacity: int
    learned_features: bool
    options: bool
    planning: bool
    privileged_diagnostic_input: bool


@dataclass(frozen=True, slots=True)
class WorkAccounting:
    environment_steps: int
    updates: int
    parameter_touches: int
    work_items: int
    replay_items_retained: int


@dataclass(frozen=True, slots=True)
class LearningTrace:
    actions: tuple[int, ...]
    rewards: tuple[int, ...]
    cumulative_reward: tuple[int, ...]
    accounting: WorkAccounting
    certification: ControlCertification


class PrimitiveTabularControl:
    """Sample-average primitive control with deterministic initial exploration."""

    def __init__(self, action_count: int) -> None:
        if action_count <= 1:
            raise ControlError("CONTROL_ACTION_SPACE_INVALID")
        self._action_count = action_count
        self._counts: dict[tuple[tuple[int, ...], int], int] = {}
        self._returns: dict[tuple[tuple[int, ...], int], int] = {}
        self._pending: tuple[tuple[int, ...], int] | None = None
        self._steps = 0
        self._updates = 0
        self._touches = 0
        self._work_items = 0

    @property
    def certification(self) -> ControlCertification:
        return ControlCertification(
            adapter_version="1.0",
            track="track_a",
            primitive_actions_only=True,
            batch_size=1,
            replay_capacity=0,
            learned_features=False,
            options=False,
            planning=False,
            privileged_diagnostic_input=False,
        )

    @property
    def accounting(self) -> WorkAccounting:
        return WorkAccounting(
            environment_steps=self._steps,
            updates=self._updates,
            parameter_touches=self._touches,
            work_items=self._work_items,
            replay_items_retained=0,
        )

    def act(self, observation: tuple[int, ...]) -> int:
        """Choose one primitive action without retaining a transition."""
        if not observation:
            raise ControlError("CONTROL_OBSERVATION_INVALID")
        if self._pending is not None:
            raise ControlError("CONTROL_UPDATE_REQUIRED")
        unseen = [action for action in range(self._action_count) if (observation, action) not in self._counts]
        action = unseen[0] if unseen else self._best_action(observation)
        self._pending = (observation, action)
        self._work_items += self._action_count
        return action

    def observe(self, reward: int) -> None:
        """Consume the one pending transition exactly once and discard it."""
        if self._pending is None:
            raise ControlError("CONTROL_ACTION_REQUIRED")
        key = self._pending
        self._counts[key] = self._counts.get(key, 0) + 1
        self._returns[key] = self._returns.get(key, 0) + reward
        self._pending = None
        self._steps += 1
        self._updates += 1
        self._touches += 2
        self._work_items += 1

    def _best_action(self, observation: tuple[int, ...]) -> int:
        def better(left: int, right: int) -> bool:
            left_key = (observation, left)
            right_key = (observation, right)
            return self._returns[left_key] * self._counts[right_key] > self._returns[right_key] * self._counts[left_key]

        best = 0
        for action in range(1, self._action_count):
            if better(action, best):
                best = action
        return best


def run_deterministic_bandit(horizon: int, target_action: int = 2) -> LearningTrace:
    """Run the certified one-state diagnostic learning curve."""
    if horizon <= 0 or target_action < 0 or target_action >= 3:
        raise ControlError("CONTROL_FIXTURE_INVALID")
    control = PrimitiveTabularControl(action_count=3)
    actions: list[int] = []
    rewards: list[int] = []
    cumulative: list[int] = []
    total = 0
    for _ in range(horizon):
        action = control.act((0,))
        reward = int(action == target_action)
        control.observe(reward)
        actions.append(action)
        rewards.append(reward)
        total += reward
        cumulative.append(total)
    return LearningTrace(
        actions=tuple(actions),
        rewards=tuple(rewards),
        cumulative_reward=tuple(cumulative),
        accounting=control.accounting,
        certification=control.certification,
    )
