//! Cognitive-artifact and lineage contract validation.

use crate::bonsai::artifact::v1::artifact_lifecycle_event::Detail;
use crate::bonsai::artifact::v1::{
    ArtifactBirth, ArtifactCost, ArtifactDisposition, ArtifactDispositionRecord,
    ArtifactLifecycleEvent, ArtifactRevision, ArtifactType, ArtifactUtility, ConsumerAction,
    ConsumerKind, ConsumerLink, LineageRelation, ParentReference, Provenance,
};
use crate::bonsai::event::v1::Availability;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineageValidationError {
    Identity,
    Sequence,
    MissingProvenance,
    ArtifactType,
    DuplicateArtifact,
    DuplicateRevision,
    UnknownArtifact,
    OrphanRevision,
    ParentReference,
    LineageCycle,
    TerminalResurrection,
    Consumer,
    Cost,
    Utility,
    Disposition,
}

impl fmt::Display for LineageValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "ARTIFACT_IDENTITY_INVALID",
            Self::Sequence => "ARTIFACT_LIFECYCLE_SEQUENCE_INVALID",
            Self::MissingProvenance => "ARTIFACT_PROVENANCE_MISSING",
            Self::ArtifactType => "ARTIFACT_TYPE_INVALID",
            Self::DuplicateArtifact => "ARTIFACT_ID_DUPLICATE",
            Self::DuplicateRevision => "ARTIFACT_REVISION_ID_DUPLICATE",
            Self::UnknownArtifact => "ARTIFACT_UNKNOWN",
            Self::OrphanRevision => "ARTIFACT_REVISION_ORPHANED",
            Self::ParentReference => "ARTIFACT_PARENT_INVALID",
            Self::LineageCycle => "ARTIFACT_LINEAGE_CYCLE",
            Self::TerminalResurrection => "ARTIFACT_TERMINAL_RESURRECTION",
            Self::Consumer => "ARTIFACT_CONSUMER_INVALID",
            Self::Cost => "ARTIFACT_COST_INVALID",
            Self::Utility => "ARTIFACT_UTILITY_INVALID",
            Self::Disposition => "ARTIFACT_DISPOSITION_INVALID",
        })
    }
}

impl Error for LineageValidationError {}

#[derive(Clone, Debug)]
struct ArtifactState {
    current_revision_id: Vec<u8>,
    next_sequence: u64,
    terminal: bool,
    consumers: HashSet<(Vec<u8>, i32, Option<Vec<u8>>)>,
}

#[derive(Default)]
struct ContractState {
    artifacts: HashMap<Vec<u8>, ArtifactState>,
    revision_owner: HashMap<Vec<u8>, Vec<u8>>,
    parent_graph: HashMap<Vec<u8>, HashSet<Vec<u8>>>,
    history_entry_ids: HashSet<Vec<u8>>,
}

/// Validate an ordered artifact lifecycle as an immutable contract trace.
///
/// This is a conformance validator, not the runtime registry scheduled for
/// BR-07. It proves that BC-07 messages can express and reject the required
/// lifecycle and provenance cases without prescribing learning internals.
///
/// # Errors
///
/// Returns the first stable contract error observed in lifecycle order.
pub fn validate_artifact_lineage_trace(
    events: &[ArtifactLifecycleEvent],
) -> Result<(), LineageValidationError> {
    let mut state = ContractState::default();
    for event in events {
        state.apply(event)?;
    }
    Ok(())
}

