from __future__ import annotations

import copy
import json
import uuid
from typing import cast

import pytest
from bonsai_reference.brdc1 import CycleError
from bonsai_reference.feature_discovery import canonical
from bonsai_reference.option_learning import (
    FEATURE_STATISTICS,
    MAX_DURATION,
    MAX_OPTIONS,
    MAX_RETAINED_BYTES,
    MAX_SERIALIZED_BYTES,
    MAX_STATES,
    MAX_UPDATE_BYTES,
    OnlineOptionControl,
    RewardMode,
)


class Admission:
    def __init__(
        self,
        *,
        deny_work_class: str | None = None,
        deny_allocation_prefix: str | None = None,
    ) -> None:
        self.deny_work_class = deny_work_class
        self.deny_allocation_prefix = deny_allocation_prefix
        self.work_requests: list[tuple[str, str, int]] = []
        self.allocation_requests: list[tuple[str, int, int]] = []

    def work(self, work_class: str, request_id: str, amount: int) -> dict[str, object]:
        self.work_requests.append((work_class, request_id, amount))
        accepted = work_class != self.deny_work_class
        return {
            "accepted": accepted,
            "outcome": "admit" if accepted else "reject",
            "reason_code": None if accepted else f"{work_class.upper()}_DENIED",
        }

    def allocation(
        self, request_id: str, allocated_bytes: int, serialized_bytes: int
    ) -> dict[str, object]:
        self.allocation_requests.append((request_id, allocated_bytes, serialized_bytes))
        accepted = self.deny_allocation_prefix is None or not request_id.startswith(
            self.deny_allocation_prefix
        )
        return {
            "accepted": accepted,
            "outcome": "admit" if accepted else "reject",
            "reason_code": None if accepted else "STATE_DENIED",
        }


def event_id(step: int) -> str:
    return str(uuid.uuid5(uuid.NAMESPACE_URL, f"option-test-event:{step}"))


def transition(
    control: OnlineOptionControl,
    admission: Admission,
    step: int,
    observation: tuple[int, ...],
    next_observation: tuple[int, ...],
    reward: int,
    *,
    terminated: bool = False,
    truncated: bool = False,
) -> dict[str, object]:
    action = control.act(observation, admission)
    audit = control.observe(
        reward,
        next_observation,
        terminated,
        truncated,
        event_id(step),
        admission,
    )
    assert audit["action"] == action
    return audit


TRACE = (
    ((0, 0), (1, 10), 1),
    ((1, 10), (0, 11), 0),
    ((0, 11), (1, 12), 1),
    ((1, 12), (0, 13), 0),
    ((0, 13), (1, 20), 1),
    ((1, 20), (0, 14), 0),
    ((0, 14), (1, 30), 1),
    ((1, 30), (0, 15), 0),
    ((0, 15), (1, 30), 1),
    ((1, 30), (0, 16), 0),
    ((0, 16), (1, 30), 1),
)


def run_trace(
    *, options_enabled: bool = True, reward_mode: RewardMode = "respecting"
) -> tuple[OnlineOptionControl, Admission, list[dict[str, object]]]:
    control = OnlineOptionControl(
        2,
        2,
        17,
        options_enabled=options_enabled,
        reward_mode=reward_mode,
    )
    admission = Admission()
    audits = [
        transition(control, admission, step, observation, next_observation, reward)
        for step, (observation, next_observation, reward) in enumerate(TRACE)
    ]
    return control, admission, audits


def execution_events(audits: list[dict[str, object]]) -> list[dict[str, object]]:
    values: list[dict[str, object]] = []
    for audit in audits:
        details = cast(dict[str, object], audit["details"])
        values.extend(cast(list[dict[str, object]], details["option_execution_events"]))
    return values


def test_configuration_is_frozen_and_tariffs_are_capacity_reservations() -> None:
    control = OnlineOptionControl(3, 2, 7)
    assert (control.max_states, control.max_options, control.max_duration) == (
        MAX_STATES,
        MAX_OPTIONS,
        MAX_DURATION,
    )
    assert control.acting_tariff == 7
    assert control.feature_generation_tariff == 176
    assert control.option_learning_tariff == 88
    with pytest.raises(AttributeError):
        control.max_states = 1  # type: ignore[misc]
    with pytest.raises(CycleError, match="ONLINE_OPTION_CONFIGURATION_INVALID"):
        OnlineOptionControl(3, 2, 7, max_states=63)


