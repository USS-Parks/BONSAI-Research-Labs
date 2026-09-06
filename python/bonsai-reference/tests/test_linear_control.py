from __future__ import annotations

import json

import pytest
from bonsai_reference.control import ControlError
from bonsai_reference.linear_control import LinearControl


def test_linear_normalized_gradient_updates_only_selected_weights() -> None:
    control = LinearControl(3, 2)
    assert control.act((3, 0)) == 0
    control.observe(1)
    assert control.weights[0] == pytest.approx((0.16, 0.12, 0.0))
    assert control.weights[1:] == ((0.0, 0.0, 0.0), (0.0, 0.0, 0.0))
    update = json.loads(control.parameter_update)
    assert update["before"] == [0.0, 0.0, 0.0]
    assert update["after"] == pytest.approx([0.16, 0.12, 0.0])
    assert update["features"] == [1.0, 0.75, 0.0]
    assert control.accounting.parameter_touches == 3
    assert control.accounting.work_items == 4


def test_linear_rejections_preserve_pending_update_and_weights() -> None:
    control = LinearControl(2, 1)
    with pytest.raises(ControlError, match="CONTROL_OBSERVATION_INVALID"):
        control.act((1, 2))
    assert control.accounting.work_items == 0
    control.act((0,))
    with pytest.raises(ControlError, match="CONTROL_UPDATE_REQUIRED"):
        control.reset_episode()
    with pytest.raises(ControlError, match="CONTROL_REWARD_INVALID"):
        control.observe(2**63)
    assert control.weights == ((0.0, 0.0), (0.0, 0.0))
    control.observe(1)
    assert control.weights[0] == (0.25, 0.0)
    with pytest.raises(ControlError, match="CONTROL_ACTION_REQUIRED"):
        control.observe(1)
    control.reset_episode()
    assert control.weights[0] == (0.25, 0.0)


def test_linear_storage_shape_and_repeated_results_are_independent_of_history_length() -> None:
    controls = [LinearControl(3, 4), LinearControl(3, 4)]
    for step in range(1_000):
        observation = (step, step % 7, 0, step % 3)
        actions = [control.act(observation) for control in controls]
        assert actions[0] == actions[1]
        for control in controls:
            control.observe(int(actions[0] == step % 3))
            assert len(control.weights) == 3
            assert all(len(row) == 5 for row in control.weights)
            assert len(control.parameter_update) < 2_048
        assert controls[0].weights == controls[1].weights
        assert controls[0].parameter_update == controls[1].parameter_update
    assert controls[0].accounting.updates == 1_000
    assert controls[0].accounting.parameter_touches == 5_000
    assert controls[0].accounting.replay_items_retained == 0