impl ContractState {
    fn apply(&mut self, event: &ArtifactLifecycleEvent) -> Result<(), LineageValidationError> {
        validate_uuid(&event.artifact_id)?;
        validate_uuid(&event.artifact_revision_id)?;
        let detail = event
            .detail
            .as_ref()
            .ok_or(LineageValidationError::Identity)?;

        if matches!(detail, Detail::Birth(_)) {
            if event.lifecycle_sequence != 1 {
                return Err(LineageValidationError::Sequence);
            }
        } else {
            let artifact = self
                .artifacts
                .get(&event.artifact_id)
                .ok_or(LineageValidationError::UnknownArtifact)?;
            if event.lifecycle_sequence != artifact.next_sequence {
                return Err(LineageValidationError::Sequence);
            }
            if !matches!(detail, Detail::Revision(_))
                && event.artifact_revision_id != artifact.current_revision_id
            {
                return Err(LineageValidationError::OrphanRevision);
            }
        }

        match detail {
            Detail::Birth(birth) => self.apply_birth(event, birth),
            Detail::Revision(revision) => self.apply_revision(event, revision),
            Detail::ConsumerLink(link) => self.apply_consumer(event, link),
            Detail::Cost(cost) => self.apply_cost(event, cost),
            Detail::Utility(utility) => self.apply_utility(event, utility),
            Detail::Disposition(disposition) => self.apply_disposition(event, disposition),
        }
    }

    fn apply_birth(
        &mut self,
        event: &ArtifactLifecycleEvent,
        birth: &ArtifactBirth,
    ) -> Result<(), LineageValidationError> {
        if self.artifacts.contains_key(&event.artifact_id) {
            return Err(LineageValidationError::DuplicateArtifact);
        }
        if self
            .revision_owner
            .contains_key(&event.artifact_revision_id)
        {
            return Err(LineageValidationError::DuplicateRevision);
        }
        validate_artifact_type(birth.artifact_type)?;
        validate_sha256(&birth.representation_sha256)?;
        validate_provenance(birth.provenance.as_ref())?;
        let parent_artifacts = self.validate_parents(&event.artifact_id, &birth.parents)?;

        self.revision_owner.insert(
            event.artifact_revision_id.clone(),
            event.artifact_id.clone(),
        );
        self.parent_graph
            .insert(event.artifact_id.clone(), parent_artifacts);
        if graph_has_cycle(&self.parent_graph) {
            self.revision_owner.remove(&event.artifact_revision_id);
            self.parent_graph.remove(&event.artifact_id);
            return Err(LineageValidationError::LineageCycle);
        }
        self.artifacts.insert(
            event.artifact_id.clone(),
            ArtifactState {
                current_revision_id: event.artifact_revision_id.clone(),
                next_sequence: 2,
                terminal: false,
                consumers: HashSet::new(),
            },
        );
        Ok(())
    }

    fn apply_revision(
        &mut self,
        event: &ArtifactLifecycleEvent,
        revision: &ArtifactRevision,
    ) -> Result<(), LineageValidationError> {
        let artifact = self
            .artifacts
            .get(&event.artifact_id)
            .ok_or(LineageValidationError::UnknownArtifact)?;
        if artifact.terminal {
            return Err(LineageValidationError::TerminalResurrection);
        }
        if revision.previous_revision_id != artifact.current_revision_id {
            return Err(LineageValidationError::OrphanRevision);
        }
        validate_uuid(&revision.previous_revision_id)?;
        validate_sha256(&revision.representation_sha256)?;
        validate_provenance(revision.provenance.as_ref())?;
        if self
            .revision_owner
            .contains_key(&event.artifact_revision_id)
        {
            return Err(LineageValidationError::DuplicateRevision);
        }
        let added_parents = self.validate_parents(&event.artifact_id, &revision.parents)?;
        let mut graph = self.parent_graph.clone();
        graph
            .entry(event.artifact_id.clone())
            .or_default()
            .extend(added_parents);
        if graph_has_cycle(&graph) {
            return Err(LineageValidationError::LineageCycle);
        }

        self.parent_graph = graph;
        self.revision_owner.insert(
            event.artifact_revision_id.clone(),
            event.artifact_id.clone(),
        );
        let artifact = self
            .artifacts
            .get_mut(&event.artifact_id)
            .ok_or(LineageValidationError::UnknownArtifact)?;
        artifact
            .current_revision_id
            .clone_from(&event.artifact_revision_id);
        artifact.next_sequence += 1;
        Ok(())
    }

