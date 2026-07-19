//! Runtime-fact-derived evaluation track classification.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Track {
    A,
    B,
    C,
    D,
    #[serde(rename = "INDETERMINATE_TRACK")]
    Indeterminate,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionAccess {
    SinglePass,
    Replay,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateSchedule {
    EventDriven,
    DenseEveryStep,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
// These are independent observed capabilities/data flows; combining them into
// one state enum would hide contradictory facts that must yield indeterminate.
#[allow(clippy::struct_excessive_bools)]
pub struct TrackDeclaration {
    pub schema_version: String,
    pub declared_track: Track,
    pub runtime_facts_complete: bool,
    pub batch_size: u64,
    pub transition_access: TransitionAccess,
    pub replay_capacity_transitions: u64,
    pub offline_updates: bool,
    pub observer_data_access: bool,
    pub privileged_state: bool,
    pub human_labels: bool,
    pub domain_feature_targets: bool,
    pub update_schedule: UpdateSchedule,
    pub fixed_external_budgets: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrackVerdict {
    pub declared: Track,
    pub derived: Track,
    pub declaration_matches: bool,
    pub reason_code: &'static str,
}

/// Derive the mutually exclusive evaluation track from runtime facts.
///
/// Declaration never overrides observed capability or data-flow facts.
#[must_use]
pub fn derive_track(input: &TrackDeclaration) -> TrackVerdict {
    let (derived, reason_code) = if input.schema_version != "1.0"
        || !input.runtime_facts_complete
        || input.batch_size == 0
    {
        (Track::Indeterminate, "TRACK_FACTS_INCOMPLETE")
    } else if input.observer_data_access {
        (Track::Indeterminate, "OBSERVER_DATA_BOUNDARY_VIOLATION")
    } else if input.privileged_state || input.human_labels || input.domain_feature_targets {
        (Track::D, "PRIVILEGED_DIAGNOSTIC_INPUT")
    } else if input.transition_access == TransitionAccess::Replay
        || input.replay_capacity_transitions > 0
        || input.offline_updates
    {
        (Track::B, "REPLAY_OR_OFFLINE_UPDATE")
    } else if input.update_schedule == UpdateSchedule::DenseEveryStep {
        (Track::C, "DENSE_UPDATE_SCHEDULE")
    } else if input.batch_size == 1 && input.fixed_external_budgets {
        (Track::A, "STRICT_EXPERIENTIAL_FACTS")
    } else {
        (Track::Indeterminate, "TRACK_FACTS_CONTRADICTORY")
    };
    TrackVerdict {
        declared: input.declared_track,
        derived,
        declaration_matches: input.declared_track == derived,
        reason_code,
    }
}

#[cfg(test)]
mod tests {
    use super::{Track, TrackDeclaration, TransitionAccess, UpdateSchedule, derive_track};

    fn strict() -> TrackDeclaration {
        TrackDeclaration {
            schema_version: "1.0".to_owned(),
            declared_track: Track::A,
            runtime_facts_complete: true,
            batch_size: 1,
            transition_access: TransitionAccess::SinglePass,
            replay_capacity_transitions: 0,
            offline_updates: false,
            observer_data_access: false,
            privileged_state: false,
            human_labels: false,
            domain_feature_targets: false,
            update_schedule: UpdateSchedule::EventDriven,
            fixed_external_budgets: true,
        }
    }

    #[test]
    fn declaration_cannot_override_hidden_replay() {
        let mut input = strict();
        input.replay_capacity_transitions = 1;
        let verdict = derive_track(&input);
        assert_eq!(verdict.derived, Track::B);
        assert!(!verdict.declaration_matches);
    }

    #[test]
    fn observer_access_is_indeterminate_not_track_a() {
        let mut input = strict();
        input.observer_data_access = true;
        assert_eq!(derive_track(&input).derived, Track::Indeterminate);
    }
}
