//! Subproblem metrics that keep reward-respecting and reward-oblivious cases distinct.

use crate::RationalValue;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

/// Whether a subproblem is charged to the original reward.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RewardMode {
    Respecting,
    Oblivious,
}

/// One diagnostic subproblem with full lineage and cost fields.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(clippy::struct_excessive_bools)]
#[serde(deny_unknown_fields)]
pub struct SubproblemRecord {
    pub subproblem_id: String,
    pub feature_id: String,
    pub reward_mode: RewardMode,
    pub original_reward: i64,
    pub attained_feature: bool,
    pub attained_intensity: i64,
    pub stopping_bonus: i64,
    pub stopping_value: i64,
    pub initiated: bool,
    pub terminated: bool,
    pub learning_progress: i64,
    pub success: bool,
    pub cost: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(clippy::struct_excessive_bools)]
#[serde(deny_unknown_fields)]
pub struct SubproblemMetricRow {
    pub subproblem_id: String,
    pub feature_id: String,
    pub reward_mode: RewardMode,
    pub reward_aligned: bool,
    pub attained_feature: bool,
    pub attained_intensity: i64,
    pub original_reward: i64,
    pub stopping_bonus: i64,
    pub stopping_value: i64,
    pub initiated: bool,
    pub terminated: bool,
    pub learning_progress: i64,
    pub success: bool,
    pub cost: RationalValue,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubproblemMetricTable {
    pub schema: String,
    pub rows: Vec<SubproblemMetricRow>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubproblemMetricError {
    Identity,
    Trace,
}

impl fmt::Display for SubproblemMetricError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "SUBPROBLEM_IDENTITY_INVALID",
            Self::Trace => "SUBPROBLEM_TRACE_INVALID",
        })
    }
}

impl Error for SubproblemMetricError {}

/// Derive exact subproblem metrics and reward-mode alignment.
///
/// A reward-respecting subproblem is aligned only when success matches a
/// positive original reward. A reward-oblivious subproblem may succeed when
/// the original reward is non-positive and is never treated as reward-respecting.
///
/// # Errors
///
/// Rejects empty identities, missing feature lineage, or unterminated success.
pub fn derive_subproblem_metrics(
    records: &[SubproblemRecord],
) -> Result<SubproblemMetricTable, SubproblemMetricError> {
    validate(records)?;
    let mut rows = records
        .iter()
        .map(|record| {
            Ok(SubproblemMetricRow {
                subproblem_id: record.subproblem_id.clone(),
                feature_id: record.feature_id.clone(),
                reward_mode: record.reward_mode,
                reward_aligned: aligned(record),
                attained_feature: record.attained_feature,
                attained_intensity: record.attained_intensity,
                original_reward: record.original_reward,
                stopping_bonus: record.stopping_bonus,
                stopping_value: record.stopping_value,
                initiated: record.initiated,
                terminated: record.terminated,
                learning_progress: record.learning_progress,
                success: record.success,
                cost: crate::normalize(RationalValue {
                    numerator: i64::try_from(record.cost)
                        .map_err(|_| SubproblemMetricError::Trace)?,
                    denominator: 1,
                }),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    rows.sort_by(|left, right| left.subproblem_id.cmp(&right.subproblem_id));
    Ok(SubproblemMetricTable {
        schema: "bonsai.subproblem-metric-table/v1".to_owned(),
        rows,
    })
}

fn aligned(record: &SubproblemRecord) -> bool {
    match record.reward_mode {
        RewardMode::Respecting => record.success == (record.original_reward > 0),
        RewardMode::Oblivious => record.success && record.original_reward <= 0,
    }
}

fn validate(records: &[SubproblemRecord]) -> Result<(), SubproblemMetricError> {
    if records.is_empty() {
        return Err(SubproblemMetricError::Trace);
    }
    let mut ids = BTreeSet::new();
    for record in records {
        if record.subproblem_id.is_empty()
            || record.feature_id.is_empty()
            || !ids.insert(record.subproblem_id.as_str())
        {
            return Err(SubproblemMetricError::Identity);
        }
        if record.success && !record.terminated {
            return Err(SubproblemMetricError::Trace);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{RewardMode, SubproblemRecord, derive_subproblem_metrics};

    fn record(id: &str, mode: RewardMode, original_reward: i64, success: bool) -> SubproblemRecord {
        SubproblemRecord {
            subproblem_id: id.to_owned(),
            feature_id: format!("feature-{id}"),
            reward_mode: mode,
            original_reward,
            attained_feature: success,
            attained_intensity: i64::from(success),
            stopping_bonus: 1,
            stopping_value: 2,
            initiated: true,
            terminated: true,
            learning_progress: 3,
            success,
            cost: 4,
        }
    }

    #[test]
    fn reward_respecting_and_oblivious_cases_are_distinguishable() {
        let table = derive_subproblem_metrics(&[
            record("respect-pass", RewardMode::Respecting, 5, true),
            record("respect-fail", RewardMode::Respecting, 0, true),
            record("oblivious", RewardMode::Oblivious, 0, true),
        ])
        .expect("metrics");
        let row = |id: &str| {
            table
                .rows
                .iter()
                .find(|row| row.subproblem_id == id)
                .expect("row")
        };
        assert_eq!(row("respect-pass").reward_mode, RewardMode::Respecting);
        assert!(row("respect-pass").reward_aligned);
        assert_eq!(row("respect-fail").reward_mode, RewardMode::Respecting);
        assert!(!row("respect-fail").reward_aligned);
        assert_eq!(row("oblivious").reward_mode, RewardMode::Oblivious);
        assert!(row("oblivious").reward_aligned);
        assert_ne!(
            row("respect-pass").reward_mode,
            row("oblivious").reward_mode
        );
        assert_eq!(row("respect-pass").feature_id, "feature-respect-pass");
    }

    #[test]
    fn committed_alignment_fixture_is_exact() {
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/subproblem-metrics/v1/expected-outcomes.json"
        ))
        .expect("fixture");
        let table = derive_subproblem_metrics(&[
            record("respect-pass", RewardMode::Respecting, 5, true),
            record("respect-fail", RewardMode::Respecting, 0, true),
            record("oblivious", RewardMode::Oblivious, 0, true),
        ])
        .expect("metrics");
        let observed = serde_json::json!({
            "schema": "bonsai.subproblem-metric-outcomes/v1",
            "alignment": table.rows.iter().map(|row| {
                (row.subproblem_id.clone(), row.reward_aligned)
            }).collect::<std::collections::BTreeMap<_, _>>()
        });
        assert_eq!(observed, expected);
    }
}
