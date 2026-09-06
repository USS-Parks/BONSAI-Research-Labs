"""Optional, pinned FrozenLake session with explicit online observation projection."""
from __future__ import annotations

import hashlib
import importlib
import importlib.metadata
import json
import operator
from dataclasses import asdict, dataclass
from typing import Protocol, SupportsIndex, cast

from bonsai_reference.scenario import ScenarioError, SessionObservation, SessionTransition

GYMNASIUM_VERSION = "1.3.0"


class _Space(Protocol):
    n: int
    def contains(self, value: object) -> bool: ...


class _Environment(Protocol):
    action_space: _Space
    observation_space: _Space
    def reset(self, *, seed: int) -> tuple[object, object]: ...
    def step(self, action: int) -> tuple[object, object, object, object, object]: ...


class GymnasiumAPI(Protocol):
    def make(self, id: str, *, map_name: str, is_slippery: bool,
             max_episode_steps: int, render_mode: None) -> _Environment: ...


@dataclass(frozen=True, slots=True)
class GymnasiumSpec:
    scenario_id: str = "gymnasium-frozen-lake"
    version: str = "1.0"
    seed: int = 42
    horizon: int = 8
    action_count: int = 4
    observation_width: int = 1
    environment_id: str = "FrozenLake-v1"
    gymnasium_version: str = GYMNASIUM_VERSION
    map_name: str = "4x4"
    is_slippery: bool = False

    def validate(self) -> None:
        if (self.scenario_id != "gymnasium-frozen-lake" or self.version != "1.0"
                or self.environment_id != "FrozenLake-v1" or self.gymnasium_version != GYMNASIUM_VERSION
                or self.map_name != "4x4" or self.is_slippery is not False
                or type(self.action_count) is not int or self.action_count != 4
                or type(self.observation_width) is not int or self.observation_width != 1
                or type(self.seed) is not int or not 0 <= self.seed < 2**64
                or type(self.horizon) is not int or not 1 <= self.horizon <= 1000):
            raise ScenarioError("GYMNASIUM_CONFIGURATION_INVALID")


def make_environment(spec: GymnasiumSpec) -> _Environment:
    spec.validate()
    try:
        installed = importlib.metadata.version("gymnasium")
    except importlib.metadata.PackageNotFoundError as error:
        raise ScenarioError("GYMNASIUM_OPTIONAL_DEPENDENCY_MISSING") from error
    if installed != GYMNASIUM_VERSION:
        raise ScenarioError("GYMNASIUM_DEPENDENCY_VERSION_MISMATCH")
    gym = cast(GymnasiumAPI, cast(object, importlib.import_module("gymnasium")))
    environment = gym.make(spec.environment_id, map_name=spec.map_name, is_slippery=False,
                           max_episode_steps=spec.horizon, render_mode=None)
    if environment.action_space.n != 4 or environment.observation_space.n != 16:
        raise ScenarioError("GYMNASIUM_SPACE_UNSUPPORTED")
    return environment


class GymnasiumSession:
    """Keep one current public observation; never expose info, model, map or future actions."""

    def __init__(self, spec: GymnasiumSpec) -> None:
        self._spec = spec
        self._environment = make_environment(spec)
        self._observation: SessionObservation | None = None
        self._done = False
        self._step = 0
        self._stream = ""

    def _project(self, raw: object, done: bool) -> SessionObservation:
        if not self._environment.observation_space.contains(raw):
            raise ScenarioError("GYMNASIUM_OBSERVATION_INVALID")
        try:
            value = operator.index(cast(SupportsIndex, raw))
        except TypeError as error:
            raise ScenarioError("GYMNASIUM_OBSERVATION_INVALID") from error
        if not 0 <= value < 16:
            raise ScenarioError("GYMNASIUM_OBSERVATION_INVALID")
        return SessionObservation(self._stream, self._step, (value,), () if done else (0, 1, 2, 3))

    def reset(self, seed: int) -> SessionObservation:
        if type(seed) is not int or not 0 <= seed < 2**64:
            raise ScenarioError("SCENARIO_SEED_INVALID")
        raw, _info = self._environment.reset(seed=seed)
        identity = {**asdict(self._spec), "seed": seed}
        self._stream = hashlib.sha256(
            json.dumps(identity, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
        self._step = 0
        self._done = False
        self._observation = self._project(raw, False)
        return self._observation

    def observe(self) -> SessionObservation:
        if self._observation is None:
            raise ScenarioError("SCENARIO_RESET_REQUIRED")
        return self._observation

    def step(self, index: int, action: int) -> SessionTransition:
        self.observe()
        if self._done:
            raise ScenarioError("SCENARIO_EPISODE_FINISHED")
        if type(index) is not int or index != self._step:
            raise ScenarioError("SCENARIO_STEP_OUT_OF_ORDER")
        if type(action) is not int or not self._environment.action_space.contains(action):
            raise ScenarioError("SCENARIO_ACTION_INVALID")
        raw, reward, terminated, truncated, _info = self._environment.step(action)
        if type(terminated) is not bool or type(truncated) is not bool:
            raise ScenarioError("GYMNASIUM_END_FLAGS_INVALID")
        if type(reward) not in (int, float) or reward not in (0, 1):
            raise ScenarioError("GYMNASIUM_REWARD_INVALID")
        self._step += 1
        self._done = terminated or truncated
        self._observation = self._project(raw, self._done)
        return SessionTransition(index, action, int(cast(int | float, reward)),
                                 self._observation, terminated, truncated)
