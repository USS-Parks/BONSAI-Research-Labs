"""Bounded feature proposals from online observations and reward sufficient statistics."""
from __future__ import annotations

import copy
import hashlib
import json
import sys
import uuid
from dataclasses import asdict, dataclass, fields, is_dataclass, replace
from typing import Protocol, cast

from bonsai_reference.brdc1 import CycleError, FeatureRecord, FeatureStage


class DiscoveryAdmission(Protocol):
    def work(self, request_id: str, amount: int) -> dict[str, object]: ...
    def allocation(self, request_id: str, allocated_bytes: int, serialized_bytes: int) -> dict[str, object]: ...


@dataclass(frozen=True, slots=True)
class Exposure:
    count: int
    reward_sum: int
    first_step: int
    last_step: int
    first_event_id: str
    last_event_id: str


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def retained_bytes(value: object) -> int:
    """Measure the unique Python object graph, excluding interpreter/allocator overhead."""
    seen: set[int] = set()

    def visit(item: object) -> int:
        identity = id(item)
        if identity in seen:
            return 0
        seen.add(identity)
        size = sys.getsizeof(item)
        if isinstance(item, dict):
            return size + sum(visit(key) + visit(child) for key, child in cast(dict[object, object], item).items())
        if isinstance(item, (tuple, list)):
            return size + sum(visit(child) for child in cast(tuple[object, ...] | list[object], item))
        if is_dataclass(item) and not isinstance(item, type):
            return size + sum(visit(getattr(item, field.name)) for field in fields(item))
        return size

    return visit(value)


