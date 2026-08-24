//! Feature metrics over controlled lineage and activation fixtures.

use crate::RationalValue;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

/// One immutable feature snapshot used for exact diagnostic metrics.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureRecord {
    pub feature_id: String,
    pub birth_step: u64,
    pub retirement_step: Option<u64>,
    pub activations: u64,
    pub last_activation_step: u64,
    pub bytes: u64,
    pub work: u64,
    pub consumers: u64,
    pub utility: Option<i64>,
    pub representation: Vec<i64>,
    pub parents: Vec<String>,
    pub obsolete_protected: bool,
}

/// Exact diagnostic class. Labels are not human-semantic utility.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureClass {
    Useful,
    Redundant,
    Dormant,
    ObsoleteProtected,
    Retired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(clippy::struct_excessive_bools)]
#[serde(deny_unknown_fields)]
pub struct FeatureMetricRow {
    pub feature_id: String,
    pub class: FeatureClass,
    pub age: u64,
    pub activations: u64,
    pub consumers: u64,
    pub novelty: bool,
    pub redundancy: u64,
    pub useful_lineage: bool,
    pub utility_per_byte: Option<RationalValue>,
    pub utility_per_work: Option<RationalValue>,
    pub dormant: bool,
    pub obsolete_protected: bool,
    pub detail_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureMetricTable {
    pub schema: String,
    pub now_step: u64,
    pub churn: RationalValue,
    pub rows: Vec<FeatureMetricRow>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FeatureMetricError {
    Identity,
    Trace,
    Arithmetic,
}

impl fmt::Display for FeatureMetricError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "FEATURE_IDENTITY_INVALID",
            Self::Trace => "FEATURE_TRACE_INVALID",
            Self::Arithmetic => "FEATURE_METRIC_ARITHMETIC_FAILED",
        })
    }
}

impl Error for FeatureMetricError {}

/// Derive exact feature metrics from one controlled fixture.
///
/// Human-semantic names are ignored. Utility is the supplied numeric estimate
/// only. Redundancy is representation identity, not a label.
///
/// # Errors
///
/// Rejects empty IDs, inverted lifetimes, and checked-arithmetic failure.
pub fn derive_feature_metrics(
    records: &[FeatureRecord],
    now_step: u64,
) -> Result<FeatureMetricTable, FeatureMetricError> {
    validate(records, now_step)?;
    let mut representation_counts: BTreeMap<&[i64], u64> = BTreeMap::new();
    for record in records {
        *representation_counts
            .entry(record.representation.as_slice())
            .or_insert(0) += 1;
    }
    let mut rows = Vec::new();
    let mut births = 0_u64;
    let mut retirements = 0_u64;
    for record in records {
        if record.birth_step <= now_step {
            births += 1;
        }
        if record.retirement_step.is_some_and(|step| step <= now_step) {
            retirements += 1;
        }
        rows.push(row_for(record, now_step, &representation_counts)?);
    }
    rows.sort_by(|left, right| left.feature_id.cmp(&right.feature_id));
    Ok(FeatureMetricTable {
        schema: "bonsai.feature-metric-table/v1".to_owned(),
        now_step,
        churn: crate::normalize(RationalValue {
            numerator: i64::try_from(births.saturating_add(retirements))
                .map_err(|_| FeatureMetricError::Arithmetic)?,
            denominator: 1,
        }),
        rows,
    })
}

fn row_for(
    record: &FeatureRecord,
    now_step: u64,
    representation_counts: &BTreeMap<&[i64], u64>,
) -> Result<FeatureMetricRow, FeatureMetricError> {
    let redundancy = representation_counts
        .get(record.representation.as_slice())
        .copied()
        .unwrap_or(1)
        .saturating_sub(1);
    let novelty = redundancy == 0;
    let retired = record.retirement_step.is_some();
    let dormant = record.activations == 0 && !retired;
    let useful = record.utility.is_some_and(|value| value > 0) && record.consumers > 0;
    Ok(FeatureMetricRow {
        feature_id: record.feature_id.clone(),
        class: classify(retired, useful, redundancy, record.obsolete_protected),
        age: now_step.saturating_sub(record.birth_step),
        activations: record.activations,
        consumers: record.consumers,
        novelty,
        redundancy,
        useful_lineage: useful && !record.parents.is_empty(),
        utility_per_byte: ratio(record.utility, record.bytes)?,
        utility_per_work: ratio(record.utility, record.work)?,
        dormant,
        obsolete_protected: record.obsolete_protected
            && record.utility.is_some_and(|value| value <= 0),
        detail_code: record
            .utility
            .is_none()
            .then(|| "FEATURE_UTILITY_UNAVAILABLE".to_owned()),
    })
}

#[allow(clippy::fn_params_excessive_bools)]
fn classify(
    retired: bool,
    useful: bool,
    redundancy: u64,
    obsolete_protected: bool,
) -> FeatureClass {
    if retired {
        FeatureClass::Retired
    } else if obsolete_protected && !useful {
        FeatureClass::ObsoleteProtected
    } else if useful {
        FeatureClass::Useful
    } else if redundancy > 0 {
        FeatureClass::Redundant
    } else {
        FeatureClass::Dormant
    }
}

