use bonsai_contracts::bonsai::event::v1::EventEnvelope;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub type EventId = [u8; 16];
pub type SourceId = [u8; 16];

#[derive(Clone, Debug, PartialEq)]
pub struct ObservedEvent {
    pub envelope: EventEnvelope,
    pub arrival_index: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrderingLimits {
    pub maximum_observations: usize,
    pub maximum_causal_parents_per_event: usize,
}

impl Default for OrderingLimits {
    fn default() -> Self {
        Self {
            maximum_observations: 100_000,
            maximum_causal_parents_per_event: 64,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrderingError {
    LimitExceeded,
    InvalidIdentity,
    DuplicateArrivalIndex,
}

impl OrderingError {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::LimitExceeded => "ORDERING_LIMIT_EXCEEDED",
            Self::InvalidIdentity => "ORDERING_ID_INVALID",
            Self::DuplicateArrivalIndex => "ORDERING_ARRIVAL_INDEX_DUPLICATE",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum OrderEdgeKind {
    SourceSequence,
    CausalParent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderEdge {
    pub before: EventId,
    pub after: EventId,
    pub kinds: BTreeSet<OrderEdgeKind>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EventPair {
    pub first: EventId,
    pub second: EventId,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MissingParent {
    pub event_id: EventId,
    pub parent_id: EventId,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SequenceConflict {
    pub source_id: SourceId,
    pub source_sequence: u64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SequenceGap {
    pub source_id: SourceId,
    pub lower_sequence: u64,
    pub higher_sequence: u64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EventOrderFlag {
    Duplicate,
    Late,
    MissingParent,
    ClockRegression,
    InCycle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventOrderClass {
    pub event_id: EventId,
    pub flags: BTreeSet<EventOrderFlag>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderingReport {
    pub events: Vec<EventOrderClass>,
    pub edges: Vec<OrderEdge>,
    pub concurrent_pairs: Vec<EventPair>,
    pub duplicate_event_ids: Vec<EventId>,
    pub late_event_ids: Vec<EventId>,
    pub clock_regression_event_ids: Vec<EventId>,
    pub missing_parents: Vec<MissingParent>,
    pub sequence_conflicts: Vec<SequenceConflict>,
    pub sequence_gaps: Vec<SequenceGap>,
    pub cycle_event_ids: Vec<EventId>,
}

/// Derive a deterministic partial order from recorded arrival indices, per-source sequences, and causal edges.
///
/// Collection iteration order and wall-clock values do not affect the report. `arrival_index` is retained only
/// for the arrival-relative `late` class.
///
/// # Errors
///
/// Rejects excessive input, invalid fixed-size identities, excessive parent fan-in, or ambiguous arrival indices.
#[allow(clippy::too_many_lines)]
pub fn classify_partial_order(
    observations: &[ObservedEvent],
    limits: OrderingLimits,
) -> Result<OrderingReport, OrderingError> {
    if observations.len() > limits.maximum_observations
        || limits.maximum_observations == 0
        || limits.maximum_causal_parents_per_event == 0
        || observations.iter().any(|observation| {
            observation.envelope.causal_parent_event_ids.len()
                > limits.maximum_causal_parents_per_event
        })
    {
        return Err(OrderingError::LimitExceeded);
    }

    let mut by_arrival = BTreeMap::new();
    for observation in observations {
        let event_id = id(&observation.envelope.event_id)?;
        let source_id = id(&observation.envelope.source_id)?;
        if by_arrival
            .insert(
                observation.arrival_index,
                (event_id, source_id, observation),
            )
            .is_some()
        {
            return Err(OrderingError::DuplicateArrivalIndex);
        }
    }

    let mut canonical = BTreeMap::<EventId, &ObservedEvent>::new();
    let mut counts = BTreeMap::<EventId, usize>::new();
    let mut late = BTreeSet::new();
    let mut maximum_seen = BTreeMap::<SourceId, u64>::new();
    for (event_id, source_id, observation) in by_arrival.values().copied() {
        *counts.entry(event_id).or_default() += 1;
        canonical.entry(event_id).or_insert(observation);
        let maximum = maximum_seen.entry(source_id).or_insert(0);
        if observation.envelope.source_sequence < *maximum {
            late.insert(event_id);
        }
        *maximum = (*maximum).max(observation.envelope.source_sequence);
    }

    let duplicates = counts
        .iter()
        .filter_map(|(event_id, count)| (*count > 1).then_some(*event_id))
        .collect::<BTreeSet<_>>();
    let mut edges = BTreeMap::<(EventId, EventId), BTreeSet<OrderEdgeKind>>::new();
    let mut missing_parents = BTreeSet::new();
    for (event_id, observation) in &canonical {
        for parent in &observation.envelope.causal_parent_event_ids {
            let parent_id = id(parent)?;
            if canonical.contains_key(&parent_id) {
                edges
                    .entry((parent_id, *event_id))
                    .or_default()
                    .insert(OrderEdgeKind::CausalParent);
            } else {
                missing_parents.insert(MissingParent {
                    event_id: *event_id,
                    parent_id,
                });
            }
        }
    }

    let mut by_source = BTreeMap::<SourceId, BTreeMap<u64, Vec<EventId>>>::new();
    for (event_id, observation) in &canonical {
        let source_id = id(&observation.envelope.source_id)?;
        by_source
            .entry(source_id)
            .or_default()
            .entry(observation.envelope.source_sequence)
            .or_default()
            .push(*event_id);
    }
    let mut sequence_conflicts = BTreeSet::new();
    let mut sequence_gaps = BTreeSet::new();
    let mut clock_regressions = BTreeSet::new();
    for (source_id, sequences) in &by_source {
        for (sequence, event_ids) in sequences {
            if event_ids.len() > 1 {
                sequence_conflicts.insert(SequenceConflict {
                    source_id: *source_id,
                    source_sequence: *sequence,
                });
            }
        }
        let groups = sequences.iter().collect::<Vec<_>>();
        for window in groups.windows(2) {
            let (lower_sequence, lower_ids) = window[0];
            let (higher_sequence, higher_ids) = window[1];
            if *higher_sequence != lower_sequence.saturating_add(1) {
                sequence_gaps.insert(SequenceGap {
                    source_id: *source_id,
                    lower_sequence: *lower_sequence,
                    higher_sequence: *higher_sequence,
                });
                continue;
            }
            if let ([lower_id], [higher_id]) = (lower_ids.as_slice(), higher_ids.as_slice()) {
                edges
                    .entry((*lower_id, *higher_id))
                    .or_default()
                    .insert(OrderEdgeKind::SourceSequence);
                let lower_time = canonical[lower_id].envelope.monotonic_time_ns;
                let higher_time = canonical[higher_id].envelope.monotonic_time_ns;
                if higher_time < lower_time {
                    clock_regressions.insert(*higher_id);
                }
            }
        }
    }

    let adjacency = adjacency(&edges);
    let event_ids = canonical.keys().copied().collect::<Vec<_>>();
    let cycle_event_ids = event_ids
        .iter()
        .copied()
        .filter(|event_id| reaches_cycle(*event_id, &adjacency))
        .collect::<BTreeSet<_>>();
    let mut concurrent_pairs = Vec::new();
    for (index, first) in event_ids.iter().enumerate() {
        for second in &event_ids[index + 1..] {
            if !reaches(*first, *second, &adjacency) && !reaches(*second, *first, &adjacency) {
                concurrent_pairs.push(EventPair {
                    first: *first,
                    second: *second,
                });
            }
        }
    }

    let missing_event_ids = missing_parents
        .iter()
        .map(|missing| missing.event_id)
        .collect::<BTreeSet<_>>();
    let events = event_ids
        .iter()
        .map(|event_id| {
            let mut flags = BTreeSet::new();
            flags.extend(
                [
                    (duplicates.contains(event_id), EventOrderFlag::Duplicate),
                    (late.contains(event_id), EventOrderFlag::Late),
                    (
                        missing_event_ids.contains(event_id),
                        EventOrderFlag::MissingParent,
                    ),
                    (
                        clock_regressions.contains(event_id),
                        EventOrderFlag::ClockRegression,
                    ),
                    (cycle_event_ids.contains(event_id), EventOrderFlag::InCycle),
                ]
                .into_iter()
                .filter_map(|(present, flag)| present.then_some(flag)),
            );
            EventOrderClass {
                event_id: *event_id,
                flags,
            }
        })
        .collect();
    Ok(OrderingReport {
        events,
        edges: edges
            .into_iter()
            .map(|((before, after), kinds)| OrderEdge {
                before,
                after,
                kinds,
            })
            .collect(),
        concurrent_pairs,
        duplicate_event_ids: duplicates.into_iter().collect(),
        late_event_ids: late.into_iter().collect(),
        clock_regression_event_ids: clock_regressions.into_iter().collect(),
        missing_parents: missing_parents.into_iter().collect(),
        sequence_conflicts: sequence_conflicts.into_iter().collect(),
        sequence_gaps: sequence_gaps.into_iter().collect(),
        cycle_event_ids: cycle_event_ids.into_iter().collect(),
    })
}

fn id(bytes: &[u8]) -> Result<EventId, OrderingError> {
    let id: EventId = bytes
        .try_into()
        .map_err(|_| OrderingError::InvalidIdentity)?;
    if id.iter().all(|byte| *byte == 0) {
        Err(OrderingError::InvalidIdentity)
    } else {
        Ok(id)
    }
}

fn adjacency(
    edges: &BTreeMap<(EventId, EventId), BTreeSet<OrderEdgeKind>>,
) -> BTreeMap<EventId, BTreeSet<EventId>> {
    let mut adjacency = BTreeMap::new();
    for (before, after) in edges.keys() {
        adjacency
            .entry(*before)
            .or_insert_with(BTreeSet::new)
            .insert(*after);
    }
    adjacency
}

fn reaches(
    start: EventId,
    target: EventId,
    adjacency: &BTreeMap<EventId, BTreeSet<EventId>>,
) -> bool {
    let mut queue = VecDeque::from([start]);
    let mut visited = BTreeSet::from([start]);
    while let Some(current) = queue.pop_front() {
        for next in adjacency.get(&current).into_iter().flatten() {
            if *next == target {
                return true;
            }
            if visited.insert(*next) {
                queue.push_back(*next);
            }
        }
    }
    false
}

fn reaches_cycle(start: EventId, adjacency: &BTreeMap<EventId, BTreeSet<EventId>>) -> bool {
    let mut queue = adjacency
        .get(&start)
        .into_iter()
        .flatten()
        .copied()
        .collect::<VecDeque<_>>();
    let mut visited = BTreeSet::new();
    while let Some(current) = queue.pop_front() {
        if current == start {
            return true;
        }
        if visited.insert(current) {
            queue.extend(adjacency.get(&current).into_iter().flatten().copied());
        }
    }
    false
}
