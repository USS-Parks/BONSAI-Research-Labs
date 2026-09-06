//! Live diagnostic bridge to the existing external work and storage governors.
use bonsai_contracts::resource::WorkClass;
use bonsai_governor::allocation::{
    AllocationAuthority, AllocationOutcome, AllocationPolicy, AllocationPurpose, AllocationRequest,
    ClassReservation, WorkAllocator,
};
use bonsai_governor::storage::{
    PathShape, PersistenceOutcome, PersistenceRequest, RetentionKind, RetentionSignals,
    StorageBroker, StoragePolicy,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::error::Error;
use std::io::{self, BufRead, Read, Write};

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Work {
        request_id: String,
        amount: u64,
    },
    Allocation {
        request_id: String,
        allocated_bytes: u64,
        serialized_bytes: u64,
    },
}

struct Governor {
    work: WorkAllocator,
    storage: StorageBroker,
    memory_limit: u64,
    pending: bool,
}

impl Governor {
    fn new(work_limit: u64, memory_limit: u64, storage_limit: u64) -> Result<Self, Box<dyn Error>> {
        if memory_limit == 0 || memory_limit > 16 * 1024 * 1024 || work_limit > 1_000_000_000 {
            return Err("FEATURE_GOVERNOR_LIMIT_INVALID".into());
        }
        let classes = [
            WorkClass::Acting,
            WorkClass::Learning,
            WorkClass::FeatureGeneration,
            WorkClass::OptionLearning,
            WorkClass::ModelLearning,
            WorkClass::Planning,
            WorkClass::Curation,
            WorkClass::Observer,
        ];
        let reservations = classes
            .into_iter()
            .map(|work_class| ClassReservation {
                work_class,
                amount: if work_class == WorkClass::FeatureGeneration {
                    work_limit
                } else {
                    1
                },
            })
            .collect();
        Ok(Self {
            work: WorkAllocator::new(AllocationPolicy {
                policy_id: "bx12.feature-work.v1".into(),
                total_capacity: work_limit + 7,
                reservations,
                evidence_flush_reservation: 1,
            })?,
            storage: StorageBroker::new(StoragePolicy {
                policy_id: "bx12.feature-state.v1".into(),
                max_bytes: storage_limit,
                max_files: 1,
                max_bounded_state_bytes: storage_limit,
                allow_transition_replay: false,
                allowed_replay_capacity_transitions: 0,
            })?,
            memory_limit,
            pending: false,
        })
    }

    fn apply(&mut self, request: Request) -> Result<Value, Box<dyn Error>> {
        match request {
            Request::Work { request_id, amount } => {
                if self.pending {
                    return Err("FEATURE_ALLOCATION_REQUIRED".into());
                }
                let decision = self.work.allocate(&AllocationRequest {
                    request_id,
                    authority: AllocationAuthority::Agent,
                    work_class: WorkClass::FeatureGeneration,
                    purpose: AllocationPurpose::Ordinary,
                    amount,
                })?;
                self.pending = decision.outcome == AllocationOutcome::Admit;
                Ok(serde_json::to_value(decision)?)
            }
            Request::Allocation {
                request_id,
                allocated_bytes,
                serialized_bytes,
            } => self.allocate(&request_id, allocated_bytes, serialized_bytes),
        }
    }

    fn allocate(
        &mut self,
        request_id: &str,
        allocated: u64,
        serialized: u64,
    ) -> Result<Value, Box<dyn Error>> {
        if !self.pending {
            return Err("FEATURE_WORK_ADMISSION_REQUIRED".into());
        }
        self.pending = false;
        if allocated > self.memory_limit {
            return Ok(
                json!({"outcome":"reject", "reason_code":"FEATURE_RETAINED_MEMORY_LIMIT",
                "request_id":request_id,"requested":allocated,"limit":self.memory_limit,
                "allocated_bytes":allocated,"serialized_bytes":serialized}),
            );
        }
        let usage = self.storage.usage();
        if usage.files_consumed > 0 && serialized <= usage.bytes_consumed {
            return Ok(
                json!({"outcome":"admit", "reason_code":"FEATURE_EXISTING_STATE_RESERVATION",
                "request_id":request_id, "allocated_bytes":allocated,
                "memory_limit":self.memory_limit, "serialized_bytes":serialized,
                "storage_usage":usage}),
            );
        }
        let decision = self.storage.persist(&PersistenceRequest {
            request_id: request_id.into(),
            logical_path: "feature-state.json".into(),
            byte_delta: serialized.saturating_sub(usage.bytes_consumed),
            file_delta: u64::from(usage.files_consumed == 0),
            declared_kind: RetentionKind::BoundedAlgorithmState,
            path_shape: PathShape::RegularFile,
            signals: RetentionSignals {
                bounded_state_bytes: serialized,
                ..RetentionSignals::empty()
            },
        })?;
        Ok(json!({
            "outcome":if decision.outcome == PersistenceOutcome::Admit { "admit" } else { "reject" },
            "reason_code":decision.reason_code,"request_id":request_id,
            "allocated_bytes":allocated,"memory_limit":self.memory_limit,
            "serialized_bytes":serialized,"storage":decision,
        }))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 3 {
        return Err("expected WORK_LIMIT MEMORY_LIMIT SERIALIZED_LIMIT".into());
    }
    let mut governor = Governor::new(args[0].parse()?, args[1].parse()?, args[2].parse()?)?;
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    loop {
        let mut bytes = Vec::new();
        let count = (&mut input).take(4097).read_until(b'\n', &mut bytes)?;
        if count == 0 {
            break;
        }
        if count > 4096 || bytes.last() != Some(&b'\n') {
            return Err("FEATURE_REQUEST_FRAME_INVALID".into());
        }
        let response = governor.apply(serde_json::from_slice(&bytes)?)?;
        serde_json::to_writer(&mut output, &response)?;
        output.write_all(b"\n")?;
        output.flush()?;
    }
    Ok(())
}
