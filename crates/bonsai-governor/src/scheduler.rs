//! Dense and event-driven scheduler contracts under matched budgets (BQ-09–BQ-12).

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SchedulerKind {
    Dense,
    EventDriven,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateClass {
    Exact,
    Approximate,
    Delayed,
    Dropped,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerEvent {
    pub event_id: String,
    pub producer: String,
    pub eligible: Vec<String>,
    pub suppressed: Vec<String>,
    pub retained_state: u64,
    pub order: u64,
    pub deadline_ns: Option<u64>,
    pub update: UpdateClass,
    pub work_charged: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerTrace {
    pub schema: String,
    pub kind: SchedulerKind,
    pub stream_id: String,
    pub seed: u64,
    pub budget: u64,
    pub events: Vec<SchedulerEvent>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerConformance {
    pub schema: String,
    pub matched: bool,
    pub dense_work: u64,
    pub event_work: u64,
    pub behavior_delta: i64,
    pub resource_delta: i64,
    pub detail_code: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerError {
    Identity,
    Declaration,
    Budget,
    Stream,
    Arithmetic,
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "SCHEDULER_IDENTITY_INVALID",
            Self::Declaration => "SCHEDULER_DECLARATION_INVALID",
            Self::Budget => "SCHEDULER_BUDGET_UNMATCHED",
            Self::Stream => "SCHEDULER_STREAM_UNMATCHED",
            Self::Arithmetic => "SCHEDULER_ARITHMETIC_OVERFLOW",
        })
    }
}

impl Error for SchedulerError {}

/// Reject an event-driven claim that omits eligibility or suppression accounting.
///
/// # Errors
///
/// Rejects empty identities or undeclared event-driven traces.
pub fn validate_scheduler_trace(trace: &SchedulerTrace) -> Result<(), SchedulerError> {
    if trace.stream_id.is_empty() || trace.events.is_empty() || trace.budget == 0 {
        return Err(SchedulerError::Identity);
    }
    for event in &trace.events {
        if event.event_id.is_empty() || event.producer.is_empty() || event.eligible.is_empty() {
            return Err(SchedulerError::Declaration);
        }
        if event
            .suppressed
            .iter()
            .any(|component| event.eligible.contains(component))
        {
            return Err(SchedulerError::Declaration);
        }
        if trace.kind == SchedulerKind::EventDriven
            && event.suppressed.is_empty()
            && event.update == UpdateClass::Dropped
        {
            return Err(SchedulerError::Declaration);
        }
    }
    Ok(())
}

/// Wake every eligible component and charge all work.
///
/// # Errors
///
/// Rejects malformed component lists.
pub fn dense_schedule(
    stream_id: &str,
    seed: u64,
    budget: u64,
    components: &[&str],
    inputs: &[u64],
) -> Result<SchedulerTrace, SchedulerError> {
    if stream_id.is_empty() || components.is_empty() || inputs.is_empty() {
        return Err(SchedulerError::Identity);
    }
    let component_count = u64::try_from(components.len()).unwrap_or(u64::MAX);
    let mut events = Vec::with_capacity(inputs.len());
    for (index, charge) in inputs.iter().enumerate() {
        events.push(SchedulerEvent {
            event_id: format!("dense-{index}"),
            producer: "semantic_input".to_owned(),
            eligible: components.iter().map(|name| (*name).to_owned()).collect(),
            suppressed: Vec::new(),
            retained_state: 0,
            order: u64::try_from(index).unwrap_or(u64::MAX),
            deadline_ns: None,
            update: UpdateClass::Exact,
            work_charged: charge
                .checked_mul(component_count)
                .ok_or(SchedulerError::Arithmetic)?,
        });
    }
    let trace = SchedulerTrace {
        schema: "bonsai.scheduler-trace/v1".to_owned(),
        kind: SchedulerKind::Dense,
        stream_id: stream_id.to_owned(),
        seed,
        budget,
        events,
    };
    validate_scheduler_trace(&trace)?;
    Ok(trace)
}

/// Queue only declared-eligible work and record deferral or drop.
///
/// # Errors
///
/// Rejects malformed eligibility.
pub fn event_schedule(
    stream_id: &str,
    seed: u64,
    budget: u64,
    eligible: &[&str],
    deferred: &[&str],
    inputs: &[u64],
) -> Result<SchedulerTrace, SchedulerError> {
    if stream_id.is_empty() || eligible.is_empty() || inputs.is_empty() {
        return Err(SchedulerError::Identity);
    }
    let deferred_set = deferred.iter().copied().collect::<BTreeSet<_>>();
    if deferred_set.iter().any(|name| eligible.contains(name)) {
        return Err(SchedulerError::Declaration);
    }
    let eligible_count = u64::try_from(eligible.len()).unwrap_or(u64::MAX);
    let mut events = Vec::with_capacity(inputs.len());
    for (index, charge) in inputs.iter().enumerate() {
        events.push(SchedulerEvent {
            event_id: format!("event-{index}"),
            producer: "semantic_input".to_owned(),
            eligible: eligible.iter().map(|name| (*name).to_owned()).collect(),
            suppressed: deferred.iter().map(|name| (*name).to_owned()).collect(),
            retained_state: u64::try_from(deferred.len()).unwrap_or(0),
            order: u64::try_from(index).unwrap_or(u64::MAX),
            deadline_ns: Some(1_000 + u64::try_from(index).unwrap_or(0)),
            update: if deferred.is_empty() {
                UpdateClass::Exact
            } else {
                UpdateClass::Delayed
            },
            work_charged: charge
                .checked_mul(eligible_count)
                .ok_or(SchedulerError::Arithmetic)?,
        });
    }
    let trace = SchedulerTrace {
        schema: "bonsai.scheduler-trace/v1".to_owned(),
        kind: SchedulerKind::EventDriven,
        stream_id: stream_id.to_owned(),
        seed,
        budget,
        events,
    };
    validate_scheduler_trace(&trace)?;
    Ok(trace)
}

/// Compare dense and event-driven traces under identical streams and budgets.
///
/// # Errors
///
/// Rejects unmatched stream, seed, budget, or missing declarations.
pub fn matched_budget_compare(
    dense: &SchedulerTrace,
    event: &SchedulerTrace,
) -> Result<SchedulerConformance, SchedulerError> {
    validate_scheduler_trace(dense)?;
    validate_scheduler_trace(event)?;
    if dense.kind != SchedulerKind::Dense || event.kind != SchedulerKind::EventDriven {
        return Err(SchedulerError::Declaration);
    }
    if dense.stream_id != event.stream_id || dense.events.len() != event.events.len() {
        return Err(SchedulerError::Stream);
    }
    if dense.seed != event.seed || dense.budget != event.budget {
        return Err(SchedulerError::Budget);
    }
    let dense_work = dense
        .events
        .iter()
        .map(|event| event.work_charged)
        .sum::<u64>();
    let event_work = event
        .events
        .iter()
        .map(|event| event.work_charged)
        .sum::<u64>();
    Ok(SchedulerConformance {
        schema: "bonsai.scheduler-conformance/v1".to_owned(),
        matched: true,
        dense_work,
        event_work,
        behavior_delta: i64::try_from(dense_work).unwrap_or(i64::MAX)
            - i64::try_from(event_work).unwrap_or(i64::MAX),
        resource_delta: i64::try_from(dense_work).unwrap_or(i64::MAX)
            - i64::try_from(event_work).unwrap_or(i64::MAX),
        detail_code: "SCHEDULER_MATCHED_STREAM".to_owned(),
    })
}
