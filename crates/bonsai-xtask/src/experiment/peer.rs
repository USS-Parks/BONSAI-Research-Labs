use super::evidence::Evidence;
use bonsai_contracts::adapter::{AdapterProtocolMachine, Peer as ProtocolPeer};
use bonsai_contracts::bonsai::adapter::v1::{self as wire, adapter_frame::Message as Kind};
use bonsai_runtime::{AgentLaunchPolicy, ChildTransport};
use prost::Message;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

pub(super) struct AdapterPeer {
    pub(super) transport: ChildTransport,
    machine: AdapterProtocolMachine,
    pub(super) name: &'static str,
    sequence: u64,
    last_deadline: u64,
    clock: Instant,
    isolation: Option<AgentLaunchPolicy>,
}

impl AdapterPeer {
    pub(super) fn new(
        name: &'static str,
        transport: ChildTransport,
        isolation: Option<AgentLaunchPolicy>,
    ) -> Self {
        Self {
            transport,
            machine: AdapterProtocolMachine::default(),
            name,
            sequence: 0,
            last_deadline: 0,
            clock: Instant::now(),
            isolation,
        }
    }

    pub(super) fn exchange(
        &mut self,
        mut kind: Kind,
        timeout: Duration,
        log: &mut Evidence<'_>,
    ) -> Result<Kind, String> {
        let elapsed = u64::try_from(self.clock.elapsed().as_nanos()).map_err(|e| e.to_string())?;
        let timeout_ns = u64::try_from(timeout.as_nanos()).map_err(|e| e.to_string())?;
        // The wire protocol requires increasing upper deadlines. The local deadline
        // can be earlier and is always enforced across both write and read.
        let deadline = elapsed
            .checked_add(timeout_ns)
            .ok_or("DEADLINE_OVERFLOW")?
            .max(
                self.last_deadline
                    .checked_add(1)
                    .ok_or("DEADLINE_OVERFLOW")?,
            );
        match &mut kind {
            Kind::Start(v) => v.deadline_monotonic_ns = deadline,
            Kind::Configure(v) => v.deadline_monotonic_ns = deadline,
            Kind::Reset(v) => v.deadline_monotonic_ns = deadline,
            Kind::Step(v) => v.deadline_monotonic_ns = deadline,
            Kind::Feedback(v) => v.deadline_monotonic_ns = deadline,
            Kind::Work(v) => v.deadline_monotonic_ns = deadline,
            Kind::Stop(v) => v.deadline_monotonic_ns = deadline,
            _ => return Err("SUPERVISOR_MESSAGE_INVALID".into()),
        }
        let request = wire::AdapterFrame {
            sequence: self.sequence,
            protocol_epoch: 1,
            protocol_minor: 0,
            capability_fingerprint_sha256: Vec::new(),
            message: Some(kind),
        };
        self.machine
            .apply(ProtocolPeer::Supervisor, &request)
            .map_err(|e| e.to_string())?;
        let encoded = request.encode_to_vec();
        if let Some(policy) = &self.isolation {
            policy
                .validate_protocol_payload(&encoded)
                .map_err(|e| e.to_string())?;
        }
        // Persistence remains inside the overall run wall budget, but action latency
        // starts at dispatch and ends after response validation. A slow observer-file
        // flush must not consume a deadline before the action is even sent.
        let dispatched = Instant::now();
        let mut sent_complete = false;
        let mut response_bytes = None;
        let exchanged = (|| {
            self.transport
                .send_with_timeout(&encoded, timeout)
                .map_err(|e| format!("{}:{}", self.name, e.code()))?;
            sent_complete = true;
            let remaining = timeout
                .checked_sub(dispatched.elapsed())
                .ok_or("ACTION_DEADLINE_AFTER_SEND")?;
            let bytes = self
                .transport
                .receive(remaining)
                .map_err(|e| format!("{}:{}", self.name, e.code()))?
                .ok_or_else(|| format!("{}:ADAPTER_EOF", self.name))?;
            response_bytes = Some(bytes);
            let response = wire::AdapterFrame::decode(
                response_bytes
                    .as_deref()
                    .ok_or("ADAPTER_RESPONSE_MISSING")?,
            )
            .map_err(|e| e.to_string())?;
            self.machine
                .apply(ProtocolPeer::Adapter, &response)
                .map_err(|e| format!("{}:{e}", self.name))?;
            if dispatched.elapsed() > timeout {
                return Err("ACTION_RESPONSE_LATE".into());
            }
            response
                .message
                .ok_or_else(|| "ADAPTER_MESSAGE_MISSING".to_owned())
        })();
        let latency = u64::try_from(dispatched.elapsed().as_nanos()).map_err(|e| e.to_string())?;
        log.append(
            "run.protocol",
            &json!({"adapter":self.name,"phase":"request_attempted",
            "frame_hex":hex(&encoded),"local_timeout_ns":timeout_ns,"sent_complete":sent_complete}),
        )?;
        if let Some(bytes) = response_bytes {
            log.append(
                "run.protocol",
                &json!({"adapter":self.name,"phase":"response_received",
                "frame_hex":hex(&bytes),"latency_ns":latency}),
            )?;
        }
        if let Err(error) = &exchanged {
            log.append(
                "run.protocol",
                &json!({"adapter":self.name,"phase":"exchange_failed",
                "latency_ns":latency,"reason_code":error}),
            )?;
        }
        let response = exchanged?;
        self.sequence += 1;
        self.last_deadline = deadline;
        Ok(response)
    }

