use bonsai_contracts::resource::{BudgetScope, WorkClass};
use bonsai_governor::decision::{DecisionInput, DecisionPolicyReference, decide};
use bonsai_governor::enforcement::{HardControl, preflight_hard_controls};
use bonsai_governor::replay::replay_decisions;
use bonsai_governor::scheduler::{
    SchedulerError, SchedulerKind, dense_schedule, event_schedule, matched_budget_compare,
    validate_scheduler_trace,
};
use bonsai_governor::{CounterKey, LimitProjection, ScopeProjection, TypedAmount};
use bonsai_platform::capability::{CapabilityMatrix, HostClass, Support, control};
use bonsai_platform::linux::detect_linux_backend;
use bonsai_platform::windows::detect_windows_backend;

fn decision(id: &str, state: LimitProjection) -> DecisionInput {
    DecisionInput {
        decision_id: id.to_owned(),
        monotonic_time_ns: 100,
        policy: DecisionPolicyReference {
            policy_id: "policy-1".to_owned(),
            policy_version: "1.0".to_owned(),
            canonical_sha256: "11".repeat(32),
        },
        work_class: WorkClass::Acting,
        request: TypedAmount {
            key: CounterKey {
                counter_id: "cpu_time".to_owned(),
                unit: "ns".to_owned(),
            },
            amount: 4,
        },
        projections: vec![ScopeProjection {
            limit_id: "limit-1".to_owned(),
            scope: BudgetScope::PerStep,
            consumed_before: Some(4),
            requested: 4,
            soft_limit: 10,
            hard_limit: 20,
            projected: Some(8),
            state,
        }],
        next_rolling_release_ns: None,
    }
}

#[test]
fn unsupported_hard_controls_reject_before_track_a() {
    let windows = detect_windows_backend();
    let rejected = preflight_hard_controls(
        &windows,
        &[HardControl {
            control_id: "job.memory".to_owned(),
            required: true,
        }],
    )
    .expect("preflight");
    assert!(!rejected.admitted);
    assert_eq!(rejected.reason_code, "ENFORCEMENT_HARD_CONTROL_UNSUPPORTED");

    let linux = detect_linux_backend();
    let linux_result = preflight_hard_controls(
        &linux,
        &[HardControl {
            control_id: "cgroup.memory.max".to_owned(),
            required: true,
        }],
    )
    .expect("linux");
    assert_eq!(
        linux_result.admitted,
        linux.hard_limit_supported("cgroup.memory.max")
    );
}

#[test]
fn measurement_support_does_not_admit_hard_preflight() {
    let matrix = CapabilityMatrix::assembled(
        "fixture-backend",
        "linux",
        HostClass::Container,
        vec![control(
            "cgroup.cpu.stat",
            Support::Supported,
            "MEASURE_ONLY",
        )],
        vec![control(
            "cgroup.memory.max",
            Support::NoPermission,
            "CGROUP_HARD_LIMIT_UNIMPLEMENTED",
        )],
        Support::Supported,
        "fixture",
    );
    let measurement = preflight_hard_controls(
        &matrix,
        &[HardControl {
            control_id: "cgroup.cpu.stat".to_owned(),
            required: true,
        }],
    )
    .expect("measurement");
    assert!(!measurement.admitted);
    assert_eq!(
        measurement.reason_code,
        "ENFORCEMENT_HARD_CONTROL_UNSUPPORTED"
    );

    let limit = preflight_hard_controls(
        &matrix,
        &[HardControl {
            control_id: "cgroup.memory.max".to_owned(),
            required: true,
        }],
    )
    .expect("limit");
    assert_eq!(
        limit.admitted,
        matrix.hard_limit_supported("cgroup.memory.max")
    );
}

#[test]
fn decision_replay_matches_and_detects_policy_divergence() {
    let first = decision("d1", LimitProjection::WithinSoft);
    let recorded = vec![decide(decision("d1", LimitProjection::WithinSoft)).expect("decide")];
    let matched = replay_decisions(&[first], &recorded).expect("replay");
    assert!(matched.matched);

    let mut altered = decision("d1", LimitProjection::WithinSoft);
    altered.policy.policy_id = "policy-altered".to_owned();
    let diverged = replay_decisions(&[altered], &recorded).expect("diverge");
    assert!(!diverged.matched);
    assert_eq!(diverged.divergences, vec!["d1".to_owned()]);
}

#[test]
fn dense_and_event_schedulers_compare_under_matched_streams() {
    let components = ["acting", "learning", "observer"];
    let dense = dense_schedule("stream-a", 7, 100, &components, &[1, 1, 1]).expect("dense");
    assert_eq!(dense.kind, SchedulerKind::Dense);
    assert!(dense.events.iter().all(|event| event.suppressed.is_empty()));
    validate_scheduler_trace(&dense).expect("dense valid");

    let event = event_schedule(
        "stream-a",
        7,
        100,
        &["acting", "observer"],
        &["learning"],
        &[1, 1, 1],
    )
    .expect("event");
    assert!(
        event
            .events
            .iter()
            .all(|item| item.suppressed == ["learning"])
    );
    let compare = matched_budget_compare(&dense, &event).expect("compare");
    assert!(compare.matched);
    assert!(compare.dense_work > compare.event_work);
    assert!(matched_budget_compare(&dense, &dense).is_err());
}

#[test]
fn scheduler_charge_overflow_is_a_stable_error() {
    assert_eq!(
        dense_schedule("stream-a", 1, 1, &["acting", "learning"], &[u64::MAX])
            .expect_err("dense overflow"),
        SchedulerError::Arithmetic
    );
    assert_eq!(
        event_schedule("stream-a", 1, 1, &["acting", "learning"], &[], &[u64::MAX])
            .expect_err("event overflow"),
        SchedulerError::Arithmetic
    );
}
