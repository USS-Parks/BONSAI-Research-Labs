//! Model and knowledge metrics with lineage-aligned error comparison.

use crate::RationalValue;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

/// Exact diagnostic class for a controlled model fixture.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelClass {
    Stale,
    Biased,
    Calibrated,
    RepresentationShift,
}

/// One diagnostic model bound to a representation and prediction target.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelRecord {
    pub model_id: String,
    pub representation_id: String,
    pub target_id: String,
    pub parent_model_id: Option<String>,
    pub one_step_error: i64,
    pub option_horizon_error: i64,
    pub reward_error: i64,
    pub stopping_error: i64,
    pub jump_length_error: i64,
    pub drift: i64,
    pub recovery: i64,
    pub uncertainty: i64,
    pub harmful_planning: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelMetricRow {
    pub model_id: String,
    pub class: ModelClass,
    pub one_step_error: i64,
    pub option_horizon_error: i64,
    pub reward_error: i64,
    pub stopping_error: i64,
    pub jump_length_error: i64,
    pub drift: i64,
    pub recovery: i64,
    pub uncertainty: i64,
    pub harmful_planning: bool,
    pub semantic_stable: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelComparison {
    pub left_model_id: String,
    pub right_model_id: String,
    pub one_step_delta: Option<RationalValue>,
    pub detail_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelMetricTable {
    pub schema: String,
    pub rows: Vec<ModelMetricRow>,
    pub comparisons: Vec<ModelComparison>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelMetricError {
    Identity,
    Trace,
}

impl fmt::Display for ModelMetricError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "MODEL_IDENTITY_INVALID",
            Self::Trace => "MODEL_TRACE_INVALID",
        })
    }
}

impl Error for ModelMetricError {}

/// Derive model metrics and refuse unaligned cross-target error comparison.
///
/// Errors may be subtracted only when both models name the same `target_id`.
/// A representation change with the same target is a representation-shift
/// class, not an implicit numeric comparison against a different target.
///
/// # Errors
///
/// Rejects empty identities or unknown parent references.
pub fn derive_model_metrics(records: &[ModelRecord]) -> Result<ModelMetricTable, ModelMetricError> {
    let indexed = index(records)?;
    let mut rows = Vec::new();
    for record in records {
        let parent = record
            .parent_model_id
            .as_ref()
            .map(|parent_id| indexed[parent_id.as_str()]);
        rows.push(ModelMetricRow {
            model_id: record.model_id.clone(),
            class: classify(record, parent),
            one_step_error: record.one_step_error,
            option_horizon_error: record.option_horizon_error,
            reward_error: record.reward_error,
            stopping_error: record.stopping_error,
            jump_length_error: record.jump_length_error,
            drift: record.drift,
            recovery: record.recovery,
            uncertainty: record.uncertainty,
            harmful_planning: record.harmful_planning,
            semantic_stable: parent.is_none_or(|parent| {
                parent.representation_id == record.representation_id
                    && parent.target_id == record.target_id
            }),
        });
    }
    rows.sort_by(|left, right| left.model_id.cmp(&right.model_id));
    Ok(ModelMetricTable {
        schema: "bonsai.model-metric-table/v1".to_owned(),
        rows,
        comparisons: compare_adjacent(records),
    })
}

fn index(records: &[ModelRecord]) -> Result<BTreeMap<&str, &ModelRecord>, ModelMetricError> {
    if records.is_empty() {
        return Err(ModelMetricError::Trace);
    }
    let mut indexed = BTreeMap::new();
    let mut ids = BTreeSet::new();
    for record in records {
        if record.model_id.is_empty()
            || record.representation_id.is_empty()
            || record.target_id.is_empty()
            || !ids.insert(record.model_id.as_str())
        {
            return Err(ModelMetricError::Identity);
        }
        indexed.insert(record.model_id.as_str(), record);
    }
    for record in records {
        if let Some(parent) = &record.parent_model_id
            && !indexed.contains_key(parent.as_str())
        {
            return Err(ModelMetricError::Trace);
        }
    }
    Ok(indexed)
}

