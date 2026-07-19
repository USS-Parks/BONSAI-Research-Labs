from __future__ import annotations

from bonsai_reference.control import ControlError, PrimitiveTabularControl, run_deterministic_bandit


def test_deterministic_fixture_learning_curve_and_accounting() -> None:
    expected = run_deterministic_bandit(10)
    for _ in range(100):
        assert run_deterministic_bandit(10) == expected
    assert expected.actions == (0, 1, 2, 2, 2, 2, 2, 2, 2, 2)
    assert expected.rewards == (0, 0, 1, 1, 1, 1, 1, 1, 1, 1)
    assert expected.cumulative_reward[-1] == 8
    assert expected.accounting.environment_steps == 10
    assert expected.accounting.updates == 10
    assert expected.accounting.parameter_touches == 20
    assert expected.accounting.work_items == 40
    assert expected.accounting.replay_items_retained == 0


def test_protocol_certification_is_strict_track_a() -> None:
    certification = PrimitiveTabularControl(3).certification
    assert certification.track == "track_a"
    assert certification.primitive_actions_only
    assert certification.batch_size == 1
    assert certification.replay_capacity == 0
    assert not certification.learned_features
    assert not certification.options
    assert not certification.planning
    assert not certification.privileged_diagnostic_input


def test_batch_one_protocol_rejects_out_of_order_calls() -> None:
    control = PrimitiveTabularControl(3)
    try:
        control.observe(1)
    except ControlError as error:
        assert error.code == "CONTROL_ACTION_REQUIRED"
    else:
        raise AssertionError("expected missing action")

    control.act((0,))
    try:
        control.act((0,))
    except ControlError as error:
        assert error.code == "CONTROL_UPDATE_REQUIRED"
    else:
        raise AssertionError("expected pending update")
