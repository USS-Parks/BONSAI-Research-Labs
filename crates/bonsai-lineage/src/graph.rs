use crate::{
    ArtifactId, ArtifactRevisionId, ArtifactVersion, ConsumerKey, HistoryEntryId, RegistrySnapshot,
};
use bonsai_contracts::bonsai::artifact::v1::{LineageRelation, ParentReference};
use bonsai_contracts::bonsai::event::v1::Availability;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineageGraphError {
    UnknownArtifact,
    ArtifactIdentity,
    RevisionOwner,
    DuplicateRevision,
    RepresentationChangeWithoutRevision,
    RevisionChain,
    ParentReference,
    DanglingEdge,
    Cycle,
    CostObservation,
    Arithmetic,
}

impl LineageGraphError {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnknownArtifact => "LINEAGE_ARTIFACT_UNKNOWN",
            Self::ArtifactIdentity => "LINEAGE_ARTIFACT_IDENTITY_INVALID",
            Self::RevisionOwner => "LINEAGE_REVISION_OWNER_INVALID",
            Self::DuplicateRevision => "LINEAGE_REVISION_ID_DUPLICATE",
            Self::RepresentationChangeWithoutRevision => {
                "LINEAGE_REPRESENTATION_CHANGE_WITHOUT_REVISION"
            }
            Self::RevisionChain => "LINEAGE_REVISION_CHAIN_INVALID",
            Self::ParentReference => "LINEAGE_PARENT_REFERENCE_INVALID",
            Self::DanglingEdge => "LINEAGE_DANGLING_EDGE",
            Self::Cycle => "LINEAGE_CYCLE",
            Self::CostObservation => "LINEAGE_COST_OBSERVATION_INVALID",
            Self::Arithmetic => "LINEAGE_COST_ARITHMETIC_FAILED",
        }
    }
}

