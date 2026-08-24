//! Option metrics that classify reliable, redundant, harmful, and unused options.

use crate::RationalValue;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

/// Exact diagnostic class. Option creation is not a benefit.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionClass {
    Reliable,
    Redundant,
    Harmful,
    Unused,
}

/// One diagnostic option with success, cost, and planning fields.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OptionRecord {
    pub option_id: String,
    pub successes: u64,
    pub attempts: u64,
    pub duration: u64,
    pub return_sum: i64,
    pub stopping_values: Vec<i64>,
    pub controllability: i64,
    pub planning_participation: u64,
    pub marginal_gain: i64,
    pub acquisition_cost: u64,
    pub maintenance_cost: u64,
    pub used: bool,
    pub representation: Vec<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OptionMetricRow {
    pub option_id: String,
    pub class: OptionClass,
    pub success: Option<RationalValue>,
    pub duration: u64,
    pub return_total: i64,
    pub stopping_value: Option<RationalValue>,
    pub controllability: i64,
    pub reliability: Option<RationalValue>,
    pub redundancy: u64,
    pub planning_participation: u64,
    pub marginal_gain: i64,
    pub acquisition_cost: u64,
    pub maintenance_cost: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OptionMetricTable {
    pub schema: String,
    pub rows: Vec<OptionMetricRow>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OptionMetricError {
    Identity,
    Trace,
    Arithmetic,
}

impl fmt::Display for OptionMetricError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "OPTION_IDENTITY_INVALID",
            Self::Trace => "OPTION_TRACE_INVALID",
            Self::Arithmetic => "OPTION_METRIC_ARITHMETIC_FAILED",
        })
    }
}

impl Error for OptionMetricError {}

/// Derive option metrics and exact diagnostic classes.
///
/// Creation is not scored as utility. Harmful options have negative marginal
/// gain. Unused options have no executions. Redundancy is representation
/// identity among used non-harmful options.
///
/// # Errors
///
/// Rejects empty IDs, attempts below successes, and arithmetic overflow.
pub fn derive_option_metrics(
    records: &[OptionRecord],
) -> Result<OptionMetricTable, OptionMetricError> {
    validate(records)?;
    let mut representation_counts: BTreeMap<&[i64], u64> = BTreeMap::new();
    for record in records {
        if record.used && record.marginal_gain >= 0 {
            *representation_counts
                .entry(record.representation.as_slice())
                .or_insert(0) += 1;
        }
    }
    let mut rows = Vec::new();
    for record in records {
        let redundancy = representation_counts
            .get(record.representation.as_slice())
            .copied()
            .unwrap_or(0)
            .saturating_sub(1);
        rows.push(OptionMetricRow {
            option_id: record.option_id.clone(),
            class: classify(record, redundancy),
            success: ratio(record.successes, record.attempts)?,
            duration: record.duration,
            return_total: record.return_sum,
            stopping_value: mean(&record.stopping_values)?,
            controllability: record.controllability,
            reliability: ratio(record.successes, record.attempts)?,
            redundancy,
            planning_participation: record.planning_participation,
            marginal_gain: record.marginal_gain,
            acquisition_cost: record.acquisition_cost,
            maintenance_cost: record.maintenance_cost,
        });
    }
    rows.sort_by(|left, right| left.option_id.cmp(&right.option_id));
    Ok(OptionMetricTable {
        schema: "bonsai.option-metric-table/v1".to_owned(),
        rows,
    })
}

fn classify(record: &OptionRecord, redundancy: u64) -> OptionClass {
    if !record.used || record.attempts == 0 {
        OptionClass::Unused
    } else if record.marginal_gain < 0 {
        OptionClass::Harmful
    } else if redundancy > 0 {
        OptionClass::Redundant
    } else {
        OptionClass::Reliable
    }
}

fn validate(records: &[OptionRecord]) -> Result<(), OptionMetricError> {
    if records.is_empty() {
        return Err(OptionMetricError::Trace);
    }
    let mut ids = BTreeSet::new();
    for record in records {
        if record.option_id.is_empty() || !ids.insert(record.option_id.as_str()) {
            return Err(OptionMetricError::Identity);
        }
        if record.successes > record.attempts {
            return Err(OptionMetricError::Trace);
        }
    }
    Ok(())
}

fn ratio(numerator: u64, denominator: u64) -> Result<Option<RationalValue>, OptionMetricError> {
    if denominator == 0 {
        return Ok(None);
    }
    Ok(Some(crate::normalize(RationalValue {
        numerator: i64::try_from(numerator).map_err(|_| OptionMetricError::Arithmetic)?,
        denominator,
    })))
}

fn mean(values: &[i64]) -> Result<Option<RationalValue>, OptionMetricError> {
    if values.is_empty() {
        return Ok(None);
    }
    let total = values.iter().try_fold(0_i64, |sum, value| {
        sum.checked_add(*value).ok_or(OptionMetricError::Arithmetic)
    })?;
    Ok(Some(crate::normalize(RationalValue {
        numerator: total,
        denominator: u64::try_from(values.len()).map_err(|_| OptionMetricError::Arithmetic)?,
    })))
}

#[cfg(test)]
mod tests {
    use super::{OptionClass, OptionRecord, derive_option_metrics};

    fn record(
        id: &str,
        successes: u64,
        attempts: u64,
        gain: i64,
        used: bool,
        representation: Vec<i64>,
    ) -> OptionRecord {
        OptionRecord {
            option_id: id.to_owned(),
            successes,
            attempts,
            duration: 2,
            return_sum: gain,
            stopping_values: vec![1, 1],
            controllability: 1,
            planning_participation: u64::from(used),
            marginal_gain: gain,
            acquisition_cost: 1,
            maintenance_cost: 1,
            used,
            representation,
        }
    }

    #[test]
    fn reliable_redundant_harmful_and_unused_are_distinct() {
        let table = derive_option_metrics(&[
            record("good", 4, 4, 3, true, vec![1]),
            record("twin-a", 3, 3, 1, true, vec![7]),
            record("twin-b", 3, 3, 1, true, vec![7]),
            record("hurt", 2, 2, -4, true, vec![2]),
            record("idle", 0, 0, 0, false, vec![3]),
        ])
        .expect("metrics");
        let class = |id: &str| {
            table
                .rows
                .iter()
                .find(|row| row.option_id == id)
                .expect("row")
                .class
        };
        assert_eq!(class("good"), OptionClass::Reliable);
        assert_eq!(class("twin-a"), OptionClass::Redundant);
        assert_eq!(class("twin-b"), OptionClass::Redundant);
        assert_eq!(class("hurt"), OptionClass::Harmful);
        assert_eq!(class("idle"), OptionClass::Unused);
    }

    #[test]
    fn committed_option_classes_match_fixture() {
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/option-metrics/v1/expected-outcomes.json"
        ))
        .expect("fixture");
        let table = derive_option_metrics(&[
            record("good", 4, 4, 3, true, vec![1]),
            record("twin-a", 3, 3, 1, true, vec![7]),
            record("twin-b", 3, 3, 1, true, vec![7]),
            record("hurt", 2, 2, -4, true, vec![2]),
            record("idle", 0, 0, 0, false, vec![3]),
        ])
        .expect("metrics");
        let classes = table
            .rows
            .iter()
            .map(|row| {
                (
                    row.option_id.clone(),
                    serde_json::to_value(row.class).expect("class"),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            serde_json::json!({"schema":"bonsai.option-metric-outcomes/v1","classes":classes}),
            expected
        );
    }
}
