"""Deterministic scenario protocol with an observer-only diagnostic channel."""

from __future__ import annotations

import hashlib
import json
from dataclasses import asdict, dataclass
from typing import Final, Literal, TypedDict

MASK_64: Final[int] = (1 << 64) - 1
Track = Literal["track_a", "track_d"]


class ScenarioError(ValueError):
    """Stable scenario protocol failure."""

    def __init__(self, code: str) -> None:
        self.code = code
        super().__init__(code)


@dataclass(frozen=True, slots=True)
class ScenarioSpec:
    scenario_id: str
    version: str
    seed: int
    horizon: int
    action_count: int
    observation_width: int
    big_world_size: int
    change_points: tuple[int, ...]


@dataclass(frozen=True, slots=True)
class PublicTransition:
    stream_id: str
    step: int
    observation: tuple[int, ...]
    allowed_actions: tuple[int, ...]
    action: int
    reward: int
    change_point: bool
    terminated: bool


@dataclass(frozen=True, slots=True)
class DiagnosticTruth:
    stream_id: str
    step: int
    latent_state: int
    target_action: int
    world_token: int


@dataclass(frozen=True, slots=True)
class ScenarioTrace:
    public: tuple[PublicTransition, ...]
    diagnostic: tuple[DiagnosticTruth, ...]
    public_sha256: str
    diagnostic_sha256: str


class ExposureFacts(TypedDict):
    privileged_diagnostic_exposed: bool


class _XorShift64:
    def __init__(self, seed: int) -> None:
        self._state = seed & MASK_64
        if self._state == 0:
            self._state = 0x9E3779B97F4A7C15

    def next(self) -> int:
        value = self._state
        value ^= (value << 13) & MASK_64
        value ^= value >> 7
        value ^= (value << 17) & MASK_64
        self._state = value & MASK_64
        return self._state


def validate_spec(spec: ScenarioSpec) -> None:
    """Validate semantic constraints not expressible by Python types."""
    if not spec.scenario_id or not spec.version:
        raise ScenarioError("SCENARIO_IDENTITY_INVALID")
    if spec.horizon <= 0 or spec.action_count <= 1 or spec.observation_width <= 0:
        raise ScenarioError("SCENARIO_SHAPE_INVALID")
    if spec.big_world_size < spec.action_count:
        raise ScenarioError("SCENARIO_BIG_WORLD_INVALID")
    if (
        tuple(sorted(set(spec.change_points))) != spec.change_points
        or any(point <= 0 or point >= spec.horizon for point in spec.change_points)
    ):
        raise ScenarioError("SCENARIO_CHANGE_POINTS_INVALID")


def semantic_stream(spec: ScenarioSpec, actions: tuple[int, ...]) -> ScenarioTrace:
    """Generate one byte-stable public stream and separate diagnostic truth."""
    validate_spec(spec)
    if len(actions) != spec.horizon:
        raise ScenarioError("SCENARIO_ACTION_SCHEDULE_INVALID")
    if any(action < 0 or action >= spec.action_count for action in actions):
        raise ScenarioError("SCENARIO_ACTION_INVALID")

    stream_id = _stream_identity(spec)
    generator = _XorShift64(spec.seed)
    public: list[PublicTransition] = []
    diagnostic: list[DiagnosticTruth] = []
    target_action = int(generator.next() % spec.action_count)
    allowed_actions = tuple(range(spec.action_count))
    for step, action in enumerate(actions):
        changed = step in spec.change_points
        if changed:
            target_action = (target_action + 1 + int(generator.next() % (spec.action_count - 1))) % spec.action_count
        latent_state = int(generator.next() % spec.big_world_size)
        world_token = int(generator.next() % spec.big_world_size)
        observation = tuple(
            int((latent_state + generator.next() + offset) % spec.big_world_size)
            for offset in range(spec.observation_width)
        )
        public.append(
            PublicTransition(
                stream_id=stream_id,
                step=step,
                observation=observation,
                allowed_actions=allowed_actions,
                action=action,
                reward=int(action == target_action),
                change_point=changed,
                terminated=step + 1 == spec.horizon,
            )
        )
        diagnostic.append(
            DiagnosticTruth(
                stream_id=stream_id,
                step=step,
                latent_state=latent_state,
                target_action=target_action,
                world_token=world_token,
            )
        )
    public_bytes = _canonical([asdict(item) for item in public])
    diagnostic_bytes = _canonical([asdict(item) for item in diagnostic])
    return ScenarioTrace(
        public=tuple(public),
        diagnostic=tuple(diagnostic),
        public_sha256=hashlib.sha256(public_bytes).hexdigest(),
        diagnostic_sha256=hashlib.sha256(diagnostic_bytes).hexdigest(),
    )


