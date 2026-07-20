use bonsai_contracts::adapter::{Peer, capability_fingerprint};
use bonsai_contracts::bonsai::adapter::v1::{
    Ack, AdapterFrame, CapabilityDeclaration, Configure, Handshake, Operation, Reset, Start, Step,
    StepResult, Stop, Stopped, VersionRange, adapter_frame,
};
use bonsai_contracts::bonsai::event::v1::EventEnvelope;
use bonsai_contracts::track::{Track, TrackDeclaration, TransitionAccess, UpdateSchedule};
use bonsai_ingest::ObservedEvent;
use bonsai_runtime::{
    AdapterCertificationInput, AdapterCertificationReport, AdapterConformanceSuite,
    AgentLaunchPolicy, CertificationCheck, CertificationError, CheckVerdict, IsolatedRunLayout,
    LifecycleRecord, LifecycleState, ProtocolExchange, TimeoutProbe,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use tempfile::TempDir;

const PROBE_HASH: &str = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

fn capabilities() -> CapabilityDeclaration {
    CapabilityDeclaration {
        reset: Some(true),
        work: Some(false),
        feedback: Some(false),
        asynchronous_events: Some(false),
        accepted_input_types: vec!["bonsai.fixture/v1".to_owned()],
        emitted_event_types: Vec::new(),
        retains_transitions: Some(false),
        offline_updates: Some(false),
        observer_data_access: Some(false),
        privileged_state_access: Some(false),
        filesystem_read: Some(false),
        filesystem_write: Some(true),
        network_access: Some(false),
    }
}

fn frame(sequence: u64, message: adapter_frame::Message) -> AdapterFrame {
    AdapterFrame {
        sequence,
        protocol_epoch: 1,
        protocol_minor: 0,
        capability_fingerprint_sha256: Vec::new(),
        message: Some(message),
    }
}

fn adapter_frame(
    sequence: u64,
    fingerprint: &[u8; 32],
    message: adapter_frame::Message,
) -> AdapterFrame {
    let mut value = frame(sequence, message);
    value.capability_fingerprint_sha256 = fingerprint.to_vec();
    value
}

#[allow(clippy::too_many_lines)]
fn transcript() -> Vec<ProtocolExchange> {
    let caps = capabilities();
    let fingerprint = capability_fingerprint(&caps);
    let input = b"observation".to_vec();
    let action = b"action".to_vec();
    vec![
        ProtocolExchange {
            peer: Peer::Supervisor,
            frame: frame(
                0,
                adapter_frame::Message::Start(Start {
                    run_id: vec![1; 16],
                    accepted_versions: Some(VersionRange {
                        minimum_epoch: 1,
                        minimum_minor: 0,
                        maximum_epoch: 1,
                        maximum_minor: 0,
                    }),
                    deterministic_seed: 42,
                    deadline_monotonic_ns: 100,
                }),
            ),
        },
        ProtocolExchange {
            peer: Peer::Adapter,
            frame: adapter_frame(
                0,
                &fingerprint,
                adapter_frame::Message::Handshake(Handshake {
                    selected_epoch: 1,
                    selected_minor: 0,
                    capabilities: Some(caps),
                }),
            ),
        },
        ProtocolExchange {
            peer: Peer::Supervisor,
            frame: frame(
                1,
                adapter_frame::Message::Configure(Configure {
                    configuration_sha256: vec![2; 32],
                    accepted_capability_fingerprint_sha256: fingerprint.to_vec(),
                    deadline_monotonic_ns: 200,
                }),
            ),
        },
        ProtocolExchange {
            peer: Peer::Adapter,
            frame: adapter_frame(
                1,
                &fingerprint,
                adapter_frame::Message::Ack(Ack {
                    operation: Operation::Configure as i32,
                }),
            ),
        },
        ProtocolExchange {
            peer: Peer::Supervisor,
            frame: frame(
                2,
                adapter_frame::Message::Reset(Reset {
                    episode_id: vec![3; 16],
                    deterministic_seed: 42,
                    deadline_monotonic_ns: 300,
                }),
            ),
        },
        ProtocolExchange {
            peer: Peer::Adapter,
            frame: adapter_frame(
                2,
                &fingerprint,
                adapter_frame::Message::Ack(Ack {
                    operation: Operation::Reset as i32,
                }),
            ),
        },
        ProtocolExchange {
            peer: Peer::Supervisor,
            frame: frame(
                3,
                adapter_frame::Message::Step(Step {
                    step_index: 0,
                    input_type: "bonsai.fixture/v1".to_owned(),
                    input_sha256: Sha256::digest(&input).to_vec(),
                    input,
                    deadline_monotonic_ns: 400,
                }),
            ),
        },
        ProtocolExchange {
            peer: Peer::Adapter,
            frame: adapter_frame(
                3,
                &fingerprint,
                adapter_frame::Message::StepResult(StepResult {
                    step_index: 0,
                    action_sha256: Sha256::digest(&action).to_vec(),
                    action,
                }),
            ),
        },
        ProtocolExchange {
            peer: Peer::Supervisor,
            frame: frame(
                4,
                adapter_frame::Message::Stop(Stop {
                    reason_code: "NORMAL_COMPLETION".to_owned(),
                    deadline_monotonic_ns: 500,
                }),
            ),
        },
        ProtocolExchange {
            peer: Peer::Adapter,
            frame: adapter_frame(
                4,
                &fingerprint,
                adapter_frame::Message::Stopped(Stopped {
                    outcome_code: "STOPPED".to_owned(),
                }),
            ),
        },
    ]
}

fn event(id: u8, sequence: u64, arrival_index: u64) -> ObservedEvent {
    ObservedEvent {
        envelope: EventEnvelope {
            run_id: vec![1; 16],
            source_id: vec![2; 16],
            event_id: vec![id; 16],
            source_sequence: sequence,
            monotonic_time_ns: sequence.saturating_add(1),
            ..EventEnvelope::default()
        },
        arrival_index,
    }
}

fn strict_track() -> TrackDeclaration {
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

fn good_input(adapter_id: &str) -> AdapterCertificationInput {
    let temporary = TempDir::new().expect("temporary directory");
    let layout = IsolatedRunLayout::create(temporary.path().join("run")).expect("layout");
    let audit = AgentLaunchPolicy::new(layout)
        .build_command("reference-adapter", Vec::new(), &[])
        .expect("isolated launch")
        .audit;
    AdapterCertificationInput {
        adapter_id: adapter_id.to_owned(),
        protocol_transcript: transcript(),
        capability_audit: Some(audit),
        observed_events: vec![event(1, 0, 0), event(2, 1, 1)],
        lifecycle_records: vec![
            LifecycleRecord {
                ordinal: 0,
                state: LifecycleState::Created,
                reason_code: None,
            },
            LifecycleRecord {
                ordinal: 1,
                state: LifecycleState::Running,
                reason_code: None,
            },
            LifecycleRecord {
                ordinal: 2,
                state: LifecycleState::Terminating,
                reason_code: Some("NORMAL_COMPLETION".to_owned()),
            },
            LifecycleRecord {
                ordinal: 3,
                state: LifecycleState::Completed,
                reason_code: Some("COMPLETED".to_owned()),
            },
        ],
        determinism_probe_sha256: vec![PROBE_HASH.to_owned(), PROBE_HASH.to_owned()],
        timeout_probe: TimeoutProbe {
            exercised: true,
            failure_code: Some("TRANSPORT_READ_TIMEOUT".to_owned()),
            process_contained: true,
            deadline_ns: 1_000,
            observed_ns: 1_050,
        },
        track_declaration: strict_track(),
        expected_track: Track::A,
    }
}

fn reports() -> BTreeMap<String, AdapterCertificationReport> {
    let suite = AdapterConformanceSuite;
    let mut cases = BTreeMap::new();
    cases.insert(
        "good".to_owned(),
        suite.certify(&good_input("good")).expect("good report"),
    );

    let mut input = good_input("bad_protocol");
    input.protocol_transcript[1].frame.protocol_epoch = 2;
    cases.insert(
        input.adapter_id.clone(),
        suite.certify(&input).expect("protocol report"),
    );

    let mut input = good_input("bad_isolation");
    input
        .capability_audit
        .as_mut()
        .expect("audit")
        .observer_path_exposed = true;
    cases.insert(
        input.adapter_id.clone(),
        suite.certify(&input).expect("isolation report"),
    );

    let mut input = good_input("bad_ordering");
    input.observed_events[1].envelope.source_sequence = 2;
    cases.insert(
        input.adapter_id.clone(),
        suite.certify(&input).expect("ordering report"),
    );

    let mut input = good_input("bad_lifecycle");
    input.lifecycle_records[1].ordinal = 2;
    cases.insert(
        input.adapter_id.clone(),
        suite.certify(&input).expect("lifecycle report"),
    );

    let mut input = good_input("bad_determinism");
    "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
        .clone_into(&mut input.determinism_probe_sha256[1]);
    cases.insert(
        input.adapter_id.clone(),
        suite.certify(&input).expect("determinism report"),
    );

    let mut input = good_input("bad_timeout");
    input.timeout_probe.process_contained = false;
    cases.insert(
        input.adapter_id.clone(),
        suite.certify(&input).expect("timeout report"),
    );

    let mut input = good_input("hidden_replay");
    input.track_declaration.transition_access = TransitionAccess::Replay;
    input.track_declaration.replay_capacity_transitions = 1;
    cases.insert(
        input.adapter_id.clone(),
        suite.certify(&input).expect("track report"),
    );

    let mut input = good_input("observer_data_access");
    input.track_declaration.observer_data_access = true;
    cases.insert(
        input.adapter_id.clone(),
        suite.certify(&input).expect("observer report"),
    );

    let mut input = good_input("missing_evidence");
    input.protocol_transcript.clear();
    input.capability_audit = None;
    input.observed_events.clear();
    input.lifecycle_records.clear();
    input.determinism_probe_sha256.truncate(1);
    input.timeout_probe = TimeoutProbe::not_run();
    cases.insert(
        input.adapter_id.clone(),
        suite.certify(&input).expect("missing report"),
    );
    cases
}

#[test]
fn committed_good_and_bad_adapter_matrix_has_exact_machine_verdicts() {
    let cases = reports()
        .into_iter()
        .map(|(name, report)| (name, summary(&report)))
        .collect::<BTreeMap<_, _>>();
    let actual = json!({
        "schema": "bonsai.adapter-conformance-outcomes/v1",
        "cases": cases,
    });
    let expected: Value = serde_json::from_str(include_str!(
        "../../../fixtures/adapter-conformance/v1/expected-outcomes.json"
    ))
    .expect("expected outcomes");
    assert_eq!(actual, expected);
}

#[test]
fn every_required_dimension_appears_once_and_scientific_quality_is_excluded() {
    for report in reports().values() {
        assert_eq!(report.checks.len(), 7);
        assert_eq!(
            report
                .checks
                .iter()
                .map(|result| result.check)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                CertificationCheck::Protocol,
                CertificationCheck::Isolation,
                CertificationCheck::Ordering,
                CertificationCheck::Lifecycle,
                CertificationCheck::Determinism,
                CertificationCheck::Timeout,
                CertificationCheck::Classification,
            ])
        );
        assert!(!report.scientific_quality_certified);
    }
}

#[test]
fn certification_is_byte_deterministic_and_invalid_identity_has_no_report() {
    let suite = AdapterConformanceSuite;
    let input = good_input("deterministic.adapter-v1");
    let first = serde_json::to_vec(&suite.certify(&input).expect("first")).expect("serialize");
    let second = serde_json::to_vec(&suite.certify(&input).expect("second")).expect("serialize");
    assert_eq!(first, second);

    let mut unsafe_identity = good_input("unsafe");
    unsafe_identity.adapter_id = "../unsafe".to_owned();
    assert_eq!(
        suite.certify(&unsafe_identity),
        Err(CertificationError::AdapterIdentity)
    );
}

fn summary(report: &AdapterCertificationReport) -> Value {
    let non_pass = report
        .checks
        .iter()
        .filter(|result| result.verdict != CheckVerdict::Pass)
        .map(|result| {
            let check = serde_json::to_value(result.check)
                .expect("serialize check")
                .as_str()
                .expect("check string")
                .to_owned();
            format!("{check}:{}", result.code)
        })
        .collect::<Vec<_>>();
    json!({
        "verdict": report.verdict,
        "derived_track": report.derived_track,
        "non_pass": non_pass,
        "scientific_quality_certified": report.scientific_quality_certified,
    })
}