    fn validate_parents(
        &self,
        child_artifact_id: &[u8],
        parents: &[ParentReference],
    ) -> Result<HashSet<Vec<u8>>, LineageValidationError> {
        let mut parent_artifacts = HashSet::new();
        let mut parent_revisions = HashSet::new();
        for parent in parents {
            validate_uuid(&parent.artifact_id)?;
            validate_uuid(&parent.artifact_revision_id)?;
            let relation = LineageRelation::try_from(parent.relation)
                .map_err(|_| LineageValidationError::ParentReference)?;
            if relation == LineageRelation::Unspecified
                || parent.artifact_id == child_artifact_id
                || !parent_revisions.insert(parent.artifact_revision_id.as_slice())
                || self.revision_owner.get(&parent.artifact_revision_id)
                    != Some(&parent.artifact_id)
            {
                return Err(LineageValidationError::ParentReference);
            }
            parent_artifacts.insert(parent.artifact_id.clone());
        }
        Ok(parent_artifacts)
    }

    fn apply_consumer(
        &mut self,
        event: &ArtifactLifecycleEvent,
        link: &ConsumerLink,
    ) -> Result<(), LineageValidationError> {
        validate_provenance(link.provenance.as_ref())?;
        let consumer = link
            .consumer
            .as_ref()
            .ok_or(LineageValidationError::Consumer)?;
        validate_uuid(&consumer.consumer_id)?;
        let kind =
            ConsumerKind::try_from(consumer.kind).map_err(|_| LineageValidationError::Consumer)?;
        let action =
            ConsumerAction::try_from(link.action).map_err(|_| LineageValidationError::Consumer)?;
        if kind == ConsumerKind::Unspecified || action == ConsumerAction::Unspecified {
            return Err(LineageValidationError::Consumer);
        }
        if kind == ConsumerKind::Artifact {
            let revision_id = consumer
                .consumer_artifact_revision_id
                .as_ref()
                .ok_or(LineageValidationError::Consumer)?;
            validate_uuid(revision_id)?;
            if self.revision_owner.get(revision_id) != Some(&consumer.consumer_id) {
                return Err(LineageValidationError::Consumer);
            }
        } else if consumer.consumer_artifact_revision_id.is_some() {
            return Err(LineageValidationError::Consumer);
        }

        let artifact = self
            .artifacts
            .get_mut(&event.artifact_id)
            .ok_or(LineageValidationError::UnknownArtifact)?;
        if artifact.terminal {
            return Err(LineageValidationError::TerminalResurrection);
        }
        let key = (
            consumer.consumer_id.clone(),
            consumer.kind,
            consumer.consumer_artifact_revision_id.clone(),
        );
        let changed = match action {
            ConsumerAction::Link => artifact.consumers.insert(key),
            ConsumerAction::Unlink => artifact.consumers.remove(&key),
            ConsumerAction::Unspecified => false,
        };
        if !changed {
            return Err(LineageValidationError::Consumer);
        }
        artifact.next_sequence += 1;
        Ok(())
    }

    fn apply_cost(
        &mut self,
        event: &ArtifactLifecycleEvent,
        cost: &ArtifactCost,
    ) -> Result<(), LineageValidationError> {
        validate_provenance(cost.provenance.as_ref())?;
        validate_uuid(&cost.cost_entry_id)?;
        if cost.counter_id.is_empty()
            || cost.unit.is_empty()
            || !self.history_entry_ids.insert(cost.cost_entry_id.clone())
            || !valid_observation(
                cost.availability,
                cost.amount.is_some(),
                cost.estimator_id.as_deref(),
                cost.estimator_version.as_deref(),
                cost.unavailable_reason.as_deref(),
            )
        {
            return Err(LineageValidationError::Cost);
        }
        self.advance(event)
    }

