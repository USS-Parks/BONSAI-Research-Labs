//! Bounded evidence utility for actual online-adapter certification.
use bonsai_contracts::adapter::{AdapterProtocolMachine, Peer};
use bonsai_contracts::bonsai::adapter::v1::{AdapterFrame, Start, VersionRange, adapter_frame};
use bonsai_contracts::decode_and_validate_event;
use bonsai_contracts::track::{Track, TrackDeclaration};
use bonsai_ingest::ObservedEvent;
use bonsai_runtime::{
    AdapterCertificationInput, AdapterConformanceSuite, AgentCapabilityAudit, AgentLaunchPolicy,
    CertificationVerdict, ChildTransport, IsolatedRunLayout, LifecycleRecord, ProtocolExchange,
    TimeoutProbe, TransportError, TransportLimits,
};
use prost::Message;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write as _,
    path::Path,
    time::{Duration, Instant},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireFrame {
    peer: String,
    hex: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Probe {
    exercised: bool,
    failure_code: Option<String>,
    process_contained: bool,
    deadline_ns: u64,
    observed_ns: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    adapter_id: String,
    protocol_transcript: Vec<WireFrame>,
    capability_audit: AgentCapabilityAudit,
    observed_events: Vec<String>,
    lifecycle_records: Vec<LifecycleRecord>,
    determinism_probe_sha256: Vec<String>,
    timeout_probe: Probe,
    track_declaration: TrackDeclaration,
}
fn hex(bytes: &[u8]) -> String {
    let mut result = String::new();
    for byte in bytes {
        use std::fmt::Write;
        write!(result, "{byte:02x}").expect("hex");
    }
    result
}
fn unhex(value: &str) -> Vec<u8> {
    assert!(value.len().is_multiple_of(2), "hex length");
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII"), 16).expect("hex"))
        .collect()
}
fn write_new(path: &Path, value: &impl serde::Serialize) {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .expect("new evidence file");
    file.write_all(&serde_json::to_vec_pretty(value).expect("JSON"))
        .expect("evidence write");
    file.sync_all().expect("sync evidence");
}
fn certify(input: &Path, output: &Path) {
    assert!(
        fs::metadata(input).expect("input metadata").len() <= 64 * 1024 * 1024,
        "bounded certification input"
    );
    let raw: Input = serde_json::from_slice(&fs::read(input).expect("input")).expect("typed input");
    let transcript = raw
        .protocol_transcript
        .into_iter()
        .map(|item| ProtocolExchange {
            peer: match item.peer.as_str() {
                "supervisor" => Peer::Supervisor,
                "adapter" => Peer::Adapter,
                _ => panic!("peer"),
            },
            frame: AdapterFrame::decode(unhex(&item.hex).as_slice()).expect("frame"),
        })
        .collect();
    let events = raw
        .observed_events
        .into_iter()
        .enumerate()
        .map(|(i, encoded)| ObservedEvent {
            envelope: decode_and_validate_event(&unhex(&encoded)).expect("validated event"),
            arrival_index: u64::try_from(i).expect("arrival"),
        })
        .collect();
    let input = AdapterCertificationInput {
        adapter_id: raw.adapter_id,
        protocol_transcript: transcript,
        capability_audit: Some(raw.capability_audit),
        observed_events: events,
        lifecycle_records: raw.lifecycle_records,
        determinism_probe_sha256: raw.determinism_probe_sha256,
        timeout_probe: TimeoutProbe {
            exercised: raw.timeout_probe.exercised,
            failure_code: raw.timeout_probe.failure_code,
            process_contained: raw.timeout_probe.process_contained,
            deadline_ns: raw.timeout_probe.deadline_ns,
            observed_ns: raw.timeout_probe.observed_ns,
        },
        track_declaration: raw.track_declaration,
        expected_track: Track::A,
    };
    let report = AdapterConformanceSuite
        .certify(&input)
        .expect("certification");
    write_new(output, &report);
    assert_eq!(report.verdict, CertificationVerdict::Certified);
    assert!(!report.scientific_quality_certified);
    println!(
        "{}",
        json!({"adapter":report.adapter_id,"verdict":"certified","checks":report.checks.len()})
    );
}
fn timeout(python: &Path, module: &std::ffi::OsStr, config: &Path, output: &Path) {
    let python = std::path::absolute(python).expect("absolute interpreter");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace");
    let layout = IsolatedRunLayout::create(output).expect("new timeout layout");
    let hash = hex(&Sha256::digest(fs::read(config).expect("configuration")));
    let grant = layout
        .grant_input("configuration", config, &hash)
        .expect("immutable configuration grant");
    let launch = AgentLaunchPolicy::new(layout)
        .build_command(
            python.as_os_str(),
            [
                "-I".into(),
                "-B".into(),
                root.join("scripts/adapter_entrypoint.py").into_os_string(),
                module.to_owned(),
            ],
            &[grant],
        )
        .expect("launch policy");
    let mut child = ChildTransport::spawn(&launch.command, TransportLimits::default())
        .expect("spawn actual adapter");
    let pid = child.process_id();
    let start = AdapterFrame {
        protocol_epoch: 1,
        message: Some(adapter_frame::Message::Start(Start {
            run_id: vec![1; 16],
            accepted_versions: Some(VersionRange {
                minimum_epoch: 1,
                minimum_minor: 0,
                maximum_epoch: 1,
                maximum_minor: 0,
            }),
            deterministic_seed: 42,
            deadline_monotonic_ns: 1,
        })),
        ..Default::default()
    };
    let mut protocol = AdapterProtocolMachine::default();
    protocol
        .apply(Peer::Supervisor, &start)
        .expect("start contract");
    child.send_message(&start).expect("send start");
    let hello: AdapterFrame = child
        .receive_message(Duration::from_secs(10))
        .expect("handshake")
        .expect("frame");
    protocol
        .apply(Peer::Adapter, &hello)
        .expect("actual adapter handshake");
    let now = Instant::now();
    assert_eq!(
        child.receive(Duration::from_millis(100)),
        Err(TransportError::ReadTimeout)
    );
    let observed_ns = u64::try_from(now.elapsed().as_nanos()).expect("duration");
    let outcome = child
        .shutdown(Duration::from_secs(2))
        .expect("owned child contained and reaped");
    assert_ne!(outcome.exit_code, Some(0));
    let probe = json!({"exercised":true,"failure_code":"TRANSPORT_READ_TIMEOUT","process_contained":true,
        "deadline_ns":100_000_000,"observed_ns":observed_ns});
    write_new(&output.join("timeout.json"), &probe);
    write_new(&output.join("launch.json"), &launch.audit);
    write_new(
        &output.join("process.json"),
        &json!({"pid":pid,"exit_code":outcome.exit_code,
        "handshake_hex":hex(&hello.encode_to_vec()),"configuration_sha256":hash,
        "failures":outcome.failures.iter().map(|item|item.code).collect::<Vec<_>>(),
        "os":std::env::consts::OS,"architecture":std::env::consts::ARCH}),
    );
    println!("{probe}");
}
fn main() {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    match args.first().and_then(|arg| arg.to_str()) {
        Some("certify") if args.len() == 3 => certify(Path::new(&args[1]), Path::new(&args[2])),
        Some("timeout") if args.len() == 5 => timeout(
            Path::new(&args[1]),
            &args[2],
            Path::new(&args[3]),
            Path::new(&args[4]),
        ),
        _ => panic!(
            "usage: adapter_certification certify INPUT NEW_OUTPUT | timeout PYTHON MODULE CONFIG NEW_ROOT"
        ),
    }
}