def test_options_are_learned_online_and_execute_at_variable_durations() -> None:
    control, _, audits = run_trace()
    snapshot = control.online_snapshot()
    options = cast(list[dict[str, object]], snapshot["options"])
    assert 1 <= len(options) <= MAX_OPTIONS
    option = next(record for record in options if record["completed_executions"] == 2)
    assert cast(int, option["q_updates"]) > 0
    assert cast(list[dict[str, object]], option["q_values"])
    assert cast(list[dict[str, object]], option["beta"])
    assert option["completed_executions"] == 2
    assert option["successful_executions"] == 2
    assert option["cumulative_duration"] == 5

    events = execution_events(audits)
    initiations = [event for event in events if event["kind"] == "initiate"]
    terminations = [event for event in events if event["kind"] == "terminate"]
    assert len(initiations) == len(terminations) == 2
    assert {event["duration"] for event in terminations} == {1, 4}
    assert {event["reason"] for event in terminations} == {"learned_beta"}
    assert {event["invocation_id"] for event in initiations} == {
        event["invocation_id"] for event in terminations
    }


def test_full_state_beta_requires_two_samples_before_termination() -> None:
    _, _, audits = run_trace()
    first_unseen = cast(dict[str, object], audits[6]["details"])
    first_beta = cast(list[dict[str, object]], first_unseen["option_beta_deltas"])[0]
    first_events = cast(list[dict[str, object]], first_unseen["option_execution_events"])
    assert first_beta["observation"] == [1, 30]
    assert first_beta["after"] == [1, 1]
    assert not first_beta["terminates"]
    assert first_events[-1]["kind"] == "continue"

    second_sample = cast(dict[str, object], audits[8]["details"])
    second_beta = cast(list[dict[str, object]], second_sample["option_beta_deltas"])[0]
    second_events = cast(list[dict[str, object]], second_sample["option_execution_events"])
    assert second_beta["observation"] == [1, 30]
    assert second_beta["after"] == [2, 2]
    assert second_beta["terminates"]
    assert second_events[-1]["kind"] == "terminate"


def test_reward_oblivious_control_removes_only_original_reward_term() -> None:
    respecting, _, respecting_audits = run_trace(reward_mode="respecting")
    oblivious, _, oblivious_audits = run_trace(reward_mode="oblivious")
    respecting_details = cast(dict[str, object], respecting_audits[4]["details"])
    oblivious_details = cast(dict[str, object], oblivious_audits[4]["details"])
    respecting_return = cast(
        list[dict[str, object]], cast(dict[str, object], respecting_details["rewards"])["subproblem_returns"]
    )[0]
    oblivious_return = cast(
        list[dict[str, object]], cast(dict[str, object], oblivious_details["rewards"])["subproblem_returns"]
    )[0]
    assert respecting_return["environment_reward"] == oblivious_return["environment_reward"] == 1
    assert respecting_return["stopping_bonus_term"] == oblivious_return["stopping_bonus_term"] == 1
    assert respecting_return["original_reward_term"] == 1
    assert oblivious_return["original_reward_term"] == 0
    assert respecting_return["return"] == 2 and oblivious_return["return"] == 1
    assert respecting_details["track"] == oblivious_details["track"] == "track_a"
    assert respecting_details["reward_respecting_claim_eligible"] is True
    assert oblivious_details["reward_respecting_claim_eligible"] is False
    assert respecting.accounting == oblivious.accounting


def test_primitive_only_disables_initiation_but_keeps_auxiliary_updates_and_tariffs() -> None:
    enabled, enabled_admission, _ = run_trace(options_enabled=True)
    disabled, disabled_admission, audits = run_trace(options_enabled=False)
    option = cast(list[dict[str, object]], disabled.online_snapshot()["options"])[0]
    assert cast(int, option["q_updates"]) > 0
    assert cast(int, option["beta_updates"]) > 0
    assert option["executions"] == 0
    assert enabled.accounting == disabled.accounting
    assert enabled_admission.work_requests == disabled_admission.work_requests
    assert not any(event["kind"] == "initiate" for event in execution_events(audits))


def test_admission_denial_preserves_exact_learner_state_and_can_retry() -> None:
    control = OnlineOptionControl(2, 2, 17)
    before_act = canonical(control.online_snapshot())
    with pytest.raises(CycleError, match="ACTING_DENIED"):
        control.act((0, 0), Admission(deny_work_class="acting"))
    assert canonical(control.online_snapshot()) == before_act

    admission = Admission()
    control.act((0, 0), admission)
    before_feedback = canonical(control.online_snapshot())
    before_update = control.parameter_update
    with pytest.raises(CycleError, match="OPTION_LEARNING_DENIED"):
        control.observe(1, (1, 10), False, False, event_id(0), Admission(deny_work_class="option_learning"))
    assert canonical(control.online_snapshot()) == before_feedback
    assert control.parameter_update == before_update
    audit = control.observe(1, (1, 10), False, False, event_id(0), Admission())
    assert audit["update"] == 1


