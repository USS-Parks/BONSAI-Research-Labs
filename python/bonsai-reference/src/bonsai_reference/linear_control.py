"""Bounded linear contextual reward prediction with one online NLMS update."""
from __future__ import annotations

import json
import math

from bonsai_reference.control import ControlError, WorkAccounting


class LinearControl:
    """Fixed weights generalize across observations; no transition history is retained."""

    def __init__(self, action_count: int, observation_width: int) -> None:
        if not 2 <= action_count <= 256 or not 1 <= observation_width <= 256:
            raise ControlError("CONTROL_DIMENSION_INVALID")
        self._actions = action_count
        self._width = observation_width
        self._weights = [[0.0] * (observation_width + 1) for _ in range(action_count)]
        self._pending: tuple[tuple[float, ...], int, float] | None = None
        self._steps = 0
        self._updates = 0
        self._touches = 0
        self._work_items = 0
        self._parameter_update = b""

    @property
    def accounting(self) -> WorkAccounting:
        return WorkAccounting(self._steps, self._updates, self._touches, self._work_items, 0)

    @property
    def parameter_update(self) -> bytes:
        return self._parameter_update

    @property
    def weights(self) -> tuple[tuple[float, ...], ...]:
        return tuple(tuple(row) for row in self._weights)

    def reset_episode(self) -> None:
        if self._pending is not None:
            raise ControlError("CONTROL_UPDATE_REQUIRED")

    def act(self, observation: tuple[int, ...]) -> int:
        if (len(observation) != self._width
                or any(type(value) is not int or not 0 <= value < 2**64 for value in observation)):
            raise ControlError("CONTROL_OBSERVATION_INVALID")
        if self._pending is not None:
            raise ControlError("CONTROL_UPDATE_REQUIRED")
        features = (1.0, *(value / (1 + value) for value in observation))
        predictions = [sum(weight * value for weight, value in zip(row, features, strict=True))
                       for row in self._weights]
        action = self._steps if self._steps < self._actions else max(
            range(self._actions), key=lambda index: predictions[index])
        self._pending = (features, action, predictions[action])
        self._work_items += self._actions
        return action

    def observe(self, reward: int) -> None:
        if self._pending is None:
            raise ControlError("CONTROL_ACTION_REQUIRED")
        if type(reward) is not int or not -(2**63) <= reward < 2**63:
            raise ControlError("CONTROL_REWARD_INVALID")
        features, action, prediction = self._pending
        before = self._weights[action]
        norm = sum(value * value for value in features)
        scale = 0.25 * (reward - prediction) / norm
        after = [weight + scale * value for weight, value in zip(before, features, strict=True)]
        if not all(math.isfinite(value) for value in after):
            raise ControlError("CONTROL_NUMERIC_INVALID")
        update = json.dumps({
            "schema": "bonsai.parameter-update/v1", "rule": "linear-nlms",
            "update": self._updates + 1, "action": action, "features": features,
            "reward": reward, "prediction": prediction, "before": before, "after": after,
        }, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()
        self._weights[action] = after
        self._parameter_update = update
        self._pending = None
        self._steps += 1
        self._updates += 1
        self._touches += self._width + 1
        self._work_items += 1