fn classify(record: &ModelRecord, parent: Option<&ModelRecord>) -> ModelClass {
    if parent.is_some_and(|parent| {
        parent.representation_id != record.representation_id && parent.target_id == record.target_id
    }) {
        ModelClass::RepresentationShift
    } else if record.drift > 0 && record.recovery == 0 {
        ModelClass::Stale
    } else if record.reward_error != 0 && record.uncertainty == 0 {
        ModelClass::Biased
    } else {
        ModelClass::Calibrated
    }
}

fn compare_adjacent(records: &[ModelRecord]) -> Vec<ModelComparison> {
    let mut comparisons = Vec::new();
    for window in records.windows(2) {
        let left = &window[0];
        let right = &window[1];
        let aligned = left.target_id == right.target_id;
        comparisons.push(ModelComparison {
            left_model_id: left.model_id.clone(),
            right_model_id: right.model_id.clone(),
            one_step_delta: aligned.then_some(crate::normalize(RationalValue {
                numerator: right.one_step_error.saturating_sub(left.one_step_error),
                denominator: 1,
            })),
            detail_code: (!aligned).then(|| "MODEL_TARGET_UNALIGNED".to_owned()),
        });
    }
    comparisons
}

#[cfg(test)]
mod tests {
    use super::{ModelClass, ModelRecord, derive_model_metrics};

    #[allow(clippy::too_many_arguments)]
    fn record(
        id: &str,
        representation: &str,
        target: &str,
        parent: Option<&str>,
        one_step: i64,
        reward: i64,
        drift: i64,
        recovery: i64,
        uncertainty: i64,
    ) -> ModelRecord {
        ModelRecord {
            model_id: id.to_owned(),
            representation_id: representation.to_owned(),
            target_id: target.to_owned(),
            parent_model_id: parent.map(ToOwned::to_owned),
            one_step_error: one_step,
            option_horizon_error: 0,
            reward_error: reward,
            stopping_error: 0,
            jump_length_error: 0,
            drift,
            recovery,
            uncertainty,
            harmful_planning: false,
        }
    }

    #[test]
    fn stale_biased_calibrated_and_shift_are_distinct() {
        let table = derive_model_metrics(&[
            record("cal", "r0", "t0", None, 0, 0, 0, 0, 1),
            record("stale", "r0", "t0", Some("cal"), 4, 0, 5, 0, 1),
            record("bias", "r0", "t0", Some("cal"), 0, 3, 0, 0, 0),
            record("shift", "r1", "t0", Some("cal"), 1, 0, 0, 0, 1),
        ])
        .expect("metrics");
        let class = |id: &str| {
            table
                .rows
                .iter()
                .find(|row| row.model_id == id)
                .expect("row")
                .class
        };
        assert_eq!(class("cal"), ModelClass::Calibrated);
        assert_eq!(class("stale"), ModelClass::Stale);
        assert_eq!(class("bias"), ModelClass::Biased);
        assert_eq!(class("shift"), ModelClass::RepresentationShift);
    }

    #[test]
    fn unaligned_targets_do_not_produce_numeric_error_deltas() {
        let table = derive_model_metrics(&[
            record("a", "r0", "t0", None, 1, 0, 0, 0, 1),
            record("b", "r0", "t1", None, 9, 0, 0, 0, 1),
        ])
        .expect("metrics");
        assert!(table.comparisons[0].one_step_delta.is_none());
        assert_eq!(
            table.comparisons[0].detail_code.as_deref(),
            Some("MODEL_TARGET_UNALIGNED")
        );
    }

    #[test]
    fn committed_model_classes_match_fixture() {
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/model-metrics/v1/expected-outcomes.json"
        ))
        .expect("fixture");
        let table = derive_model_metrics(&[
            record("cal", "r0", "t0", None, 0, 0, 0, 0, 1),
            record("stale", "r0", "t0", Some("cal"), 4, 0, 5, 0, 1),
            record("bias", "r0", "t0", Some("cal"), 0, 3, 0, 0, 0),
            record("shift", "r1", "t0", Some("cal"), 1, 0, 0, 0, 1),
        ])
        .expect("metrics");
        let classes = table
            .rows
            .iter()
            .map(|row| {
                (
                    row.model_id.clone(),
                    serde_json::to_value(row.class).expect("class"),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            serde_json::json!({"schema":"bonsai.model-metric-outcomes/v1","classes":classes}),
            expected
        );
    }
}