def test_audit_is_exact_bounded_parameter_update_with_reconstructible_accounting() -> None:
    control, admission, audits = run_trace()
    audit = audits[-1]
    assert json.loads(control.parameter_update) == audit
    assert canonical(audit) == control.parameter_update
    assert len(control.parameter_update) <= MAX_UPDATE_BYTES
    assert set(audit) == {
        "schema",
        "update",
        "action",
        "reward",
        "work_by_class",
        "parameter_touches",
        "state_before",
        "state_after",
        "allocated_bytes",
        "serialized_bytes",
        "details",
    }
    work = cast(dict[str, int], audit["work_by_class"])
    assert work == {
        "acting": 11 * 6,
        "learning": 11,
        "feature_generation": 11 * 176,
        "option_learning": 11 * 80,
    }
    assert control.accounting.environment_steps == control.accounting.updates == 11
    assert control.accounting.work_items == sum(work.values())
    assert control.accounting.parameter_touches == audit["parameter_touches"]
    assert control.accounting.replay_items_retained == 0
    causal = cast(dict[str, object], cast(dict[str, object], audit["details"])["causal_transition"])
    assert causal == {
        "observation": [0, 16],
        "action": audit["action"],
        "reward": 1,
        "next_observation": [1, 30],
        "terminated": False,
        "truncated": False,
        "source_event_id": event_id(10),
    }
    assert audit["serialized_bytes"] == len(canonical(control.online_snapshot()))
    assert audit["allocated_bytes"] == control.allocated_bytes()
    assert admission.work_requests[:4] == [
        ("acting", "option-acting-0", 6),
        ("learning", "learning-0", 1),
        ("feature_generation", "feature-work-0", 176),
        ("option_learning", "option-learning-0", 80),
    ]


def test_new_option_has_no_supplied_policy_or_termination_parameters() -> None:
    control = OnlineOptionControl(2, 2, 17)
    admission = Admission()
    audits: list[dict[str, object]] = []
    for step, (observation, next_observation, reward) in enumerate(TRACE[:3]):
        audits.append(transition(control, admission, step, observation, next_observation, reward))
    option = cast(list[dict[str, object]], control.online_snapshot()["options"])[0]
    assert option["q_values"] == []
    assert option["beta"] == []
    assert option["executions"] == 0
    construction = cast(
        list[dict[str, object]], cast(dict[str, object], audits[-1]["details"])["option_construction"]
    )[0]
    assert construction["kind"] == "construction"
    assert construction["parents"] == [construction["feature_revision_id"]]


def test_feature_revision_lineage_is_exact_in_audit_and_bounded_in_state() -> None:
    control, _, audits = run_trace()
    maintenance = [
        event
        for audit in audits
        for event in cast(
            list[dict[str, object]],
            cast(dict[str, object], audit["details"])["option_construction"],
        )
        if event["kind"] == "maintenance"
    ]
    assert maintenance
    for event in maintenance:
        assert event["parents"] == [event["feature_revision_id"]]
    subproblems = cast(list[dict[str, object]], control.online_snapshot()["subproblems"])
    revised = next(record for record in subproblems if cast(int, record["feature_revision_count"]) > 1)
    assert revised["feature_birth_revision_id"] != revised["feature_latest_revision_id"]
    assert "feature_revision_ids" not in revised


def test_state_tables_stop_at_capacity_without_replay_or_hidden_growth() -> None:
    control = OnlineOptionControl(2, 2, 9, options_enabled=False)
    admission = Admission()
    last: dict[str, object] = {}
    for step in range(MAX_STATES + 6):
        last = transition(
            control,
            admission,
            step,
            (step, step + 100),
            (step + 1, step + 101),
            0,
        )
    snapshot = control.online_snapshot()
    assert len(cast(list[dict[str, object]], snapshot["primitive"])) == MAX_STATES
    assert snapshot["replay_items_retained"] == 0
    details = cast(dict[str, object], last["details"])
    primitive = cast(dict[str, object], details["primitive_delta"])
    assert primitive["applied"] is False and primitive["reason"] == "state_capacity"
    assert control.accounting.parameter_touches == MAX_STATES * 2 + FEATURE_STATISTICS * 2
    assert len(control.parameter_update) <= MAX_UPDATE_BYTES


