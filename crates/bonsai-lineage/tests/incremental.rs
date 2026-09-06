use bonsai_contracts::bonsai::artifact::v1::artifact_lifecycle_event::Detail;
use bonsai_contracts::bonsai::artifact::v1::{
    ArtifactBirth, ArtifactCost, ArtifactDisposition, ArtifactDispositionRecord,
    ArtifactLifecycleEvent, ArtifactRevision, ArtifactType, ArtifactUtility, ConsumerAction,
    ConsumerKind, ConsumerLink, ConsumerReference, LineageRelation, ParentReference, Provenance,
};
use bonsai_contracts::bonsai::event::v1::Availability;
use bonsai_contracts::lineage::{
    IncrementalLineageValidator, LineageValidationError, validate_artifact_lineage_trace,
};
use bonsai_lineage::ArtifactLifecycleRegistry;
use bonsai_lineage::persistent::{PersistentLimits, PersistentLineageRegistry};

fn id(value: u64) -> Vec<u8> {
    let mut bytes = vec![1; 16];
    bytes[..8].copy_from_slice(&value.to_le_bytes());
    bytes
}
fn provenance() -> Provenance {
    Provenance {
        producer_id: "BX-07-generated".into(),
        producer_version: "1.0".into(),
        source_event_ids: vec![id(900_000)],
        method_ids: vec!["deterministic-generator/v1".into()],
    }
}
fn birth(value: u64, parents: Vec<ParentReference>) -> ArtifactLifecycleEvent {
    ArtifactLifecycleEvent {
        artifact_id: id(value),
        artifact_revision_id: id(100_000 + value),
        lifecycle_sequence: 1,
        detail: Some(Detail::Birth(ArtifactBirth {
            artifact_type: ArtifactType::Feature as i32,
            representation_sha256: vec![1; 32],
            provenance: Some(provenance()),
            parents,
        })),
    }
}
fn parent(value: u64) -> ParentReference {
    ParentReference {
        artifact_id: id(value),
        artifact_revision_id: id(100_000 + value),
        relation: LineageRelation::DerivedFrom as i32,
    }
}
fn cost(value: u64, sequence: u64, entry: u64, amount: Option<u64>) -> ArtifactLifecycleEvent {
    ArtifactLifecycleEvent {
        artifact_id: id(value),
        artifact_revision_id: id(100_000 + value),
        lifecycle_sequence: sequence,
        detail: Some(Detail::Cost(ArtifactCost {
            cost_entry_id: id(entry),
            counter_id: "work_items".into(),
            unit: "1".into(),
            amount,
            availability: Availability::Measured as i32,
            estimator_id: None,
            estimator_version: None,
            unavailable_reason: None,
            provenance: Some(provenance()),
        })),
    }
}

#[test]
fn invalid_history_measurement_does_not_reserve_its_identity() {
    for utility in [false, true] {
        let mut state = IncrementalLineageValidator::new();
        state.apply(&birth(1, vec![])).expect("birth");
        let mut event = cost(1, 2, 200_001, None);
        if utility {
            event.detail = Some(Detail::Utility(ArtifactUtility {
                utility_entry_id: id(200_001),
                metric_id: "utility".into(),
                metric_version: "1.0".into(),
                unit: "1".into(),
                estimate: None,
                availability: Availability::Measured as i32,
                estimator_id: None,
                estimator_version: None,
                unavailable_reason: None,
                provenance: Some(provenance()),
            }));
        }
        let before = state.clone();
        assert!(state.apply(&event).is_err());
        assert_eq!(state, before);
        match event.detail.as_mut().expect("detail") {
            Detail::Cost(value) => value.amount = Some(1),
            Detail::Utility(value) => value.estimate = Some(0.5),
            _ => unreachable!(),
        }
        state
            .apply(&event)
            .expect("same identity is still available");
    }
}

