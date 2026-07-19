use bonsai_bundle::SegmentWriter;
use bonsai_contracts::bonsai::event::v1::{Availability, EventEnvelope, Precision};
use bonsai_ingest::{
    EventIngestor, IngestOutcome, IngestPolicy, SchemaAuthorization, SourceAuthorization,
};
use prost::Message;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

fn event() -> EventEnvelope {
    let payload = b"fixture".to_vec();
    EventEnvelope {
        run_id: vec![1; 16],
        source_id: vec![2; 16],
        event_id: vec![3; 16],
        source_sequence: 0,
        causal_parent_event_ids: Vec::new(),
        monotonic_time_ns: 1,
        wall_time_unix_ns: None,
        event_type: "fixture.event/v1".to_owned(),
        payload_schema_epoch: 1,
        payload_schema_minor: 0,
        payload_sha256: Sha256::digest(&payload).to_vec(),
        availability: Availability::Measured as i32,
        precision: Some(Precision {
            representation: "bytes".to_owned(),
            significant_bits: None,
        }),
        payload,
    }
}

fn policy() -> IngestPolicy {
    IngestPolicy {
        run_id: [1; 16],
        maximum_envelope_bytes: 4096,
        maximum_causal_parents: 4,
        rate_window_ns: 100,
        maximum_rejection_records: 8,
        maximum_rejection_bytes: 2048,
        sources: BTreeMap::from([(
            [2; 16],
            SourceAuthorization {
                allowed_event_types: BTreeSet::from(["fixture.event/v1".to_owned()]),
                maximum_payload_bytes: 64,
                maximum_events_per_window: 2,
            },
        )]),
        schemas: BTreeMap::from([(
            "fixture.event/v1".to_owned(),
            SchemaAuthorization {
                epoch: 1,
                maximum_minor: 0,
            },
        )]),
    }
}

#[test]
fn only_fully_validated_events_reach_the_segment_writer() {
    let directory = tempfile::tempdir().expect("tempdir");
    let mut writer = SegmentWriter::create(directory.path(), 0, 4096).expect("writer");
    let mut ingestor = EventIngestor::new(&mut writer, policy()).expect("policy");
    assert!(matches!(
        ingestor.ingest(&event().encode_to_vec(), 1),
        IngestOutcome::Rejected(rejection) if rejection.code == "INGEST_LIFECYCLE_PRECONDITION"
    ));
    ingestor.start().expect("start");

    let mut wrong_run = event();
    wrong_run.run_id = vec![9; 16];
    let mut wrong_source = event();
    wrong_source.source_id = vec![9; 16];
    let mut wrong_type = event();
    wrong_type.event_type = "other.event/v1".to_owned();
    let mut wrong_schema = event();
    wrong_schema.payload_schema_minor = 1;
    let mut wrong_hash = event();
    wrong_hash.payload_sha256 = vec![0; 32];
    for (candidate, expected) in [
        (wrong_run, "INGEST_RUN_UNAUTHORIZED"),
        (wrong_source, "INGEST_SOURCE_UNAUTHORIZED"),
        (wrong_type, "INGEST_EVENT_TYPE_UNAUTHORIZED"),
        (wrong_schema, "INGEST_SCHEMA_UNAUTHORIZED"),
        (wrong_hash, "EVENT_PAYLOAD_HASH_MISMATCH"),
    ] {
        assert!(matches!(
            ingestor.ingest(&candidate.encode_to_vec(), 1),
            IngestOutcome::Rejected(rejection) if rejection.code == expected
        ));
    }

    let valid = event().encode_to_vec();
    assert!(matches!(
        ingestor.ingest(&valid, 1),
        IngestOutcome::Accepted { event_id } if event_id == [3; 16]
    ));
    drop(ingestor);
    assert_eq!(writer.finalize().expect("finalize").frame_count, 1);
}

#[test]
fn rate_and_rejection_evidence_remain_bounded() {
    let directory = tempfile::tempdir().expect("tempdir");
    let mut writer = SegmentWriter::create(directory.path(), 0, 4096).expect("writer");
    let mut ingestor = EventIngestor::new(&mut writer, policy()).expect("policy");
    ingestor.start().expect("start");
    let valid = event().encode_to_vec();
    assert!(matches!(
        ingestor.ingest(&valid, 1),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        ingestor.ingest(&valid, 2),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        ingestor.ingest(&valid, 3),
        IngestOutcome::Rejected(rejection) if rejection.code == "INGEST_RATE_LIMIT"
    ));
    for _ in 0..100 {
        let _ = ingestor.ingest(b"not-protobuf", 4);
    }
    let ledger = ingestor.rejection_ledger();
    assert!(ledger.records.len() <= 8);
    assert!(ledger.retained_bytes <= 2048);
    assert!(ledger.dropped_records > 0);
    assert!(ledger.records.iter().all(|record| record.len() <= 2048));
}

#[test]
fn deterministic_fuzz_corpus_never_panics_or_appends_invalid_bytes() {
    let directory = tempfile::tempdir().expect("tempdir");
    let mut writer = SegmentWriter::create(directory.path(), 0, 4096).expect("writer");
    let mut ingestor = EventIngestor::new(&mut writer, policy()).expect("policy");
    ingestor.start().expect("start");
    let mut state = 0x9e37_79b9_u32;
    for length in 0..2048_usize {
        let mut bytes = vec![0_u8; length];
        for byte in &mut bytes {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            *byte = state.to_le_bytes()[0];
        }
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ingestor.ingest(&bytes, length as u64)
        }));
        assert!(matches!(outcome, Ok(IngestOutcome::Rejected(_))));
    }
    drop(ingestor);
    assert_eq!(writer.finalize().expect("finalize").frame_count, 0);
}

#[test]
fn committed_outcome_matrix_names_the_exact_rejection_codes() {
    let matrix: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/event-ingest/v1/expected-outcomes.json"
    ))
    .expect("outcome matrix");
    assert_eq!(matrix["cases"]["valid"], "accepted");
    for (case, code) in [
        ("wrong_run", "INGEST_RUN_UNAUTHORIZED"),
        ("wrong_source", "INGEST_SOURCE_UNAUTHORIZED"),
        ("wrong_event_type", "INGEST_EVENT_TYPE_UNAUTHORIZED"),
        ("wrong_schema", "INGEST_SCHEMA_UNAUTHORIZED"),
        ("wrong_payload_hash", "EVENT_PAYLOAD_HASH_MISMATCH"),
        ("rate_flood", "INGEST_RATE_LIMIT"),
        ("out_of_lifecycle", "INGEST_LIFECYCLE_PRECONDITION"),
    ] {
        assert_eq!(matrix["cases"][case], code);
    }
}
