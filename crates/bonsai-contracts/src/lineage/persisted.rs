//! Bounded working-set adapter for a durable lineage index.
//!
//! Legality remains in the same contract engine as in-memory admission. The
//! caller supplies indexed historical facts and commits the returned change
//! atomically with its durable event prefix.
use super::incremental::ValidationMode;
use super::{ContractState, LineageValidationError, LineageValidationWork};
use crate::bonsai::artifact::v1::{ArtifactLifecycleEvent, artifact_lifecycle_event::Detail};
use std::collections::HashSet;

pub use super::ArtifactState as PersistedArtifactState;

/// Indexed historical reads. Implementations must bound each artifact's live
/// consumers and perform historical reachability without retaining the prefix.
pub trait LineageStateReader {
    type Error;
    /// # Errors
    /// Returns the backend error if indexed facts cannot be read.
    fn artifact(&self, id: &[u8]) -> Result<Option<PersistedArtifactState>, Self::Error>;
    /// # Errors
    /// Returns the backend error if indexed facts cannot be read.
    fn revision_owner(&self, id: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    /// # Errors
    /// Returns the backend error if indexed facts cannot be read.
    fn history_contains(&self, id: &[u8]) -> Result<bool, Self::Error>;
    /// # Errors
    /// Returns the backend error if indexed facts cannot be read.
    fn reaches(&self, ancestor: &[u8], target: &[u8]) -> Result<bool, Self::Error>;
}

/// A change produced only after the shared contract engine accepts an event.
pub struct ValidatedLineageChange {
    pub artifact_id: Vec<u8>,
    pub state: PersistedArtifactState,
    pub new_revision: Option<Vec<u8>>,
    pub added_parents: HashSet<Vec<u8>>,
    pub history_id: Option<Vec<u8>>,
}

/// Validate against the current indexed state, retaining only this event's
/// referenced identities and one artifact's bounded live consumer set.
///
/// # Errors
///
/// Storage errors are separate from unchanged contract rejection codes.
/// The reader must represent one consistent transaction snapshot.
pub fn validate_persisted_transition<R: LineageStateReader>(
    reader: &R,
    event: &ArtifactLifecycleEvent,
) -> Result<Result<ValidatedLineageChange, LineageValidationError>, R::Error> {
    let mut state = ContractState::default();
    if let Some(artifact) = reader.artifact(&event.artifact_id)? {
        state.artifacts.insert(event.artifact_id.clone(), artifact);
    }
    let mut revisions = vec![event.artifact_revision_id.as_slice()];
    let mut parents = &[][..];
    let mut history = None;
    if let Some(detail) = &event.detail {
        match detail {
            Detail::Birth(value) => parents = value.parents.as_slice(),
            Detail::Revision(value) => parents = value.parents.as_slice(),
            Detail::ConsumerLink(value) => {
                if let Some(id) = value
                    .consumer
                    .as_ref()
                    .and_then(|c| c.consumer_artifact_revision_id.as_deref())
                {
                    revisions.push(id);
                }
            }
            Detail::Cost(value) => history = Some(value.cost_entry_id.clone()),
            Detail::Utility(value) => history = Some(value.utility_entry_id.clone()),
            Detail::Disposition(value) => {
                if let Some(id) = &value.successor_artifact_id
                    && let Some(artifact) = reader.artifact(id)?
                {
                    state.artifacts.insert(id.clone(), artifact);
                }
            }
        }
    }
    revisions.extend(parents.iter().map(|p| p.artifact_revision_id.as_slice()));
    for id in revisions {
        if let Some(owner) = reader.revision_owner(id)? {
            state.revision_owner.insert(id.to_vec(), owner);
        }
    }
    if let Some(id) = &history
        && reader.history_contains(id)?
    {
        state.history_entry_ids.insert(id.clone());
    }
    // Reachability is supplied by the indexed backend, but the contract engine
    // decides whether and where a cycle error takes precedence.
    let mut cycle = false;
    if matches!(&event.detail, Some(Detail::Revision(_))) {
        for parent in parents {
            if reader.reaches(&parent.artifact_id, &event.artifact_id)? {
                cycle = true;
                break;
            }
        }
    }
    if let Err(error) = state.apply(
        event,
        ValidationMode::Persisted(cycle),
        &mut LineageValidationWork::default(),
    ) {
        return Ok(Err(error));
    }
    let Some(artifact) = state.artifacts.remove(&event.artifact_id) else {
        return Ok(Err(LineageValidationError::UnknownArtifact));
    };
    let new_revision = matches!(&event.detail, Some(Detail::Birth(_) | Detail::Revision(_)))
        .then(|| event.artifact_revision_id.clone());
    Ok(Ok(ValidatedLineageChange {
        artifact_id: event.artifact_id.clone(),
        state: artifact,
        new_revision,
        added_parents: state
            .parent_graph
            .remove(&event.artifact_id)
            .unwrap_or_default(),
        history_id: history,
    }))
}
