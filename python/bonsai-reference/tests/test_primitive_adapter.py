from __future__ import annotations

import hashlib
import json
from pathlib import Path

import pytest
from bonsai.adapter.v1 import adapter_pb2 as wire
from bonsai_reference.adapter_wire import decode_into, encode
from bonsai_reference.primitive_adapter import ACCOUNTING, OBSERVATION, REWARD
from test_environment_adapter import ROOT, LiveEnvironment


@pytest.mark.parametrize("module", ["primitive_adapter", "linear_adapter"])
def test_live_online_learners_have_exact_accounting_and_retain_learning_on_reset(tmp_path: Path, module: str) -> None:
    configuration = tmp_path / "agent.json"
    config = {"action_count": 3}
    if module == "linear_adapter":
        config["observation_width"] = 1
    configuration.write_text(json.dumps(config), encoding="utf-8")
    agent = LiveEnvironment(configuration, [
        "-I", "-B", str(ROOT / "scripts/adapter_entrypoint.py"), module,
        "--bonsai-input", "configuration=" + str(configuration), "--bonsai-work-dir", str(tmp_path),
    ])
    try:
        agent.configure()
        actions: list[int] = []
        for episode in range(2):
            agent.reset()
            for step in range(6):
                observation = encode(wire.CausalObservation(
                    stream_id="constant-observation", step=step, observation=[0], allowed_actions=[0, 1, 2],
                ))
                response = agent.step(OBSERVATION, observation)
                action = wire.CausalAction()
                decode_into(action, response.step_result.action)
                assert action.step == step
                actions.append(action.action)
                reward = encode(wire.PrimitiveReward(step=step, reward=int(action.action == 2)))
                feedback = agent.exchange(wire.AdapterFrame(feedback=wire.Feedback(
                    feedback_id=bytes([step + 1] * 16), signal_type=REWARD, signal=reward,
                    signal_sha256=hashlib.sha256(reward).digest(), deadline_monotonic_ns=agent.deadline(),
                )))
                assert feedback.ack.operation == wire.OPERATION_FEEDBACK
            result = agent.exchange(wire.AdapterFrame(work=wire.Work(
                work_item_id=bytes([episode + 1] * 16), work_class=ACCOUNTING,
                payload_sha256=hashlib.sha256(b"").digest(), deadline_monotonic_ns=agent.deadline(),
            )))
            assert result.work_result.outcome == "MEASURED"
            assert result.work_result.result_sha256 == hashlib.sha256(result.work_result.result).digest()
            measured = wire.PrimitiveAccounting()
            decode_into(measured, result.work_result.result)
            count = 6 * (episode + 1)
            assert measured.environment_steps == measured.updates == count
            assert measured.parameter_touches == 2 * count
            assert measured.work_items == 4 * count
            assert measured.replay_items_retained == 0
            update = json.loads(measured.parameter_update)
            assert update["schema"] == "bonsai.parameter-update/v1"
            assert update["update"] == count
            assert update["before"] != update["after"]
        assert actions == [0, 1, *([2] * 10)]
        agent.stop()
    finally:
        agent.close()
    assert sorted(path.name for path in tmp_path.iterdir()) == ["agent.json"]


def test_pending_action_cannot_be_reset_or_counted_as_completed() -> None:
    from bonsai_reference.control import ControlError, PrimitiveTabularControl

    control = PrimitiveTabularControl(3)
    assert control.act((0,)) == 0
    try:
        control.reset_episode()
    except ControlError as error:
        assert error.code == "CONTROL_UPDATE_REQUIRED"
    else:
        raise AssertionError("pending transition was silently discarded")
    assert control.accounting.environment_steps == control.accounting.updates == 0
    assert control.accounting.work_items == 3
    control.observe(1)
    control.reset_episode()
    assert control.accounting.updates == 1
