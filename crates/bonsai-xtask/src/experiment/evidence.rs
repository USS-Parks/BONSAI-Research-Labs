use bonsai_bundle::SegmentWriter;
use bonsai_contracts::bonsai::event::v1::{Availability, EventEnvelope, Precision};
use bonsai_ingest::{
    EventIngestor, IngestOutcome, IngestPolicy, SchemaAuthorization, SourceAuthorization,
};
use prost::Message;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

const TYPES: [&str; 6] = [
    "run.protocol",
    "run.action",
    "run.reward",
    "run.work",
    "run.resource",
    "run.status",
];
const SOURCE: [u8; 16] = [42; 16];
const FRAME_MAX: u32 = 256 * 1024;
const CLOSEOUT_RESERVE: u64 = 64 * 1024;

#[derive(Clone, Copy, Default)]
pub(super) struct EvidenceSummary {
    pub(super) events: u64,
    pub(super) bytes: u64,
    pub(super) rewards: u64,
    pub(super) reward_sum: i64,
}

pub(super) struct Evidence<'a> {
    ingestor: EventIngestor<'a>,
    run_id: [u8; 16],
    sequence: u64,
    previous: Option<Vec<u8>>,
    started: Instant,
    bytes: u64,
    maximum: u64,
    rewards: u64,
    reward_sum: i64,
}

impl<'a> Evidence<'a> {
    pub(super) fn new(
        writer: &'a mut SegmentWriter,
        run_id: [u8; 16],
        maximum: u64,
    ) -> Result<Self, String> {
        if maximum < 2 * CLOSEOUT_RESERVE {
            return Err("OBSERVER_BUDGET_TOO_SMALL".into());
        }
        let policy = IngestPolicy {
            run_id,
            maximum_envelope_bytes: FRAME_MAX,
            maximum_causal_parents: 1,
            rate_window_ns: 1_000_000_000,
            maximum_rejection_records: 16,
            maximum_rejection_bytes: 4096,
            sources: BTreeMap::from([(
                SOURCE,
                SourceAuthorization {
                    allowed_event_types: TYPES.iter().map(|s| (*s).into()).collect::<BTreeSet<_>>(),
                    maximum_payload_bytes: FRAME_MAX - 1024,
                    maximum_events_per_window: 100_000,
                },
            )]),
            schemas: TYPES
                .iter()
                .map(|s| {
                    (
                        (*s).into(),
                        SchemaAuthorization {
                            epoch: 1,
                            maximum_minor: 0,
                        },
                    )
                })
                .collect(),
        };
        let mut ingestor = EventIngestor::new(writer, policy).map_err(|e| e.code().to_owned())?;
        ingestor.start().map_err(|e| e.code().to_owned())?;
        Ok(Self {
            ingestor,
            run_id,
            sequence: 0,
            previous: None,
            started: Instant::now(),
            bytes: 148,
            maximum,
            rewards: 0,
            reward_sum: 0,
        })
    }

    pub(super) fn append(&mut self, kind: &str, value: &Value) -> Result<(), String> {
        let (rewards, reward_sum) = if kind == "run.reward" {
            (
                self.rewards.checked_add(1).ok_or("REWARD_COUNT_OVERFLOW")?,
                self.reward_sum
                    .checked_add(value["reward"].as_i64().ok_or("REWARD_EVENT_INVALID")?)
                    .ok_or("REWARD_SUM_OVERFLOW")?,
            )
        } else {
            (self.rewards, self.reward_sum)
        };
        let payload = serde_json::to_vec(value).map_err(|e| e.to_string())?;
        let mut identity = Sha256::new();
        identity.update(self.run_id);
        identity.update(self.sequence.to_le_bytes());
        let event_id = identity.finalize()[..16].to_vec();
        let now = u64::try_from(self.started.elapsed().as_nanos()).map_err(|e| e.to_string())?;
        let event = EventEnvelope {
            run_id: self.run_id.to_vec(),
            source_id: SOURCE.to_vec(),
            event_id: event_id.clone(),
            source_sequence: self.sequence,
            causal_parent_event_ids: self.previous.iter().cloned().collect(),
            monotonic_time_ns: now,
            wall_time_unix_ns: None,
            event_type: kind.into(),
            payload_schema_epoch: 1,
            payload_schema_minor: 0,
            payload_sha256: Sha256::digest(&payload).to_vec(),
            availability: Availability::Measured as i32,
            precision: Some(Precision {
                representation: "json".into(),
                significant_bits: None,
            }),
            payload,
        }
        .encode_to_vec();
        let next = self
            .bytes
            .checked_add(u64::try_from(event.len()).map_err(|e| e.to_string())? + 44)
            .ok_or("OBSERVER_SIZE_OVERFLOW")?;
        let ceiling = if kind == "run.status"
            || kind == "run.resource"
                && ["cleanup", "before_cleanup"]
                    .iter()
                    .any(|phase| value["phase"] == *phase)
        {
            self.maximum
        } else {
            self.maximum - CLOSEOUT_RESERVE
        };
        if next > ceiling {
            return Err("OBSERVER_STORAGE_EXHAUSTED".into());
        }
        match self.ingestor.ingest(&event, now) {
            IngestOutcome::Accepted { .. } => {
                self.sequence += 1;
                self.previous = Some(event_id);
                self.bytes = next;
                self.rewards = rewards;
                self.reward_sum = reward_sum;
                Ok(())
            }
            IngestOutcome::Rejected(rejection) => Err(rejection.code.into()),
        }
    }

    pub(super) fn finish(mut self) -> Result<EvidenceSummary, String> {
        self.ingestor
            .begin_termination()
            .map_err(|e| e.code().to_owned())?;
        self.ingestor.stop().map_err(|e| e.code().to_owned())?;
        let rejected = self.ingestor.rejection_ledger();
        if !rejected.records.is_empty() || rejected.dropped_records != 0 {
            return Err("REQUIRED_EVENT_REJECTED".into());
        }
        Ok(EvidenceSummary {
            events: self.sequence,
            bytes: self.bytes,
            rewards: self.rewards,
            reward_sum: self.reward_sum,
        })
    }
}