    fn apply_utility(
        &mut self,
        event: &ArtifactLifecycleEvent,
        utility: &ArtifactUtility,
    ) -> Result<(), LineageValidationError> {
        validate_provenance(utility.provenance.as_ref())?;
        validate_uuid(&utility.utility_entry_id)?;
        if utility.metric_id.is_empty()
            || utility.metric_version.is_empty()
            || utility.unit.is_empty()
            || utility.estimate.is_some_and(|value| !value.is_finite())
            || !self
                .history_entry_ids
                .insert(utility.utility_entry_id.clone())
            || !valid_observation(
                utility.availability,
                utility.estimate.is_some(),
                utility.estimator_id.as_deref(),
                utility.estimator_version.as_deref(),
                utility.unavailable_reason.as_deref(),
            )
        {
            return Err(LineageValidationError::Utility);
        }
        self.advance(event)
    }

    fn apply_disposition(
        &mut self,
        event: &ArtifactLifecycleEvent,
        record: &ArtifactDispositionRecord,
    ) -> Result<(), LineageValidationError> {
        validate_provenance(record.provenance.as_ref())?;
        let disposition = ArtifactDisposition::try_from(record.disposition)
            .map_err(|_| LineageValidationError::Disposition)?;
        if disposition == ArtifactDisposition::Unspecified {
            return Err(LineageValidationError::Disposition);
        }
        let terminal = matches!(
            disposition,
            ArtifactDisposition::Replaced
                | ArtifactDisposition::Retired
                | ArtifactDisposition::Removed
        );
        if disposition == ArtifactDisposition::Replaced {
            let successor = record
                .successor_artifact_id
                .as_ref()
                .ok_or(LineageValidationError::Disposition)?;
            validate_uuid(successor)?;
            if successor == &event.artifact_id || !self.artifacts.contains_key(successor) {
                return Err(LineageValidationError::Disposition);
            }
        } else if record.successor_artifact_id.is_some() {
            return Err(LineageValidationError::Disposition);
        }

        let artifact = self
            .artifacts
            .get_mut(&event.artifact_id)
            .ok_or(LineageValidationError::UnknownArtifact)?;
        if artifact.terminal {
            return Err(LineageValidationError::TerminalResurrection);
        }
        artifact.terminal = terminal;
        artifact.next_sequence += 1;
        Ok(())
    }

    fn advance(&mut self, event: &ArtifactLifecycleEvent) -> Result<(), LineageValidationError> {
        let artifact = self
            .artifacts
            .get_mut(&event.artifact_id)
            .ok_or(LineageValidationError::UnknownArtifact)?;
        artifact.next_sequence += 1;
        Ok(())
    }
}

fn validate_uuid(bytes: &[u8]) -> Result<(), LineageValidationError> {
    if bytes.len() != 16 || bytes.iter().all(|byte| *byte == 0) {
        return Err(LineageValidationError::Identity);
    }
    Ok(())
}

fn validate_sha256(bytes: &[u8]) -> Result<(), LineageValidationError> {
    if bytes.len() != 32 {
        return Err(LineageValidationError::Identity);
    }
    Ok(())
}

fn validate_artifact_type(value: i32) -> Result<(), LineageValidationError> {
    let artifact_type =
        ArtifactType::try_from(value).map_err(|_| LineageValidationError::ArtifactType)?;
    if artifact_type == ArtifactType::Unspecified {
        return Err(LineageValidationError::ArtifactType);
    }
    Ok(())
}

fn validate_provenance(value: Option<&Provenance>) -> Result<(), LineageValidationError> {
    let provenance = value.ok_or(LineageValidationError::MissingProvenance)?;
    if provenance.producer_id.is_empty()
        || provenance.producer_version.is_empty()
        || provenance.source_event_ids.is_empty()
        || provenance
            .source_event_ids
            .iter()
            .any(|event_id| validate_uuid(event_id).is_err())
        || provenance.method_ids.iter().any(String::is_empty)
    {
        return Err(LineageValidationError::MissingProvenance);
    }
    Ok(())
}

