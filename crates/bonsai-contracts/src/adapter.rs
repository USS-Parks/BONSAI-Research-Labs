//! BONSAI adapter protocol validation and deterministic state machine.

use crate::bonsai::adapter::v1::{
    self as wire, AdapterFrame, CapabilityDeclaration, Operation, adapter_frame,
};
use prost::Message;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt;

pub const PROTOCOL_EPOCH: u32 = 1;
pub const PROTOCOL_MINOR: u32 = 0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Peer {
    Supervisor,
    Adapter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolState {
    Created,
    AwaitingHandshake,
    AwaitingConfigure,
    AwaitingConfigureAck,
    Ready,
    AwaitingResetAck,
    Active,
    AwaitingStepResult,
    AwaitingWorkResult,
    AwaitingFeedbackAck,
    AwaitingStopped,
    Stopped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolViolation {
    MissingMessage,
    OutOfOrder {
        state: ProtocolState,
        message: &'static str,
    },
    WrongPeer,
    Sequence,
    VersionMismatch,
    CapabilityDeclaration,
    CapabilityChanged,
    CapabilityNotDeclared(&'static str),
    InvalidField(&'static str),
    Deadline,
    Digest(&'static str),
    PostStop,
}

impl fmt::Display for ProtocolViolation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ADAPTER_PROTOCOL_{self:?}")
    }
}

impl Error for ProtocolViolation {}

#[derive(Debug)]
pub struct AdapterProtocolMachine {
    state: ProtocolState,
    next_supervisor_sequence: u64,
    next_adapter_sequence: u64,
    accepted_versions: Option<wire::VersionRange>,
    capability_fingerprint: Option<[u8; 32]>,
    capabilities: Option<CapabilityDeclaration>,
    last_deadline_ns: u64,
}

impl Default for AdapterProtocolMachine {
    fn default() -> Self {
        Self {
            state: ProtocolState::Created,
            next_supervisor_sequence: 0,
            next_adapter_sequence: 0,
            accepted_versions: None,
            capability_fingerprint: None,
            capabilities: None,
            last_deadline_ns: 0,
        }
    }
}

impl AdapterProtocolMachine {
    #[must_use]
    pub const fn state(&self) -> ProtocolState {
        self.state
    }

    /// Apply one decoded frame after validating sender, sequence, version, and state.
    ///
    /// # Errors
    ///
    /// Returns a stable protocol violation without changing state when the frame is invalid.
    #[allow(clippy::too_many_lines)]
    pub fn apply(&mut self, peer: Peer, frame: &AdapterFrame) -> Result<(), ProtocolViolation> {
        if self.state == ProtocolState::Stopped {
            return Err(ProtocolViolation::PostStop);
        }
        let message = frame
            .message
            .as_ref()
            .ok_or(ProtocolViolation::MissingMessage)?;
        self.validate_sequence(peer, frame.sequence)?;
        Self::validate_peer(peer, message)?;
        if !matches!(
            self.state,
            ProtocolState::Created | ProtocolState::AwaitingHandshake
        ) {
            self.validate_version_and_fingerprint(frame, peer == Peer::Adapter)?;
        }

        let mut next = self.state;
        match (self.state, message) {
            (ProtocolState::Created, adapter_frame::Message::Start(start)) => {
                validate_uuid(&start.run_id, "start.run_id")?;
                let versions = start
                    .accepted_versions
                    .as_ref()
                    .ok_or(ProtocolViolation::InvalidField("accepted_versions"))?;
                validate_version_range(versions)?;
                validate_deadline(start.deadline_monotonic_ns, self.last_deadline_ns)?;
                self.accepted_versions = Some(*versions);
                self.last_deadline_ns = start.deadline_monotonic_ns;
                next = ProtocolState::AwaitingHandshake;
            }
            (ProtocolState::AwaitingHandshake, adapter_frame::Message::Handshake(handshake)) => {
                let range = self
                    .accepted_versions
                    .as_ref()
                    .ok_or(ProtocolViolation::VersionMismatch)?;
                if !version_in_range(handshake.selected_epoch, handshake.selected_minor, range)
                    || frame.protocol_epoch != handshake.selected_epoch
                    || frame.protocol_minor != handshake.selected_minor
                {
                    return Err(ProtocolViolation::VersionMismatch);
                }
                let capabilities = handshake
                    .capabilities
                    .as_ref()
                    .ok_or(ProtocolViolation::CapabilityDeclaration)?;
                validate_capabilities(capabilities)?;
                let fingerprint = capability_fingerprint(capabilities);
                if frame.capability_fingerprint_sha256.as_slice() != fingerprint {
                    return Err(ProtocolViolation::CapabilityDeclaration);
                }
                self.capability_fingerprint = Some(fingerprint);
                self.capabilities = Some(capabilities.clone());
                next = ProtocolState::AwaitingConfigure;
            }
            (ProtocolState::AwaitingConfigure, adapter_frame::Message::Configure(configure)) => {
                validate_digest(&configure.configuration_sha256, "configuration_sha256")?;
                if configure.accepted_capability_fingerprint_sha256.as_slice()
                    != self
                        .capability_fingerprint
                        .ok_or(ProtocolViolation::CapabilityDeclaration)?
                {
                    return Err(ProtocolViolation::CapabilityChanged);
                }
                validate_deadline(configure.deadline_monotonic_ns, self.last_deadline_ns)?;
                self.last_deadline_ns = configure.deadline_monotonic_ns;
                next = ProtocolState::AwaitingConfigureAck;
            }
            (ProtocolState::AwaitingConfigureAck, adapter_frame::Message::Ack(ack))
                if ack.operation == Operation::Configure as i32 =>
            {
                next = ProtocolState::Ready;
            }
            (
                ProtocolState::Ready | ProtocolState::Active,
                adapter_frame::Message::Reset(reset),
            ) => {
                self.require_capability(|caps| caps.reset == Some(true), "reset")?;
                validate_uuid(&reset.episode_id, "reset.episode_id")?;
                validate_deadline(reset.deadline_monotonic_ns, self.last_deadline_ns)?;
                self.last_deadline_ns = reset.deadline_monotonic_ns;
                next = ProtocolState::AwaitingResetAck;
            }
            (ProtocolState::AwaitingResetAck, adapter_frame::Message::Ack(ack))
                if ack.operation == Operation::Reset as i32 =>
            {
                next = ProtocolState::Active;
            }
            (ProtocolState::Active, adapter_frame::Message::Step(step)) => {
                self.require_input_type(&step.input_type)?;
                validate_payload(&step.input, &step.input_sha256, "step.input")?;
                validate_deadline(step.deadline_monotonic_ns, self.last_deadline_ns)?;
                self.last_deadline_ns = step.deadline_monotonic_ns;
                next = ProtocolState::AwaitingStepResult;
            }
            (ProtocolState::AwaitingStepResult, adapter_frame::Message::StepResult(result)) => {
                validate_payload(&result.action, &result.action_sha256, "step_result.action")?;
                next = ProtocolState::Active;
            }
            (ProtocolState::Active, adapter_frame::Message::Work(work)) => {
                self.require_capability(|caps| caps.work == Some(true), "work")?;
                validate_uuid(&work.work_item_id, "work.work_item_id")?;
                validate_payload(&work.payload, &work.payload_sha256, "work.payload")?;
                validate_deadline(work.deadline_monotonic_ns, self.last_deadline_ns)?;
                self.last_deadline_ns = work.deadline_monotonic_ns;
                next = ProtocolState::AwaitingWorkResult;
            }
            (ProtocolState::AwaitingWorkResult, adapter_frame::Message::WorkResult(result)) => {
                validate_uuid(&result.work_item_id, "work_result.work_item_id")?;
                validate_payload(&result.result, &result.result_sha256, "work_result.result")?;
                next = ProtocolState::Active;
            }
            (ProtocolState::Active, adapter_frame::Message::Feedback(feedback)) => {
                self.require_capability(|caps| caps.feedback == Some(true), "feedback")?;
                validate_uuid(&feedback.feedback_id, "feedback.feedback_id")?;
                validate_payload(&feedback.signal, &feedback.signal_sha256, "feedback.signal")?;
                validate_deadline(feedback.deadline_monotonic_ns, self.last_deadline_ns)?;
                self.last_deadline_ns = feedback.deadline_monotonic_ns;
                next = ProtocolState::AwaitingFeedbackAck;
            }
            (ProtocolState::AwaitingFeedbackAck, adapter_frame::Message::Ack(ack))
                if ack.operation == Operation::Feedback as i32 =>
            {
                next = ProtocolState::Active;
            }
            (ProtocolState::Active, adapter_frame::Message::Event(event)) => {
                self.require_capability(
                    |caps| caps.asynchronous_events == Some(true),
                    "asynchronous_events",
                )?;
                if event.event_envelope.is_empty() {
                    return Err(ProtocolViolation::InvalidField("event_envelope"));
                }
            }
            (_, adapter_frame::Message::Stop(stop)) => {
                if peer != Peer::Supervisor || stop.reason_code.is_empty() {
                    return Err(ProtocolViolation::InvalidField("stop"));
                }
                validate_deadline(stop.deadline_monotonic_ns, self.last_deadline_ns)?;
                self.last_deadline_ns = stop.deadline_monotonic_ns;
                next = ProtocolState::AwaitingStopped;
            }
            (ProtocolState::AwaitingStopped, adapter_frame::Message::Stopped(stopped)) => {
                if stopped.outcome_code.is_empty() {
                    return Err(ProtocolViolation::InvalidField("outcome_code"));
                }
                next = ProtocolState::Stopped;
            }
            _ => {
                return Err(ProtocolViolation::OutOfOrder {
                    state: self.state,
                    message: message_name(message),
                });
            }
        }

        match peer {
            Peer::Supervisor => self.next_supervisor_sequence += 1,
            Peer::Adapter => self.next_adapter_sequence += 1,
        }
        self.state = next;
        Ok(())
    }

    fn validate_sequence(&self, peer: Peer, sequence: u64) -> Result<(), ProtocolViolation> {
        let expected = match peer {
            Peer::Supervisor => self.next_supervisor_sequence,
            Peer::Adapter => self.next_adapter_sequence,
        };
        if sequence == expected {
            Ok(())
        } else {
            Err(ProtocolViolation::Sequence)
        }
    }

    fn validate_peer(
        peer: Peer,
        message: &adapter_frame::Message,
    ) -> Result<(), ProtocolViolation> {
        let supervisor = matches!(
            message,
            adapter_frame::Message::Start(_)
                | adapter_frame::Message::Configure(_)
                | adapter_frame::Message::Reset(_)
                | adapter_frame::Message::Step(_)
                | adapter_frame::Message::Work(_)
                | adapter_frame::Message::Feedback(_)
                | adapter_frame::Message::Stop(_)
        );
        if supervisor == (peer == Peer::Supervisor) {
            Ok(())
        } else {
            Err(ProtocolViolation::WrongPeer)
        }
    }

    fn validate_version_and_fingerprint(
        &self,
        frame: &AdapterFrame,
        adapter_originated: bool,
    ) -> Result<(), ProtocolViolation> {
        if frame.protocol_epoch != PROTOCOL_EPOCH || frame.protocol_minor > PROTOCOL_MINOR {
            return Err(ProtocolViolation::VersionMismatch);
        }
        if adapter_originated
            && frame.capability_fingerprint_sha256.as_slice()
                != self
                    .capability_fingerprint
                    .ok_or(ProtocolViolation::CapabilityDeclaration)?
        {
            return Err(ProtocolViolation::CapabilityChanged);
        }
        Ok(())
    }

    fn require_capability(
        &self,
        predicate: impl FnOnce(&CapabilityDeclaration) -> bool,
        name: &'static str,
    ) -> Result<(), ProtocolViolation> {
        let capabilities = self
            .capabilities
            .as_ref()
            .ok_or(ProtocolViolation::CapabilityDeclaration)?;
        if predicate(capabilities) {
            Ok(())
        } else {
            Err(ProtocolViolation::CapabilityNotDeclared(name))
        }
    }

    fn require_input_type(&self, input_type: &str) -> Result<(), ProtocolViolation> {
        if !input_type.is_empty()
            && self
                .capabilities
                .as_ref()
                .ok_or(ProtocolViolation::CapabilityDeclaration)?
                .accepted_input_types
                .iter()
                .any(|declared| declared == input_type)
        {
            Ok(())
        } else {
            Err(ProtocolViolation::CapabilityNotDeclared("input_type"))
        }
    }
}

#[must_use]
pub fn capability_fingerprint(capabilities: &CapabilityDeclaration) -> [u8; 32] {
    Sha256::digest(capabilities.encode_to_vec()).into()
}

fn validate_version_range(range: &wire::VersionRange) -> Result<(), ProtocolViolation> {
    if range.minimum_epoch == 0
        || (range.minimum_epoch, range.minimum_minor) > (range.maximum_epoch, range.maximum_minor)
    {
        Err(ProtocolViolation::VersionMismatch)
    } else {
        Ok(())
    }
}

fn version_in_range(epoch: u32, minor: u32, range: &wire::VersionRange) -> bool {
    (epoch, minor) >= (range.minimum_epoch, range.minimum_minor)
        && (epoch, minor) <= (range.maximum_epoch, range.maximum_minor)
}

fn validate_capabilities(capabilities: &CapabilityDeclaration) -> Result<(), ProtocolViolation> {
    let explicit_flags = [
        capabilities.reset,
        capabilities.work,
        capabilities.feedback,
        capabilities.asynchronous_events,
        capabilities.retains_transitions,
        capabilities.offline_updates,
        capabilities.observer_data_access,
        capabilities.privileged_state_access,
        capabilities.filesystem_read,
        capabilities.filesystem_write,
        capabilities.network_access,
    ];
    if explicit_flags.iter().any(Option::is_none)
        || capabilities.accepted_input_types.is_empty()
        || capabilities
            .accepted_input_types
            .iter()
            .chain(&capabilities.emitted_event_types)
            .any(String::is_empty)
    {
        return Err(ProtocolViolation::CapabilityDeclaration);
    }
    let mut inputs = capabilities.accepted_input_types.clone();
    inputs.sort();
    inputs.dedup();
    let mut outputs = capabilities.emitted_event_types.clone();
    outputs.sort();
    outputs.dedup();
    if inputs.len() != capabilities.accepted_input_types.len()
        || outputs.len() != capabilities.emitted_event_types.len()
        || (capabilities.asynchronous_events != Some(true) && !outputs.is_empty())
    {
        return Err(ProtocolViolation::CapabilityDeclaration);
    }
    Ok(())
}

fn validate_uuid(value: &[u8], field: &'static str) -> Result<(), ProtocolViolation> {
    if value.len() == 16 && value.iter().any(|byte| *byte != 0) {
        Ok(())
    } else {
        Err(ProtocolViolation::InvalidField(field))
    }
}

fn validate_digest(value: &[u8], field: &'static str) -> Result<(), ProtocolViolation> {
    if value.len() == 32 {
        Ok(())
    } else {
        Err(ProtocolViolation::Digest(field))
    }
}

fn validate_payload(
    payload: &[u8],
    digest: &[u8],
    field: &'static str,
) -> Result<(), ProtocolViolation> {
    if digest == Sha256::digest(payload).as_slice() {
        Ok(())
    } else {
        Err(ProtocolViolation::Digest(field))
    }
}

fn validate_deadline(deadline: u64, previous: u64) -> Result<(), ProtocolViolation> {
    if deadline > previous {
        Ok(())
    } else {
        Err(ProtocolViolation::Deadline)
    }
}

const fn message_name(message: &adapter_frame::Message) -> &'static str {
    match message {
        adapter_frame::Message::Start(_) => "start",
        adapter_frame::Message::Handshake(_) => "handshake",
        adapter_frame::Message::Configure(_) => "configure",
        adapter_frame::Message::Reset(_) => "reset",
        adapter_frame::Message::Step(_) => "step",
        adapter_frame::Message::StepResult(_) => "step_result",
        adapter_frame::Message::Work(_) => "work",
        adapter_frame::Message::WorkResult(_) => "work_result",
        adapter_frame::Message::Feedback(_) => "feedback",
        adapter_frame::Message::Ack(_) => "ack",
        adapter_frame::Message::Event(_) => "event",
        adapter_frame::Message::Stop(_) => "stop",
        adapter_frame::Message::Stopped(_) => "stopped",
        adapter_frame::Message::Error(_) => "error",
    }
}
