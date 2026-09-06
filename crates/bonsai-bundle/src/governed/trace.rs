use super::snapshot::{Snapshot, digest};
use super::{Result, ensure, number, text};
use bonsai_contracts::adapter::{AdapterProtocolMachine, Peer};
use bonsai_contracts::bonsai::adapter::v1::{AdapterFrame, adapter_frame::Message as Kind};
use bonsai_contracts::bonsai::event::v1::{Availability, EventEnvelope};
use prost::Message;
use serde_json::Value;
use sha2::{Digest, Sha256};

pub(super) const SEGMENT: &str = "telemetry/segment-00000000000000000000.bseg";

struct Event {
    kind: String,
    body: Value,
}

pub(super) struct Trace {
    events: std::vec::IntoIter<Event>,
    machines: [AdapterProtocolMachine; 2],
    pub(super) count: u64,
    pub(super) last_time: u64,
}

impl Trace {
    pub(super) fn load(snapshot: &Snapshot, steps: u64) -> Result<Self> {
        let run = unhex(&snapshot.run_id.replace('-', ""))?;
        ensure(
            run.len() == 16 && run.iter().any(|b| *b != 0),
            "RUN_ID_INVALID",
        )?;
        let bytes = snapshot.bytes(SEGMENT)?;
        ensure(bytes.len() <= 64 * 1024 * 1024, "RUN_TRACE_BOUND_EXCEEDED")?;
        let mut events = Vec::new();
        let mut previous = None;
        let mut last_time = 0;
        let mut failure = None;
        let summary = crate::visit_segment_bytes(bytes, |frame| {
            if failure.is_some() {
                return;
            }
            let result = envelope(frame, &run, previous.as_deref(), events.len(), last_time)
                .and_then(|event| {
                    ensure(
                        u64::try_from(events.len()).map_err(|_| "RUN_SIZE_OVERFLOW")?
                            < steps * 32 + 128,
                        "RUN_EVENT_BOUND_EXCEEDED",
                    )?;
                    last_time = event.monotonic_time_ns;
                    previous = Some(event.event_id);
                    let body = serde_json::from_slice(&event.payload)
                        .map_err(|_| "RUN_EVENT_JSON_INVALID")?;
                    events.push(Event {
                        kind: event.event_type,
                        body,
                    });
                    Ok(())
                });
            if let Err(error) = result {
                failure = Some(error);
            }
        })
        .map_err(|_| "RUN_SEGMENT_INVALID")?;
        if let Some(error) = failure {
            return Err(error);
        }
        ensure(
            summary.sequence == 0
                && summary.maximum_frame_size == 262_144
                && summary.frame_count
                    == u64::try_from(events.len()).map_err(|_| "RUN_SIZE_OVERFLOW")?,
            "RUN_SEGMENT_IDENTITY_INVALID",
        )?;
        Ok(Self {
            count: summary.frame_count,
            events: events.into_iter(),
            machines: std::array::from_fn(|_| AdapterProtocolMachine::default()),
            last_time,
        })
    }

    pub(super) fn take(&mut self, kind: &str, phase: Option<&str>) -> Result<Value> {
        let event = self.events.next().ok_or("RUN_REQUIRED_EVENT_MISSING")?;
        ensure(
            event.kind == kind && phase.is_none_or(|phase| event.body["phase"] == phase),
            "RUN_EVENT_ORDER_INVALID",
        )?;
        Ok(event.body)
    }

