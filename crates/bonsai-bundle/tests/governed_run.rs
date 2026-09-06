use bonsai_bundle::{
    BlobId, BundleSchemas, SegmentWriter, governed::verify_governed_run, visit_segment_bytes,
};
use bonsai_contracts::bonsai::event::v1::EventEnvelope;
use prost::Message;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/governed-run/v1")
}
fn schemas() -> BundleSchemas {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schemas");
    let read =
        |name| serde_json::from_slice(&fs::read(root.join(name)).expect("schema")).expect("json");
    BundleSchemas {
        bundle_manifest: read("bundle-manifest-v1.json"),
        experiment_manifest: read("experiment-manifest-v1.json"),
        track_declaration: read("track-declaration-v1.json"),
        platform_inventory: read("platform-inventory-v1.json"),
        resource_policy: read("resource-policy-v1.json"),
        metric_estimate: read("metric-estimate-v1.json"),
    }
}
fn pin() -> String {
    let value: Value = read(&fixture().join("operator-receipt.json"));
    value["receipt_sha256"]
        .as_str()
        .expect("operator pin")
        .into()
}
fn copy(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("directory");
    for entry in fs::read_dir(source).expect("source") {
        let entry = entry.expect("entry");
        if entry.file_type().expect("kind").is_dir() {
            copy(&entry.path(), &destination.join(entry.file_name()));
        } else {
            fs::copy(entry.path(), destination.join(entry.file_name())).expect("copy");
        }
    }
}
fn run_copy() -> TempDir {
    let result = tempfile::tempdir().expect("temporary bundle");
    copy(&fixture().join("positive"), result.path());
    result
}
fn read(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("file")).expect("JSON")
}
fn write(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec(value).expect("JSON")).expect("write");
}
fn hash(path: &Path) -> String {
    BlobId::digest(&fs::read(path).expect("hash input")).to_hex()
}
fn seal(root: &Path) -> String {
    let path = root.join("bundle-manifest.json");
    let mut bundle = read(&path);
    for file in bundle["files"].as_array_mut().expect("files") {
        file["sha256"] = json!(hash(&root.join(file["path"].as_str().expect("path"))));
    }
    write(&path, &bundle);
    let path = root.join("artifact-index.json");
    let mut index = read(&path);
    for file in index["files"].as_array_mut().expect("files") {
        let path = root.join(file["path"].as_str().expect("path"));
        file["sha256"] = json!(hash(&path));
        file["bytes"] = json!(fs::metadata(path).expect("size").len());
    }
    write(&path, &index);
    let mut status = read(&root.join("run-status.json"));
    status["artifact_index_sha256"] = json!(hash(&path));
    write(&root.join("run-status.json"), &status);
    let mut receipt = read(&root.join("run-receipt.json"));
    receipt["index_sha256"] = status["artifact_index_sha256"].clone();
    receipt["status_sha256"] = json!(hash(&root.join("run-status.json")));
    write(&root.join("run-receipt.json"), &receipt);
    hash(&root.join("run-receipt.json"))
}
fn mutate_event(root: &Path, phase: &str, mutate: impl FnOnce(&mut Value)) {
    let path = root.join("telemetry/segment-00000000000000000000.bseg");
    let mut events = Vec::new();
    visit_segment_bytes(&fs::read(&path).expect("segment"), |bytes| {
        events.push(EventEnvelope::decode(bytes).expect("event"));
    })
    .expect("valid segment");
    let event = events
        .iter_mut()
        .find(|event| {
            let body: Value = serde_json::from_slice(&event.payload).expect("payload");
            body["phase"] == phase
        })
        .expect("phase");
    let mut body: Value = serde_json::from_slice(&event.payload).expect("payload");
    mutate(&mut body);
    event.payload = serde_json::to_vec(&body).expect("payload");
    event.payload_sha256 = Sha256::digest(&event.payload).to_vec();
    let staging = tempfile::tempdir().expect("segment staging");
    let mut writer = SegmentWriter::create(staging.path(), 0, 262_144).expect("writer");
    for event in events {
        writer.append(&event.encode_to_vec()).expect("frame");
    }
    writer.finalize().expect("footer");
    fs::copy(
        staging.path().join("segment-00000000000000000000.bseg"),
        path,
    )
    .expect("replace fixture");
}

#[test]
fn real_linux_trace_reconstructs_portably_without_compliance_flags() {
    let result =
        verify_governed_run(fixture().join("positive"), &pin(), &schemas()).expect("verified run");
    let value = serde_json::to_value(result).expect("verdict");
    assert_eq!(value["facts"]["environment_steps"], 20);
    assert_eq!(value["facts"]["derived_track"], "A");
    assert_eq!(value["claims"]["rows"][0]["c0"], "pass");
    assert_eq!(value["claims"]["rows"][0]["c1"], "pass");
    let declaration = read(&fixture().join("positive/track-declaration.json"));
    assert_eq!(declaration["runtime_facts_complete"], false);
}

