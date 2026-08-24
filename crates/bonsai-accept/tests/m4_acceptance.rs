use bonsai_accept::{
    enforce_graph_and_artifact_bounds, redact, reject_secret_leak, run_ci_duration_probe,
    sign_bytes, verify_bytes, verify_host_attestations, verify_l_manifest, verify_operator_handoff,
    verify_release_candidate, verify_supply_chain, verify_threat_model, workspace_root,
};
use bonsai_bundle::{BundleSchemas, OverallVerdict, SegmentWriter, validate_result_bundle};
use bonsai_contracts::bonsai::event::v1::{Availability, EventEnvelope, Precision};
use bonsai_governor::storage::{
    PathShape, PersistenceOutcome, PersistenceRequest, RetentionKind, RetentionSignals,
    StorageBroker, StoragePolicy,
};
use bonsai_ingest::{
    EventIngestor, IngestOutcome, IngestPolicy, SchemaAuthorization, SourceAuthorization,
};
use bonsai_runtime::{LifecycleState, RunSupervisor};
use prost::Message;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

fn event_with(id: u8, payload: &[u8]) -> EventEnvelope {
    EventEnvelope {
        run_id: vec![1; 16],
        source_id: vec![2; 16],
        event_id: vec![id; 16],
        source_sequence: u64::from(id),
        causal_parent_event_ids: Vec::new(),
        monotonic_time_ns: u64::from(id) + 1,
        wall_time_unix_ns: None,
        event_type: "fixture.event/v1".to_owned(),
        payload_schema_epoch: 1,
        payload_schema_minor: 0,
        payload_sha256: Sha256::digest(payload).to_vec(),
        availability: Availability::Measured as i32,
        precision: Some(Precision {
            representation: "bytes".to_owned(),
            significant_bits: None,
        }),
        payload: payload.to_vec(),
    }
}

