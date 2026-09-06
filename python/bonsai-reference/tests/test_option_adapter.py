from __future__ import annotations

import hashlib
import json
from pathlib import Path

import pytest
from bonsai.adapter.v1 import adapter_pb2 as wire
from bonsai_reference.adapter_wire import decode_into, encode, message_kind
from bonsai_reference.environment_adapter import ACT
from bonsai_reference.option_adapter import ACCOUNTING, ADMISSION, OBSERVATION, TRANSITION, tariffs
from test_environment_adapter import ROOT, LiveEnvironment


def agent_config(enabled: bool = True, mode: str = "respecting", retained: int = 1024 * 1024) -> dict[str, object]:
    return {"action_count": 2, "observation_width": 1, "seed": 7, "options_enabled": enabled,
            "reward_mode": mode, "retained_state_limit_bytes": retained, "serialized_state_limit_bytes": 256 * 1024}


def launch_agent(tmp_path: Path, config: dict[str, object]) -> LiveEnvironment:
    path = tmp_path / "agent.json"
    path.write_text(json.dumps(config), encoding="utf-8")
    return LiveEnvironment(path, ["-I", "-B", str(ROOT / "scripts/adapter_entrypoint.py"), "option_adapter",
                                  "--bonsai-input", "configuration=" + str(path), "--bonsai-work-dir", str(tmp_path)])


def grant(agent: LiveEnvironment, total: int, retained: int = 1024 * 1024) -> wire.AdapterFrame:
    payload = json.dumps({"schema": "bonsai.online-admission/v2", "step": total,
                          "work_per_step": [
                              {"work_class": key, "amount": amount} for key, amount in tariffs(2, 1).items()
                          ],
                          "retained_state_limit_bytes": retained, "serialized_state_limit_bytes": 256 * 1024}).encode()
    return agent.exchange(wire.AdapterFrame(work=wire.Work(
        work_item_id=(total + 1).to_bytes(16, "little"), work_class=ADMISSION, payload=payload,
        payload_sha256=hashlib.sha256(payload).digest(), deadline_monotonic_ns=agent.deadline(),
    )))


def accounting(agent: LiveEnvironment, total: int) -> wire.PrimitiveAccounting:
    result = agent.exchange(wire.AdapterFrame(work=wire.Work(
        work_item_id=(total + 1).to_bytes(16, "little"), work_class=ACCOUNTING,
        payload_sha256=hashlib.sha256(b"").digest(), deadline_monotonic_ns=agent.deadline(),
    )))
    assert result.work_result.outcome == "MEASURED"
    assert result.work_result.result_sha256 == hashlib.sha256(result.work_result.result).digest()
    measured = wire.PrimitiveAccounting()
    decode_into(measured, result.work_result.result)
    return measured


@pytest.mark.parametrize("enabled,mode", [(True, "respecting"), (False, "respecting"), (True, "oblivious")])
def test_two_live_children_learn_and_execute_options_with_separate_rewards(
    tmp_path: Path, enabled: bool, mode: str,
) -> None:
    spec = {"scenario_id": "feature-attainment-chain", "version": "1.0", "seed": 7, "horizon": 24,
            "action_count": 2, "observation_width": 1, "size": 5, "reward_state": 4, "goal_terminates": False}
    path = tmp_path / "environment.json"
    path.write_text(json.dumps(spec), encoding="utf-8")
    environment = LiveEnvironment(path, ["-I", "-B", str(ROOT / "scripts/adapter_entrypoint.py"),
                                          "chain_adapter", "--spec", str(path)])
    agent = launch_agent(tmp_path, agent_config(enabled, mode))
    totals = tariffs(2, 1)
    executions: dict[str, list[int]] = {}
    durations: set[int] = set()
    learned_terminations = 0
    births = 0
    changed_policy = 0
    try:
        environment.configure()
        agent.configure()
        observed = wire.CausalObservation()
        for total in range(96):
            if total % 24 == 0:
                environment.reset()
                agent.reset()
                observed = environment.observe()
                assert list(observed.observation) == [42 % 4]
            assert grant(agent, total).work_result.outcome == "ADMITTED"
            result = agent.step(OBSERVATION, encode(observed))
            assert message_kind(result) == "step_result", result
            action = wire.CausalAction()
            decode_into(action, result.step_result.action)
            response = environment.step(ACT, encode(action))
            assert message_kind(response) == "step_result", response
            transition = wire.CausalTransition()
            decode_into(transition, response.step_result.action)
            expected = (observed.observation[0] + (1 if action.action == 0 else -1)) % 5
            assert list(transition.next.observation) == [expected]
            assert transition.reward == (4 if expected == 4 else -1)
            assert transition.truncated == (total % 24 == 23)
            assert not transition.terminated
            signal = encode(transition)
            feedback = agent.exchange(wire.AdapterFrame(feedback=wire.Feedback(
                feedback_id=(total + 1).to_bytes(16, "little"), signal_type=TRANSITION, signal=signal,
                signal_sha256=hashlib.sha256(signal).digest(), deadline_monotonic_ns=agent.deadline(),
            )))
            assert feedback.ack.operation == wire.OPERATION_FEEDBACK, feedback
            measured = accounting(agent, total)
            update = json.loads(measured.parameter_update)
            assert measured.environment_steps == measured.updates == total + 1
            assert measured.work_items == (total + 1) * sum(totals.values())
            assert measured.replay_items_retained == 0
            assert update["work_by_class"] == {key: (total + 1) * value for key, value in totals.items()}
            assert update["reward"] == update["details"]["rewards"]["environment"] == transition.reward
            details = update["details"]
            assert details["options_enabled"] is enabled and details["reward_mode"] == mode
            births += sum(event["kind"] == "construction" for event in details["option_construction"])
            changed_policy += sum(delta["before"] != delta["after"] for delta in details["option_q_deltas"])
            for reward in details["rewards"]["subproblem_returns"]:
                assert reward["environment_reward"] == transition.reward
                assert reward["original_reward_term"] == (transition.reward if mode == "respecting" else 0)
                assert reward["return"] == reward["original_reward_term"] + reward["stopping_bonus_term"]
            for event in details["option_execution_events"]:
                if event["kind"] == "initiate":
                    assert enabled and event["invocation_id"] not in executions
                    executions[event["invocation_id"]] = []
                elif event["kind"] == "action" and event["invocation_id"] is not None:
                    rewards = executions[event["invocation_id"]]
                    assert event["action"] == transition.action
                    assert event["duration"] == len(rewards) + 1
                    rewards.append(transition.reward)
                elif event["kind"] == "terminate":
                    rewards = executions[event["invocation_id"]]
                    assert event["duration"] == len(rewards) and event["environment_return"] == sum(rewards)
                    assert 1 <= event["duration"] <= 16
                    durations.add(event["duration"])
                    if event["reason"] == "learned_beta":
                        learned_terminations += 1
                        assert expected == 4 and event["attained"]
            observed = transition.next
        assert births > 0 and changed_policy > 0
        assert learned_terminations > 0 if enabled else not executions
        if enabled:
            assert len(durations) >= 2 and max(durations) > 1
        agent.stop()
        environment.stop()
    finally:
        agent.close()
        environment.close()
    assert agent.child.returncode == environment.child.returncode == 0
    assert sorted(item.name for item in tmp_path.iterdir()) == ["agent.json", "environment.json"]


