//! Reconstruct admissions and violations from immutable policy (BQ-08).

use crate::decision::{DecisionEvidence, DecisionInput, decide};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayReport {
    pub schema: String,
    pub matched: bool,
    pub divergences: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayError {
    Empty,
    Decision,
}

impl fmt::Display for ReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "GOVERNOR_REPLAY_EMPTY",
            Self::Decision => "GOVERNOR_REPLAY_DECISION_INVALID",
        })
    }
}

impl Error for ReplayError {}

/// Replay each recorded decision from the original input. Agent work is not run.
///
/// # Errors
///
/// Rejects an empty transcript or a decision that cannot be reconstructed.
pub fn replay_decisions(
    inputs: &[DecisionInput],
    recorded: &[DecisionEvidence],
) -> Result<ReplayReport, ReplayError> {
    if inputs.is_empty() || inputs.len() != recorded.len() {
        return Err(ReplayError::Empty);
    }
    let mut divergences = Vec::new();
    for (input, expected) in inputs.iter().zip(recorded) {
        let replayed = decide(clone_input(input)).map_err(|_| ReplayError::Decision)?;
        if replayed.canonical_sha256 != expected.canonical_sha256 {
            divergences.push(input.decision_id.clone());
        }
    }
    Ok(ReplayReport {
        schema: "bonsai.governor-replay/v1".to_owned(),
        matched: divergences.is_empty(),
        divergences,
    })
}

fn clone_input(input: &DecisionInput) -> DecisionInput {
    DecisionInput {
        decision_id: input.decision_id.clone(),
        monotonic_time_ns: input.monotonic_time_ns,
        policy: input.policy.clone(),
        work_class: input.work_class,
        request: input.request.clone(),
        projections: input.projections.clone(),
        next_rolling_release_ns: input.next_rolling_release_ns,
    }
}
