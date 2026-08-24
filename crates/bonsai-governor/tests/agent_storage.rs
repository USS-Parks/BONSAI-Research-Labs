use bonsai_contracts::track::Track;
use bonsai_governor::storage::{
    PathShape, PersistenceOutcome, PersistenceRequest, RetentionKind, RetentionSignals,
    StorageBroker, StorageError, StoragePolicy, inspect_path_shape, resolve_under_work_root,
};
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;

fn track_a_policy() -> StoragePolicy {
    StoragePolicy {
        policy_id: "m2.agent-storage.v1".to_owned(),
        max_bytes: 512,
        max_files: 4,
        max_bounded_state_bytes: 64,
        allow_transition_replay: false,
        allowed_replay_capacity_transitions: 0,
    }
}

fn request(
    request_id: &str,
    logical_path: &str,
    byte_delta: u64,
    file_delta: u64,
    declared_kind: RetentionKind,
    signals: RetentionSignals,
) -> PersistenceRequest {
    PersistenceRequest {
        request_id: request_id.to_owned(),
        logical_path: logical_path.to_owned(),
        byte_delta,
        file_delta,
        declared_kind,
        path_shape: PathShape::RegularFile,
        signals,
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn adversarial_sequence_classifies_replay_and_allows_legitimate_state() {
    let mut broker = StorageBroker::new(track_a_policy()).expect("broker");
    let requests = [
        request(
            "model-params",
            "models/weights.bin",
            128,
            1,
            RetentionKind::ModelParameters,
            RetentionSignals {
                parameter_tensor_count: 2,
                ..RetentionSignals::empty()
            },
        ),
        request(
            "algo-state",
            "state/eligibility.bin",
            32,
            1,
            RetentionKind::BoundedAlgorithmState,
            RetentionSignals {
                bounded_state_bytes: 32,
                ..RetentionSignals::empty()
            },
        ),
        request(
            "replay-buffer",
            "buffers/replay_buffer.json",
            200,
            1,
            RetentionKind::Unclassified,
            RetentionSignals {
                transition_record_count: 16,
                observation_fields: true,
                action_fields: true,
                reward_fields: true,
                next_observation_fields: true,
                ..RetentionSignals::empty()
            },
        ),
        request(
            "hidden-name",
            "cache/experience_replay.dat",
            40,
            1,
            RetentionKind::ModelParameters,
            RetentionSignals::empty(),
        ),
        request(
            "byte-exhaust",
            "models/more_weights.bin",
            400,
            1,
            RetentionKind::ModelParameters,
            RetentionSignals {
                parameter_tensor_count: 1,
                ..RetentionSignals::empty()
            },
        ),
        {
            let mut symlink = request(
                "symlink-escape",
                "work/alias.bin",
                8,
                1,
                RetentionKind::BoundedAlgorithmState,
                RetentionSignals {
                    bounded_state_bytes: 8,
                    ..RetentionSignals::empty()
                },
            );
            symlink.path_shape = PathShape::Symlink;
            symlink
        },
        request(
            "observer-path",
            "observer/telemetry/segment.bin",
            8,
            1,
            RetentionKind::ModelParameters,
            RetentionSignals {
                parameter_tensor_count: 1,
                ..RetentionSignals::empty()
            },
        ),
        request(
            "traversal",
            "../agent/work/escape.bin",
            8,
            1,
            RetentionKind::ModelParameters,
            RetentionSignals {
                parameter_tensor_count: 1,
                ..RetentionSignals::empty()
            },
        ),
    ];
    let decisions = requests
        .iter()
        .map(|request| broker.persist(request).expect("decision"))
        .map(|decision| {
            json!({
                "sequence": decision.sequence,
                "request_id": decision.request_id,
                "classified_kind": decision.classified_kind,
                "outcome": decision.outcome,
                "reason_code": decision.reason_code,
                "bytes_before": decision.bytes_before,
                "bytes_after": decision.bytes_after,
                "files_before": decision.files_before,
                "files_after": decision.files_after,
                "track_fact_replay_capacity": decision.track_fact_replay_capacity,
            })
        })
        .collect::<Vec<_>>();
    let actual = json!({
        "schema": "bonsai.agent-storage-outcomes/v1",
        "decisions": decisions,
        "usage": broker.usage(),
        "derived_track": format!("{:?}", broker.derived_track(Track::A)),
    });
    let expected: Value = serde_json::from_str(include_str!(
        "../../../fixtures/agent-storage/v1/expected-outcomes.json"
    ))
    .expect("expected outcomes");
    assert_eq!(actual, expected);
}

#[test]
fn legitimate_state_survives_while_replay_is_rejected_without_meter_growth() {
    let mut broker = StorageBroker::new(track_a_policy()).expect("broker");
    let admitted = broker
        .persist(&request(
            "params",
            "models/q.bin",
            64,
            1,
            RetentionKind::ModelParameters,
            RetentionSignals {
                parameter_tensor_count: 1,
                ..RetentionSignals::empty()
            },
        ))
        .expect("params");
    assert_eq!(admitted.outcome, PersistenceOutcome::Admit);
    let before = broker.usage();
    let denied = broker
        .persist(&request(
            "transitions",
            "work/transitions.json",
            64,
            1,
            RetentionKind::TransitionReplay,
            RetentionSignals {
                transition_record_count: 4,
                observation_fields: true,
                action_fields: true,
                reward_fields: true,
                next_observation_fields: false,
                ..RetentionSignals::empty()
            },
        ))
        .expect("replay");
    assert_eq!(denied.outcome, PersistenceOutcome::Reject);
    assert_eq!(denied.classified_kind, RetentionKind::TransitionReplay);
    assert_eq!(broker.usage(), before);
    assert_eq!(broker.derived_track(Track::A), Track::A);
}

#[test]
fn allowed_replay_policy_classifies_and_forces_track_b() {
    let policy = StoragePolicy {
        policy_id: "m2.agent-storage.replay.v1".to_owned(),
        max_bytes: 1_024,
        max_files: 8,
        max_bounded_state_bytes: 64,
        allow_transition_replay: true,
        allowed_replay_capacity_transitions: 32,
    };
    let mut broker = StorageBroker::new(policy).expect("broker");
    let decision = broker
        .persist(&request(
            "replay-ok",
            "buffers/replay_buffer.bin",
            100,
            1,
            RetentionKind::TransitionReplay,
            RetentionSignals {
                transition_record_count: 16,
                observation_fields: true,
                action_fields: true,
                reward_fields: true,
                next_observation_fields: true,
                ..RetentionSignals::empty()
            },
        ))
        .expect("decision");
    assert_eq!(decision.outcome, PersistenceOutcome::Admit);
    assert_eq!(
        decision.reason_code,
        "TRANSITION_REPLAY_RETENTION_CLASSIFIED"
    );
    assert_eq!(broker.derived_track(Track::A), Track::B);
}

#[test]
fn malformed_policies_and_live_path_helpers_fail_closed() {
    let mut bad = track_a_policy();
    bad.max_bytes = 0;
    assert!(matches!(
        StorageBroker::new(bad),
        Err(StorageError::PolicyBounds)
    ));

    let mut inconsistent = track_a_policy();
    inconsistent.allow_transition_replay = true;
    inconsistent.allowed_replay_capacity_transitions = 0;
    assert!(matches!(
        StorageBroker::new(inconsistent),
        Err(StorageError::PolicyBounds)
    ));

    let root = unique_temp_dir();
    let work = root.join("agent").join("work");
    fs::create_dir_all(&work).expect("work");
    let file = work.join("state.bin");
    fs::write(&file, b"abc").expect("write");
    assert_eq!(
        inspect_path_shape(&file).expect("shape"),
        PathShape::RegularFile
    );
    assert_eq!(
        resolve_under_work_root(&work, "state.bin").expect("resolve"),
        work.join("state.bin")
    );
    assert!(resolve_under_work_root(&work, "../escape.bin").is_none());
    assert!(resolve_under_work_root(&work, "nested/../../escape.bin").is_none());
    let _ = fs::remove_dir_all(&root);
}

fn unique_temp_dir() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "bonsai-bq06-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    fs::create_dir_all(&path).expect("temp");
    path
}
