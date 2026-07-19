use bonsai_contracts::bonsai::event::v1::EventEnvelope;
use bonsai_ingest::{ObservedEvent, OrderEdgeKind, OrderingLimits, classify_partial_order};

fn observed(
    id: u8,
    source: u8,
    sequence: u64,
    monotonic_time_ns: u64,
    arrival_index: u64,
    parents: &[u8],
) -> ObservedEvent {
    ObservedEvent {
        envelope: EventEnvelope {
            event_id: vec![id; 16],
            source_id: vec![source; 16],
            source_sequence: sequence,
            monotonic_time_ns,
            causal_parent_event_ids: parents.iter().map(|parent| vec![*parent; 16]).collect(),
            wall_time_unix_ns: Some(i64::from(id) * -1_000_000),
            ..EventEnvelope::default()
        },
        arrival_index,
    }
}

fn fixture() -> Vec<ObservedEvent> {
    vec![
        observed(1, 10, 0, 10, 0, &[]),
        observed(3, 10, 2, 11, 1, &[]),
        observed(4, 20, 0, 5, 2, &[1]),
        observed(2, 10, 1, 9, 3, &[]),
        observed(5, 30, 0, 7, 4, &[9]),
        observed(1, 10, 0, 10, 5, &[]),
    ]
}

#[test]
fn exact_partial_order_classes_are_derived_without_wall_time() {
    let report = classify_partial_order(&fixture(), OrderingLimits::default()).expect("report");
    assert_eq!(report.duplicate_event_ids, vec![[1; 16]]);
    assert_eq!(report.late_event_ids, vec![[1; 16], [2; 16]]);
    assert_eq!(report.clock_regression_event_ids, vec![[2; 16]]);
    assert_eq!(report.missing_parents.len(), 1);
    assert_eq!(report.missing_parents[0].event_id, [5; 16]);
    assert_eq!(report.missing_parents[0].parent_id, [9; 16]);
    assert!(report.edges.iter().any(|edge| {
        edge.before == [1; 16]
            && edge.after == [2; 16]
            && edge.kinds.contains(&OrderEdgeKind::SourceSequence)
    }));
    assert!(report.edges.iter().any(|edge| {
        edge.before == [1; 16]
            && edge.after == [4; 16]
            && edge.kinds.contains(&OrderEdgeKind::CausalParent)
    }));
    assert!(
        report
            .concurrent_pairs
            .iter()
            .any(|pair| { pair.first == [2; 16] && pair.second == [4; 16] })
    );
    assert!(
        report
            .concurrent_pairs
            .iter()
            .any(|pair| pair.second == [5; 16])
    );
}

#[test]
fn randomized_collection_order_preserves_the_exact_report() {
    let baseline = classify_partial_order(&fixture(), OrderingLimits::default()).expect("baseline");
    let original = fixture();
    for rotation in 0..original.len() {
        let mut candidate = original.clone();
        candidate.rotate_left(rotation);
        if rotation % 2 == 1 {
            candidate.reverse();
        }
        assert_eq!(
            classify_partial_order(&candidate, OrderingLimits::default()),
            Ok(baseline.clone())
        );
    }
}

#[test]
fn cycles_conflicts_and_gaps_are_explicit_not_silently_ordered() {
    let observations = vec![
        observed(1, 10, 0, 1, 0, &[3]),
        observed(2, 10, 0, 2, 1, &[]),
        observed(3, 10, 2, 3, 2, &[1]),
    ];
    let report = classify_partial_order(&observations, OrderingLimits::default()).expect("report");
    assert_eq!(report.sequence_conflicts.len(), 1);
    assert_eq!(report.sequence_gaps.len(), 1);
    assert_eq!(report.cycle_event_ids, vec![[1; 16], [3; 16]]);
    assert!(
        !report
            .edges
            .iter()
            .any(|edge| { edge.kinds.contains(&OrderEdgeKind::SourceSequence) })
    );
}

#[test]
fn committed_matrix_freezes_partial_order_not_wall_time() {
    let matrix: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/event-ordering/v1/expected-outcomes.json"
    ))
    .expect("ordering matrix");
    assert_eq!(matrix["wall_time_orders_events"], false);
    assert_eq!(matrix["collection_order_changes_report"], false);
    assert_eq!(matrix["classes"].as_array().map(Vec::len), Some(10));
}
