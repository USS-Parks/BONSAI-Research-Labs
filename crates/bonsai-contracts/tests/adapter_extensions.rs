use bonsai_contracts::adapter::{
    AdapterProtocolMachine, Peer, ProtocolState, ProtocolViolation, capability_fingerprint,
};
use bonsai_contracts::bonsai::adapter::v1::{
    AdapterFrame, CapabilityDeclaration, Handshake, Start, Stop, VersionRange, adapter_frame,
};
use prost::Message;
use sha2::{Digest, Sha256};

fn caps() -> CapabilityDeclaration {
    CapabilityDeclaration {
        reset: Some(true),
        work: Some(true),
        feedback: Some(true),
        asynchronous_events: Some(false),
        accepted_input_types: vec!["bonsai.fixture/v1".into()],
        retains_transitions: Some(false),
        offline_updates: Some(false),
        observer_data_access: Some(false),
        privileged_state_access: Some(false),
        filesystem_read: Some(false),
        filesystem_write: Some(true),
        network_access: Some(false),
        ..Default::default()
    }
}
fn begin() -> AdapterProtocolMachine {
    let mut machine = AdapterProtocolMachine::default();
    machine
        .apply(
            Peer::Supervisor,
            &AdapterFrame {
                protocol_epoch: 1,
                message: Some(adapter_frame::Message::Start(Start {
                    run_id: vec![1; 16],
                    accepted_versions: Some(VersionRange {
                        minimum_epoch: 1,
                        minimum_minor: 0,
                        maximum_epoch: 2,
                        maximum_minor: 10,
                    }),
                    deterministic_seed: 1,
                    deadline_monotonic_ns: 1,
                })),
                ..Default::default()
            },
        )
        .expect("start with broad peer offer");
    machine
}
fn handshake(epoch: u32, minor: u32, caps: CapabilityDeclaration) -> AdapterFrame {
    AdapterFrame {
        protocol_epoch: epoch,
        protocol_minor: minor,
        capability_fingerprint_sha256: capability_fingerprint(&caps).to_vec(),
        message: Some(adapter_frame::Message::Handshake(Handshake {
            selected_epoch: epoch,
            selected_minor: minor,
            capabilities: Some(caps),
        })),
        ..Default::default()
    }
}
#[test]
fn compatible_versions_are_pinned_and_breaking_versions_refused_even_when_offered() {
    for minor in [0, 1] {
        let mut machine = begin();
        machine
            .apply(Peer::Adapter, &handshake(1, minor, caps()))
            .expect("compatible");
        let mut stop = AdapterFrame {
            sequence: 1,
            protocol_epoch: 1,
            protocol_minor: 1 - minor,
            message: Some(adapter_frame::Message::Stop(Stop {
                reason_code: "COMPATIBILITY_PROBE".into(),
                deadline_monotonic_ns: 2,
            })),
            ..Default::default()
        };
        assert_eq!(
            machine.apply(Peer::Supervisor, &stop),
            Err(ProtocolViolation::VersionMismatch)
        );
        assert_eq!(machine.state(), ProtocolState::AwaitingConfigure);
        stop.protocol_minor = minor;
        machine
            .apply(Peer::Supervisor, &stop)
            .expect("same sequence remains usable");
    }
    for (epoch, minor) in [(2, 0), (1, 2)] {
        let mut machine = begin();
        assert_eq!(
            machine.apply(Peer::Adapter, &handshake(epoch, minor, caps())),
            Err(ProtocolViolation::VersionMismatch)
        );
        machine
            .apply(Peer::Adapter, &handshake(1, 0, caps()))
            .expect("rejection did not change handshake state");
    }
}
#[test]
fn required_extensions_fail_closed_and_optional_names_confer_no_authority() {
    for name in [
        "bonsai.observation/v1",
        "bonsai.action/v1",
        "bonsai.work/v1",
        "bonsai.feedback/v1",
    ] {
        let mut c = caps();
        c.required_capabilities.push(name.into());
        begin()
            .apply(Peer::Adapter, &handshake(1, 1, c))
            .expect("supported requirement");
    }
    for name in [
        "vendor.unknown/v1",
        "bonsai.checkpoint/v1",
        "bonsai.artifact-events/v1",
    ] {
        let mut c = caps();
        c.required_capabilities.push(name.into());
        assert_eq!(
            begin().apply(Peer::Adapter, &handshake(1, 1, c)),
            Err(ProtocolViolation::RequiredCapabilityUnsupported)
        );
    }
    let mut c = caps();
    c.optional_capabilities.push("vendor.unknown/v1".into());
    begin()
        .apply(Peer::Adapter, &handshake(1, 1, c.clone()))
        .expect("unknown optional hint");
    assert_eq!(
        begin().apply(Peer::Adapter, &handshake(1, 0, c.clone())),
        Err(ProtocolViolation::VersionMismatch)
    );
    c.required_capabilities.push("vendor.unknown/v1".into());
    assert_eq!(
        begin().apply(Peer::Adapter, &handshake(1, 1, c)),
        Err(ProtocolViolation::CapabilityDeclaration)
    );
    let mut c = caps();
    c.work = Some(false);
    c.required_capabilities.push("bonsai.work/v1".into());
    assert_eq!(
        begin().apply(Peer::Adapter, &handshake(1, 1, c)),
        Err(ProtocolViolation::RequiredCapabilityUnsupported)
    );
}
#[test]
fn unknown_optional_envelope_fields_are_ignored_but_unknown_capability_bytes_cannot_change_identity()
 {
    let frame = handshake(1, 1, caps());
    let mut bytes = frame.encode_to_vec();
    // Unknown field 100, varint 7: endpoint skips it; evidence retains original raw bytes.
    bytes.extend_from_slice(&[0xa0, 0x06, 0x07]);
    let decoded = AdapterFrame::decode(bytes.as_slice()).expect("unknown optional field");
    assert_eq!(decoded, frame);
    begin()
        .apply(Peer::Adapter, &decoded)
        .expect("optional frame extension");
    let mut changed = frame;
    let mut unknown_caps = caps().encode_to_vec();
    unknown_caps.extend_from_slice(&[0xa0, 0x06, 0x07]);
    let decoded_caps =
        CapabilityDeclaration::decode(unknown_caps.as_slice()).expect("future field");
    assert_eq!(decoded_caps, caps());
    changed.capability_fingerprint_sha256 = Sha256::digest(unknown_caps).to_vec();
    assert_eq!(
        begin().apply(Peer::Adapter, &changed),
        Err(ProtocolViolation::CapabilityDeclaration)
    );
}