#[test]
fn modified_file_cannot_become_eligible() {
    let root = run_copy();
    fs::write(root.path().join("agent-configuration.json"), b"{}").expect("mutation");
    assert!(matches!(
        verify_governed_run(root.path(), &pin(), &schemas()),
        Err("RUN_FILE_HASH_MISMATCH")
    ));
}

#[test]
fn missing_controller_remains_ineligible_after_fixture_resealing() {
    let root = run_copy();
    mutate_event(root.path(), "controls_before_launch", |body| {
        body["agent"]
            .as_object_mut()
            .expect("controls")
            .remove("cpu_max");
    });
    assert!(matches!(
        verify_governed_run(root.path(), &seal(root.path()), &schemas()),
        Err("RUN_CONTROLLER_MISSING_OR_CHANGED")
    ));
}

#[test]
fn history_exposure_cannot_hide_behind_a_false_declaration() {
    let root = run_copy();
    mutate_event(root.path(), "agent_launch_policy", |body| {
        body["audit"]["observer_path_exposed"] = json!(false);
        body["audit"]["arguments"]
            .as_array_mut()
            .expect("arguments")
            .push(json!("/observer/history"));
    });
    assert!(matches!(
        verify_governed_run(root.path(), &seal(root.path()), &schemas()),
        Err("RUN_INFORMATION_BOUNDARY_VIOLATION")
    ));
}

#[test]
fn unknown_track_and_incomplete_status_fail_closed() {
    for (file, field, value, expected) in [
        (
            "track-declaration.json",
            "declared_track",
            "INDETERMINATE_TRACK",
            "RUN_TRACK_UNSUPPORTED",
        ),
        (
            "run-status.json",
            "status",
            "INCOMPLETE",
            "RUN_NOT_COMPLETE",
        ),
    ] {
        let root = run_copy();
        let path = root.path().join(file);
        let mut record = read(&path);
        record[field] = json!(value);
        write(&path, &record);
        let result = verify_governed_run(root.path(), &seal(root.path()), &schemas());
        assert!(matches!(result, Err(code) if code == expected));
    }
}

#[test]
fn resealed_policy_substitution_cannot_change_execution_limits() {
    let root = run_copy();
    let path = root.path().join("resource-policy.json");
    let mut policy = read(&path);
    policy["limits"][0]["soft_limit"] = json!(4);
    policy["limits"][0]["hard_limit"] = json!(4);
    write(&path, &policy);
    mutate_event(root.path(), "controls_before_launch", |body| {
        body["policy_sha256"] = json!(hash(&path));
    });
    assert!(matches!(
        verify_governed_run(root.path(), &seal(root.path()), &schemas()),
        Err("RUN_POLICY_LIMIT_SUBSTITUTED")
    ));
}

#[test]
fn required_event_gap_is_rejected_even_with_rebuilt_sequence_and_hashes() {
    let root = run_copy();
    let path = root
        .path()
        .join("telemetry/segment-00000000000000000000.bseg");
    let mut events = Vec::new();
    visit_segment_bytes(&fs::read(&path).expect("segment"), |bytes| {
        events.push(EventEnvelope::decode(bytes).expect("event"));
    })
    .expect("valid segment");
    let position = events
        .iter()
        .position(|event| event.event_type == "run.reward")
        .expect("reward");
    events.remove(position);
    let mut previous = None;
    let staging = tempfile::tempdir().expect("staging");
    let mut writer = SegmentWriter::create(staging.path(), 0, 262_144).expect("writer");
    for (sequence, mut event) in events.into_iter().enumerate() {
        event.source_sequence = u64::try_from(sequence).expect("sequence");
        let mut hash = Sha256::new();
        hash.update(&event.run_id);
        hash.update(event.source_sequence.to_le_bytes());
        event.event_id = hash.finalize()[..16].to_vec();
        event.causal_parent_event_ids = previous.into_iter().collect();
        previous = Some(event.event_id.clone());
        writer.append(&event.encode_to_vec()).expect("frame");
    }
    writer.finalize().expect("footer");
    fs::copy(
        staging.path().join("segment-00000000000000000000.bseg"),
        path,
    )
    .expect("replace");
    assert!(matches!(
        verify_governed_run(root.path(), &seal(root.path()), &schemas()),
        Err("RUN_EVENT_ORDER_INVALID")
    ));
}