class OnlineFeatureStage(FeatureStage):
    """Extend the original lifecycle with finite, experience-generated equality features."""

    def __init__(self, width: int, seed: int, *, enabled: bool = True,
                 max_statistics: int = 64, max_features: int = 16) -> None:
        super().__init__()
        if (type(width) is not int or not 1 <= width <= 8
                or type(seed) is not int or not 0 <= seed < 2**64
                or type(enabled) is not bool
                or type(max_statistics) is not int or not width <= max_statistics <= 128
                or type(max_features) is not int or not 1 <= max_features <= 32):
            raise CycleError("FEATURE_DISCOVERY_CONFIGURATION_INVALID")
        self._width = width
        self._seed = seed
        self._enabled = enabled
        self._max_statistics = max_statistics
        self._max_features = max_features
        self._step = 0
        self._statistics: dict[tuple[int, int], Exposure] = {}
        self._versions: dict[str, tuple[int, str, int]] = {}

    @property
    def width(self) -> int:
        return self._width

    @property
    def seed(self) -> int:
        return self._seed

    @property
    def enabled(self) -> bool:
        return self._enabled

    @property
    def max_statistics(self) -> int:
        return self._max_statistics

    @property
    def max_features(self) -> int:
        return self._max_features

    def produce(self, feature_id: str, representation: tuple[int, ...], step: int, *,
                semantic_label: str | None = None, parents: tuple[str, ...] = ()) -> FeatureRecord:
        raise CycleError("FEATURE_ONLINE_OBSERVATION_REQUIRED")

    def revise(self, feature_id: str, representation: tuple[int, ...], step: int) -> FeatureRecord:
        raise CycleError("FEATURE_ONLINE_OBSERVATION_REQUIRED")

    def retire(self, feature_id: str, step: int) -> FeatureRecord:
        raise CycleError("FEATURE_ONLINE_OBSERVATION_REQUIRED")

    def add_consumer(self, feature_id: str) -> FeatureRecord:
        raise CycleError("FEATURE_ONLINE_OBSERVATION_REQUIRED")

    def set_utility(self, feature_id: str, utility: int) -> FeatureRecord:
        raise CycleError("FEATURE_ONLINE_OBSERVATION_REQUIRED")

    def allocated_bytes(self) -> int:
        return sys.getsizeof(self) + retained_bytes(vars(self))

    def online_snapshot(self) -> dict[str, object]:
        return {
            "schema": "bonsai.online-feature-state/v1",
            "width": self.width, "seed": self.seed, "enabled": self.enabled,
            "max_statistics": self.max_statistics, "max_features": self.max_features,
            "next_step": self._step, "feature_work": self.work,
            "features": [asdict(record) for record in self.snapshot()],
            "statistics": [{"coordinate": key[0], "value": key[1], **asdict(value)}
                           for key, value in sorted(self._statistics.items())],
            "versions": dict(self._versions),
            "replay_items_retained": 0,
        }

    def observe(self, step: int, observation: tuple[int, ...], reward: int,
                source_event_id: str, admission: DiscoveryAdmission) -> dict[str, object]:
        self._validate(step, observation, reward, source_event_id)
        before = hashlib.sha256(canonical(self.online_snapshot())).hexdigest()
        # A fixed reservation tariff based on capacity, not a CPU-time measurement.
        amount = 1 if not self.enabled else 8 + 4 * self.width + 4 * self.max_statistics + 8 * self.max_features
        work = admission.work(f"feature-work-{step}", amount)
        if work.get("outcome") != "admit":
            return {"accepted": False, "reason_code": work.get("reason_code"),
                    "state_before": before, "state_after": before, "work": work, "events": []}
        prospective = copy.deepcopy(self)
        events = prospective._update(observation, reward, source_event_id)
        prospective._step += 1
        encoded = canonical(prospective.online_snapshot())
        allocated = prospective.allocated_bytes()
        allocation = admission.allocation(f"feature-state-{step}", allocated, len(encoded))
        if allocation.get("outcome") != "admit":
            return {"accepted": False, "reason_code": allocation.get("reason_code"),
                    "state_before": before, "state_after": before, "work": work,
                    "allocation": allocation, "events": [],
                    "proposed_serialized_bytes": len(encoded), "proposed_allocated_bytes": allocated}
        self.__dict__ = prospective.__dict__
        return {"accepted": True, "reason_code": "FEATURE_STATE_ADMITTED",
                "state_before": before, "state_after": hashlib.sha256(encoded).hexdigest(),
                "work": work, "allocation": allocation, "events": events,
                "serialized_bytes": len(encoded), "allocated_bytes": allocated,
                "feature_count": len(self._features), "statistics_count": len(self._statistics)}

    def _validate(self, step: int, observation: tuple[int, ...], reward: int, event_id: str) -> None:
        if type(step) is not int or step != self._step or not 0 <= step < 2**32:
            raise CycleError("FEATURE_DISCOVERY_STEP_INVALID")
        if (type(observation) is not tuple or len(observation) != self.width
                or any(type(value) is not int or not 0 <= value < 2**31 for value in observation)):
            raise CycleError("FEATURE_DISCOVERY_OBSERVATION_INVALID")
        if type(reward) is not int or not -(2**31) <= reward < 2**31:
            raise CycleError("FEATURE_DISCOVERY_REWARD_INVALID")
        try:
            parsed = uuid.UUID(event_id)
        except (ValueError, TypeError, AttributeError) as error:
            raise CycleError("FEATURE_DISCOVERY_EVENT_INVALID") from error
        if parsed.int == 0 or str(parsed) != event_id:
            raise CycleError("FEATURE_DISCOVERY_EVENT_INVALID")

    def _update(self, observation: tuple[int, ...], reward: int, event_id: str) -> list[dict[str, object]]:
        if not self.enabled:
            return []
        eligible: list[tuple[int, int]] = []
        for coordinate, value in enumerate(observation):
            key = (coordinate, value)
            previous = self._statistics.get(key)
            if previous is None:
                if len(self._statistics) >= self.max_statistics:
                    continue
                current = Exposure(1, reward, self._step, self._step, event_id, event_id)
            else:
                current = replace(previous, count=previous.count + 1, reward_sum=previous.reward_sum + reward,
                                  last_step=self._step, last_event_id=event_id)
            self._statistics[key] = current
            if current.reward_sum > 0 and (current.count == 2 or current.count % 4 == 0):
                eligible.append(key)
        eligible.sort(key=lambda key: hashlib.sha256(canonical([self.seed, self._step, key])).digest())
        for key in eligible:
            feature_id = str(uuid.uuid5(uuid.NAMESPACE_URL, f"bonsai-feature:{self.seed}:{key[0]}:{key[1]}"))
            if feature_id not in self._versions and len(self._features) >= self.max_features:
                continue
            return [self._candidate(feature_id, key)]
        return []

    def _candidate(self, feature_id: str, key: tuple[int, int]) -> dict[str, object]:
        exposure = self._statistics[key]
        representation = (0, key[0], key[1], exposure.reward_sum, exposure.count)
        old = self._versions.get(feature_id)
        version = 1 if old is None else old[0] + 1
        revision_id = str(uuid.uuid5(uuid.UUID(feature_id), str(version)))
        if old is None:
            record = super().produce(feature_id, representation, self._step)
            sequence = 1
        else:
            record = super().revise(feature_id, representation, self._step)
            sequence = old[2] + 1
        # The old legality fixtures retain their original counter. Online records use bytes.
        self._features[feature_id] = replace(record, bytes=len(canonical(representation)))
        self._versions[feature_id] = (version, revision_id, sequence + 1)
        return {
            "artifact_id": feature_id, "artifact_revision_id": revision_id,
            "lifecycle_sequence": sequence, "kind": "birth" if old is None else "revision",
            "previous_revision_id": None if old is None else old[1],
            "representation": list(representation),
            "representation_sha256": hashlib.sha256(canonical(representation)).hexdigest(),
            "parents": [], "exposure": asdict(exposure),
            "provenance": {"producer_id": "bonsai-online-features", "producer_version": "1.0.0",
                           "source_event_ids": sorted({exposure.first_event_id, exposure.last_event_id}),
                           "method_ids": ["observed-equality-positive-support-v1"]},
            "utility": None, "parameter_count": 2,
            "representation_serialized_bytes": len(canonical(representation)),
        }

    def evaluate(self, observation: tuple[int, ...]) -> tuple[tuple[str, float], ...]:
        if len(observation) != self.width:
            raise CycleError("FEATURE_DISCOVERY_OBSERVATION_INVALID")
        return tuple((record.feature_id, record.representation[3] / record.representation[4]
                      if observation[record.representation[1]] == record.representation[2] else 0.0)
                     for record in self.snapshot())