def classify_diagnostic_exposure(facts: ExposureFacts) -> Track:
    """Force Track D whenever privileged diagnostic truth reaches an agent."""
    return "track_d" if facts["privileged_diagnostic_exposed"] else "track_a"


def _stream_identity(spec: ScenarioSpec) -> str:
    return hashlib.sha256(_canonical(asdict(spec))).hexdigest()


def _canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode("ascii")


@dataclass(frozen=True, slots=True)
class SessionObservation:
    stream_id: str
    step: int
    observation: tuple[int, ...]
    allowed_actions: tuple[int, ...]


@dataclass(frozen=True, slots=True)
class SessionTransition:
    step: int
    action: int
    reward: int
    next: SessionObservation
    terminated: bool
    truncated: bool


@dataclass(frozen=True, slots=True)
class SessionDiagnostic:
    stream_id: str
    step: int
    latent_state: int
    goal_state: int


class ScenarioSession:
    """Online causal ring diagnostic; archived semantic_stream remains unchanged.

    Action a advances the ring by a+1. Reaching the private goal terminates;
    exhausting the horizon truncates. Observation noise consumes a fixed seeded
    stream and the next observation also depends on the action's state change.
    Only the current state is retained, with no future action schedule or trace.
    """

    def __init__(self, spec: ScenarioSpec) -> None:
        if (
            spec.action_count > 256
            or spec.observation_width > 256
            or spec.big_world_size > MASK_64
            or spec.horizon > 1_000_000_000
            or len(spec.change_points) > 4096
        ):
            raise ScenarioError("SCENARIO_SESSION_LIMIT_INVALID")
        validate_spec(spec)
        self._spec = spec
        self._generator = _XorShift64(0)
        self._stream_id = ""
        self._state = 0
        self._goal = 0
        self._step = 0
        self._started = False
        self._done = False
        self._observation: SessionObservation | None = None

    def reset(self, seed: int) -> SessionObservation:
        """Reset one episode without receiving any future actions."""
        if type(seed) is not int or not 0 <= seed <= MASK_64:
            raise ScenarioError("SCENARIO_SEED_INVALID")
        self._generator = _XorShift64(seed)
        self._state = self._generator.next() % self._spec.big_world_size
        offset = 1 + self._generator.next() % min(self._spec.action_count, self._spec.big_world_size - 1)
        self._goal = (self._state + offset) % self._spec.big_world_size
        identity = asdict(self._spec)
        identity["seed"] = seed
        self._stream_id = hashlib.sha256(
            _canonical({"dynamics": "bonsai.causal-ring/v1", "spec": identity})
        ).hexdigest()
        self._step = 0
        self._started = True
        self._done = False
        self._observation = self._observe()
        return self._observation

    def observe(self) -> SessionObservation:
        """Return only the cached current public observation; do not advance RNG."""
        if self._observation is None:
            raise ScenarioError("SCENARIO_RESET_REQUIRED")
        return self._observation

    def step(self, index: int, action: int) -> SessionTransition:
        """Apply exactly one chosen action; rejected requests do not change state."""
        if not self._started:
            raise ScenarioError("SCENARIO_RESET_REQUIRED")
        if self._done:
            raise ScenarioError("SCENARIO_EPISODE_FINISHED")
        if type(index) is not int or index != self._step:
            raise ScenarioError("SCENARIO_STEP_OUT_OF_ORDER")
        if type(action) is not int or not 0 <= action < self._spec.action_count:
            raise ScenarioError("SCENARIO_ACTION_INVALID")
        if index in self._spec.change_points:
            shift = 1 + self._generator.next() % (self._spec.big_world_size - 1)
            self._goal = (self._goal + shift) % self._spec.big_world_size
        self._state = (self._state + action + 1) % self._spec.big_world_size
        self._step += 1
        terminated = self._state == self._goal
        truncated = not terminated and self._step == self._spec.horizon
        self._done = terminated or truncated
        self._observation = self._observe()
        return SessionTransition(index, action, int(terminated), self._observation, terminated, truncated)

    def diagnostic(self) -> SessionDiagnostic:
        """Observer-only truth; never serialize this through the learner channel."""
        if not self._started:
            raise ScenarioError("SCENARIO_RESET_REQUIRED")
        return SessionDiagnostic(self._stream_id, self._step, self._state, self._goal)

    def _observe(self) -> SessionObservation:
        values = tuple(
            (self._state + self._generator.next() + offset) % self._spec.big_world_size
            for offset in range(self._spec.observation_width)
        )
        actions = () if self._done else tuple(range(self._spec.action_count))
        return SessionObservation(self._stream_id, self._step, values, actions)
