//! Live version compatibility evidence using the unchanged generic runtime.
use bonsai_contracts::adapter::Peer;
use bonsai_contracts::bonsai::adapter::v1::{self as wire, AdapterFrame, adapter_frame};
use bonsai_contracts::bonsai::event::v1::EventEnvelope;
use bonsai_contracts::track::{Track, TrackDeclaration, TransitionAccess, UpdateSchedule};
use bonsai_ingest::ObservedEvent;
use bonsai_runtime::{
    AdapterCertificationInput, AdapterConformanceSuite, AgentLaunchPolicy, CertificationVerdict,
    ChildTransport, IsolatedRunLayout, LifecycleRecord, LifecycleState, ProtocolExchange,
    RunSupervisor, TimeoutProbe, TransportError, TransportLimits,
};
use prost::Message;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

fn frame(minor: u32, sequence: u64, message: adapter_frame::Message) -> AdapterFrame {
    AdapterFrame {
        sequence,
        protocol_epoch: 1,
        protocol_minor: minor,
        message: Some(message),
        ..Default::default()
    }
}
fn start(minor: u32, seed: u64) -> AdapterFrame {
    frame(
        minor,
        0,
        adapter_frame::Message::Start(wire::Start {
            run_id: vec![1; 16],
            deterministic_seed: seed,
            deadline_monotonic_ns: 1,
            accepted_versions: Some(wire::VersionRange {
                minimum_epoch: 1,
                minimum_minor: 0,
                maximum_epoch: 1,
                maximum_minor: 1,
            }),
        }),
    )
}
fn exchange(
    child: &mut ChildTransport,
    request: AdapterFrame,
    transcript: &mut Vec<ProtocolExchange>,
) -> AdapterFrame {
    child.send_message(&request).expect("send request");
    transcript.push(ProtocolExchange {
        peer: Peer::Supervisor,
        frame: request,
    });
    let response: AdapterFrame = child
        .receive_message(Duration::from_secs(10))
        .expect("receive")
        .expect("response");
    transcript.push(ProtocolExchange {
        peer: Peer::Adapter,
        frame: response.clone(),
    });
    response
}
fn launch(root: &Path, python: &Path, script: &Path, minor: u32) -> bonsai_runtime::IsolatedLaunch {
    AgentLaunchPolicy::new(IsolatedRunLayout::create(root).expect("new isolated layout"))
        .build_command(
            python.as_os_str(),
            [
                std::ffi::OsString::from("-I"),
                std::ffi::OsString::from("-B"),
                script.as_os_str().to_owned(),
                minor.to_string().into(),
            ],
            &[],
        )
        .expect("isolated command")
}
fn timeout(root: &Path, python: &Path, script: &Path, minor: u32) -> TimeoutProbe {
    let spec = launch(root, python, script, minor);
    let mut child = ChildTransport::spawn(&spec.command, TransportLimits::default())
        .expect("spawn timeout probe");
    exchange(&mut child, start(minor, 0), &mut Vec::new());
    // The adapter has completed startup and now waits for the next request.
    let started = Instant::now();
    assert_eq!(
        child.receive(Duration::from_millis(100)),
        Err(TransportError::ReadTimeout)
    );
    let elapsed = u64::try_from(started.elapsed().as_nanos()).expect("duration");
    let outcome = child
        .shutdown(Duration::from_secs(2))
        .expect("contained timeout child");
    assert!(outcome.exit_code != Some(0));
    fs::write(
        root.join("timeout.json"),
        serde_json::to_vec_pretty(&json!({
            "observed_ns":elapsed,"deadline_ns":100_000_000,"exit_code":outcome.exit_code,
            "failures":outcome.failures.iter().map(|failure|failure.code).collect::<Vec<_>>(),
        }))
        .expect("json"),
    )
    .expect("timeout evidence");
    TimeoutProbe {
        exercised: true,
        failure_code: Some("TRANSPORT_READ_TIMEOUT".into()),
        process_contained: true,
        deadline_ns: 100_000_000,
        observed_ns: elapsed,
    }
}
fn track() -> TrackDeclaration {
    TrackDeclaration {
        schema_version: "1.0".into(),
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
#[allow(clippy::too_many_lines)]
fn run(
    root: &Path,
    python: &Path,
    script: &Path,
    minor: u32,
    seed: u64,
) -> (AdapterCertificationInput, String) {
    let spec = launch(root, python, script, minor);
    let life_root = root.join("supervisor");
    let mut supervisor = RunSupervisor::create(&life_root).expect("supervisor");
    let mut child =
        ChildTransport::spawn(&spec.command, TransportLimits::default()).expect("spawn adapter");
    supervisor
        .transition_to(LifecycleState::Running, None)
        .expect("running");
    let mut transcript = Vec::new();
    let hello = exchange(&mut child, start(minor, seed), &mut transcript);
    let fingerprint = hello.capability_fingerprint_sha256;
    exchange(
        &mut child,
        frame(
            minor,
            1,
            adapter_frame::Message::Configure(wire::Configure {
                configuration_sha256: vec![2; 32],
                accepted_capability_fingerprint_sha256: fingerprint,
                deadline_monotonic_ns: 2,
            }),
        ),
        &mut transcript,
    );
    exchange(
        &mut child,
        frame(
            minor,
            2,
            adapter_frame::Message::Reset(wire::Reset {
                episode_id: vec![3; 16],
                deterministic_seed: seed,
                deadline_monotonic_ns: 3,
            }),
        ),
        &mut transcript,
    );
    let input = seed.to_le_bytes().to_vec();
    let response = exchange(
        &mut child,
        frame(
            minor,
            3,
            adapter_frame::Message::Step(wire::Step {
                step_index: 0,
                input_type: "bonsai.fixture/v1".into(),
                input_sha256: Sha256::digest(&input).to_vec(),
                input,
                deadline_monotonic_ns: 4,
            }),
        ),
        &mut transcript,
    );
    let action = match response.message.expect("step response") {
        adapter_frame::Message::StepResult(result) => result.action,
        _ => panic!("expected action"),
    };
    supervisor
        .transition_to(LifecycleState::Terminating, Some("NORMAL_COMPLETION"))
        .expect("terminating");
    exchange(
        &mut child,
        frame(
            minor,
            4,
            adapter_frame::Message::Stop(wire::Stop {
                reason_code: "NORMAL_COMPLETION".into(),
                deadline_monotonic_ns: 5,
            }),
        ),
        &mut transcript,
    );
    let outcome = child
        .shutdown(Duration::from_secs(5))
        .expect("clean shutdown");
    assert_eq!(outcome.exit_code, Some(0));
    assert!(outcome.failures.is_empty());
    supervisor
        .transition_to(LifecycleState::Completed, Some("COMPLETED"))
        .expect("completed");
    let records: Vec<LifecycleRecord> = fs::read_to_string(life_root.join("lifecycle.jsonl"))
        .expect("journal")
        .lines()
        .map(|line| serde_json::from_str(line).expect("record"))
        .collect();
    let events = transcript
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let sequence = u64::try_from(i).expect("sequence");
            let mut id = vec![0; 16];
            id[..8].copy_from_slice(&(sequence + 1).to_le_bytes());
            ObservedEvent {
                envelope: EventEnvelope {
                    run_id: vec![1; 16],
                    source_id: vec![2; 16],
                    event_id: id,
                    source_sequence: sequence,
                    monotonic_time_ns: sequence + 1,
                    ..Default::default()
                },
                arrival_index: sequence,
            }
        })
        .collect();
    let raw = transcript
        .iter()
        .map(|item| {
            json!({
                "peer":if item.peer==Peer::Supervisor {"supervisor"} else {"adapter"},
                "bytes":item.frame.encode_to_vec(),
            })
        })
        .collect::<Vec<_>>();
    fs::write(
        root.join("transcript.json"),
        serde_json::to_vec_pretty(&raw).expect("json"),
    )
    .expect("transcript");
    fs::write(
        root.join("launch.json"),
        serde_json::to_vec_pretty(&spec.audit).expect("json"),
    )
    .expect("audit");
    let mut hash = String::new();
    for byte in Sha256::digest(action) {
        use std::fmt::Write;
        write!(hash, "{byte:02x}").expect("hex");
    }
    (
        AdapterCertificationInput {
            adapter_id: format!("compatibility-1.{minor}-seed-{seed}"),
            protocol_transcript: transcript,
            capability_audit: Some(spec.audit),
            observed_events: events,
            lifecycle_records: records,
            determinism_probe_sha256: Vec::new(),
            timeout_probe: TimeoutProbe::not_run(),
            track_declaration: track(),
            expected_track: Track::A,
        },
        hash,
    )
}
fn main() {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    assert_eq!(
        args.len(),
        2,
        "usage: adapter_compatibility PYTHON NEW_OUTPUT"
    );
    let python = std::path::absolute(Path::new(&args[0])).expect("absolute Python path");
    assert!(python.is_file(), "Python executable");
    let output = Path::new(&args[1]);
    fs::create_dir(output).expect("new output only");
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/adapter-compatibility/v1/adapter.py")
        .canonicalize()
        .expect("fixture");
    let mut reports = Vec::new();
    for minor in [0, 1] {
        let probe = timeout(
            &output.join(format!("v{minor}-timeout")),
            &python,
            &script,
            minor,
        );
        for seed in [7, 42, 91] {
            let mut runs = Vec::new();
            for trial in 0..2 {
                runs.push(run(
                    &output.join(format!("v{minor}-s{seed}-t{trial}")),
                    &python,
                    &script,
                    minor,
                    seed,
                ));
            }
            let hashes = runs
                .iter()
                .map(|(_, hash)| hash.clone())
                .collect::<Vec<_>>();
            for (mut input, _) in runs {
                input.determinism_probe_sha256.clone_from(&hashes);
                input.timeout_probe = probe.clone();
                let report = AdapterConformanceSuite.certify(&input).expect("report");
                assert_eq!(report.verdict, CertificationVerdict::Certified);
                assert!(!report.scientific_quality_certified);
                reports.push(report);
            }
        }
    }
    fs::write(
        output.join("reports.json"),
        serde_json::to_vec_pretty(&reports).expect("json"),
    )
    .expect("reports");
    println!(
        "{}",
        json!({"certified":reports.len(),"versions":["1.0","1.1"],"seeds":[7,42,91],
        "trials_per_seed":2,"scientific_quality_certified":false})
    );
}