    pub(super) fn initialize(
        &mut self,
        run_id: [u8; 16],
        seed: u64,
        config: &[u8],
        log: &mut Evidence<'_>,
    ) -> Result<(), String> {
        let response = self.exchange(
            Kind::Start(wire::Start {
                run_id: run_id.to_vec(),
                deterministic_seed: seed,
                accepted_versions: Some(wire::VersionRange {
                    minimum_epoch: 1,
                    minimum_minor: 0,
                    maximum_epoch: 1,
                    maximum_minor: 0,
                }),
                deadline_monotonic_ns: 0,
            }),
            Duration::from_secs(5),
            log,
        )?;
        let Kind::Handshake(handshake) = response else {
            return Err("HANDSHAKE_REQUIRED".into());
        };
        let capabilities = handshake.capabilities.ok_or("CAPABILITIES_REQUIRED")?;
        self.exchange(
            Kind::Configure(wire::Configure {
                configuration_sha256: Sha256::digest(config).to_vec(),
                accepted_capability_fingerprint_sha256: Sha256::digest(
                    capabilities.encode_to_vec(),
                )
                .to_vec(),
                deadline_monotonic_ns: 0,
            }),
            Duration::from_secs(1),
            log,
        )?;
        Ok(())
    }

    pub(super) fn reset(
        &mut self,
        episode: u64,
        seed: u64,
        log: &mut Evidence<'_>,
    ) -> Result<(), String> {
        let mut id = [1_u8; 16];
        id[..8].copy_from_slice(&episode.to_le_bytes());
        self.exchange(
            Kind::Reset(wire::Reset {
                episode_id: id.to_vec(),
                deterministic_seed: seed,
                deadline_monotonic_ns: 0,
            }),
            Duration::from_secs(1),
            log,
        )?;
        Ok(())
    }

    pub(super) fn step(
        &mut self,
        index: u64,
        input_type: &str,
        input: Vec<u8>,
        timeout: Duration,
        log: &mut Evidence<'_>,
    ) -> Result<Vec<u8>, String> {
        let response = self.exchange(
            Kind::Step(wire::Step {
                step_index: index,
                input_type: input_type.into(),
                input_sha256: Sha256::digest(&input).to_vec(),
                input,
                deadline_monotonic_ns: 0,
            }),
            timeout,
            log,
        )?;
        let Kind::StepResult(result) = response else {
            return Err("STEP_RESULT_REQUIRED".into());
        };
        if result.step_index != index {
            return Err("STEP_RESULT_INDEX_MISMATCH".into());
        }
        Ok(result.action)
    }
}

pub(super) fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        output.push(char::from(DIGITS[usize::from(b >> 4)]));
        output.push(char::from(DIGITS[usize::from(b & 15)]));
    }
    output
}