fn valid_observation(
    availability: i32,
    has_value: bool,
    estimator_id: Option<&str>,
    estimator_version: Option<&str>,
    unavailable_reason: Option<&str>,
) -> bool {
    let Ok(availability) = Availability::try_from(availability) else {
        return false;
    };
    match availability {
        Availability::Measured => {
            has_value
                && estimator_id.is_none()
                && estimator_version.is_none()
                && unavailable_reason.is_none()
        }
        Availability::Estimated => {
            has_value
                && estimator_id.is_some_and(|value| !value.is_empty())
                && estimator_version.is_some_and(|value| !value.is_empty())
                && unavailable_reason.is_none()
        }
        Availability::Unavailable | Availability::Excluded => {
            !has_value
                && estimator_id.is_none()
                && estimator_version.is_none()
                && unavailable_reason.is_some_and(|value| !value.is_empty())
        }
        Availability::Unspecified => false,
    }
}

fn graph_has_cycle(graph: &HashMap<Vec<u8>, HashSet<Vec<u8>>>) -> bool {
    fn visit(
        node: &[u8],
        graph: &HashMap<Vec<u8>, HashSet<Vec<u8>>>,
        visiting: &mut HashSet<Vec<u8>>,
        visited: &mut HashSet<Vec<u8>>,
    ) -> bool {
        if visited.contains(node) {
            return false;
        }
        if !visiting.insert(node.to_vec()) {
            return true;
        }
        if graph.get(node).is_some_and(|parents| {
            parents
                .iter()
                .any(|parent| visit(parent, graph, visiting, visited))
        }) {
            return true;
        }
        visiting.remove(node);
        visited.insert(node.to_vec());
        false
    }

    let mut visited = HashSet::new();
    for node in graph.keys() {
        if visit(node, graph, &mut HashSet::new(), &mut visited) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{LineageValidationError, validate_artifact_lineage_trace};
    use crate::bonsai::artifact::v1::artifact_lifecycle_event::Detail;
    use crate::bonsai::artifact::v1::{
        ArtifactBirth, ArtifactCost, ArtifactDisposition, ArtifactDispositionRecord,
        ArtifactLifecycleEvent, ArtifactRevision, ArtifactType, ArtifactUtility, ConsumerAction,
        ConsumerKind, ConsumerLink, ConsumerReference, LineageRelation, ParentReference,
        Provenance,
    };
    use crate::bonsai::event::v1::Availability;

    fn id(byte: u8) -> Vec<u8> {
        vec![byte; 16]
    }

    fn provenance(byte: u8) -> Provenance {
        Provenance {
            producer_id: "fixture-producer".to_owned(),
            producer_version: "1.0".to_owned(),
            source_event_ids: vec![id(byte)],
            method_ids: vec!["fixture-method/v1".to_owned()],
        }
    }

    fn birth(
        artifact: u8,
        revision: u8,
        artifact_type: ArtifactType,
        parents: Vec<ParentReference>,
    ) -> ArtifactLifecycleEvent {
        ArtifactLifecycleEvent {
            artifact_id: id(artifact),
            artifact_revision_id: id(revision),
            lifecycle_sequence: 1,
            detail: Some(Detail::Birth(ArtifactBirth {
                artifact_type: artifact_type as i32,
                representation_sha256: vec![revision; 32],
                provenance: Some(provenance(revision)),
                parents,
            })),
        }
    }

    fn revision(
        artifact: u8,
        current_revision: u8,
        next_revision: u8,
        sequence: u64,
        parents: Vec<ParentReference>,
    ) -> ArtifactLifecycleEvent {
        ArtifactLifecycleEvent {
            artifact_id: id(artifact),
            artifact_revision_id: id(next_revision),
            lifecycle_sequence: sequence,
            detail: Some(Detail::Revision(ArtifactRevision {
                previous_revision_id: id(current_revision),
                representation_sha256: vec![next_revision; 32],
                provenance: Some(provenance(next_revision)),
                parents,
            })),
        }
    }

    fn parent(artifact: u8, revision: u8) -> ParentReference {
        ParentReference {
            artifact_id: id(artifact),
            artifact_revision_id: id(revision),
            relation: LineageRelation::DerivedFrom as i32,
        }
    }

    fn terminal(
        artifact: u8,
        revision: u8,
        disposition: ArtifactDisposition,
    ) -> ArtifactLifecycleEvent {
        ArtifactLifecycleEvent {
            artifact_id: id(artifact),
            artifact_revision_id: id(revision),
            lifecycle_sequence: 2,
            detail: Some(Detail::Disposition(ArtifactDispositionRecord {
                disposition: disposition as i32,
                successor_artifact_id: None,
                provenance: Some(provenance(90)),
            })),
        }
    }

    #[test]
    fn full_lifecycle_records_consumers_cost_utility_and_revision() {
        let events = [
            birth(1, 21, ArtifactType::Feature, vec![]),
            ArtifactLifecycleEvent {
                artifact_id: id(1),
                artifact_revision_id: id(21),
                lifecycle_sequence: 2,
                detail: Some(Detail::ConsumerLink(ConsumerLink {
                    consumer: Some(ConsumerReference {
                        consumer_id: id(60),
                        kind: ConsumerKind::Component as i32,
                        consumer_artifact_revision_id: None,
                    }),
                    action: ConsumerAction::Link as i32,
                    provenance: Some(provenance(61)),
                })),
            },
            ArtifactLifecycleEvent {
                artifact_id: id(1),
                artifact_revision_id: id(21),
                lifecycle_sequence: 3,
                detail: Some(Detail::Cost(ArtifactCost {
                    cost_entry_id: id(62),
                    counter_id: "cpu_time_ns".to_owned(),
                    unit: "ns".to_owned(),
                    amount: Some(10),
                    availability: Availability::Measured as i32,
                    estimator_id: None,
                    estimator_version: None,
                    unavailable_reason: None,
                    provenance: Some(provenance(63)),
                })),
            },
            ArtifactLifecycleEvent {
                artifact_id: id(1),
                artifact_revision_id: id(21),
                lifecycle_sequence: 4,
                detail: Some(Detail::Utility(ArtifactUtility {
                    utility_entry_id: id(64),
                    metric_id: "downstream-control-gain".to_owned(),
                    metric_version: "1.0".to_owned(),
                    unit: "1".to_owned(),
                    estimate: Some(0.25),
                    availability: Availability::Estimated as i32,
                    estimator_id: Some("fixture-estimator".to_owned()),
                    estimator_version: Some("1.0".to_owned()),
                    unavailable_reason: None,
                    provenance: Some(provenance(65)),
                })),
            },
            ArtifactLifecycleEvent {
                artifact_id: id(1),
                artifact_revision_id: id(21),
                lifecycle_sequence: 5,
                detail: Some(Detail::Disposition(ArtifactDispositionRecord {
                    disposition: ArtifactDisposition::Retained as i32,
                    successor_artifact_id: None,
                    provenance: Some(provenance(66)),
                })),
            },
            revision(1, 21, 22, 6, vec![]),
        ];
        assert_eq!(validate_artifact_lineage_trace(&events), Ok(()));
    }

    #[test]
    fn every_registered_artifact_type_has_a_valid_root_lifecycle() {
        let artifact_types = [
            ArtifactType::Feature,
            ArtifactType::Subproblem,
            ArtifactType::Option,
            ArtifactType::Model,
            ArtifactType::Planner,
            ArtifactType::Policy,
            ArtifactType::ValueFunction,
        ];
        for (index, artifact_type) in artifact_types.into_iter().enumerate() {
            let byte = u8::try_from(index + 1).expect("fixture index fits");
            assert_eq!(
                validate_artifact_lineage_trace(&[birth(byte, byte + 20, artifact_type, vec![])]),
                Ok(())
            );
        }
    }

    #[test]
    fn property_orphan_revisions_are_rejected_for_every_wrong_predecessor() {
        for wrong_predecessor in 40..48 {
            let mut candidate = revision(1, 21, 22, 2, vec![]);
            let Some(Detail::Revision(detail)) = candidate.detail.as_mut() else {
                unreachable!();
            };
            detail.previous_revision_id = id(wrong_predecessor);
            assert_eq!(
                validate_artifact_lineage_trace(&[
                    birth(1, 21, ArtifactType::Feature, vec![]),
                    candidate,
                ]),
                Err(LineageValidationError::OrphanRevision)
            );
        }
    }

    #[test]
    fn property_missing_provenance_is_rejected_at_birth_and_revision() {
        for missing_at_revision in [false, true] {
            let mut events = vec![birth(1, 21, ArtifactType::Feature, vec![])];
            if missing_at_revision {
                let mut next = revision(1, 21, 22, 2, vec![]);
                let Some(Detail::Revision(detail)) = next.detail.as_mut() else {
                    unreachable!();
                };
                detail.provenance = None;
                events.push(next);
            } else {
                let Some(Detail::Birth(detail)) = events[0].detail.as_mut() else {
                    unreachable!();
                };
                detail.provenance = None;
            }
            assert_eq!(
                validate_artifact_lineage_trace(&events),
                Err(LineageValidationError::MissingProvenance)
            );
        }
    }

    #[test]
    fn property_parent_cycles_are_rejected_at_every_cycle_length() {
        for cycle_length in 2..=6 {
            let mut events = Vec::new();
            events.push(birth(1, 21, ArtifactType::Feature, vec![]));
            for node in 2..=cycle_length {
                events.push(birth(
                    node,
                    node + 20,
                    ArtifactType::Feature,
                    vec![parent(node - 1, node + 19)],
                ));
            }
            events.push(revision(
                1,
                21,
                31,
                2,
                vec![parent(cycle_length, cycle_length + 20)],
            ));
            assert_eq!(
                validate_artifact_lineage_trace(&events),
                Err(LineageValidationError::LineageCycle)
            );
        }
    }

    #[test]
    fn property_terminal_dispositions_require_a_new_artifact_identity() {
        for disposition in [ArtifactDisposition::Retired, ArtifactDisposition::Removed] {
            let old_birth = birth(1, 21, ArtifactType::Feature, vec![]);
            let retirement = terminal(1, 21, disposition);
            assert_eq!(
                validate_artifact_lineage_trace(&[
                    old_birth.clone(),
                    retirement.clone(),
                    revision(1, 21, 22, 3, vec![]),
                ]),
                Err(LineageValidationError::TerminalResurrection)
            );
            assert_eq!(
                validate_artifact_lineage_trace(&[
                    old_birth,
                    retirement,
                    birth(2, 22, ArtifactType::Feature, vec![parent(1, 21)]),
                ]),
                Ok(())
            );
        }

        let old_birth = birth(1, 21, ArtifactType::Feature, vec![]);
        let successor_birth = birth(2, 22, ArtifactType::Feature, vec![parent(1, 21)]);
        let replaced = ArtifactLifecycleEvent {
            artifact_id: id(1),
            artifact_revision_id: id(21),
            lifecycle_sequence: 2,
            detail: Some(Detail::Disposition(ArtifactDispositionRecord {
                disposition: ArtifactDisposition::Replaced as i32,
                successor_artifact_id: Some(id(2)),
                provenance: Some(provenance(91)),
            })),
        };
        assert_eq!(
            validate_artifact_lineage_trace(&[
                old_birth,
                successor_birth,
                replaced,
                revision(1, 21, 23, 3, vec![]),
            ]),
            Err(LineageValidationError::TerminalResurrection)
        );
    }
}