impl fmt::Display for LineageGraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl Error for LineageGraphError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CostScope {
    ArtifactOnly,
    DescendantsInclusive,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CostRollupKey {
    pub counter_id: String,
    pub unit: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CostRollup {
    pub measured_amount: Option<u64>,
    pub estimated_amount: Option<u64>,
    pub unavailable_entries: u64,
    pub excluded_entries: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UtilitySource {
    pub entry_id: HistoryEntryId,
    pub metric_id: String,
    pub metric_version: String,
    pub estimate: Option<f64>,
    pub availability: Availability,
    pub estimator_id: Option<String>,
    pub estimator_version: Option<String>,
    pub source_event_ids: Vec<[u8; 16]>,
    pub method_ids: Vec<String>,
}

/// Validated, read-only graph projection over one registry snapshot.
#[derive(Clone, Debug)]
pub struct LineageGraph {
    snapshot: RegistrySnapshot,
    parents: BTreeMap<ArtifactId, BTreeSet<ArtifactId>>,
    children: BTreeMap<ArtifactId, BTreeSet<ArtifactId>>,
}

impl LineageGraph {
    /// Build and validate a graph without changing the supplied registry view.
    ///
    /// # Errors
    ///
    /// Rejects inconsistent identities, revision chains/owners, duplicate
    /// revisions, silent representation changes, dangling edges, and cycles.
    pub fn build(snapshot: &RegistrySnapshot) -> Result<Self, LineageGraphError> {
        let mut revision_owners = BTreeMap::new();
        let mut revision_hashes = BTreeMap::new();
        let mut parent_references = Vec::new();
        for (artifact_id, record) in &snapshot.artifacts {
            validate_artifact(
                *artifact_id,
                record,
                &mut revision_owners,
                &mut revision_hashes,
                &mut parent_references,
            )?;
        }
        if revision_owners != snapshot.revision_owners {
            return Err(LineageGraphError::RevisionOwner);
        }

        let mut parents = snapshot
            .artifacts
            .keys()
            .map(|artifact_id| (*artifact_id, BTreeSet::new()))
            .collect::<BTreeMap<_, _>>();
        let mut children = parents.clone();
        for (child, reference) in parent_references {
            let parent_artifact = exact_id(&reference.artifact_id)?;
            let parent_revision = exact_id(&reference.artifact_revision_id)?;
            if revision_owners.get(&parent_revision) != Some(&parent_artifact) {
                return Err(LineageGraphError::DanglingEdge);
            }
            parents.entry(child).or_default().insert(parent_artifact);
            children.entry(parent_artifact).or_default().insert(child);
        }
        if graph_has_cycle(&parents) {
            return Err(LineageGraphError::Cycle);
        }
        Ok(Self {
            snapshot: snapshot.clone(),
            parents,
            children,
        })
    }

    /// Return every transitive provenance ancestor in stable artifact-ID order.
    ///
    /// # Errors
    ///
    /// Returns `LINEAGE_ARTIFACT_UNKNOWN` when the artifact is absent.
    pub fn ancestry(&self, artifact_id: &ArtifactId) -> Result<Vec<ArtifactId>, LineageGraphError> {
        self.traverse(artifact_id, &self.parents)
    }

    /// Return every transitive provenance descendant in stable artifact-ID order.
    ///
    /// # Errors
    ///
    /// Returns `LINEAGE_ARTIFACT_UNKNOWN` when the artifact is absent.
    pub fn descendants(
        &self,
        artifact_id: &ArtifactId,
    ) -> Result<Vec<ArtifactId>, LineageGraphError> {
        self.traverse(artifact_id, &self.children)
    }

    /// Return the immutable revision chain in lifecycle order.
    ///
    /// # Errors
    ///
    /// Returns `LINEAGE_ARTIFACT_UNKNOWN` when the artifact is absent.
    pub fn revisions(
        &self,
        artifact_id: &ArtifactId,
    ) -> Result<&[ArtifactVersion], LineageGraphError> {
        self.snapshot
            .artifacts
            .get(artifact_id)
            .map(|record| record.versions.as_slice())
            .ok_or(LineageGraphError::UnknownArtifact)
    }

    #[must_use]
    pub fn revision_owner(&self, revision_id: &ArtifactRevisionId) -> Option<ArtifactId> {
        self.snapshot.revision_owners.get(revision_id).copied()
    }

    /// Return the exact active-consumer view in stable key order.
    ///
    /// # Errors
    ///
    /// Returns `LINEAGE_ARTIFACT_UNKNOWN` when the artifact is absent.
    pub fn active_consumers(
        &self,
        artifact_id: &ArtifactId,
    ) -> Result<Vec<ConsumerKey>, LineageGraphError> {
        self.snapshot
            .artifacts
            .get(artifact_id)
            .map(|record| record.active_consumers.iter().cloned().collect())
            .ok_or(LineageGraphError::UnknownArtifact)
    }

    /// Return exact utility-entry provenance without inferring causality.
    ///
    /// # Errors
    ///
    /// Rejects an absent artifact or malformed retained availability/provenance.
    pub fn utility_sources(
        &self,
        artifact_id: &ArtifactId,
    ) -> Result<Vec<UtilitySource>, LineageGraphError> {
        let record = self
            .snapshot
            .artifacts
            .get(artifact_id)
            .ok_or(LineageGraphError::UnknownArtifact)?;
        record
            .utilities
            .iter()
            .map(|(entry_id, utility)| {
                let availability = Availability::try_from(utility.availability)
                    .map_err(|_| LineageGraphError::CostObservation)?;
                let provenance = utility
                    .provenance
                    .as_ref()
                    .ok_or(LineageGraphError::ParentReference)?;
                let source_event_ids = provenance
                    .source_event_ids
                    .iter()
                    .map(|event_id| exact_id(event_id))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(UtilitySource {
                    entry_id: *entry_id,
                    metric_id: utility.metric_id.clone(),
                    metric_version: utility.metric_version.clone(),
                    estimate: utility.estimate,
                    availability,
                    estimator_id: utility.estimator_id.clone(),
                    estimator_version: utility.estimator_version.clone(),
                    source_event_ids,
                    method_ids: provenance.method_ids.clone(),
                })
            })
            .collect()
    }

    /// Roll up cost observations for one artifact or its descendant subgraph.
    ///
    /// Measured and estimated totals remain separate. Unavailable/excluded
    /// observations are counted and never represented as zero-valued amounts.
    ///
    /// # Errors
    ///
    /// Rejects unknown artifacts, malformed observations, and checked overflow.
    pub fn cost_rollup(
        &self,
        artifact_id: &ArtifactId,
        scope: CostScope,
    ) -> Result<BTreeMap<CostRollupKey, CostRollup>, LineageGraphError> {
        if !self.snapshot.artifacts.contains_key(artifact_id) {
            return Err(LineageGraphError::UnknownArtifact);
        }
        let mut artifacts = BTreeSet::from([*artifact_id]);
        if scope == CostScope::DescendantsInclusive {
            artifacts.extend(self.descendants(artifact_id)?);
        }
        let mut rollups = BTreeMap::new();
        for current in artifacts {
            let record = self
                .snapshot
                .artifacts
                .get(&current)
                .ok_or(LineageGraphError::UnknownArtifact)?;
            for cost in record.costs.values() {
                let key = CostRollupKey {
                    counter_id: cost.counter_id.clone(),
                    unit: cost.unit.clone(),
                };
                accumulate_cost(rollups.entry(key).or_default(), cost)?;
            }
        }
        Ok(rollups)
    }

    fn traverse(
        &self,
        artifact_id: &ArtifactId,
        edges: &BTreeMap<ArtifactId, BTreeSet<ArtifactId>>,
    ) -> Result<Vec<ArtifactId>, LineageGraphError> {
        if !self.snapshot.artifacts.contains_key(artifact_id) {
            return Err(LineageGraphError::UnknownArtifact);
        }
        let mut queue = edges
            .get(artifact_id)
            .into_iter()
            .flatten()
            .copied()
            .collect::<VecDeque<_>>();
        let mut found = BTreeSet::new();
        while let Some(current) = queue.pop_front() {
            if found.insert(current) {
                queue.extend(edges.get(&current).into_iter().flatten().copied());
            }
        }
        Ok(found.into_iter().collect())
    }
}

fn validate_artifact(
    artifact_id: ArtifactId,
    record: &crate::ArtifactRecord,
    revision_owners: &mut BTreeMap<ArtifactRevisionId, ArtifactId>,
    revision_hashes: &mut BTreeMap<ArtifactRevisionId, [u8; 32]>,
    parent_references: &mut Vec<(ArtifactId, ParentReference)>,
) -> Result<(), LineageGraphError> {
    if artifact_id != record.artifact_id || record.versions.is_empty() {
        return Err(LineageGraphError::ArtifactIdentity);
    }
    for (index, version) in record.versions.iter().enumerate() {
        if let Some(existing_hash) = revision_hashes.get(&version.revision_id) {
            return if existing_hash == &version.representation_sha256 {
                Err(LineageGraphError::DuplicateRevision)
            } else {
                Err(LineageGraphError::RepresentationChangeWithoutRevision)
            };
        }
        let expected_previous = index
            .checked_sub(1)
            .map(|previous| record.versions[previous].revision_id);
        if version.previous_revision_id != expected_previous {
            return Err(LineageGraphError::RevisionChain);
        }
        revision_hashes.insert(version.revision_id, version.representation_sha256);
        revision_owners.insert(version.revision_id, artifact_id);
        for parent in &version.parents {
            let relation = LineageRelation::try_from(parent.relation)
                .map_err(|_| LineageGraphError::ParentReference)?;
            if relation == LineageRelation::Unspecified {
                return Err(LineageGraphError::ParentReference);
            }
            parent_references.push((artifact_id, parent.clone()));
        }
    }
    if record.current_revision_id
        != record
            .versions
            .last()
            .ok_or(LineageGraphError::RevisionChain)?
            .revision_id
    {
        return Err(LineageGraphError::RevisionChain);
    }
    Ok(())
}

fn accumulate_cost(
    rollup: &mut CostRollup,
    cost: &bonsai_contracts::bonsai::artifact::v1::ArtifactCost,
) -> Result<(), LineageGraphError> {
    let availability = Availability::try_from(cost.availability)
        .map_err(|_| LineageGraphError::CostObservation)?;
    match availability {
        Availability::Measured => add_amount(&mut rollup.measured_amount, cost.amount)?,
        Availability::Estimated => add_amount(&mut rollup.estimated_amount, cost.amount)?,
        Availability::Unavailable => {
            if cost.amount.is_some() {
                return Err(LineageGraphError::CostObservation);
            }
            rollup.unavailable_entries = rollup
                .unavailable_entries
                .checked_add(1)
                .ok_or(LineageGraphError::Arithmetic)?;
        }
        Availability::Excluded => {
            if cost.amount.is_some() {
                return Err(LineageGraphError::CostObservation);
            }
            rollup.excluded_entries = rollup
                .excluded_entries
                .checked_add(1)
                .ok_or(LineageGraphError::Arithmetic)?;
        }
        Availability::Unspecified => return Err(LineageGraphError::CostObservation),
    }
    Ok(())
}

fn add_amount(target: &mut Option<u64>, amount: Option<u64>) -> Result<(), LineageGraphError> {
    let amount = amount.ok_or(LineageGraphError::CostObservation)?;
    *target = Some(
        target
            .unwrap_or_default()
            .checked_add(amount)
            .ok_or(LineageGraphError::Arithmetic)?,
    );
    Ok(())
}

fn exact_id(bytes: &[u8]) -> Result<[u8; 16], LineageGraphError> {
    let id: [u8; 16] = bytes
        .try_into()
        .map_err(|_| LineageGraphError::ParentReference)?;
    if id.iter().all(|byte| *byte == 0) {
        Err(LineageGraphError::ParentReference)
    } else {
        Ok(id)
    }
}

fn graph_has_cycle(parents: &BTreeMap<ArtifactId, BTreeSet<ArtifactId>>) -> bool {
    fn visit(
        node: ArtifactId,
        parents: &BTreeMap<ArtifactId, BTreeSet<ArtifactId>>,
        visiting: &mut BTreeSet<ArtifactId>,
        visited: &mut BTreeSet<ArtifactId>,
    ) -> bool {
        if visited.contains(&node) {
            return false;
        }
        if !visiting.insert(node) {
            return true;
        }
        if parents.get(&node).is_some_and(|direct| {
            direct
                .iter()
                .any(|parent| visit(*parent, parents, visiting, visited))
        }) {
            return true;
        }
        visiting.remove(&node);
        visited.insert(node);
        false
    }

    let mut visited = BTreeSet::new();
    parents
        .keys()
        .any(|node| visit(*node, parents, &mut BTreeSet::new(), &mut visited))
}

#[cfg(test)]
mod tests {
    use super::{CostRollupKey, CostScope, LineageGraph, LineageGraphError};
    use crate::ArtifactLifecycleRegistry;
    use bonsai_contracts::bonsai::artifact::v1::artifact_lifecycle_event::Detail;
    use bonsai_contracts::bonsai::artifact::v1::{
        ArtifactBirth, ArtifactCost, ArtifactLifecycleEvent, ArtifactRevision, ArtifactType,
        ArtifactUtility, ConsumerAction, ConsumerKind, ConsumerLink, ConsumerReference,
        LineageRelation, ParentReference, Provenance,
    };
    use bonsai_contracts::bonsai::event::v1::Availability;

    fn id(byte: u8) -> Vec<u8> {
        vec![byte; 16]
    }

    fn provenance(byte: u8) -> Provenance {
        Provenance {
            producer_id: "br-08-fixture".to_owned(),
            producer_version: "1.0".to_owned(),
            source_event_ids: vec![id(byte)],
            method_ids: vec![format!("source-{byte}")],
        }
    }

    fn parent(artifact: u8, revision: u8) -> ParentReference {
        ParentReference {
            artifact_id: id(artifact),
            artifact_revision_id: id(revision),
            relation: LineageRelation::ConstructedFrom as i32,
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

    fn known_graph() -> ArtifactLifecycleRegistry {
        let events = vec![
            birth(1, 21, ArtifactType::Feature, vec![]),
            event(
                1,
                21,
                2,
                Detail::ConsumerLink(ConsumerLink {
                    consumer: Some(ConsumerReference {
                        consumer_id: id(90),
                        kind: ConsumerKind::Component as i32,
                        consumer_artifact_revision_id: None,
                    }),
                    action: ConsumerAction::Link as i32,
                    provenance: Some(provenance(91)),
                }),
            ),
            event(
                1,
                21,
                3,
                Detail::Cost(ArtifactCost {
                    cost_entry_id: id(61),
                    counter_id: "work_items".to_owned(),
                    unit: "1".to_owned(),
                    amount: Some(2),
                    availability: Availability::Measured as i32,
                    estimator_id: None,
                    estimator_version: None,
                    unavailable_reason: None,
                    provenance: Some(provenance(61)),
                }),
            ),
            event(
                1,
                22,
                4,
                Detail::Revision(ArtifactRevision {
                    previous_revision_id: id(21),
                    representation_sha256: vec![22; 32],
                    provenance: Some(provenance(22)),
                    parents: vec![],
                }),
            ),
            birth(2, 23, ArtifactType::Subproblem, vec![parent(1, 22)]),
            event(
                2,
                23,
                2,
                Detail::Cost(ArtifactCost {
                    cost_entry_id: id(62),
                    counter_id: "work_items".to_owned(),
                    unit: "1".to_owned(),
                    amount: Some(3),
                    availability: Availability::Estimated as i32,
                    estimator_id: Some("fixture-estimator".to_owned()),
                    estimator_version: Some("1.0".to_owned()),
                    unavailable_reason: None,
                    provenance: Some(provenance(62)),
                }),
            ),
            birth(3, 24, ArtifactType::Option, vec![parent(2, 23)]),
            event(
                3,
                24,
                2,
                Detail::Utility(ArtifactUtility {
                    utility_entry_id: id(63),
                    metric_id: "exact-marginal-gain".to_owned(),
                    metric_version: "1.0".to_owned(),
                    unit: "reward".to_owned(),
                    estimate: Some(4.0),
                    availability: Availability::Measured as i32,
                    estimator_id: None,
                    estimator_version: None,
                    unavailable_reason: None,
                    provenance: Some(provenance(63)),
                }),
            ),
            birth(4, 25, ArtifactType::Model, vec![parent(3, 24)]),
            event(
                4,
                25,
                2,
                Detail::Cost(ArtifactCost {
                    cost_entry_id: id(64),
                    counter_id: "energy_nj".to_owned(),
                    unit: "nJ".to_owned(),
                    amount: None,
                    availability: Availability::Unavailable as i32,
                    estimator_id: None,
                    estimator_version: None,
                    unavailable_reason: Some("COUNTER_UNAVAILABLE".to_owned()),
                    provenance: Some(provenance(64)),
                }),
            ),
        ];
        ArtifactLifecycleRegistry::reconstruct(&events).expect("known graph")
    }

    #[test]
    fn known_graph_returns_exact_queries_without_mutating_registry() {
        let registry = known_graph();
        let before = registry.snapshot().clone();
        let graph = LineageGraph::build(registry.snapshot()).expect("graph");
        assert_eq!(
            graph.ancestry(&[4; 16]).expect("ancestry"),
            vec![[1; 16], [2; 16], [3; 16]]
        );
        assert_eq!(
            graph.descendants(&[1; 16]).expect("descendants"),
            vec![[2; 16], [3; 16], [4; 16]]
        );
        assert_eq!(
            graph.active_consumers(&[1; 16]).expect("consumers").len(),
            1
        );
        assert_eq!(graph.revisions(&[1; 16]).expect("revisions").len(), 2);
        assert_eq!(graph.revision_owner(&[22; 16]), Some([1; 16]));
        let utility = graph.utility_sources(&[3; 16]).expect("utility");
        assert_eq!(utility.len(), 1);
        assert_eq!(utility[0].source_event_ids, vec![[63; 16]]);
        assert_eq!(registry.snapshot(), &before);
    }

    #[test]
    fn cost_rollup_keeps_measured_estimated_and_unavailable_distinct() {
        let registry = known_graph();
        let graph = LineageGraph::build(registry.snapshot()).expect("graph");
        let costs = graph
            .cost_rollup(&[1; 16], CostScope::DescendantsInclusive)
            .expect("costs");
        let work = costs
            .get(&CostRollupKey {
                counter_id: "work_items".to_owned(),
                unit: "1".to_owned(),
            })
            .expect("work");
        assert_eq!(work.measured_amount, Some(2));
        assert_eq!(work.estimated_amount, Some(3));
        let energy = costs
            .get(&CostRollupKey {
                counter_id: "energy_nj".to_owned(),
                unit: "nJ".to_owned(),
            })
            .expect("energy");
        assert_eq!(energy.measured_amount, None);
        assert_eq!(energy.unavailable_entries, 1);
    }

    #[test]
    fn cycle_and_dangling_edge_are_explicit_failures() {
        let registry = known_graph();
        let mut dangling = registry.snapshot().clone();
        dangling
            .artifacts
            .get_mut(&[1; 16])
            .expect("feature")
            .versions[0]
            .parents
            .push(parent(99, 99));
        assert_eq!(
            LineageGraph::build(&dangling).expect_err("dangling"),
            LineageGraphError::DanglingEdge
        );

        let mut cyclic = registry.snapshot().clone();
        cyclic
            .artifacts
            .get_mut(&[1; 16])
            .expect("feature")
            .versions[0]
            .parents
            .push(parent(4, 25));
        assert_eq!(
            LineageGraph::build(&cyclic).expect_err("cycle"),
            LineageGraphError::Cycle
        );
    }

    #[test]
    fn duplicate_revision_and_silent_representation_change_are_distinct() {
        let registry = known_graph();
        let mut duplicate = registry.snapshot().clone();
        let version = duplicate.artifacts[&[1; 16]].versions[0].clone();
        duplicate
            .artifacts
            .get_mut(&[1; 16])
            .expect("feature")
            .versions
            .push(version.clone());
        assert_eq!(
            LineageGraph::build(&duplicate).expect_err("duplicate"),
            LineageGraphError::DuplicateRevision
        );

        let mut changed = registry.snapshot().clone();
        let mut changed_version = version;
        changed_version.representation_sha256 = [99; 32];
        changed
            .artifacts
            .get_mut(&[1; 16])
            .expect("feature")
            .versions
            .push(changed_version);
        assert_eq!(
            LineageGraph::build(&changed).expect_err("changed"),
            LineageGraphError::RepresentationChangeWithoutRevision
        );
    }

    #[test]
    fn revision_chain_and_owner_index_must_match_exactly() {
        let registry = known_graph();
        let mut wrong_chain = registry.snapshot().clone();
        wrong_chain
            .artifacts
            .get_mut(&[1; 16])
            .expect("feature")
            .versions[1]
            .previous_revision_id = Some([99; 16]);
        assert_eq!(
            LineageGraph::build(&wrong_chain).expect_err("chain"),
            LineageGraphError::RevisionChain
        );

        let mut wrong_owner = registry.snapshot().clone();
        wrong_owner.revision_owners.insert([22; 16], [2; 16]);
        assert_eq!(
            LineageGraph::build(&wrong_owner).expect_err("owner"),
            LineageGraphError::RevisionOwner
        );
    }

    #[test]
    fn unknown_artifact_queries_fail_instead_of_returning_empty() {
        let registry = known_graph();
        let graph = LineageGraph::build(registry.snapshot()).expect("graph");
        assert_eq!(
            graph.ancestry(&[99; 16]),
            Err(LineageGraphError::UnknownArtifact)
        );
        assert_eq!(
            graph.cost_rollup(&[99; 16], CostScope::ArtifactOnly),
            Err(LineageGraphError::UnknownArtifact)
        );
    }
}
