use bonsai_contracts::resource::WorkClass;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

const ALLOCATED_CLASSES: [WorkClass; 8] = [
    WorkClass::Acting,
    WorkClass::Learning,
    WorkClass::FeatureGeneration,
    WorkClass::OptionLearning,
    WorkClass::ModelLearning,
    WorkClass::Planning,
    WorkClass::Curation,
    WorkClass::Observer,
];
const OBSERVER_INDEX: usize = 7;

/// One exclusive hard partition of the allocator's capacity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassReservation {
    pub work_class: WorkClass,
    pub amount: u64,
}

/// Fixed externally-owned work allocation policy.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AllocationPolicy {
    pub policy_id: String,
    pub total_capacity: u64,
    pub reservations: Vec<ClassReservation>,
    pub evidence_flush_reservation: u64,
}

/// Authority that submitted a work-allocation request.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AllocationAuthority {
    Agent,
    Observer,
}

/// Purpose relevant to the protected observer evidence-flush partition.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AllocationPurpose {
    Ordinary,
    EvidenceFlush,
}

/// One immutable request to consume work from a named class partition.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AllocationRequest {
    pub request_id: String,
    pub authority: AllocationAuthority,
    pub work_class: WorkClass,
    pub purpose: AllocationPurpose,
    pub amount: u64,
}

/// Machine outcome for a work allocation request.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AllocationOutcome {
    Admit,
    Defer,
    Reject,
}

/// Deterministic decision retaining the applicable reservation and exact usage transition.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AllocationDecision {
    pub sequence: u64,
    pub request_id: String,
    pub authority: AllocationAuthority,
    pub work_class: WorkClass,
    pub purpose: AllocationPurpose,
    pub requested: u64,
    pub reservation: u64,
    pub consumed_before: u64,
    pub consumed_after: u64,
    pub outcome: AllocationOutcome,
    pub reason_code: String,
}

/// Stable usage view in canonical work-class order.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassUsage {
    pub work_class: WorkClass,
    pub consumed: u64,
    pub reservation: u64,
}

/// Validated allocator input/state failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllocationError {
    PolicyIdentity,
    PolicyCapacity,
    PolicyClassCoverage,
    PolicyReservation,
    RequestIdentity,
    Arithmetic,
}

impl AllocationError {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::PolicyIdentity => "ALLOCATION_POLICY_ID_INVALID",
            Self::PolicyCapacity => "ALLOCATION_POLICY_CAPACITY_INVALID",
            Self::PolicyClassCoverage => "ALLOCATION_POLICY_CLASS_COVERAGE_INVALID",
            Self::PolicyReservation => "ALLOCATION_POLICY_RESERVATION_INVALID",
            Self::RequestIdentity => "ALLOCATION_REQUEST_INVALID",
            Self::Arithmetic => "ALLOCATION_ARITHMETIC_FAILED",
        }
    }
}

impl fmt::Display for AllocationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl Error for AllocationError {}

impl AllocationPolicy {
    /// Validate complete, non-overlapping reservations for every allocatable class.
    ///
    /// # Errors
    ///
    /// Rejects malformed identity/capacity, missing or duplicate classes, a sum mismatch,
    /// environment allocation, or an evidence reserve outside the observer partition.
    pub fn validate(&self) -> Result<(), AllocationError> {
        if self.policy_id.is_empty()
            || self.policy_id.len() > 96
            || !self
                .policy_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        {
            return Err(AllocationError::PolicyIdentity);
        }
        if self.total_capacity == 0 {
            return Err(AllocationError::PolicyCapacity);
        }
        let mut seen = [false; ALLOCATED_CLASSES.len()];
        let mut amounts = [0_u64; ALLOCATED_CLASSES.len()];
        for reservation in &self.reservations {
            let index =
                class_index(reservation.work_class).ok_or(AllocationError::PolicyClassCoverage)?;
            if reservation.amount == 0 || seen[index] {
                return Err(AllocationError::PolicyClassCoverage);
            }
            seen[index] = true;
            amounts[index] = reservation.amount;
        }
        if seen.iter().any(|present| !present) {
            return Err(AllocationError::PolicyClassCoverage);
        }
        let sum = amounts.iter().try_fold(0_u64, |total, amount| {
            total
                .checked_add(*amount)
                .ok_or(AllocationError::Arithmetic)
        })?;
        if sum != self.total_capacity {
            return Err(AllocationError::PolicyReservation);
        }
        let observer = amounts[OBSERVER_INDEX];
        if self.evidence_flush_reservation == 0 || self.evidence_flush_reservation > observer {
            return Err(AllocationError::PolicyReservation);
        }
        Ok(())
    }
}

/// Stateful deterministic reservation allocator.
#[derive(Clone, Debug)]
pub struct WorkAllocator {
    policy: AllocationPolicy,
    reservations: [u64; ALLOCATED_CLASSES.len()],
    consumed: [u64; ALLOCATED_CLASSES.len()],
    observer_ordinary_consumed: u64,
    next_sequence: u64,
}

impl WorkAllocator {
    /// Construct an empty allocator after validating exact partition coverage.
    ///
    /// # Errors
    ///
    /// Returns the policy validation failure without retaining partial state.
    pub fn new(policy: AllocationPolicy) -> Result<Self, AllocationError> {
        policy.validate()?;
        let mut reservations = [0_u64; ALLOCATED_CLASSES.len()];
        for reservation in &policy.reservations {
            let index =
                class_index(reservation.work_class).ok_or(AllocationError::PolicyClassCoverage)?;
            reservations[index] = reservation.amount;
        }
        Ok(Self {
            policy,
            reservations,
            consumed: [0_u64; ALLOCATED_CLASSES.len()],
            observer_ordinary_consumed: 0,
            next_sequence: 0,
        })
    }

