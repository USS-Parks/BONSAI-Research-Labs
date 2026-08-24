use bonsai_contracts::resource::WorkClass;
use bonsai_governor::allocation::{
    AllocationAuthority, AllocationError, AllocationOutcome, AllocationPolicy, AllocationPurpose,
    AllocationRequest, ClassReservation, WorkAllocator,
};
use serde_json::{Value, json};

fn policy() -> AllocationPolicy {
    AllocationPolicy {
        policy_id: "m2.work-allocation.v1".to_owned(),
        total_capacity: 100,
        reservations: vec![
            reservation(WorkClass::Acting, 30),
            reservation(WorkClass::Learning, 15),
            reservation(WorkClass::FeatureGeneration, 10),
            reservation(WorkClass::OptionLearning, 10),
            reservation(WorkClass::ModelLearning, 10),
            reservation(WorkClass::Planning, 10),
            reservation(WorkClass::Curation, 5),
            reservation(WorkClass::Observer, 10),
        ],
        evidence_flush_reservation: 4,
    }
}

const fn reservation(work_class: WorkClass, amount: u64) -> ClassReservation {
    ClassReservation { work_class, amount }
}

fn request(
    request_id: &str,
    authority: AllocationAuthority,
    work_class: WorkClass,
    purpose: AllocationPurpose,
    amount: u64,
) -> AllocationRequest {
    AllocationRequest {
        request_id: request_id.to_owned(),
        authority,
        work_class,
        purpose,
        amount,
    }
}

fn agent(request_id: &str, work_class: WorkClass, amount: u64) -> AllocationRequest {
    request(
        request_id,
        AllocationAuthority::Agent,
        work_class,
        AllocationPurpose::Ordinary,
        amount,
    )
}

#[test]
fn adversarial_sequence_preserves_acting_and_evidence_flush_reservations() {
    let mut allocator = WorkAllocator::new(policy()).expect("allocator");
    let requests = [
        agent("learning-fill", WorkClass::Learning, 15),
        agent("learning-over", WorkClass::Learning, 1),
        agent("forged-observer", WorkClass::Observer, 1),
        request(
            "observer-ordinary-fill",
            AllocationAuthority::Observer,
            WorkClass::Observer,
            AllocationPurpose::Ordinary,
            6,
        ),
        request(
            "observer-ordinary-over",
            AllocationAuthority::Observer,
            WorkClass::Observer,
            AllocationPurpose::Ordinary,
            1,
        ),
        request(
            "evidence-flush",
            AllocationAuthority::Observer,
            WorkClass::Observer,
            AllocationPurpose::EvidenceFlush,
            4,
        ),
        agent("acting-fill", WorkClass::Acting, 30),
        agent("acting-over", WorkClass::Acting, 1),
    ];
    let decisions = requests
        .iter()
        .map(|request| allocator.allocate(request).expect("decision"))
        .map(|decision| {
            json!({
                "sequence": decision.sequence,
                "request_id": decision.request_id,
                "outcome": decision.outcome,
                "reason_code": decision.reason_code,
                "consumed_before": decision.consumed_before,
                "consumed_after": decision.consumed_after,
                "reservation": decision.reservation,
            })
        })
        .collect::<Vec<_>>();
    let actual = json!({
        "schema": "bonsai.work-allocation-outcomes/v1",
        "decisions": decisions,
        "usage": allocator.usage(),
    });
    let expected: Value = serde_json::from_str(include_str!(
        "../../../fixtures/work-allocation/v1/expected-outcomes.json"
    ))
    .expect("expected outcomes");
    assert_eq!(actual, expected);
}

#[test]
fn every_agent_class_is_bounded_without_cross_partition_consumption() {
    let mut allocator = WorkAllocator::new(policy()).expect("allocator");
    for (index, usage) in allocator.usage().into_iter().enumerate() {
        if usage.work_class == WorkClass::Observer {
            continue;
        }
        let class = usage.work_class;
        let admitted = allocator
            .allocate(&agent(&format!("fill-{index}"), class, usage.reservation))
            .expect("fill decision");
        assert_eq!(admitted.outcome, AllocationOutcome::Admit);
        let before = allocator.usage();
        let deferred = allocator
            .allocate(&agent(&format!("over-{index}"), class, 1))
            .expect("over decision");
        assert_eq!(deferred.outcome, AllocationOutcome::Defer);
        assert_eq!(deferred.reason_code, "WORK_CLASS_RESERVATION_EXHAUSTED");
        assert_eq!(allocator.usage(), before);
    }
    assert!(allocator.usage().iter().all(|usage| {
        usage.consumed <= usage.reservation
            && (usage.work_class != WorkClass::Observer || usage.consumed == 0)
    }));
}

#[test]
fn request_order_cannot_starve_acting_or_let_agent_claim_observer_authority() {
    let adversarial_classes = [
        WorkClass::Learning,
        WorkClass::FeatureGeneration,
        WorkClass::OptionLearning,
        WorkClass::ModelLearning,
        WorkClass::Planning,
        WorkClass::Curation,
        WorkClass::Environment,
        WorkClass::Observer,
    ];
    for rotation in 0..adversarial_classes.len() {
        let mut allocator = WorkAllocator::new(policy()).expect("allocator");
        for offset in 0..adversarial_classes.len() {
            let class = adversarial_classes[(rotation + offset) % adversarial_classes.len()];
            let decision = allocator
                .allocate(&agent(&format!("attack-{rotation}-{offset}"), class, 1_000))
                .expect("adversarial decision");
            assert_ne!(decision.outcome, AllocationOutcome::Admit);
        }
        let acting = allocator
            .allocate(&agent("acting-reserved", WorkClass::Acting, 30))
            .expect("acting decision");
        assert_eq!(acting.outcome, AllocationOutcome::Admit);
        let observer = allocator
            .usage()
            .into_iter()
            .find(|usage| usage.work_class == WorkClass::Observer)
            .expect("observer usage");
        assert_eq!(observer.consumed, 0);
    }
}

#[test]
fn malformed_partition_policies_fail_before_state_exists() {
    let mut missing = policy();
    missing.reservations.pop();
    assert!(matches!(
        WorkAllocator::new(missing),
        Err(AllocationError::PolicyClassCoverage)
    ));

    let mut duplicate = policy();
    duplicate.reservations[7].work_class = WorkClass::Acting;
    assert!(matches!(
        WorkAllocator::new(duplicate),
        Err(AllocationError::PolicyClassCoverage)
    ));

    let mut wrong_sum = policy();
    wrong_sum.reservations[0].amount = 29;
    assert!(matches!(
        WorkAllocator::new(wrong_sum),
        Err(AllocationError::PolicyReservation)
    ));

    let mut no_flush = policy();
    no_flush.evidence_flush_reservation = 0;
    assert!(matches!(
        WorkAllocator::new(no_flush),
        Err(AllocationError::PolicyReservation)
    ));
}
