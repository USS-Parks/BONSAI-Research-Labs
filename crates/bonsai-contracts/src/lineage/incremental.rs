use super::{ContractState, LineageValidationError};
use crate::bonsai::artifact::v1::ArtifactLifecycleEvent;
use serde::Serialize;

/// Work performed by validation, excluding event payload cloning and storage.
/// Map/set probes count requested keys; ancestry counts actual visited relations.
/// Replay counters remain zero on incremental admission.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct LineageValidationWork {
    pub events: u64,
    pub artifact_lookups: u64,
    pub revision_lookups: u64,
    pub parent_references: u64,
    pub consumer_lookups: u64,
    pub history_lookups: u64,
    pub ancestry_nodes: u64,
    pub ancestry_edges: u64,
    pub whole_graph_scans: u64,
    pub graph_clones: u64,
}

#[derive(Clone, Copy)]
pub(super) enum ValidationMode {
    Replay,
    Incremental,
    Persisted(bool),
}

/// Persistent state extracted from the contract validator.
///
/// Only accepted events change semantic state. The full replay entry point
/// retains its whole-graph cycle check as an independent traversal oracle.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IncrementalLineageValidator {
    state: ContractState,
}

impl IncrementalLineageValidator {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate and commit one event without replaying the accepted prefix.
    ///
    /// # Errors
    /// Returns the same first contract error as full replay. Rejection is atomic.
    pub fn apply(&mut self, event: &ArtifactLifecycleEvent) -> Result<(), LineageValidationError> {
        self.apply_measured(event).0
    }

    /// Admit one event and return its observed validation work, including errors.
    /// Measurements are returned separately and never mutate semantic state.
    pub fn apply_measured(
        &mut self,
        event: &ArtifactLifecycleEvent,
    ) -> (Result<(), LineageValidationError>, LineageValidationWork) {
        let mut work = LineageValidationWork::default();
        let result = self
            .state
            .apply(event, ValidationMode::Incremental, &mut work);
        (result, work)
    }
}