fn flood_policy() -> IngestPolicy {
    IngestPolicy {
        run_id: [1; 16],
        maximum_envelope_bytes: 512,
        maximum_causal_parents: 2,
        rate_window_ns: 1_000,
        maximum_rejection_records: 4,
        maximum_rejection_bytes: 1_024,
        sources: BTreeMap::from([(
            [2; 16],
            SourceAuthorization {
                allowed_event_types: BTreeSet::from(["fixture.event/v1".to_owned()]),
                maximum_payload_bytes: 16,
                maximum_events_per_window: 1,
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

fn schemas(root: &Path) -> BundleSchemas {
    BundleSchemas {
        bundle_manifest: read_json(&root.join("schemas/bundle-manifest-v1.json")),
        experiment_manifest: read_json(&root.join("schemas/experiment-manifest-v1.json")),
        track_declaration: read_json(&root.join("schemas/track-declaration-v1.json")),
        platform_inventory: read_json(&root.join("schemas/platform-inventory-v1.json")),
        resource_policy: read_json(&root.join("schemas/resource-policy-v1.json")),
        metric_estimate: read_json(&root.join("schemas/metric-estimate-v1.json")),
    }
}

fn read_json(path: &Path) -> serde_json::Value {
    serde_json::from_slice(&fs::read(path).expect("schema"))
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

#[test]
fn bv11_threat_model_covers_every_trust_boundary() {
    verify_threat_model(&workspace_root()).expect("BV-11 threat model");
}

#[test]
fn bv12_fuzz_property_and_adversarial_suite_bounds_tamper_and_flood() {
    let key = b"operator-local-key-not-a-kms";
    let payload = br#"{"bundle":"honest"}"#;
    let envelope = sign_bytes(key, payload);
    verify_bytes(key, payload, &envelope).expect("honest signature");
    assert_eq!(
        verify_bytes(key, b"tampered-bundle", &envelope)
            .expect_err("tamper")
            .0,
        "TAMPER_DETECTED"
    );

    let mut flooded = String::from("token=super-secret");
    for _ in 0..64 {
        flooded.push_str(" super-secret");
    }
    let redacted = redact(&flooded, &["super-secret"]);
    reject_secret_leak(&redacted, &["super-secret"]).expect("redacted");
    assert!(redacted.contains("<redacted>"));
    assert!(!redacted.contains("super-secret"));

    assert_eq!(
        enforce_graph_and_artifact_bounds(9, 3, 8, 4)
            .expect_err("bomb")
            .0,
        "ARTIFACT_BOMB_BOUNDED"
    );
    assert_eq!(
        enforce_graph_and_artifact_bounds(2, 9, 8, 4)
            .expect_err("depth")
            .0,
        "GRAPH_DEPTH_BOUNDED"
    );
    enforce_graph_and_artifact_bounds(2, 3, 8, 4).expect("within bounds");

    let directory = tempfile::tempdir().expect("tempdir");
    let mut writer = SegmentWriter::create(directory.path(), 0, 4096).expect("writer");
    let mut ingestor = EventIngestor::new(&mut writer, flood_policy()).expect("policy");
    ingestor.start().expect("start");
    let mut accepted = 0_u32;
    let mut rejected = 0_u32;
    for index in 1_u8..=8 {
        match ingestor.ingest(&event_with(index, b"ok").encode_to_vec(), 10) {
            IngestOutcome::Accepted { .. } => accepted += 1,
            IngestOutcome::Rejected(rejection) => {
                rejected += 1;
                assert!(
                    rejection.code == "INGEST_RATE_LIMIT"
                        || rejection.code == "INGEST_ENVELOPE_TOO_LARGE"
                        || rejection.code == "INGEST_PAYLOAD_TOO_LARGE"
                );
            }
        }
    }
    let bomb = ingestor.ingest(&vec![0xFF; 2_048], 11);
    assert!(
        matches!(bomb, IngestOutcome::Rejected(rejection) if rejection.code == "INGEST_ENVELOPE_TOO_LARGE")
    );
    let ledger = ingestor.rejection_ledger();
    assert_eq!(accepted, 1);
    assert!(rejected >= 7);
    assert!(ledger.records.len() <= 4);
    assert!(ledger.retained_bytes <= 1_024);
    assert!(ledger.dropped_records > 0 || ledger.records.len() <= 4);

    let mut broker = StorageBroker::new(StoragePolicy {
        policy_id: "m4.flood".to_owned(),
        max_bytes: 64,
        max_files: 2,
        max_bounded_state_bytes: 16,
        allow_transition_replay: false,
        allowed_replay_capacity_transitions: 0,
    })
    .expect("broker");
    let flood = PersistenceRequest {
        request_id: "bomb".to_owned(),
        logical_path: "../escape/artifact.bin".to_owned(),
        byte_delta: 4_096,
        file_delta: 1,
        declared_kind: RetentionKind::Unclassified,
        path_shape: PathShape::Symlink,
        signals: RetentionSignals::empty(),
    };
    let decision = broker.persist(&flood).expect("bounded deny");
    assert_eq!(decision.outcome, PersistenceOutcome::Reject);
    assert!(
        decision.reason_code == "STORAGE_SYMLINK_DENIED"
            || decision.reason_code == "STORAGE_PATH_TRAVERSAL_DENIED"
            || decision.reason_code == "STORAGE_BYTE_BUDGET_EXHAUSTED"
    );

    let root = tempfile::tempdir().expect("run root");
    let run = root.path().join("run");
    let mut supervisor = RunSupervisor::create(&run).expect("create");
    supervisor
        .transition_to(LifecycleState::Running, None)
        .expect("running");
    drop(supervisor);
    let (_, report) = RunSupervisor::open_and_recover(&run).expect("recover");
    assert_eq!(report.final_state, LifecycleState::Recovered);
}

#[test]
fn bv13_supply_chain_has_no_silent_critical_waiver() {
    let (cargo, uv) = verify_supply_chain(&workspace_root()).expect("BV-13");
    assert_eq!(cargo.len(), 64);
    assert_eq!(uv.len(), 64);
}

#[test]
fn bv14_l_harness_records_honest_not_run_and_refuses_ci_as_acceptance() {
    let root = workspace_root();
    verify_l_manifest(&root).expect("L manifest");
    let hosts = verify_host_attestations(&root).expect("attestations");
    assert_eq!(hosts.len(), 3);
    assert!(hosts.iter().all(|host| host.status == "not-run"));
    let probe = run_ci_duration_probe(false).expect("ci probe");
    assert!(!probe.long_duration_claim);
    assert_eq!(
        run_ci_duration_probe(true).expect_err("must not promote").0,
        "L_ACCEPTANCE_NOT_CLAIMED"
    );
}

#[test]
fn bv15_operator_handoff_lets_a_new_user_reproduce_m1_without_overclaim() {
    verify_operator_handoff(&workspace_root()).expect("BV-15");
}

#[test]
fn bv16_release_candidate_verifies_independently_without_publication() {
    let root = workspace_root();
    verify_release_candidate(&root).expect("RC files");
    let report = validate_result_bundle(
        &root,
        "fixtures/bundle-validation/v1/manifest.json",
        &schemas(&root),
    )
    .expect("independent bundle verify");
    assert_eq!(report.verdict, OverallVerdict::Valid);
    let tampered = validate_result_bundle(
        &root,
        "fixtures/bundle-validation/tampered/manifest.json",
        &schemas(&root),
    )
    .expect("tampered report");
    assert_eq!(tampered.verdict, OverallVerdict::Invalid);
}
