"""Online primitive tabular learner using the shared adapter lifecycle."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import cast

from bonsai.adapter.v1 import adapter_pb2 as wire

from bonsai_reference.adapter_protocol import MAXIMUM, OrderedAdapter, serve
from bonsai_reference.adapter_wire import decode_into, encode
from bonsai_reference.control import ControlError, PrimitiveTabularControl
from bonsai_reference.scenario import ScenarioError

OBSERVATION = "bonsai.agent.observation/v1"
REWARD = "bonsai.agent.reward/v1"
ACCOUNTING = "bonsai.agent.accounting/v1"


def capabilities() -> wire.CapabilityDeclaration:
    return wire.CapabilityDeclaration(
        reset=True, work=True, feedback=True, asynchronous_events=False,
        accepted_input_types=[OBSERVATION, REWARD, ACCOUNTING], emitted_event_types=[],
        retains_transitions=False, offline_updates=False, observer_data_access=False,
        privileged_state_access=False, filesystem_read=True, filesystem_write=False, network_access=False,
    )


class PrimitiveAdapter(OrderedAdapter):
    def __init__(self, action_count: int, configuration_sha256: bytes) -> None:
        super().__init__(capabilities(), configuration_sha256)
        self.control = PrimitiveTabularControl(action_count)
        self._actions = action_count
        self._step_index = 0
        self._pending = False

    def _reset(self, seed: int) -> None:
        try:
            self.control.reset_episode()
        except ControlError as error:
            raise ScenarioError(error.code) from error
        self._step_index = 0
        self._pending = False

    def _step(self, request: wire.Step) -> wire.AdapterFrame:
        self._check_deadline(request.deadline_monotonic_ns)
        if self._pending or request.step_index != self._step_index:
            raise ScenarioError("ADAPTER_STEP_OUT_OF_ORDER")
        if request.input_type != OBSERVATION or hashlib.sha256(request.input).digest() != request.input_sha256:
            raise ScenarioError("ADAPTER_INPUT_INVALID")
        observation = wire.CausalObservation()
        decode_into(observation, request.input)
        if (observation.step != self._step_index or not observation.stream_id
                or not observation.observation or len(observation.observation) > 256
                or list(observation.allowed_actions) != list(range(self._actions))):
            raise ScenarioError("ADAPTER_OBSERVATION_INVALID")
        action = self.control.act(tuple(observation.observation))
        self._pending = True
        payload = encode(wire.CausalAction(step=self._step_index, action=action))
        return wire.AdapterFrame(step_result=wire.StepResult(
            step_index=self._step_index, action=payload, action_sha256=hashlib.sha256(payload).digest(),
        ))

    def _feedback(self, request: wire.Feedback) -> wire.AdapterFrame:
        self._check_deadline(request.deadline_monotonic_ns)
        if (not self._pending or request.signal_type != REWARD
                or len(request.feedback_id) != 16 or not any(request.feedback_id)
                or hashlib.sha256(request.signal).digest() != request.signal_sha256):
            raise ScenarioError("ADAPTER_FEEDBACK_INVALID")
        reward = wire.PrimitiveReward()
        decode_into(reward, request.signal)
        if reward.step != self._step_index:
            raise ScenarioError("ADAPTER_FEEDBACK_OUT_OF_ORDER")
        self.control.observe(reward.reward)
        self._pending = False
        self._step_index += 1
        return wire.AdapterFrame(ack=wire.Ack(operation=wire.OPERATION_FEEDBACK))

    def _work(self, request: wire.Work) -> wire.AdapterFrame:
        self._check_deadline(request.deadline_monotonic_ns)
        if (self._pending or request.work_class != ACCOUNTING or request.payload
                or len(request.work_item_id) != 16 or not any(request.work_item_id)
                or request.payload_sha256 != hashlib.sha256(b"").digest()):
            raise ScenarioError("ADAPTER_ACCOUNTING_QUERY_INVALID")
        measured = self.control.accounting
        result = encode(wire.PrimitiveAccounting(
            environment_steps=measured.environment_steps, updates=measured.updates,
            parameter_touches=measured.parameter_touches, work_items=measured.work_items,
            replay_items_retained=measured.replay_items_retained,
        ))
        return wire.AdapterFrame(work_result=wire.WorkResult(
            work_item_id=request.work_item_id, outcome="MEASURED", result=result,
            result_sha256=hashlib.sha256(result).digest(),
        ))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bonsai-input", action="append", required=True)
    parser.add_argument("--bonsai-work-dir", type=Path, required=True)
    arguments = parser.parse_args()
    if len(arguments.bonsai_input) != 1 or not arguments.bonsai_input[0].startswith("configuration="):
        raise ScenarioError("AGENT_CONFIGURATION_INVALID")
    path = Path(arguments.bonsai_input[0].split("=", 1)[1])
    with path.open("rb") as source:
        payload = source.read(MAXIMUM + 1)
    if len(payload) > MAXIMUM:
        raise ScenarioError("AGENT_CONFIGURATION_TOO_LARGE")
    raw: object = json.loads(payload)
    if not isinstance(raw, dict):
        raise ScenarioError("AGENT_CONFIGURATION_INVALID")
    value = cast(dict[str, object], raw)
    action_count = value.get("action_count")
    if (set(value) != {"action_count"}
            or type(action_count) is not int or not 2 <= action_count <= 256):
        raise ScenarioError("AGENT_CONFIGURATION_INVALID")
    return serve(PrimitiveAdapter(action_count, hashlib.sha256(payload).digest()))


if __name__ == "__main__":
    raise SystemExit(main())
