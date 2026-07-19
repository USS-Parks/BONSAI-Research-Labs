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