    #[must_use]
    pub fn policy(&self) -> &AllocationPolicy {
        &self.policy
    }

    #[must_use]
    pub fn usage(&self) -> Vec<ClassUsage> {
        ALLOCATED_CLASSES
            .iter()
            .enumerate()
            .map(|(index, work_class)| ClassUsage {
                work_class: *work_class,
                consumed: self.consumed[index],
                reservation: self.reservations[index],
            })
            .collect()
    }

    /// Decide and, only on admission, atomically consume one class reservation.
    ///
    /// # Errors
    ///
    /// Rejects malformed request identity or checked-arithmetic failure. Authority and
    /// capacity denials are retained as normal machine decisions and never mutate usage.
    pub fn allocate(
        &mut self,
        request: &AllocationRequest,
    ) -> Result<AllocationDecision, AllocationError> {
        validate_request(request)?;
        let sequence = self.next_sequence;
        let next_sequence = sequence.checked_add(1).ok_or(AllocationError::Arithmetic)?;
        let Some(index) = class_index(request.work_class) else {
            self.next_sequence = next_sequence;
            return Ok(decision(
                sequence,
                request,
                0,
                0,
                AllocationOutcome::Reject,
                "ALLOCATION_WORK_CLASS_UNSUPPORTED",
            ));
        };
        let reservation = self.reservations[index];
        let consumed_before = self.consumed[index];
        if !authority_matches(request) {
            self.next_sequence = next_sequence;
            return Ok(decision(
                sequence,
                request,
                reservation,
                consumed_before,
                AllocationOutcome::Reject,
                "ALLOCATION_AUTHORITY_MISMATCH",
            ));
        }

        let projected = consumed_before
            .checked_add(request.amount)
            .ok_or(AllocationError::Arithmetic)?;
        let observer_ordinary_projection = self
            .observer_ordinary_consumed
            .checked_add(request.amount)
            .ok_or(AllocationError::Arithmetic)?;
        let observer_ordinary_limit = reservation
            .checked_sub(self.policy.evidence_flush_reservation)
            .ok_or(AllocationError::Arithmetic)?;
        let ordinary_hits_flush_reserve = request.work_class == WorkClass::Observer
            && request.purpose == AllocationPurpose::Ordinary
            && observer_ordinary_projection > observer_ordinary_limit;
        if ordinary_hits_flush_reserve {
            self.next_sequence = next_sequence;
            return Ok(decision(
                sequence,
                request,
                reservation,
                consumed_before,
                AllocationOutcome::Defer,
                "OBSERVER_EVIDENCE_FLUSH_RESERVATION_PROTECTED",
            ));
        }
        if projected > reservation {
            self.next_sequence = next_sequence;
            return Ok(decision(
                sequence,
                request,
                reservation,
                consumed_before,
                AllocationOutcome::Defer,
                "WORK_CLASS_RESERVATION_EXHAUSTED",
            ));
        }

        self.consumed[index] = projected;
        if request.work_class == WorkClass::Observer
            && request.purpose == AllocationPurpose::Ordinary
        {
            self.observer_ordinary_consumed = observer_ordinary_projection;
        }
        self.next_sequence = next_sequence;
        Ok(decision(
            sequence,
            request,
            reservation,
            consumed_before,
            AllocationOutcome::Admit,
            "WORK_CLASS_RESERVATION_AVAILABLE",
        ))
    }
}

fn validate_request(request: &AllocationRequest) -> Result<(), AllocationError> {
    if request.request_id.is_empty()
        || request.request_id.len() > 96
        || request.amount == 0
        || !request
            .request_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        Err(AllocationError::RequestIdentity)
    } else {
        Ok(())
    }
}

const fn authority_matches(request: &AllocationRequest) -> bool {
    match request.authority {
        AllocationAuthority::Agent => {
            !matches!(
                request.work_class,
                WorkClass::Observer | WorkClass::Environment
            ) && matches!(request.purpose, AllocationPurpose::Ordinary)
        }
        AllocationAuthority::Observer => {
            matches!(request.work_class, WorkClass::Observer)
        }
    }
}

fn decision(
    sequence: u64,
    request: &AllocationRequest,
    reservation: u64,
    consumed: u64,
    outcome: AllocationOutcome,
    reason_code: &str,
) -> AllocationDecision {
    let consumed_after = if outcome == AllocationOutcome::Admit {
        consumed.saturating_add(request.amount)
    } else {
        consumed
    };
    AllocationDecision {
        sequence,
        request_id: request.request_id.clone(),
        authority: request.authority,
        work_class: request.work_class,
        purpose: request.purpose,
        requested: request.amount,
        reservation,
        consumed_before: consumed,
        consumed_after,
        outcome,
        reason_code: reason_code.to_owned(),
    }
}

const fn class_index(work_class: WorkClass) -> Option<usize> {
    match work_class {
        WorkClass::Acting => Some(0),
        WorkClass::Learning => Some(1),
        WorkClass::FeatureGeneration => Some(2),
        WorkClass::OptionLearning => Some(3),
        WorkClass::ModelLearning => Some(4),
        WorkClass::Planning => Some(5),
        WorkClass::Curation => Some(6),
        WorkClass::Observer => Some(7),
        WorkClass::Environment => None,
    }
}
