//! Event validation and bounded rejection evidence before immutable append.

#![forbid(unsafe_code)]

use bonsai_bundle::SegmentWriter;
use bonsai_contracts::bonsai::event::v1::EventEnvelope;
use bonsai_contracts::decode_and_validate_event;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IngestLifecycle {
    Created,
    Running,
    Terminating,
    Stopped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaAuthorization {
    pub epoch: u32,
    pub maximum_minor: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceAuthorization {
    pub allowed_event_types: BTreeSet<String>,
    pub maximum_payload_bytes: u32,
    pub maximum_events_per_window: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IngestPolicy {
    pub run_id: [u8; 16],
    pub maximum_envelope_bytes: u32,
    pub maximum_causal_parents: usize,
    pub rate_window_ns: u64,
    pub maximum_rejection_records: usize,
    pub maximum_rejection_bytes: usize,
    pub sources: BTreeMap<[u8; 16], SourceAuthorization>,
    pub schemas: BTreeMap<String, SchemaAuthorization>,
}

impl IngestPolicy {
    /// Validate that every bound and authorization set is usable.
    ///
    /// # Errors
    ///
    /// Returns `INGEST_POLICY_INVALID` for an empty or zero bound.
    pub fn validate(&self) -> Result<(), IngestStateError> {
        if self.run_id.iter().all(|byte| *byte == 0)
            || self.maximum_envelope_bytes == 0
            || self.maximum_causal_parents == 0
            || self.rate_window_ns == 0
            || self.maximum_rejection_records == 0
            || self.maximum_rejection_bytes < 64
            || self.sources.is_empty()
            || self.schemas.is_empty()
            || self.sources.values().any(|source| {
                source.allowed_event_types.is_empty()
                    || source.maximum_payload_bytes == 0
                    || source.maximum_events_per_window == 0
            })
            || self.schemas.values().any(|schema| schema.epoch == 0)
        {
            Err(IngestStateError::PolicyInvalid)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IngestStateError {
    PolicyInvalid,
    Lifecycle,
}

impl IngestStateError {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::PolicyInvalid => "INGEST_POLICY_INVALID",
            Self::Lifecycle => "INGEST_LIFECYCLE_INVALID",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IngestOutcome {
    Accepted { event_id: [u8; 16] },
    Rejected(IngestRejection),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IngestRejection {
    pub code: &'static str,
    pub observed_envelope_bytes: usize,
    pub source_id: Option<[u8; 16]>,
    pub event_id: Option<[u8; 16]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectionLedgerSnapshot {
    pub records: Vec<Vec<u8>>,
    pub retained_bytes: usize,
    pub dropped_records: u64,
}

#[derive(Debug)]
struct RejectionLedger {
    records: VecDeque<Vec<u8>>,
    retained_bytes: usize,
    dropped_records: u64,
}

#[derive(Clone, Copy, Debug)]
struct RateState {
    window_start_ns: u64,
    last_arrival_ns: u64,
    count: u32,
}

/// Stateful validator that is the only route into its borrowed segment writer.
pub struct EventIngestor<'writer> {
    writer: &'writer mut SegmentWriter,
    policy: IngestPolicy,
    lifecycle: IngestLifecycle,
    rates: BTreeMap<[u8; 16], RateState>,
    rejections: RejectionLedger,
}

impl<'writer> EventIngestor<'writer> {
    /// Create a stopped-by-default validator around one open immutable segment.
    ///
    /// # Errors
    ///
    /// Rejects an invalid policy before retaining the writer.
    pub fn new(
        writer: &'writer mut SegmentWriter,
        policy: IngestPolicy,
    ) -> Result<Self, IngestStateError> {
        policy.validate()?;
        Ok(Self {
            writer,
            policy,
            lifecycle: IngestLifecycle::Created,
            rates: BTreeMap::new(),
            rejections: RejectionLedger {
                records: VecDeque::new(),
                retained_bytes: 0,
                dropped_records: 0,
            },
        })
    }

    /// Enter the only lifecycle state that permits append.
    ///
    /// # Errors
    ///
    /// Rejects repeated or out-of-order lifecycle transitions.
    pub fn start(&mut self) -> Result<(), IngestStateError> {
        self.transition(IngestLifecycle::Created, IngestLifecycle::Running)
    }

    /// Stop accepting new events while evidence is finalized.
    ///
    /// # Errors
    ///
    /// Rejects repeated or out-of-order lifecycle transitions.
    pub fn begin_termination(&mut self) -> Result<(), IngestStateError> {
        self.transition(IngestLifecycle::Running, IngestLifecycle::Terminating)
    }

    /// Enter the terminal state after segment finalization coordination.
    ///
    /// # Errors
    ///
    /// Rejects repeated or out-of-order lifecycle transitions.
    pub fn stop(&mut self) -> Result<(), IngestStateError> {
        self.transition(IngestLifecycle::Terminating, IngestLifecycle::Stopped)
    }

    #[must_use]
    pub const fn lifecycle(&self) -> IngestLifecycle {
        self.lifecycle
    }

    /// Validate and append the exact encoded event bytes, or emit bounded rejection evidence.
    #[must_use]
    pub fn ingest(&mut self, encoded: &[u8], observer_arrival_ns: u64) -> IngestOutcome {
        match self.validate_candidate(encoded, observer_arrival_ns) {
            Ok((event, source_id, event_id, next_rate)) => {
                if let Err(error) = self.writer.append(encoded) {
                    return self.reject(
                        encoded.len(),
                        Some(source_id),
                        Some(event_id),
                        error.code(),
                    );
                }
                self.rates.insert(source_id, next_rate);
                let _ = event;
                IngestOutcome::Accepted { event_id }
            }
            Err(rejection) => self.reject(
                encoded.len(),
                rejection.source_id,
                rejection.event_id,
                rejection.code,
            ),
        }
    }

    #[must_use]
    pub fn rejection_ledger(&self) -> RejectionLedgerSnapshot {
        RejectionLedgerSnapshot {
            records: self.rejections.records.iter().cloned().collect(),
            retained_bytes: self.rejections.retained_bytes,
            dropped_records: self.rejections.dropped_records,
        }
    }

    fn validate_candidate(
        &self,
        encoded: &[u8],
        observer_arrival_ns: u64,
    ) -> Result<(EventEnvelope, [u8; 16], [u8; 16], RateState), IngestRejection> {
        if self.lifecycle != IngestLifecycle::Running {
            return Err(rejection("INGEST_LIFECYCLE_PRECONDITION", encoded.len()));
        }
        if encoded.len() > self.policy.maximum_envelope_bytes as usize {
            return Err(rejection("INGEST_ENVELOPE_TOO_LARGE", encoded.len()));
        }
        let event = decode_and_validate_event(encoded)
            .map_err(|error| rejection(error.code(), encoded.len()))?;
        let source_id = exact_id(&event.source_id)
            .ok_or_else(|| rejection("EVENT_ID_INVALID", encoded.len()))?;
        let event_id = exact_id(&event.event_id)
            .ok_or_else(|| rejection("EVENT_ID_INVALID", encoded.len()))?;
        let run_id =
            exact_id(&event.run_id).ok_or_else(|| rejection("EVENT_ID_INVALID", encoded.len()))?;
        let contextual = |code| IngestRejection {
            code,
            observed_envelope_bytes: encoded.len(),
            source_id: Some(source_id),
            event_id: Some(event_id),
        };
        if run_id != self.policy.run_id {
            return Err(contextual("INGEST_RUN_UNAUTHORIZED"));
        }
        let source = self
            .policy
            .sources
            .get(&source_id)
            .ok_or_else(|| contextual("INGEST_SOURCE_UNAUTHORIZED"))?;
        if !source.allowed_event_types.contains(&event.event_type) {
            return Err(contextual("INGEST_EVENT_TYPE_UNAUTHORIZED"));
        }
        if event.payload.len() > source.maximum_payload_bytes as usize {
            return Err(contextual("INGEST_PAYLOAD_TOO_LARGE"));
        }
        if event.causal_parent_event_ids.len() > self.policy.maximum_causal_parents {
            return Err(contextual("INGEST_CAUSAL_PARENT_LIMIT"));
        }
        let schema = self
            .policy
            .schemas
            .get(&event.event_type)
            .ok_or_else(|| contextual("INGEST_SCHEMA_UNAUTHORIZED"))?;
        if event.payload_schema_epoch != schema.epoch
            || event.payload_schema_minor > schema.maximum_minor
        {
            return Err(contextual("INGEST_SCHEMA_UNAUTHORIZED"));
        }
        let next_rate = next_rate_state(
            self.rates.get(&source_id).copied(),
            observer_arrival_ns,
            self.policy.rate_window_ns,
            source.maximum_events_per_window,
        )
        .map_err(contextual)?;
        Ok((event, source_id, event_id, next_rate))
    }

    fn reject(
        &mut self,
        observed_envelope_bytes: usize,
        source_id: Option<[u8; 16]>,
        event_id: Option<[u8; 16]>,
        code: &'static str,
    ) -> IngestOutcome {
        let rejection = IngestRejection {
            code,
            observed_envelope_bytes,
            source_id,
            event_id,
        };
        self.rejections.retain(&rejection, &self.policy);
        IngestOutcome::Rejected(rejection)
    }

    fn transition(
        &mut self,
        expected: IngestLifecycle,
        next: IngestLifecycle,
    ) -> Result<(), IngestStateError> {
        if self.lifecycle != expected {
            return Err(IngestStateError::Lifecycle);
        }
        self.lifecycle = next;
        Ok(())
    }
}

impl RejectionLedger {
    fn retain(&mut self, rejection: &IngestRejection, policy: &IngestPolicy) {
        let evidence = encode_rejection(rejection);
        if evidence.len() > policy.maximum_rejection_bytes {
            self.dropped_records = self.dropped_records.saturating_add(1);
            return;
        }
        while self.records.len() >= policy.maximum_rejection_records
            || self.retained_bytes.saturating_add(evidence.len()) > policy.maximum_rejection_bytes
        {
            let Some(removed) = self.records.pop_front() else {
                break;
            };
            self.retained_bytes = self.retained_bytes.saturating_sub(removed.len());
            self.dropped_records = self.dropped_records.saturating_add(1);
        }
        self.retained_bytes = self.retained_bytes.saturating_add(evidence.len());
        self.records.push_back(evidence);
    }
}

#[derive(Serialize)]
struct RejectionEvidence<'code> {
    schema: &'static str,
    code: &'code str,
    observed_envelope_bytes: usize,
    source_id_hex: Option<String>,
    event_id_hex: Option<String>,
}

fn encode_rejection(rejection: &IngestRejection) -> Vec<u8> {
    let evidence = RejectionEvidence {
        schema: "bonsai.ingest-rejection/v1",
        code: rejection.code,
        observed_envelope_bytes: rejection.observed_envelope_bytes,
        source_id_hex: rejection.source_id.map(hex_id),
        event_id_hex: rejection.event_id.map(hex_id),
    };
    serde_json::to_vec(&evidence)
        .unwrap_or_else(|_| b"{\"code\":\"INGEST_REJECTION_ENCODING\"}".to_vec())
}

fn rejection(code: &'static str, observed_envelope_bytes: usize) -> IngestRejection {
    IngestRejection {
        code,
        observed_envelope_bytes,
        source_id: None,
        event_id: None,
    }
}

fn exact_id(bytes: &[u8]) -> Option<[u8; 16]> {
    bytes.try_into().ok()
}

fn next_rate_state(
    current: Option<RateState>,
    arrival_ns: u64,
    window_ns: u64,
    maximum: u32,
) -> Result<RateState, &'static str> {
    let Some(current) = current else {
        return Ok(RateState {
            window_start_ns: arrival_ns,
            last_arrival_ns: arrival_ns,
            count: 1,
        });
    };
    if arrival_ns < current.last_arrival_ns {
        return Err("INGEST_ARRIVAL_CLOCK_REGRESSION");
    }
    if arrival_ns.saturating_sub(current.window_start_ns) >= window_ns {
        return Ok(RateState {
            window_start_ns: arrival_ns,
            last_arrival_ns: arrival_ns,
            count: 1,
        });
    }
    if current.count >= maximum {
        return Err("INGEST_RATE_LIMIT");
    }
    Ok(RateState {
        window_start_ns: current.window_start_ns,
        last_arrival_ns: arrival_ns,
        count: current.count + 1,
    })
}

fn hex_id(id: [u8; 16]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(32);
    for byte in id {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}
