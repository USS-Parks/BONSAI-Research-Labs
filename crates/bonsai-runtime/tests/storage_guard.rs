use bonsai_runtime::{
    AgentStorageBroker, IsolatedRunLayout, PersistOutcome, PersistRequest, PersistenceClass,
    StorageError, StoragePolicy,
};
use serde_json::{Value, json};

fn policy() -> StoragePolicy {
    StoragePolicy {
        policy_id: "m2.storage-guard.v1".to_owned(),
        max_bytes: 160,
        max_files: 2,
        max_file_bytes: 80,
        allow_replay: false,
    }
}

fn request(relative_path: &str, declared: PersistenceClass, bytes: &[u8]) -> PersistRequest {
    PersistRequest {
        relative_path: relative_path.to_owned(),
        declared_class: declared,
        bytes: bytes.to_vec(),
    }
}

fn model_weights() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "kind": "model_parameters",
        "values": [1, 0, -1]
    }))
    .expect("model parameters")
}

fn algorithm_state() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "kind": "algorithm_state",
        "slots": {"counts": [1, 2], "returns": [0, 1]}
    }))
    .expect("algorithm state")
}

fn replay_buffer() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "kind": "replay_buffer",
        "transitions": [{
            "state": [0],
            "action": 1,
            "reward": 0,
            "next_state": [1]
        }]
    }))
    .expect("replay buffer")
}

fn hidden_replay() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "kind": "model_parameters",
        "values": [9],
        "transitions": [{
            "state": [2],
            "action": 0,
            "reward": 1,
            "next_state": [3]
        }]
    }))
    .expect("hidden replay")
}

fn oversized_model() -> Vec<u8> {
    let mut object = serde_json::Map::new();
    object.insert(
        "kind".to_owned(),
        Value::String("model_parameters".to_owned()),
    );
    object.insert(
        "values".to_owned(),
        Value::Array((0..40).map(Value::from).collect()),
    );
    serde_json::to_vec(&Value::Object(object)).expect("oversized model")
}

fn extra_state() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "kind": "algorithm_state",
        "slots": {"bias": [1]}
    }))
    .expect("extra state")
}

fn broker() -> (tempfile::TempDir, AgentStorageBroker) {
    let directory = tempfile::tempdir().expect("temporary directory");
    let layout = IsolatedRunLayout::create(directory.path().join("run")).expect("layout");
    let broker = AgentStorageBroker::new(layout, policy()).expect("broker");
    (directory, broker)
}

#[test]
fn authorized_parameters_and_state_are_metered_and_replay_is_classified() {
    let (_directory, mut broker) = broker();
    let expected: Value = serde_json::from_str(include_str!(
        "../../../fixtures/storage-guard/v1/expected-outcomes.json"
    ))
    .expect("committed outcomes");
    let requests = [
        request(
            "model/weights.json",
            PersistenceClass::ModelParameters,
            &model_weights(),
        ),
        request(
            "state/counters.json",
            PersistenceClass::BoundedAlgorithmState,
            &algorithm_state(),
        ),
        request(
            "replay/transitions.json",
            PersistenceClass::ReplayBuffer,
            &replay_buffer(),
        ),
        request(
            "model/hidden-replay.json",
            PersistenceClass::ModelParameters,
            &hidden_replay(),
        ),
        request(
            "model/too-large.json",
            PersistenceClass::ModelParameters,
            &oversized_model(),
        ),
        request(
            "state/extra.json",
            PersistenceClass::BoundedAlgorithmState,
            &extra_state(),
        ),
    ];
    let decisions = Value::Array(
        requests
            .iter()
            .map(|item| {
                serde_json::to_value(broker.persist(item).expect("decision")).expect("json")
            })
            .collect(),
    );
    assert_eq!(decisions, expected["decisions"]);
    let inspection = serde_json::to_value(broker.inspect().expect("inspect")).expect("json");
    assert_eq!(inspection, expected["inspection"]);
    assert_eq!(broker.used_files(), 2);
}

#[test]
fn path_traversal_and_observer_targets_fail_closed() {
    let (_directory, mut broker) = broker();
    assert!(matches!(
        broker.persist(&request(
            "../observer/telemetry/stolen.json",
            PersistenceClass::ModelParameters,
            &model_weights()
        )),
        Err(StorageError::PathTraversal)
    ));
    assert!(matches!(
        broker.persist(&request(
            "/tmp/escape.json",
            PersistenceClass::ModelParameters,
            &model_weights()
        )),
        Err(StorageError::PathTraversal)
    ));
    assert!(matches!(
        broker.persist(&request(
            "model/./weights.json",
            PersistenceClass::ModelParameters,
            &model_weights()
        )),
        Err(StorageError::PathTraversal)
    ));
}

#[test]
fn symlink_ancestors_and_destinations_are_denied() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let layout = IsolatedRunLayout::create(directory.path().join("run")).expect("layout");
    let linked = layout.writable_root().join("linked");
    let created = create_symlink(layout.observer_root(), &linked);
    let mut broker = AgentStorageBroker::new(layout, policy()).expect("broker");
    if created {
        assert!(matches!(
            broker.persist(&request(
                "linked/canary.json",
                PersistenceClass::ModelParameters,
                &model_weights()
            )),
            Err(StorageError::SymlinkDenied)
        ));
    }
    let (_directory, mut plain) = broker_pair_without_symlink();
    assert_eq!(
        plain
            .persist(&request(
                "model/weights.json",
                PersistenceClass::ModelParameters,
                &model_weights()
            ))
            .expect("admit")
            .outcome,
        PersistOutcome::Admit
    );
}

fn broker_pair_without_symlink() -> (tempfile::TempDir, AgentStorageBroker) {
    broker()
}

fn create_symlink(original: &std::path::Path, link: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(original, link).is_ok()
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(original, link).is_ok()
    }
}

#[test]
fn leftover_on_disk_overwrite_rejects_without_overflow() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let layout = IsolatedRunLayout::create(directory.path().join("run")).expect("layout");
    std::fs::write(layout.writable_root().join("leftover.bin"), [0_u8; 50]).expect("leftover");
    let mut broker = AgentStorageBroker::new(layout, policy()).expect("broker");
    let decision = broker
        .persist(&request(
            "leftover.bin",
            PersistenceClass::ModelParameters,
            &model_weights(),
        ))
        .expect("decision");
    assert_eq!(decision.outcome, PersistOutcome::Reject);
    assert_eq!(decision.reason_code, "STORAGE_METER_INCONSISTENT");
    assert_eq!(broker.used_bytes(), 0);
}

#[test]
fn malformed_policy_is_rejected_before_broker_state_exists() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let layout = IsolatedRunLayout::create(directory.path().join("run")).expect("layout");
    let mut invalid = policy();
    invalid.max_file_bytes = 256;
    assert!(matches!(
        AgentStorageBroker::new(layout, invalid),
        Err(StorageError::PolicyBound)
    ));
}