fn validate(records: &[FeatureRecord], now_step: u64) -> Result<(), FeatureMetricError> {
    if records.is_empty() {
        return Err(FeatureMetricError::Trace);
    }
    let mut ids = BTreeSet::new();
    for record in records {
        if record.feature_id.is_empty() || !ids.insert(record.feature_id.as_str()) {
            return Err(FeatureMetricError::Identity);
        }
        if record.birth_step > now_step
            || record
                .retirement_step
                .is_some_and(|step| step < record.birth_step || step > now_step)
        {
            return Err(FeatureMetricError::Trace);
        }
    }
    Ok(())
}

fn ratio(utility: Option<i64>, denom: u64) -> Result<Option<RationalValue>, FeatureMetricError> {
    match utility {
        Some(value) if denom > 0 => Ok(Some(crate::normalize(RationalValue {
            numerator: value,
            denominator: denom,
        }))),
        Some(_) => Err(FeatureMetricError::Arithmetic),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::{FeatureClass, FeatureRecord, derive_feature_metrics};
    use serde_json::{Value, json};
    use std::collections::BTreeMap;

    fn fixture() -> Vec<FeatureRecord> {
        vec![
            FeatureRecord {
                feature_id: "useful".to_owned(),
                birth_step: 0,
                retirement_step: None,
                activations: 3,
                last_activation_step: 0,
                bytes: 2,
                work: 4,
                consumers: 2,
                utility: Some(8),
                representation: vec![1, 0],
                parents: vec!["root".to_owned()],
                obsolete_protected: false,
            },
            FeatureRecord {
                feature_id: "copy-a".to_owned(),
                birth_step: 1,
                retirement_step: None,
                activations: 1,
                last_activation_step: 1,
                bytes: 2,
                work: 4,
                consumers: 0,
                utility: Some(0),
                representation: vec![9],
                parents: Vec::new(),
                obsolete_protected: false,
            },
            FeatureRecord {
                feature_id: "copy-b".to_owned(),
                birth_step: 1,
                retirement_step: None,
                activations: 1,
                last_activation_step: 1,
                bytes: 2,
                work: 4,
                consumers: 0,
                utility: Some(0),
                representation: vec![9],
                parents: Vec::new(),
                obsolete_protected: false,
            },
            FeatureRecord {
                feature_id: "sleep".to_owned(),
                birth_step: 2,
                retirement_step: None,
                activations: 0,
                last_activation_step: 2,
                bytes: 2,
                work: 4,
                consumers: 0,
                utility: Some(0),
                representation: vec![3],
                parents: Vec::new(),
                obsolete_protected: false,
            },
            FeatureRecord {
                feature_id: "old".to_owned(),
                birth_step: 0,
                retirement_step: Some(4),
                activations: 2,
                last_activation_step: 3,
                bytes: 2,
                work: 4,
                consumers: 0,
                utility: Some(-1),
                representation: vec![2],
                parents: Vec::new(),
                obsolete_protected: true,
            },
        ]
    }

    #[test]
    fn useful_redundant_and_dormant_cases_are_exact() {
        let table = derive_feature_metrics(&fixture(), 5).expect("metrics");
        let class = |id: &str| {
            table
                .rows
                .iter()
                .find(|row| row.feature_id == id)
                .expect("row")
                .class
        };
        assert_eq!(class("useful"), FeatureClass::Useful);
        assert_eq!(class("copy-a"), FeatureClass::Redundant);
        assert_eq!(class("copy-b"), FeatureClass::Redundant);
        assert_eq!(class("sleep"), FeatureClass::Dormant);
        assert_eq!(class("old"), FeatureClass::Retired);
        assert!(
            table
                .rows
                .iter()
                .any(|row| row.useful_lineage && row.feature_id == "useful")
        );
        assert_eq!(table.churn.numerator, 6);
    }

    #[test]
    fn committed_feature_classes_match_fixture() {
        let expected: Value = serde_json::from_str(include_str!(
            "../../../fixtures/feature-metrics/v1/expected-outcomes.json"
        ))
        .expect("fixture");
        let table = derive_feature_metrics(&fixture(), 5).expect("metrics");
        let classes = table
            .rows
            .iter()
            .map(|row| {
                (
                    row.feature_id.clone(),
                    serde_json::to_value(row.class).expect("class"),
                )
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            json!({"schema": "bonsai.feature-metric-outcomes/v1", "classes": classes}),
            expected
        );
    }

    #[test]
    fn unavailable_utility_does_not_become_zero() {
        let mut records = fixture();
        records.truncate(1);
        records[0].utility = None;
        let table = derive_feature_metrics(&records, 1).expect("metrics");
        assert!(table.rows[0].utility_per_byte.is_none());
        assert_eq!(
            table.rows[0].detail_code.as_deref(),
            Some("FEATURE_UTILITY_UNAVAILABLE")
        );
    }
}
