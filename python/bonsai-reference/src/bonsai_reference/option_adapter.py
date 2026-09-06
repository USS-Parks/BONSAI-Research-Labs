"""Learned temporal actions on the shared ordered BONSAI process protocol."""
from __future__ import annotations

import hashlib
import json
import uuid
from typing import Literal

from bonsai.adapter.v1 import adapter_pb2 as wire

from bonsai_reference.adapter_protocol import OrderedAdapter
from bonsai_reference.adapter_wire import decode_into, encode
from bonsai_reference.brdc1 import CycleError
from bonsai_reference.control import ControlError
from bonsai_reference.online_adapter import ACCOUNTING, OBSERVATION, run_adapter
from bonsai_reference.option_learning import OnlineOptionControl
from bonsai_reference.scenario import ScenarioError

TRANSITION = "bonsai.agent.causal-transition/v1"
ADMISSION = "bonsai.agent.admission/v2"


def tariffs(actions: int, width: int) -> dict[str, int]:
    return {"acting": actions + 4, "learning": 1, "feature_generation": 168 + 4 * width,
            "option_learning": 16 + 4 * (8 + 2 * actions + 2 * width)}


def capabilities() -> wire.CapabilityDeclaration:
    return wire.CapabilityDeclaration(
        reset=True, work=True, feedback=True, asynchronous_events=False,
        accepted_input_types=[OBSERVATION, TRANSITION, ACCOUNTING, ADMISSION], emitted_event_types=[],
        retains_transitions=False, offline_updates=False, observer_data_access=False,
        privileged_state_access=False, filesystem_read=True, filesystem_write=False, network_access=False,
    )


class GrantedAdmission:
    """Consume a single supervisor grant; the supervisor owns the budget decision."""

    def __init__(self, payload: bytes, step: int, expected: dict[str, int], retained: int, serialized: int) -> None:
        if len(payload) > 4096:
            raise ScenarioError("ADAPTER_ADMISSION_TOO_LARGE")
        try:
            raw: object = json.loads(payload)
        except (ValueError, UnicodeError) as error:
            raise ScenarioError("ADAPTER_ADMISSION_INVALID") from error
        wanted = {"schema": "bonsai.online-admission/v2", "step": step,
                  "work_per_step": [{"work_class": key, "amount": amount} for key, amount in expected.items()],
                  "retained_state_limit_bytes": retained, "serialized_state_limit_bytes": serialized}
        if (json.dumps(raw, sort_keys=True, separators=(",", ":"))
                != json.dumps(wanted, sort_keys=True, separators=(",", ":"))):
            raise ScenarioError("ADAPTER_ADMISSION_MISMATCH")
        self._expected = expected
        self._used: set[str] = set()
        self._allocations: set[str] = set()
        self._retained, self._serialized = retained, serialized
        self._step = step
        self._digest = hashlib.sha256(payload).hexdigest()

    def work(self, work_class: str, request_id: str, amount: int) -> dict[str, object]:
        if (work_class not in self._expected or work_class in self._used
                or type(amount) is not int or amount != self._expected[work_class]
                or not request_id or len(request_id.encode()) > 128):
            return {"outcome": "reject", "reason_code": "GRANTED_WORK_MISMATCH"}
        self._used.add(work_class)
        return {"outcome": "admit", "reason_code": "SUPERVISOR_WORK_GRANTED", "work_class": work_class,
                "request_id": request_id, "amount": amount, "step": self._step, "grant_sha256": self._digest}

    def allocation(self, request_id: str, allocated_bytes: int, serialized_bytes: int) -> dict[str, object]:
        if (not request_id or len(request_id.encode()) > 128 or request_id in self._allocations
                or len(self._allocations) >= 4
                or type(allocated_bytes) is not int or type(serialized_bytes) is not int
                or allocated_bytes <= 0 or serialized_bytes <= 0):
            return {"outcome": "reject", "reason_code": "GRANTED_ALLOCATION_INVALID"}
        self._allocations.add(request_id)
        admitted = allocated_bytes <= self._retained and serialized_bytes <= self._serialized
        return {"outcome": "admit" if admitted else "reject",
                "reason_code": "SUPERVISOR_STATE_GRANTED" if admitted else "GRANTED_STATE_LIMIT_EXCEEDED",
                "request_id": request_id, "allocated_bytes": allocated_bytes, "serialized_bytes": serialized_bytes,
                "retained_state_limit_bytes": self._retained, "serialized_state_limit_bytes": self._serialized,
                "step": self._step, "grant_sha256": self._digest}

    def complete(self) -> None:
        if self._used != set(self._expected) or not self._allocations:
            raise ScenarioError("ADAPTER_ADMISSION_NOT_CONSUMED")