    pub(super) fn exchange(&mut self, agent: bool, timeout: u64) -> Result<(Kind, Kind)> {
        let name = if agent { "agent" } else { "environment" };
        let request = self.take("run.protocol", Some("request_attempted"))?;
        ensure(
            request["adapter"] == name
                && request["sent_complete"] == true
                && number(&request, "local_timeout_ns")? == timeout,
            "RUN_PROTOCOL_DISPATCH_INVALID",
        )?;
        let request: AdapterFrame = decode(&unhex(text(&request, "frame_hex")?)?)?;
        let machine = &mut self.machines[usize::from(!agent)];
        machine
            .apply(Peer::Supervisor, &request)
            .map_err(|_| "RUN_PROTOCOL_REQUEST_INVALID")?;
        let response = self.take("run.protocol", Some("response_received"))?;
        ensure(
            response["adapter"] == name && number(&response, "latency_ns")? <= timeout,
            "RUN_PROTOCOL_DEADLINE_VIOLATION",
        )?;
        let response: AdapterFrame = decode(&unhex(text(&response, "frame_hex")?)?)?;
        self.machines[usize::from(!agent)]
            .apply(Peer::Adapter, &response)
            .map_err(|_| "RUN_PROTOCOL_RESPONSE_INVALID")?;
        Ok((
            request.message.ok_or("RUN_PROTOCOL_MESSAGE_MISSING")?,
            response.message.ok_or("RUN_PROTOCOL_MESSAGE_MISSING")?,
        ))
    }

    pub(super) fn finish(self) -> Result<()> {
        ensure(self.events.len() == 0, "RUN_UNEXPECTED_TRAILING_EVENT")
    }
}

fn envelope(
    frame: &[u8],
    run: &[u8],
    previous: Option<&[u8]>,
    sequence: usize,
    time: u64,
) -> Result<EventEnvelope> {
    ensure(frame.len() <= 262_144, "RUN_EVENT_BOUND_EXCEEDED")?;
    let event: EventEnvelope = decode(frame)?;
    let sequence = u64::try_from(sequence).map_err(|_| "RUN_SIZE_OVERFLOW")?;
    let mut hash = Sha256::new();
    hash.update(run);
    hash.update(sequence.to_le_bytes());
    let parents = previous.map_or_else(Vec::new, |id| vec![id.to_vec()]);
    ensure(
        event.run_id == run
            && event.source_id == [42; 16]
            && event.event_id == hash.finalize()[..16]
            && event.source_sequence == sequence
            && event.causal_parent_event_ids == parents
            && event.monotonic_time_ns >= time
            && event.wall_time_unix_ns.is_none()
            && event.payload_schema_epoch == 1
            && event.payload_schema_minor == 0
            && event.availability == Availability::Measured as i32
            && event
                .precision
                .as_ref()
                .is_some_and(|v| v.representation == "json" && v.significant_bits.is_none())
            && event.payload_sha256 == Sha256::digest(&event.payload).as_slice(),
        "RUN_EVENT_CONTINUITY_INVALID",
    )?;
    ensure(
        [
            "run.protocol",
            "run.action",
            "run.reward",
            "run.work",
            "run.resource",
            "run.status",
        ]
        .contains(&event.event_type.as_str()),
        "RUN_EVENT_TYPE_UNSUPPORTED",
    )?;
    Ok(event)
}

pub(super) fn decode<T: Message + Default>(bytes: &[u8]) -> Result<T> {
    let message = T::decode(bytes).map_err(|_| "RUN_WIRE_INVALID")?;
    // Unknown fields, duplicate scalar tags and alternate encodings cannot carry
    // undeclared inputs through an otherwise valid supported reference trace.
    ensure(message.encode_to_vec() == bytes, "RUN_WIRE_NONCANONICAL")?;
    Ok(message)
}

pub(super) fn unhex(value: &str) -> Result<Vec<u8>> {
    ensure(
        value.len().is_multiple_of(2)
            && value.len() <= 524_288
            && value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "RUN_HEX_INVALID",
    )?;
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).map_err(|_| "RUN_HEX_INVALID")?;
            u8::from_str_radix(pair, 16).map_err(|_| "RUN_HEX_INVALID")
        })
        .collect()
}

pub(super) fn hash_matches(value: &Value, key: &str, bytes: &[u8]) -> Result<()> {
    ensure(
        text(value, key)? == digest(bytes),
        "RUN_EVIDENCE_LINK_MISMATCH",
    )
}