#[test]
fn diamond_cycle_rejection_preserves_revision_ownership_and_sequence() {
    let mut state = IncrementalLineageValidator::new();
    let mut accepted = vec![
        birth(1, vec![]),
        birth(2, vec![parent(1)]),
        birth(3, vec![parent(1)]),
        birth(4, vec![parent(2), parent(3)]),
    ];
    for event in &accepted {
        state.apply(event).expect("DAG");
    }
    let mut candidate = ArtifactLifecycleEvent {
        artifact_id: id(1),
        artifact_revision_id: id(300_001),
        lifecycle_sequence: 2,
        detail: Some(Detail::Revision(ArtifactRevision {
            previous_revision_id: id(100_001),
            representation_sha256: vec![2; 32],
            parents: vec![parent(4)],
            provenance: Some(provenance()),
        })),
    };
    let before = state.clone();
    let (result, work) = state.apply_measured(&candidate);
    assert_eq!(result, Err(LineageValidationError::LineageCycle));
    assert_eq!(state, before);
    assert!((2..=4).contains(&work.ancestry_nodes));
    assert_eq!(work.whole_graph_scans, 0);
    assert_eq!(work.graph_clones, 0);
    let Some(Detail::Revision(value)) = candidate.detail.as_mut() else {
        unreachable!()
    };
    value.parents.clear();
    state
        .apply(&candidate)
        .expect("rejected revision did not reserve id");
    accepted.push(candidate);
    assert_eq!(validate_artifact_lineage_trace(&accepted), Ok(()));
}

fn generated(
    registry: &ArtifactLifecycleRegistry,
    attempt: u64,
    selector: u64,
) -> ArtifactLifecycleEvent {
    if registry.is_empty() || attempt.is_multiple_of(7) {
        return generated_birth(registry, attempt);
    }
    let index = usize::try_from(selector).expect("bounded selector") % registry.len();
    let record = registry
        .snapshot()
        .artifacts
        .values()
        .nth(index)
        .expect("artifact");
    let sequence = u64::try_from(
        registry
            .events()
            .iter()
            .filter(|event| event.artifact_id == record.artifact_id)
            .count(),
    )
    .expect("bounded count")
        + 1;
    let mut event = ArtifactLifecycleEvent {
        artifact_id: record.artifact_id.to_vec(),
        artifact_revision_id: record.current_revision_id.to_vec(),
        lifecycle_sequence: sequence,
        detail: None,
    };
    event.detail = Some(match attempt % 5 {
        0 => {
            event.artifact_revision_id = id(300_000 + attempt);
            let parent = registry
                .snapshot()
                .artifacts
                .values()
                .next()
                .expect("parent");
            Detail::Revision(ArtifactRevision {
                previous_revision_id: record.current_revision_id.to_vec(),
                representation_sha256: vec![2; 32],
                provenance: Some(provenance()),
                parents: vec![ParentReference {
                    artifact_id: parent.artifact_id.to_vec(),
                    artifact_revision_id: parent.current_revision_id.to_vec(),
                    relation: LineageRelation::DerivedFrom as i32,
                }],
            })
        }
        1 => Detail::ConsumerLink(ConsumerLink {
            consumer: Some(ConsumerReference {
                consumer_id: id(400_000 + attempt),
                kind: ConsumerKind::Component as i32,
                consumer_artifact_revision_id: None,
            }),
            action: if selector.is_multiple_of(3) {
                ConsumerAction::Unlink
            } else {
                ConsumerAction::Link
            } as i32,
            provenance: Some(provenance()),
        }),
        2 => Detail::Cost(ArtifactCost {
            cost_entry_id: id(500_000 + attempt),
            counter_id: "cpu_time_ns".into(),
            unit: "ns".into(),
            amount: Some(10),
            availability: Availability::Measured as i32,
            estimator_id: None,
            estimator_version: None,
            unavailable_reason: None,
            provenance: Some(provenance()),
        }),
        3 => Detail::Utility(ArtifactUtility {
            utility_entry_id: id(500_000 + attempt),
            metric_id: "gain".into(),
            metric_version: "1.0".into(),
            unit: "1".into(),
            estimate: Some(0.25),
            availability: Availability::Measured as i32,
            estimator_id: None,
            estimator_version: None,
            unavailable_reason: None,
            provenance: Some(provenance()),
        }),
        _ => Detail::Disposition(ArtifactDispositionRecord {
            disposition: if selector.is_multiple_of(11) {
                ArtifactDisposition::Retired
            } else {
                ArtifactDisposition::Retained
            } as i32,
            successor_artifact_id: None,
            provenance: Some(provenance()),
        }),
    });
    event
}