@pytest.mark.parametrize("case", ["missing_grant", "wrong_grant", "memory_denial"])
def test_live_option_admission_refuses_unapproved_work_and_reaps(tmp_path: Path, case: str) -> None:
    retained = 1 if case == "memory_denial" else 1024 * 1024
    agent = launch_agent(tmp_path, agent_config(retained=retained))
    try:
        agent.configure()
        agent.reset()
        if case == "wrong_grant":
            response = grant(agent, 1)
            assert response.error.reason_code == "ADAPTER_ADMISSION_MISMATCH"
        else:
            if case == "memory_denial":
                assert grant(agent, 0, retained).work_result.outcome == "ADMITTED"
            observed = encode(wire.CausalObservation(stream_id="live-boundary", step=0,
                                                     observation=[2], allowed_actions=[0, 1]))
            response = agent.step(OBSERVATION, observed)
            assert message_kind(response) == "error"
            assert response.error.reason_code == (
                "GRANTED_STATE_LIMIT_EXCEEDED" if case == "memory_denial" else "ADAPTER_STEP_OUT_OF_ORDER")
        assert len(encode(response)) < 256
        assert agent.child.wait(timeout=2) == 2
    finally:
        agent.close()


@pytest.mark.parametrize("fault", ["action", "stream", "step", "hash", "actions"])
def test_live_option_feedback_rejects_contradictory_causality_and_reaps(tmp_path: Path, fault: str) -> None:
    agent = launch_agent(tmp_path, agent_config())
    try:
        agent.configure()
        agent.reset()
        assert grant(agent, 0).work_result.outcome == "ADMITTED"
        observation = wire.CausalObservation(stream_id="causal-boundary", step=0,
                                             observation=[2], allowed_actions=[0, 1])
        response = agent.step(OBSERVATION, encode(observation))
        action = wire.CausalAction()
        decode_into(action, response.step_result.action)
        transition = wire.CausalTransition(step=0, action=action.action, reward=-1,
            next=wire.CausalObservation(stream_id="causal-boundary", step=1,
                                       observation=[3], allowed_actions=[0, 1]))
        if fault == "action":
            transition.action = 1 - action.action
        elif fault == "stream":
            transition.next.stream_id = "other-stream"
        elif fault == "step":
            transition.next.step = 2
        elif fault == "actions":
            transition.next.allowed_actions[:] = []
        signal = encode(transition)
        response = agent.exchange(wire.AdapterFrame(feedback=wire.Feedback(
            feedback_id=(1).to_bytes(16, "little"), signal_type=TRANSITION, signal=signal,
            signal_sha256=b"x" * 32 if fault == "hash" else hashlib.sha256(signal).digest(),
            deadline_monotonic_ns=agent.deadline(),
        )))
        assert message_kind(response) == "error"
        assert response.error.reason_code in {"ADAPTER_TRANSITION_INVALID", "ADAPTER_FEEDBACK_INVALID"}
        assert len(encode(response)) < 256
        assert agent.child.wait(timeout=2) == 2
    finally:
        agent.close()
