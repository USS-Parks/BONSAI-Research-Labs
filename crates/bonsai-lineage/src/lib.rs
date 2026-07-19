//! Deterministic runtime registry for immutable cognitive-artifact lifecycles.

#![forbid(unsafe_code)]

use bonsai_contracts::bonsai::artifact::v1::artifact_lifecycle_event::Detail;
use bonsai_contracts::bonsai::artifact::v1::{
    ArtifactBirth, ArtifactCost, ArtifactDisposition, ArtifactDispositionRecord,
    ArtifactLifecycleEvent, ArtifactRevision, ArtifactType, ArtifactUtility, ConsumerAction,
    ConsumerKind, ConsumerLink, ParentReference, Provenance,
};
use bonsai_contracts::lineage::{LineageValidationError, validate_artifact_lineage_trace};
use std::collections::{BTreeMap, BTreeSet};

pub type ArtifactId = [u8; 16];
pub type ArtifactRevisionId = [u8; 16];
pub type HistoryEntryId = [u8; 16];
pub type RepresentationHash = [u8; 32];

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ConsumerKey {
    pub consumer_id: [u8; 16],
    pub kind: ConsumerKind,
    pub consumer_artifact_revision_id: Option<ArtifactRevisionId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArtifactVersion {
    pub revision_id: ArtifactRevisionId,
    pub previous_revision_id: Option<ArtifactRevisionId>,
    pub representation_sha256: RepresentationHash,
    pub parents: Vec<ParentReference>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArtifactRecord {
    pub artifact_id: ArtifactId,
    pub artifact_type: ArtifactType,
    pub versions: Vec<ArtifactVersion>,
    pub current_revision_id: ArtifactRevisionId,
    pub active_consumers: BTreeSet<ConsumerKey>,
    pub costs: BTreeMap<HistoryEntryId, ArtifactCost>,
    pub utilities: BTreeMap<HistoryEntryId, ArtifactUtility>,
    pub disposition_history: Vec<ArtifactDispositionRecord>,
    pub terminal: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RegistrySnapshot {
    pub artifacts: BTreeMap<ArtifactId, ArtifactRecord>,
    pub revision_owners: BTreeMap<ArtifactRevisionId, ArtifactId>,
}

/// Runtime state derived exclusively from the accepted immutable event prefix.
#[derive(Clone, Debug, Default)]
pub struct ArtifactLifecycleRegistry {
    events: Vec<ArtifactLifecycleEvent>,
    snapshot: RegistrySnapshot,
}

impl ArtifactLifecycleRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reconstruct a registry by applying the supplied lifecycle events in order.
    ///
    /// # Errors
    ///
    /// Returns the first BC-07 lifecycle error without retaining a partial result.
    pub fn reconstruct(events: &[ArtifactLifecycleEvent]) -> Result<Self, LineageValidationError> {
        validate_artifact_lineage_trace(events)?;
        let mut registry = Self::new();
        for event in events {
            registry.apply_validated(event)?;
        }
        registry.events = events.to_vec();
        Ok(registry)
    }

    /// Atomically validate and retain one lifecycle event.
    ///
    /// The BC-07 contract validator remains the sole legality oracle. A rejected
    /// event cannot alter the registry or its accepted event prefix.
    ///
    /// # Errors
    ///
    /// Returns the stable BC-07 error for the first invalid transition.
    pub fn apply(&mut self, event: ArtifactLifecycleEvent) -> Result<(), LineageValidationError> {
        let mut candidate = self.events.clone();
        candidate.push(event.clone());
        validate_artifact_lineage_trace(&candidate)?;
        self.apply_validated(&event)?;
        self.events.push(event);
        Ok(())
    }

    #[must_use]
    pub fn snapshot(&self) -> &RegistrySnapshot {
        &self.snapshot
    }

    #[must_use]
    pub fn events(&self) -> &[ArtifactLifecycleEvent] {
        &self.events
    }

    #[must_use]
    pub fn artifact(&self, artifact_id: &ArtifactId) -> Option<&ArtifactRecord> {
        self.snapshot.artifacts.get(artifact_id)
    }

    #[must_use]
    pub fn revision_owner(&self, revision_id: &ArtifactRevisionId) -> Option<ArtifactId> {
        self.snapshot.revision_owners.get(revision_id).copied()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.snapshot.artifacts.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.snapshot.artifacts.is_empty()
    }

    fn apply_validated(
        &mut self,
        event: &ArtifactLifecycleEvent,
    ) -> Result<(), LineageValidationError> {
        let artifact_id = exact_id(&event.artifact_id)?;
        let revision_id = exact_id(&event.artifact_revision_id)?;
        let detail = event
            .detail
            .as_ref()
            .ok_or(LineageValidationError::Identity)?;
        match detail {
            Detail::Birth(birth) => self.apply_birth(artifact_id, revision_id, birth)?,
            Detail::Revision(revision) => {
                self.apply_revision(artifact_id, revision_id, revision)?;
            }
            Detail::ConsumerLink(link) => self.apply_consumer(artifact_id, link)?,
            Detail::Cost(cost) => self.apply_cost(artifact_id, cost)?,
            Detail::Utility(utility) => self.apply_utility(artifact_id, utility)?,
            Detail::Disposition(disposition) => {
                self.apply_disposition(artifact_id, disposition)?;
            }
        }
        Ok(())
    }

    fn apply_birth(
        &mut self,
        artifact_id: ArtifactId,
        revision_id: ArtifactRevisionId,
        birth: &ArtifactBirth,
    ) -> Result<(), LineageValidationError> {
        let artifact_type = ArtifactType::try_from(birth.artifact_type)
            .map_err(|_| LineageValidationError::ArtifactType)?;
        let version = ArtifactVersion {
            revision_id,
            previous_revision_id: None,
            representation_sha256: exact_hash(&birth.representation_sha256)?,
            parents: birth.parents.clone(),
            provenance: birth
                .provenance
                .clone()
                .ok_or(LineageValidationError::MissingProvenance)?,
        };
        self.snapshot
            .revision_owners
            .insert(revision_id, artifact_id);
        self.snapshot.artifacts.insert(
            artifact_id,
            ArtifactRecord {
                artifact_id,
                artifact_type,
                versions: vec![version],
                current_revision_id: revision_id,
                active_consumers: BTreeSet::new(),
                costs: BTreeMap::new(),
                utilities: BTreeMap::new(),
                disposition_history: Vec::new(),
                terminal: false,
            },
        );
        Ok(())
    }

    fn apply_revision(
        &mut self,
        artifact_id: ArtifactId,
        revision_id: ArtifactRevisionId,
        revision: &ArtifactRevision,
    ) -> Result<(), LineageValidationError> {
        let version = ArtifactVersion {
            revision_id,
            previous_revision_id: Some(exact_id(&revision.previous_revision_id)?),
            representation_sha256: exact_hash(&revision.representation_sha256)?,
            parents: revision.parents.clone(),
            provenance: revision
                .provenance
                .clone()
                .ok_or(LineageValidationError::MissingProvenance)?,
        };
        let record = self
            .snapshot
            .artifacts
            .get_mut(&artifact_id)
            .ok_or(LineageValidationError::UnknownArtifact)?;
        record.current_revision_id = revision_id;
        record.versions.push(version);
        self.snapshot
            .revision_owners
            .insert(revision_id, artifact_id);
        Ok(())
    }

    fn apply_consumer(
        &mut self,
        artifact_id: ArtifactId,
        link: &ConsumerLink,
    ) -> Result<(), LineageValidationError> {
        let consumer = link
            .consumer
            .as_ref()
            .ok_or(LineageValidationError::Consumer)?;
        let key = ConsumerKey {
            consumer_id: exact_id(&consumer.consumer_id)?,
            kind: ConsumerKind::try_from(consumer.kind)
                .map_err(|_| LineageValidationError::Consumer)?,
            consumer_artifact_revision_id: consumer
                .consumer_artifact_revision_id
                .as_deref()
                .map(exact_id)
                .transpose()?,
        };
        let record = self
            .snapshot
            .artifacts
            .get_mut(&artifact_id)
            .ok_or(LineageValidationError::UnknownArtifact)?;
        match ConsumerAction::try_from(link.action).map_err(|_| LineageValidationError::Consumer)? {
            ConsumerAction::Link => {
                record.active_consumers.insert(key);
            }
            ConsumerAction::Unlink => {
                record.active_consumers.remove(&key);
            }
            ConsumerAction::Unspecified => return Err(LineageValidationError::Consumer),
        }
        Ok(())
    }

    fn apply_cost(
        &mut self,
        artifact_id: ArtifactId,
        cost: &ArtifactCost,
    ) -> Result<(), LineageValidationError> {
        let entry_id = exact_id(&cost.cost_entry_id)?;
        self.snapshot
            .artifacts
            .get_mut(&artifact_id)
            .ok_or(LineageValidationError::UnknownArtifact)?
            .costs
            .insert(entry_id, cost.clone());
        Ok(())
    }

    fn apply_utility(
        &mut self,
        artifact_id: ArtifactId,
        utility: &ArtifactUtility,
    ) -> Result<(), LineageValidationError> {
        let entry_id = exact_id(&utility.utility_entry_id)?;
        self.snapshot
            .artifacts
            .get_mut(&artifact_id)
            .ok_or(LineageValidationError::UnknownArtifact)?
            .utilities
            .insert(entry_id, utility.clone());
        Ok(())
    }

    fn apply_disposition(
        &mut self,
        artifact_id: ArtifactId,
        disposition: &ArtifactDispositionRecord,
    ) -> Result<(), LineageValidationError> {
        let terminal = matches!(
            ArtifactDisposition::try_from(disposition.disposition)
                .map_err(|_| LineageValidationError::Disposition)?,
            ArtifactDisposition::Replaced
                | ArtifactDisposition::Retired
                | ArtifactDisposition::Removed
        );
        let record = self
            .snapshot
            .artifacts
            .get_mut(&artifact_id)
            .ok_or(LineageValidationError::UnknownArtifact)?;
        record.disposition_history.push(disposition.clone());
        record.terminal = terminal;
        Ok(())
    }
}

fn exact_id(bytes: &[u8]) -> Result<[u8; 16], LineageValidationError> {
    let id: [u8; 16] = bytes
        .try_into()
        .map_err(|_| LineageValidationError::Identity)?;
    if id.iter().all(|byte| *byte == 0) {
        Err(LineageValidationError::Identity)
    } else {
        Ok(id)
    }
}

fn exact_hash(bytes: &[u8]) -> Result<[u8; 32], LineageValidationError> {
    bytes
        .try_into()
        .map_err(|_| LineageValidationError::Identity)
}

#[cfg(test)]
mod tests {
    use super::ArtifactLifecycleRegistry;
    use bonsai_contracts::bonsai::artifact::v1::artifact_lifecycle_event::Detail;
    use bonsai_contracts::bonsai::artifact::v1::{
        ArtifactBirth, ArtifactCost, ArtifactDisposition, ArtifactDispositionRecord,
        ArtifactLifecycleEvent, ArtifactRevision, ArtifactType, ArtifactUtility, ConsumerAction,
        ConsumerKind, ConsumerLink, ConsumerReference, LineageRelation, ParentReference,
        Provenance,
    };
    use bonsai_contracts::bonsai::event::v1::Availability;
    use bonsai_contracts::lineage::LineageValidationError;

    fn id(byte: u8) -> Vec<u8> {
        vec![byte; 16]
    }

    fn provenance(byte: u8) -> Provenance {
        Provenance {
            producer_id: "registry-model".to_owned(),
            producer_version: "1.0".to_owned(),
            source_event_ids: vec![id(byte)],
            method_ids: vec!["br-07-model/v1".to_owned()],
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

    fn event(artifact: u8, revision: u8, sequence: u64, detail: Detail) -> ArtifactLifecycleEvent {
        ArtifactLifecycleEvent {
            artifact_id: id(artifact),
            artifact_revision_id: id(revision),
            lifecycle_sequence: sequence,
            detail: Some(detail),
        }
    }

    fn parent(artifact: u8, revision: u8) -> ParentReference {
        ParentReference {
            artifact_id: id(artifact),
            artifact_revision_id: id(revision),
            relation: LineageRelation::ConstructedFrom as i32,
        }
    }

    fn lifecycle_trace(terminal: ArtifactDisposition) -> Vec<ArtifactLifecycleEvent> {
        vec![
            birth(1, 21, ArtifactType::Feature, vec![]),
            event(
                1,
                21,
                2,
                Detail::ConsumerLink(ConsumerLink {
                    consumer: Some(ConsumerReference {
                        consumer_id: id(60),
                        kind: ConsumerKind::Component as i32,
                        consumer_artifact_revision_id: None,
                    }),
                    action: ConsumerAction::Link as i32,
                    provenance: Some(provenance(61)),
                }),
            ),
            event(
                1,
                21,
                3,
                Detail::Cost(ArtifactCost {
                    cost_entry_id: id(62),
                    counter_id: "work_items".to_owned(),
                    unit: "1".to_owned(),
                    amount: Some(7),
                    availability: Availability::Measured as i32,
                    estimator_id: None,
                    estimator_version: None,
                    unavailable_reason: None,
                    provenance: Some(provenance(63)),
                }),
            ),
            event(
                1,
                21,
                4,
                Detail::Utility(ArtifactUtility {
                    utility_entry_id: id(64),
                    metric_id: "diagnostic_gain".to_owned(),
                    metric_version: "1.0".to_owned(),
                    unit: "reward".to_owned(),
                    estimate: Some(2.0),
                    availability: Availability::Estimated as i32,
                    estimator_id: Some("exact-ablation".to_owned()),
                    estimator_version: Some("1.0".to_owned()),
                    unavailable_reason: None,
                    provenance: Some(provenance(65)),
                }),
            ),
            event(
                1,
                22,
                5,
                Detail::Revision(ArtifactRevision {
                    previous_revision_id: id(21),
                    representation_sha256: vec![22; 32],
                    provenance: Some(provenance(66)),
                    parents: vec![],
                }),
            ),
            event(
                1,
                22,
                6,
                Detail::ConsumerLink(ConsumerLink {
                    consumer: Some(ConsumerReference {
                        consumer_id: id(60),
                        kind: ConsumerKind::Component as i32,
                        consumer_artifact_revision_id: None,
                    }),
                    action: ConsumerAction::Unlink as i32,
                    provenance: Some(provenance(67)),
                }),
            ),
            event(
                1,
                22,
                7,
                Detail::Disposition(ArtifactDispositionRecord {
                    disposition: terminal as i32,
                    successor_artifact_id: None,
                    provenance: Some(provenance(68)),
                }),
            ),
        ]
    }

    #[test]
    fn model_covers_every_artifact_type_and_lifecycle_transition() {
        let types = [
            ArtifactType::Feature,
            ArtifactType::Subproblem,
            ArtifactType::Option,
            ArtifactType::Model,
            ArtifactType::Planner,
            ArtifactType::Policy,
            ArtifactType::ValueFunction,
        ];
        let mut registry = ArtifactLifecycleRegistry::new();
        for (index, artifact_type) in types.into_iter().enumerate() {
            let artifact = u8::try_from(index + 1).expect("fixture index");
            registry
                .apply(birth(artifact, artifact + 20, artifact_type, vec![]))
                .expect("birth");
        }
        assert_eq!(registry.len(), 7);

        for terminal in [ArtifactDisposition::Retired, ArtifactDisposition::Removed] {
            let registry = ArtifactLifecycleRegistry::reconstruct(&lifecycle_trace(terminal))
                .expect("complete lifecycle");
            let record = registry.artifact(&[1; 16]).expect("artifact");
            assert_eq!(record.versions.len(), 2);
            assert_eq!(record.current_revision_id, [22; 16]);
            assert!(record.active_consumers.is_empty());
            assert_eq!(record.costs.len(), 1);
            assert_eq!(record.utilities.len(), 1);
            assert_eq!(record.disposition_history.len(), 1);
            assert!(record.terminal);
        }
    }

    #[test]
    fn retained_deprioritized_and_replaced_dispositions_are_exact() {
        let mut events = vec![
            birth(1, 21, ArtifactType::Feature, vec![]),
            event(
                1,
                21,
                2,
                Detail::Disposition(ArtifactDispositionRecord {
                    disposition: ArtifactDisposition::Retained as i32,
                    successor_artifact_id: None,
                    provenance: Some(provenance(70)),
                }),
            ),
            event(
                1,
                21,
                3,
                Detail::Disposition(ArtifactDispositionRecord {
                    disposition: ArtifactDisposition::Deprioritized as i32,
                    successor_artifact_id: None,
                    provenance: Some(provenance(71)),
                }),
            ),
            birth(2, 22, ArtifactType::Feature, vec![parent(1, 21)]),
        ];
        events.push(event(
            1,
            21,
            4,
            Detail::Disposition(ArtifactDispositionRecord {
                disposition: ArtifactDisposition::Replaced as i32,
                successor_artifact_id: Some(id(2)),
                provenance: Some(provenance(72)),
            }),
        ));
        let registry = ArtifactLifecycleRegistry::reconstruct(&events).expect("dispositions");
        let record = registry.artifact(&[1; 16]).expect("artifact");
        assert_eq!(record.disposition_history.len(), 3);
        assert!(record.terminal);
    }

    #[test]
    fn invalid_transition_matrix_is_atomic() {
        let invalid = [
            (
                event(
                    2,
                    21,
                    1,
                    Detail::Disposition(ArtifactDispositionRecord {
                        disposition: ArtifactDisposition::Retired as i32,
                        successor_artifact_id: None,
                        provenance: Some(provenance(80)),
                    }),
                ),
                LineageValidationError::UnknownArtifact,
            ),
            (
                birth(1, 22, ArtifactType::Feature, vec![]),
                LineageValidationError::DuplicateArtifact,
            ),
            (
                event(
                    1,
                    22,
                    3,
                    Detail::Revision(ArtifactRevision {
                        previous_revision_id: id(21),
                        representation_sha256: vec![22; 32],
                        provenance: Some(provenance(81)),
                        parents: vec![],
                    }),
                ),
                LineageValidationError::Sequence,
            ),
            (
                event(
                    1,
                    22,
                    2,
                    Detail::Revision(ArtifactRevision {
                        previous_revision_id: id(99),
                        representation_sha256: vec![22; 32],
                        provenance: Some(provenance(82)),
                        parents: vec![],
                    }),
                ),
                LineageValidationError::OrphanRevision,
            ),
        ];
        for (candidate, expected) in invalid {
            let mut registry = ArtifactLifecycleRegistry::reconstruct(&[birth(
                1,
                21,
                ArtifactType::Feature,
                vec![],
            )])
            .expect("base");
            let before = registry.snapshot().clone();
            let events = registry.events().to_vec();
            assert_eq!(registry.apply(candidate), Err(expected));
            assert_eq!(registry.snapshot(), &before);
            assert_eq!(registry.events(), events);
        }
    }

    #[test]
    fn terminal_artifact_cannot_be_resurrected() {
        let mut registry =
            ArtifactLifecycleRegistry::reconstruct(&lifecycle_trace(ArtifactDisposition::Retired))
                .expect("terminal model");
        let candidate = event(
            1,
            23,
            8,
            Detail::Revision(ArtifactRevision {
                previous_revision_id: id(22),
                representation_sha256: vec![23; 32],
                provenance: Some(provenance(83)),
                parents: vec![],
            }),
        );
        assert_eq!(
            registry.apply(candidate),
            Err(LineageValidationError::TerminalResurrection)
        );
    }

    #[test]
    fn reconstruction_is_deterministic_and_revision_ownership_is_stable() {
        let events = lifecycle_trace(ArtifactDisposition::Retired);
        let direct = ArtifactLifecycleRegistry::reconstruct(&events).expect("direct");
        let mut incremental = ArtifactLifecycleRegistry::new();
        for event in events.clone() {
            incremental.apply(event).expect("incremental");
        }
        assert_eq!(direct.snapshot(), incremental.snapshot());
        assert_eq!(direct.events(), incremental.events());
        assert_eq!(direct.revision_owner(&[21; 16]), Some([1; 16]));
        assert_eq!(direct.revision_owner(&[22; 16]), Some([1; 16]));
    }
}
