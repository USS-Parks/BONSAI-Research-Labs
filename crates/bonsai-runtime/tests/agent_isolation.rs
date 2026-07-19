use bonsai_contracts::track::{
    Track, TrackDeclaration, TransitionAccess, UpdateSchedule, derive_track,
};
use bonsai_runtime::{
    AgentLaunchPolicy, ChildTransport, IsolatedRunLayout, IsolationError, ObserverArtifactClass,
    TransportLimits,
};
use serde_json::Value;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const INSPECTION_PROCESS_TIMEOUT: Duration = Duration::from_secs(20);

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn python() -> OsString {
    let root = workspace_root();
    let local = if cfg!(windows) {
        root.join(".venv/Scripts/python.exe")
    } else {
        root.join(".venv/bin/python")
    };
    let candidates = std::env::var_os("PYTHON")
        .into_iter()
        .chain(local.exists().then(|| local.into_os_string()))
        .chain([OsString::from("python3"), OsString::from("python")]);
    for candidate in candidates {
        if Command::new(&candidate)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
        {
            return candidate;
        }
    }
    panic!("BR-06 requires Python on every supported CI host");
}

fn strict_track_a() -> TrackDeclaration {
    TrackDeclaration {
        schema_version: "1.0".to_owned(),
        declared_track: Track::A,
        runtime_facts_complete: true,
        batch_size: 1,
        transition_access: TransitionAccess::SinglePass,
        replay_capacity_transitions: 0,
        offline_updates: false,
        observer_data_access: false,
        privileged_state: false,
        human_labels: false,
        domain_feature_targets: false,
        update_schedule: UpdateSchedule::EventDriven,
        fixed_external_budgets: true,
    }
}

#[test]
fn inspection_adapter_sees_only_granted_inputs_and_agent_owned_storage() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let source = directory.path().join("manifest-input.txt");
    fs::write(&source, "authorized-observation").expect("write input source");
    let layout = IsolatedRunLayout::create(directory.path().join("run")).expect("layout");
    fs::write(
        layout.telemetry_root().join("observer-canary.txt"),
        "observer-only",
    )
    .expect("write canary");
    let granted = layout
        .grant_input(
            "observation.txt",
            &source,
            "a4f6d3fee8c664e8e70753b04e33c74bb4ea31501232cda862dcf3d5b7fffd79",
        )
        .expect("grant input");
    assert!(
        fs::metadata(granted.path())
            .expect("granted input metadata")
            .permissions()
            .readonly()
    );
    let policy = AgentLaunchPolicy::new(layout.clone());
    let fixture =
        workspace_root().join("python/bonsai-reference/tests/fixtures/isolation_adapter.py");
    let launch = policy
        .build_command(python(), [fixture.into_os_string()], &[granted])
        .expect("isolated launch");

    assert!(launch.command.clear_environment);
    assert!(!launch.audit.observer_path_exposed);
    assert_eq!(
        launch.audit.inherited_handles,
        [
            "stdin:protocol",
            "stdout:protocol",
            "stderr:bounded-diagnostic"
        ]
    );
    assert_eq!(
        launch.audit.environment_keys,
        ["BONSAI_AGENT_ROOT", "BONSAI_INPUT_ROOT", "BONSAI_WORK_ROOT"]
    );

    let mut child = ChildTransport::spawn(&launch.command, TransportLimits::default())
        .expect("spawn inspection adapter");
    let frame = child
        .receive(INSPECTION_PROCESS_TIMEOUT)
        .expect("receive inspection")
        .expect("inspection frame");
    let result: Value = serde_json::from_slice(&frame).expect("inspection JSON");
    assert_eq!(
        result["granted_inputs"]["observation.txt"],
        "authorized-observation"
    );
    assert_eq!(result["observer_canary_discovered"], false);
    assert_eq!(result["work_probe_written"], true);
    assert_eq!(
        Path::new(result["current_directory"].as_str().expect("cwd"))
            .canonicalize()
            .expect("canonical cwd"),
        layout.agent_root()
    );
    let outcome = child
        .shutdown(INSPECTION_PROCESS_TIMEOUT)
        .expect("inspection shutdown");
    assert_eq!(outcome.exit_code, Some(0));
    assert!(outcome.failures.is_empty());
}

#[test]
fn observer_paths_are_rejected_from_args_and_protocol_before_launch() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let layout = IsolatedRunLayout::create(directory.path().join("run")).expect("layout");
    let policy = AgentLaunchPolicy::new(layout.clone());
    assert!(matches!(
        policy.build_command(python(), [layout.report_root().as_os_str().to_owned()], &[]),
        Err(IsolationError::ObserverPathExposure)
    ));
    assert!(matches!(
        policy.validate_protocol_payload(layout.index_root().to_string_lossy().as_bytes()),
        Err(IsolationError::ObserverPathExposure)
    ));
    policy
        .validate_protocol_payload(b"manifest-authorized-online-input")
        .expect("ordinary protocol payload");
    assert!(matches!(
        layout.grant_input(
            "../observer",
            directory.path(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        ),
        Err(IsolationError::InvalidInputName)
    ));
}

#[test]
fn manifest_hash_mismatch_never_creates_a_grant() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let source = directory.path().join("manifest-input.txt");
    fs::write(&source, "authorized-observation").expect("write input source");
    let layout = IsolatedRunLayout::create(directory.path().join("run")).expect("layout");
    assert!(matches!(
        layout.grant_input(
            "observation.txt",
            &source,
            "0000000000000000000000000000000000000000000000000000000000000000"
        ),
        Err(IsolationError::InputHashMismatch)
    ));
    assert!(!layout.input_root().join("observation.txt").exists());
}

#[test]
fn deliberate_observer_access_is_denied_and_forces_non_track_a() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let layout = IsolatedRunLayout::create(directory.path().join("run")).expect("layout");
    let policy = AgentLaunchPolicy::new(layout);
    let declaration = strict_track_a();
    assert_eq!(derive_track(&declaration).derived, Track::A);

    let denial = policy.deny_observer_access(&declaration, ObserverArtifactClass::Telemetry);
    assert!(!denial.allowed);
    assert_eq!(denial.code, "OBSERVER_ACCESS_DENIED");
    assert_eq!(denial.derived_track, Track::Indeterminate);
    assert_eq!(denial.reason_code, "OBSERVER_DATA_BOUNDARY_VIOLATION");
    assert!(denial.observed_declaration.observer_data_access);
    assert_ne!(derive_track(&denial.observed_declaration).derived, Track::A);
}

#[test]
fn committed_matrix_freezes_the_isolation_and_denial_outcomes() {
    let matrix: Value = serde_json::from_str(include_str!(
        "../../../fixtures/agent-isolation/v1/expected-outcomes.json"
    ))
    .expect("valid committed matrix");
    assert_eq!(matrix["schema"], "bonsai.agent-isolation-outcomes/v1");
    assert_eq!(
        matrix["inspection_adapter"]["observer_canary_discovered"],
        false
    );
    assert_eq!(matrix["observer_access_request"]["allowed"], false);
    assert_eq!(
        matrix["observer_access_request"]["derived_track"],
        "INDETERMINATE_TRACK"
    );
}
