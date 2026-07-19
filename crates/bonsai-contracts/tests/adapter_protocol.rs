use bonsai_contracts::adapter::{
    AdapterProtocolMachine, Peer, ProtocolState, ProtocolViolation, capability_fingerprint,
};
use bonsai_contracts::bonsai::adapter::v1::{
    Ack, AdapterFrame, CapabilityDeclaration, Configure, Feedback, Handshake, Operation, Reset,
    Start, Step, StepResult, Stop, Stopped, VersionRange, Work, WorkResult, adapter_frame,
};
use sha2::{Digest, Sha256};

fn capabilities() -> CapabilityDeclaration {
    CapabilityDeclaration {
        reset: Some(true),
        work: Some(true),
        feedback: Some(true),
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

fn configured_machine() -> (AdapterProtocolMachine, [u8; 32]) {
    let caps = capabilities();
    let fingerprint = capability_fingerprint(&caps);
    let mut machine = AdapterProtocolMachine::default();
    machine
        .apply(
            Peer::Supervisor,
            &frame(
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
        )
        .expect("start");
    let mut handshake = frame(
        0,
        adapter_frame::Message::Handshake(Handshake {
            selected_epoch: 1,
            selected_minor: 0,
            capabilities: Some(caps),
        }),
    );
    handshake.capability_fingerprint_sha256 = fingerprint.to_vec();
    machine.apply(Peer::Adapter, &handshake).expect("handshake");
    machine
        .apply(
            Peer::Supervisor,
            &frame(
                1,
                adapter_frame::Message::Configure(Configure {
                    configuration_sha256: vec![2; 32],
                    accepted_capability_fingerprint_sha256: fingerprint.to_vec(),
                    deadline_monotonic_ns: 200,
                }),
            ),
        )
        .expect("configure");
    let mut ack = frame(
        1,
        adapter_frame::Message::Ack(Ack {
            operation: Operation::Configure as i32,
        }),
    );
    ack.capability_fingerprint_sha256 = fingerprint.to_vec();
    machine.apply(Peer::Adapter, &ack).expect("configure ack");
    (machine, fingerprint)
}

#[test]
fn valid_handshake_reaches_ready_with_frozen_capabilities() {
    let (machine, _) = configured_machine();
    assert_eq!(machine.state(), ProtocolState::Ready);
}

#[test]
fn out_of_order_and_version_mismatch_fail_without_advancing() {
    let mut machine = AdapterProtocolMachine::default();
    let result = machine.apply(
        Peer::Supervisor,
        &frame(
            0,
            adapter_frame::Message::Configure(Configure {
                configuration_sha256: vec![2; 32],
                accepted_capability_fingerprint_sha256: vec![3; 32],
                deadline_monotonic_ns: 10,
            }),
        ),
    );
    assert!(matches!(result, Err(ProtocolViolation::OutOfOrder { .. })));
    assert_eq!(machine.state(), ProtocolState::Created);

    let caps = capabilities();
    let mut machine = AdapterProtocolMachine::default();
    let mut start = frame(
        0,
        adapter_frame::Message::Start(Start {
            run_id: vec![1; 16],
            accepted_versions: Some(VersionRange {
                minimum_epoch: 1,
                minimum_minor: 0,
                maximum_epoch: 1,
                maximum_minor: 0,
            }),
            deterministic_seed: 7,
            deadline_monotonic_ns: 10,
        }),
    );
    machine.apply(Peer::Supervisor, &start).expect("start");
    start.sequence = 0;
    let mut handshake = frame(
        0,
        adapter_frame::Message::Handshake(Handshake {
            selected_epoch: 2,
            selected_minor: 0,
            capabilities: Some(caps.clone()),
        }),
    );
    handshake.protocol_epoch = 2;
    handshake.capability_fingerprint_sha256 = capability_fingerprint(&caps).to_vec();
    assert_eq!(
        machine.apply(Peer::Adapter, &handshake),
        Err(ProtocolViolation::VersionMismatch)
    );
}

#[test]
fn changed_capability_fingerprint_is_rejected() {
    let (mut machine, fingerprint) = configured_machine();
    machine
        .apply(
            Peer::Supervisor,
            &frame(
                2,
                adapter_frame::Message::Stop(Stop {
                    reason_code: "NORMAL_COMPLETION".to_owned(),
                    deadline_monotonic_ns: 300,
                }),
            ),
        )
        .expect("stop");
    let mut stopped = frame(
        2,
        adapter_frame::Message::Stopped(Stopped {
            outcome_code: "STOPPED".to_owned(),
        }),
    );
    stopped.capability_fingerprint_sha256 = fingerprint.to_vec();
    stopped.capability_fingerprint_sha256[0] ^= 1;
    assert_eq!(
        machine.apply(Peer::Adapter, &stopped),
        Err(ProtocolViolation::CapabilityChanged)
    );
}

#[test]
fn omitted_replay_or_privilege_flag_is_not_treated_as_false() {
    let mut machine = AdapterProtocolMachine::default();
    machine
        .apply(
            Peer::Supervisor,
            &frame(
                0,
                adapter_frame::Message::Start(Start {
                    run_id: vec![1; 16],
                    accepted_versions: Some(VersionRange {
                        minimum_epoch: 1,
                        minimum_minor: 0,
                        maximum_epoch: 1,
                        maximum_minor: 0,
                    }),
                    deterministic_seed: 0,
                    deadline_monotonic_ns: 100,
                }),
            ),
        )
        .expect("start");
    let mut caps = capabilities();
    caps.observer_data_access = None;
    let mut handshake = frame(
        0,
        adapter_frame::Message::Handshake(Handshake {
            selected_epoch: 1,
            selected_minor: 0,
            capabilities: Some(caps.clone()),
        }),
    );
    handshake.capability_fingerprint_sha256 = capability_fingerprint(&caps).to_vec();
    assert_eq!(
        machine.apply(Peer::Adapter, &handshake),
        Err(ProtocolViolation::CapabilityDeclaration)
    );
}

#[test]
fn every_post_stop_frame_is_rejected() {
    let (mut machine, fingerprint) = configured_machine();
    machine
        .apply(
            Peer::Supervisor,
            &frame(
                2,
                adapter_frame::Message::Stop(Stop {
                    reason_code: "NORMAL_COMPLETION".to_owned(),
                    deadline_monotonic_ns: 300,
                }),
            ),
        )
        .expect("stop");
    let mut stopped = frame(
        2,
        adapter_frame::Message::Stopped(Stopped {
            outcome_code: "STOPPED".to_owned(),
        }),
    );
    stopped.capability_fingerprint_sha256 = fingerprint.to_vec();
    machine.apply(Peer::Adapter, &stopped).expect("stopped");
    assert_eq!(machine.state(), ProtocolState::Stopped);
    assert_eq!(
        machine.apply(
            Peer::Supervisor,
            &frame(
                3,
                adapter_frame::Message::Stop(Stop {
                    reason_code: "LATE".to_owned(),
                    deadline_monotonic_ns: 400,
                })
            )
        ),
        Err(ProtocolViolation::PostStop)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn reset_step_work_and_feedback_follow_the_declared_state_machine() {
    let (mut machine, fingerprint) = configured_machine();
    machine
        .apply(
            Peer::Supervisor,
            &frame(
                2,
                adapter_frame::Message::Reset(Reset {
                    episode_id: vec![3; 16],
                    deterministic_seed: 99,
                    deadline_monotonic_ns: 300,
                }),
            ),
        )
        .expect("reset");
    let mut reset_ack = frame(
        2,
        adapter_frame::Message::Ack(Ack {
            operation: Operation::Reset as i32,
        }),
    );
    reset_ack.capability_fingerprint_sha256 = fingerprint.to_vec();
    machine.apply(Peer::Adapter, &reset_ack).expect("reset ack");

    let input = b"observation".to_vec();
    machine
        .apply(
            Peer::Supervisor,
            &frame(
                3,
                adapter_frame::Message::Step(Step {
                    step_index: 0,
                    input_type: "bonsai.fixture/v1".to_owned(),
                    input_sha256: Sha256::digest(&input).to_vec(),
                    input,
                    deadline_monotonic_ns: 400,
                }),
            ),
        )
        .expect("step");
    let action = b"left".to_vec();
    let mut result = frame(
        3,
        adapter_frame::Message::StepResult(StepResult {
            step_index: 0,
            action_sha256: Sha256::digest(&action).to_vec(),
            action,
        }),
    );
    result.capability_fingerprint_sha256 = fingerprint.to_vec();
    machine.apply(Peer::Adapter, &result).expect("step result");

    let payload = b"update".to_vec();
    machine
        .apply(
            Peer::Supervisor,
            &frame(
                4,
                adapter_frame::Message::Work(Work {
                    work_item_id: vec![4; 16],
                    work_class: "learning".to_owned(),
                    payload_sha256: Sha256::digest(&payload).to_vec(),
                    payload,
                    deadline_monotonic_ns: 500,
                }),
            ),
        )
        .expect("work");
    let work_bytes = b"done".to_vec();
    let mut work_result = frame(
        4,
        adapter_frame::Message::WorkResult(WorkResult {
            work_item_id: vec![4; 16],
            outcome: "completed".to_owned(),
            result_sha256: Sha256::digest(&work_bytes).to_vec(),
            result: work_bytes,
        }),
    );
    work_result.capability_fingerprint_sha256 = fingerprint.to_vec();
    machine
        .apply(Peer::Adapter, &work_result)
        .expect("work result");

    let signal = b"reward".to_vec();
    machine
        .apply(
            Peer::Supervisor,
            &frame(
                5,
                adapter_frame::Message::Feedback(Feedback {
                    feedback_id: vec![5; 16],
                    signal_type: "reward/v1".to_owned(),
                    signal_sha256: Sha256::digest(&signal).to_vec(),
                    signal,
                    deadline_monotonic_ns: 600,
                }),
            ),
        )
        .expect("feedback");
    let mut feedback_ack = frame(
        5,
        adapter_frame::Message::Ack(Ack {
            operation: Operation::Feedback as i32,
        }),
    );
    feedback_ack.capability_fingerprint_sha256 = fingerprint.to_vec();
    machine
        .apply(Peer::Adapter, &feedback_ack)
        .expect("feedback ack");
    assert_eq!(machine.state(), ProtocolState::Active);
}