fn mutate(event: &mut ArtifactLifecycleEvent, selector: u64) {
    match selector % 8 {
        0 => event.artifact_id.clear(),
        1 => event.lifecycle_sequence = 0,
        2 => event.artifact_revision_id = id(999_999),
        3 => event.detail = None,
        _ => match event.detail.as_mut().expect("detail") {
            Detail::Birth(value) => value.provenance = None,
            Detail::Revision(value) => value.previous_revision_id = id(888_888),
            Detail::ConsumerLink(value) => value.action = ConsumerAction::Unspecified as i32,
            Detail::Cost(value) => value.amount = None,
            Detail::Utility(value) => value.estimate = Some(f64::NAN),
            Detail::Disposition(value) => {
                value.disposition = ArtifactDisposition::Unspecified as i32;
            }
        },
    }
}

#[test]
fn generated_valid_and_invalid_admissions_match_full_replay_and_state() {
    let mut accepted_total = 0;
    let mut rejected_total = 0;
    for seed in 1..=12_u64 {
        let temporary = tempfile::tempdir().expect("temporary");
        let durable_root = temporary.path().join("durable");
        let mut durable = PersistentLineageRegistry::create(
            &durable_root,
            PersistentLimits {
                maximum_live_artifacts: 1000,
                maximum_consumers: 64,
                maximum_frame_bytes: 65536,
                database_bytes: 8 * 1024 * 1024,
                output_bytes: 64 * 1024 * 1024,
            },
        )
        .expect("create durable");
        let mut registry = ArtifactLifecycleRegistry::new();
        let mut incremental = IncrementalLineageValidator::new();
        let mut random = seed;
        for attempt in 0..240 {
            random = random
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let selector = random >> 32;
            let mut event = generated(&registry, attempt, selector);
            if attempt > 0 && selector.is_multiple_of(3) {
                mutate(&mut event, selector >> 3);
            }
            let mut candidate = registry.events().to_vec();
            candidate.push(event.clone());
            let expected = validate_artifact_lineage_trace(&candidate);
            let durable_before = durable.artifact(&event.artifact_id).expect("read current");
            assert_eq!(
                durable.append(&event),
                expected.map_err(|e| e.to_string()),
                "durable seed={seed} attempt={attempt}"
            );
            if expected.is_err() {
                assert_eq!(
                    durable.artifact(&event.artifact_id).expect("read rejected"),
                    durable_before
                );
            }
            let before = incremental.clone();
            let snapshot_before = registry.snapshot().clone();
            let length_before = registry.events().len();
            assert_eq!(
                incremental.apply(&event),
                expected,
                "seed={seed} attempt={attempt}"
            );
            let (actual, work) = registry.apply_measured(event);
            assert_eq!(actual, expected, "seed={seed} attempt={attempt}");
            assert_eq!(
                (work.events, work.whole_graph_scans, work.graph_clones),
                (1, 0, 0)
            );
            if actual.is_err() {
                rejected_total += 1;
                assert_eq!(incremental, before);
                assert_eq!(registry.snapshot(), &snapshot_before);
                assert_eq!(registry.events().len(), length_before);
            } else {
                accepted_total += 1;
                let reconstructed =
                    ArtifactLifecycleRegistry::reconstruct(registry.events()).expect("replay");
                assert_eq!(registry.snapshot(), reconstructed.snapshot());
                assert_eq!(registry.events(), reconstructed.events());
            }
        }
        let checkpoint = durable.commit().expect("commit corpus");
        assert_eq!(checkpoint.events, registry.events().len() as u64);
        drop(durable);
        let (recovered, report) =
            PersistentLineageRegistry::open(&durable_root).expect("recover corpus");
        assert_eq!(report.committed_events, checkpoint.events);
        for record in registry.snapshot().artifacts.values() {
            let current = recovered
                .artifact(&record.artifact_id)
                .expect("read")
                .expect("artifact");
            assert_eq!(current.current_revision_id, record.current_revision_id);
            assert_eq!(current.terminal, record.terminal);
            assert_eq!(current.consumers.len(), record.active_consumers.len());
        }
    }
    println!(
        "BX-07/BX-08 generated corpus: seeds=12 attempts=2880 accepted={accepted_total} rejected={rejected_total}"
    );
    assert!(accepted_total > 1000 && rejected_total > 500);
}

fn generated_birth(registry: &ArtifactLifecycleRegistry, attempt: u64) -> ArtifactLifecycleEvent {
    let parents = registry
        .snapshot()
        .artifacts
        .values()
        .next()
        .map_or_else(Vec::new, |record| {
            vec![ParentReference {
                artifact_id: record.artifact_id.to_vec(),
                artifact_revision_id: record.current_revision_id.to_vec(),
                relation: LineageRelation::DerivedFrom as i32,
            }]
        });
    birth(attempt + 1, parents)
}