class OptionAdapter(OrderedAdapter):
    def __init__(self, actions: int, width: int, seed: int, options_enabled: bool,
                 reward_mode: Literal["respecting", "oblivious"], retained: int, serialized: int,
                 configuration_sha256: bytes) -> None:
        super().__init__(capabilities(), configuration_sha256)
        self.control = OnlineOptionControl(actions, width, seed,
                                           options_enabled=options_enabled, reward_mode=reward_mode)
        self._actions, self._width = actions, width
        self._retained, self._serialized = retained, serialized
        self._step_index = 0
        self._pending: tuple[int, str] | None = None
        self._admission: GrantedAdmission | None = None

    def _reset(self, seed: int) -> None:
        if self._pending is not None or self._admission is not None:
            raise ScenarioError("ADAPTER_UPDATE_REQUIRED")
        try:
            self.control.reset_episode()
        except (CycleError, ControlError) as error:
            raise ScenarioError(error.code) from error
        self._step_index = 0

    def _step(self, request: wire.Step) -> wire.AdapterFrame:
        self._check_deadline(request.deadline_monotonic_ns)
        if self._pending is not None or self._admission is None or request.step_index != self._step_index:
            raise ScenarioError("ADAPTER_STEP_OUT_OF_ORDER")
        if request.input_type != OBSERVATION or hashlib.sha256(request.input).digest() != request.input_sha256:
            raise ScenarioError("ADAPTER_INPUT_INVALID")
        observation = wire.CausalObservation()
        decode_into(observation, request.input)
        if (observation.step != self._step_index or not observation.stream_id
                or len(observation.observation) != self._width
                or list(observation.allowed_actions) != list(range(self._actions))):
            raise ScenarioError("ADAPTER_OBSERVATION_INVALID")
        try:
            action = self.control.act(tuple(observation.observation), self._admission)
        except (CycleError, ControlError) as error:
            raise ScenarioError(error.code) from error
        self._pending = (action, observation.stream_id)
        result = encode(wire.CausalAction(step=self._step_index, action=action))
        return wire.AdapterFrame(step_result=wire.StepResult(
            step_index=self._step_index, action=result, action_sha256=hashlib.sha256(result).digest(),
        ))

    def _feedback(self, request: wire.Feedback) -> wire.AdapterFrame:
        self._check_deadline(request.deadline_monotonic_ns)
        if (self._pending is None or self._admission is None or request.signal_type != TRANSITION
                or len(request.feedback_id) != 16 or not any(request.feedback_id)
                or hashlib.sha256(request.signal).digest() != request.signal_sha256):
            raise ScenarioError("ADAPTER_FEEDBACK_INVALID")
        transition = wire.CausalTransition()
        decode_into(transition, request.signal)
        next_observation = transition.next
        if (transition.step != self._step_index or transition.action != self._pending[0]
                or not transition.HasField("next") or next_observation.step != self._step_index + 1
                or next_observation.stream_id != self._pending[1]
                or len(next_observation.observation) != self._width
                or (transition.terminated and transition.truncated)
                or list(next_observation.allowed_actions) != (
                    [] if transition.terminated or transition.truncated else list(range(self._actions)))):
            raise ScenarioError("ADAPTER_TRANSITION_INVALID")
        try:
            self.control.observe(transition.reward, tuple(next_observation.observation), transition.terminated,
                                 transition.truncated, str(uuid.UUID(bytes=request.feedback_id)), self._admission)
        except (CycleError, ControlError) as error:
            raise ScenarioError(error.code) from error
        self._admission.complete()
        self._admission, self._pending = None, None
        self._step_index += 1
        return wire.AdapterFrame(ack=wire.Ack(operation=wire.OPERATION_FEEDBACK))

    def _work(self, request: wire.Work) -> wire.AdapterFrame:
        self._check_deadline(request.deadline_monotonic_ns)
        if (self._pending is not None or len(request.work_item_id) != 16 or not any(request.work_item_id)
                or request.payload_sha256 != hashlib.sha256(request.payload).digest()):
            raise ScenarioError("ADAPTER_WORK_INVALID")
        if request.work_class == ADMISSION and self._admission is None:
            self._admission = GrantedAdmission(request.payload, self.control.accounting.environment_steps,
                                               tariffs(self._actions, self._width), self._retained, self._serialized)
            result, outcome = b"", "ADMITTED"
        elif request.work_class == ACCOUNTING and not request.payload and self._admission is None:
            measured = self.control.accounting
            result = encode(wire.PrimitiveAccounting(
                environment_steps=measured.environment_steps, updates=measured.updates,
                parameter_touches=measured.parameter_touches, work_items=measured.work_items,
                replay_items_retained=measured.replay_items_retained, parameter_update=self.control.parameter_update,
            ))
            outcome = "MEASURED"
        else:
            raise ScenarioError("ADAPTER_WORK_OUT_OF_ORDER")
        return wire.AdapterFrame(work_result=wire.WorkResult(
            work_item_id=request.work_item_id, outcome=outcome, result=result,
            result_sha256=hashlib.sha256(result).digest(),
        ))


def create(value: dict[str, object], digest: bytes) -> OptionAdapter:
    keys = {"action_count", "observation_width", "seed", "options_enabled", "reward_mode",
            "retained_state_limit_bytes", "serialized_state_limit_bytes"}
    actions, width, seed = value.get("action_count"), value.get("observation_width"), value.get("seed")
    enabled, mode = value.get("options_enabled"), value.get("reward_mode")
    retained, serialized = value.get("retained_state_limit_bytes"), value.get("serialized_state_limit_bytes")
    if (set(value) != keys or type(actions) is not int or not 2 <= actions <= 8
            or type(width) is not int or not 1 <= width <= 8 or type(seed) is not int or not 0 <= seed < 2**64
            or type(enabled) is not bool or mode not in ("respecting", "oblivious")
            or type(retained) is not int or not 1 <= retained <= 16 * 1024 * 1024
            or type(serialized) is not int or not 1 <= serialized <= 1024 * 1024):
        raise ScenarioError("AGENT_CONFIGURATION_INVALID")
    return OptionAdapter(actions, width, seed, enabled, mode,
                         retained, serialized, digest)


def main() -> int:
    return run_adapter(create)


if __name__ == "__main__":
    raise SystemExit(main())