def test_declared_maximum_table_population_fits_storage_limits() -> None:
    control = OnlineOptionControl(8, 8, 23, options_enabled=False)
    admission = Admission()
    audit: dict[str, object] = {}
    for step in range(MAX_STATES * 10):
        state_index = step % MAX_STATES
        next_index = (step + 1) % MAX_STATES
        audit = transition(
            control,
            admission,
            step,
            (state_index, state_index // 8, 0, 1, 2, 3, 4, 5),
            (next_index, next_index // 8, 0, 1, 2, 3, 4, 5),
            1,
        )
    snapshot = control.online_snapshot()
    assert len(cast(list[dict[str, object]], snapshot["options"])) == MAX_OPTIONS
    assert audit["allocated_bytes"] == control.allocated_bytes() <= MAX_RETAINED_BYTES
    assert audit["serialized_bytes"] == len(canonical(snapshot)) <= MAX_SERIALIZED_BYTES
    assert len(control.parameter_update) <= MAX_UPDATE_BYTES


def test_snapshot_copy_cannot_mutate_pending_or_learned_state() -> None:
    control, _, _ = run_trace()
    before = canonical(control.online_snapshot())
    snapshot = control.online_snapshot()
    cast(list[dict[str, object]], snapshot["options"]).clear()
    cast(dict[str, int], snapshot["work_by_class"])["acting"] = 0
    assert canonical(control.online_snapshot()) == before
    assert copy.deepcopy(control.online_snapshot()) == control.online_snapshot()

    pending_control = OnlineOptionControl(2, 2, 5)
    pending_control.act((0, 0), Admission())
    pending_before = canonical(pending_control.online_snapshot())
    pending = cast(dict[str, object], pending_control.online_snapshot()["pending"])
    cast(list[dict[str, object]], pending["events"]).clear()
    assert canonical(pending_control.online_snapshot()) == pending_before


@pytest.mark.parametrize("work_class", ["learning", "feature_generation", "option_learning"])
def test_populated_feedback_work_denial_keeps_all_authoritative_state(work_class: str) -> None:
    control, _, _ = run_trace()
    control.act((0, 16), Admission())
    before = canonical(control.online_snapshot())
    previous_update = control.parameter_update
    with pytest.raises(CycleError):
        control.observe(0, (0, 16), False, False, event_id(11), Admission(deny_work_class=work_class))
    assert canonical(control.online_snapshot()) == before
    assert control.parameter_update == previous_update


@pytest.mark.parametrize("prefix", ["feature-state-", "option-state-"])
def test_populated_feedback_allocation_denial_keeps_all_authoritative_state(prefix: str) -> None:
    control, _, _ = run_trace()
    control.act((0, 16), Admission())
    before = canonical(control.online_snapshot())
    previous_update = control.parameter_update
    with pytest.raises(CycleError, match="STATE_DENIED"):
        control.observe(0, (0, 16), False, False, event_id(11), Admission(deny_allocation_prefix=prefix))
    assert canonical(control.online_snapshot()) == before
    assert control.parameter_update == previous_update


def test_populated_act_allocation_denial_keeps_invocation_and_accounting_unchanged() -> None:
    control, _, _ = run_trace()
    before = canonical(control.online_snapshot())
    with pytest.raises(CycleError, match="STATE_DENIED"):
        control.act((0, 16), Admission(deny_allocation_prefix="option-act-state-"))
    assert canonical(control.online_snapshot()) == before


@pytest.mark.parametrize("reason", ["duration_cap", "environment_terminated", "environment_truncated"])
def test_learned_execution_respects_duration_and_episode_end_boundaries(reason: str) -> None:
    control, _, _ = run_trace()
    admission = Admission()
    count = MAX_DURATION if reason == "duration_cap" else 2
    for index in range(count):
        audit = transition(control, admission, 11 + index, (0, 16), (0, 16), 0,
                           terminated=reason == "environment_terminated" and index + 1 == count,
                           truncated=reason == "environment_truncated" and index + 1 == count)
        events = execution_events([audit])
        if index + 1 < count:
            assert events[-1]["kind"] == "continue"
        else:
            assert events[-1]["kind"] == "terminate"
            assert events[-1]["reason"] == reason
            assert events[-1]["duration"] == count
            assert events[-1]["environment_return"] == 0
    assert control.online_snapshot()["active_execution"] is None
    control.reset_episode()
