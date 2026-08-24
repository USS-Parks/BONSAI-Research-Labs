//! Feature metrics over controlled lineage and activation traces (BK-05).

use crate::{MetricKey, RationalValue};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

/// Lifecycle / consumer event for one feature identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureEvent {
    pub step: u64,
    pub feature_id: String,
    pub kind: FeatureEventKind,
    pub representation_id: String,
    pub bytes: u64,
    pub work_cost: u64,
    pub consumer_delta: i64,
    pub marginal_utility: i64,
}

/// Ordered feature lifecycle verbs.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureEventKind {
    Birth,
    Activate,
    Deactivate,
    Retire,
    ConsumerUpdate,
    UtilityUpdate,
}

/// Machine classification for one feature identity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureClass {
    Useful,
    Redundant,
    Dormant,
    ObsoleteProtected,
    Unclassified,
}

/// One versioned feature metric outcome.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureMetric {
    pub key: MetricKey,
    pub unit: String,
    pub window: String,
    pub value: Option<RationalValue>,
    pub detail_code: Option<String>,
}

/// Per-feature classification retained with exact counters.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureClassification {
    pub feature_id: String,
    pub class: FeatureClass,
    pub age: u64,
    pub activations: u64,
    pub consumers: u64,
    pub marginal_utility: i64,
    pub bytes: u64,
    pub work_cost: u64,
    pub representation_id: String,
}

/// Deterministic feature metric table.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureMetricTable {
    pub schema: String,
    pub metrics: Vec<FeatureMetric>,
    pub classifications: Vec<FeatureClassification>,
}

/// Failures for malformed feature traces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FeatureMetricError {
    Trace,
    Arithmetic,
    Lifecycle,
}

impl fmt::Display for FeatureMetricError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Trace => "FEATURE_TRACE_INVALID",
            Self::Arithmetic => "FEATURE_METRIC_ARITHMETIC_FAILED",
            Self::Lifecycle => "FEATURE_LIFECYCLE_INVALID",
        })
    }
}

impl Error for FeatureMetricError {}

#[derive(Clone, Debug, Default)]
struct FeatureState {
    born_at: Option<u64>,
    retired_at: Option<u64>,
    representation_id: String,
    bytes: u64,
    work_cost: u64,
    activations: u64,
    last_activation: Option<u64>,
    consumers: i64,
    marginal_utility: i64,
}

/// Derive BK-05 feature metrics from one ordered lineage/activation trace.
///
/// # Errors
///
/// Rejects empty/non-monotonic traces, duplicate births, activate/retire before birth,
/// and arithmetic overflow. Human-semantic labels are never consulted.
#[allow(clippy::too_many_lines)]
pub fn derive_feature_metrics(
    events: &[FeatureEvent],
) -> Result<FeatureMetricTable, FeatureMetricError> {
    if events.is_empty() {
        return Err(FeatureMetricError::Trace);
    }
    let mut previous_step = None;
    let mut states: BTreeMap<String, FeatureState> = BTreeMap::new();
    let mut birth_count = 0_u64;
    let mut retirement_count = 0_u64;
    let mut activation_count = 0_u64;

    for event in events {
        if event.feature_id.is_empty() || event.representation_id.is_empty() {
            return Err(FeatureMetricError::Trace);
        }
        if let Some(previous) = previous_step
            && event.step < previous
        {
            return Err(FeatureMetricError::Trace);
        }
        previous_step = Some(event.step);
        match event.kind {
            FeatureEventKind::Birth => {
                if states.contains_key(&event.feature_id) {
                    return Err(FeatureMetricError::Lifecycle);
                }
                states.insert(
                    event.feature_id.clone(),
                    FeatureState {
                        born_at: Some(event.step),
                        representation_id: event.representation_id.clone(),
                        bytes: event.bytes,
                        work_cost: event.work_cost,
                        ..FeatureState::default()
                    },
                );
                birth_count = birth_count
                    .checked_add(1)
                    .ok_or(FeatureMetricError::Arithmetic)?;
            }
            FeatureEventKind::Activate => {
                let state = states
                    .get_mut(&event.feature_id)
                    .ok_or(FeatureMetricError::Lifecycle)?;
                if state.born_at.is_none() || state.retired_at.is_some() {
                    return Err(FeatureMetricError::Lifecycle);
                }
                state.activations = state
                    .activations
                    .checked_add(1)
                    .ok_or(FeatureMetricError::Arithmetic)?;
                state.last_activation = Some(event.step);
                activation_count = activation_count
                    .checked_add(1)
                    .ok_or(FeatureMetricError::Arithmetic)?;
            }
            FeatureEventKind::Deactivate => {
                let state = states
                    .get_mut(&event.feature_id)
                    .ok_or(FeatureMetricError::Lifecycle)?;
                if state.born_at.is_none() || state.retired_at.is_some() {
                    return Err(FeatureMetricError::Lifecycle);
                }
            }
            FeatureEventKind::Retire => {
                let state = states
                    .get_mut(&event.feature_id)
                    .ok_or(FeatureMetricError::Lifecycle)?;
                if state.born_at.is_none() || state.retired_at.is_some() {
                    return Err(FeatureMetricError::Lifecycle);
                }
                state.retired_at = Some(event.step);
                retirement_count = retirement_count
                    .checked_add(1)
                    .ok_or(FeatureMetricError::Arithmetic)?;
            }
            FeatureEventKind::ConsumerUpdate | FeatureEventKind::UtilityUpdate => {
                let state = states
                    .get_mut(&event.feature_id)
                    .ok_or(FeatureMetricError::Lifecycle)?;
                if state.born_at.is_none() {
                    return Err(FeatureMetricError::Lifecycle);
                }
                state.consumers = state
                    .consumers
                    .checked_add(event.consumer_delta)
                    .ok_or(FeatureMetricError::Arithmetic)?;
                if state.consumers < 0 {
                    return Err(FeatureMetricError::Lifecycle);
                }
                state.marginal_utility = state
                    .marginal_utility
                    .checked_add(event.marginal_utility)
                    .ok_or(FeatureMetricError::Arithmetic)?;
                if event.bytes > 0 {
                    state.bytes = event.bytes;
                }
                if event.work_cost > 0 {
                    state.work_cost = event.work_cost;
                }
            }
        }
    }

    let end_step = events.last().map_or(0, |event| event.step);
    let mut classifications = Vec::new();
    let mut representation_owners: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut total_consumers = 0_u64;
    let mut total_marginal = 0_i64;
    let mut total_bytes = 0_u64;
    let mut total_work = 0_u64;
    let mut age_sum = 0_u64;
    let mut useful_lineage = 0_u64;
    let mut dormant = 0_u64;
    let mut obsolete_protected = 0_u64;

    for (feature_id, state) in &states {
        let born_at = state.born_at.ok_or(FeatureMetricError::Lifecycle)?;
        let age = end_step
            .checked_sub(born_at)
            .ok_or(FeatureMetricError::Arithmetic)?;
        age_sum = age_sum
            .checked_add(age)
            .ok_or(FeatureMetricError::Arithmetic)?;
        total_consumers = total_consumers
            .checked_add(
                u64::try_from(state.consumers).map_err(|_| FeatureMetricError::Arithmetic)?,
            )
            .ok_or(FeatureMetricError::Arithmetic)?;
        total_marginal = total_marginal
            .checked_add(state.marginal_utility)
            .ok_or(FeatureMetricError::Arithmetic)?;
        total_bytes = total_bytes
            .checked_add(state.bytes)
            .ok_or(FeatureMetricError::Arithmetic)?;
        total_work = total_work
            .checked_add(state.work_cost)
            .ok_or(FeatureMetricError::Arithmetic)?;
        representation_owners
            .entry(state.representation_id.clone())
            .or_default()
            .push(feature_id.clone());
    }

    let mut redundant_ids = BTreeSet::new();
    for owners in representation_owners.values() {
        if owners.len() > 1 {
            for id in owners {
                redundant_ids.insert(id.clone());
            }
        }
    }

    for (feature_id, state) in &states {
        let Some(born_at) = state.born_at else {
            return Err(FeatureMetricError::Lifecycle);
        };
        let age = end_step.saturating_sub(born_at);
        let consumers = u64::try_from(state.consumers).unwrap_or(0);
        let dormant_gap = state
            .last_activation
            .map_or(age, |last| end_step.saturating_sub(last));
        let class = if state.retired_at.is_some() && consumers > 0 {
            obsolete_protected = obsolete_protected.saturating_add(1);
            FeatureClass::ObsoleteProtected
        } else if redundant_ids.contains(feature_id) && state.marginal_utility <= 0 {
            FeatureClass::Redundant
        } else if state.activations == 0 || dormant_gap >= 3 && consumers == 0 {
            dormant = dormant.saturating_add(1);
            FeatureClass::Dormant
        } else if state.marginal_utility > 0 && consumers > 0 {
            useful_lineage = useful_lineage.saturating_add(1);
            FeatureClass::Useful
        } else {
            FeatureClass::Unclassified
        };
        classifications.push(FeatureClassification {
            feature_id: feature_id.clone(),
            class,
            age,
            activations: state.activations,
            consumers,
            marginal_utility: state.marginal_utility,
            bytes: state.bytes,
            work_cost: state.work_cost,
            representation_id: state.representation_id.clone(),
        });
    }
    classifications.sort_by(|left, right| left.feature_id.cmp(&right.feature_id));

    let feature_count = u64::try_from(states.len()).map_err(|_| FeatureMetricError::Arithmetic)?;
    let unique_representations =
        u64::try_from(representation_owners.len()).map_err(|_| FeatureMetricError::Arithmetic)?;
    let redundant_count =
        u64::try_from(redundant_ids.len()).map_err(|_| FeatureMetricError::Arithmetic)?;
    let churn_events = birth_count
        .checked_add(retirement_count)
        .ok_or(FeatureMetricError::Arithmetic)?;

    let metrics = vec![
        metric(
            "feature_birth_count",
            "count",
            Some(rational(birth_count, 1)?),
            None,
        ),
        metric(
            "feature_mean_age",
            "steps",
            if feature_count == 0 {
                None
            } else {
                Some(rational(age_sum, feature_count)?)
            },
            if feature_count == 0 {
                Some("FEATURE_MEAN_AGE_UNAVAILABLE")
            } else {
                None
            },
        ),
        metric(
            "feature_activation_count",
            "count",
            Some(rational(activation_count, 1)?),
            None,
        ),
        metric(
            "feature_retirement_count",
            "count",
            Some(rational(retirement_count, 1)?),
            None,
        ),
        metric(
            "feature_novelty",
            "ratio",
            if birth_count == 0 {
                None
            } else {
                Some(rational(unique_representations, birth_count)?)
            },
            if birth_count == 0 {
                Some("FEATURE_NOVELTY_UNAVAILABLE")
            } else {
                None
            },
        ),
        metric(
            "feature_redundancy",
            "count",
            Some(rational(redundant_count, 1)?),
            None,
        ),
        metric(
            "feature_consumers",
            "count",
            Some(rational(total_consumers, 1)?),
            None,
        ),
        metric(
            "feature_marginal_contribution",
            "utility",
            Some(RationalValue {
                numerator: total_marginal,
                denominator: 1,
            }),
            None,
        ),
        metric(
            "feature_useful_lineage",
            "count",
            Some(rational(useful_lineage, 1)?),
            None,
        ),
        metric(
            "feature_utility_per_byte",
            "utility_per_byte",
            if total_bytes == 0 {
                None
            } else {
                Some(RationalValue {
                    numerator: total_marginal,
                    denominator: total_bytes,
                })
            },
            if total_bytes == 0 {
                Some("FEATURE_UTILITY_PER_BYTE_UNAVAILABLE")
            } else {
                None
            },
        ),
        metric(
            "feature_utility_per_work",
            "utility_per_work",
            if total_work == 0 {
                None
            } else {
                Some(RationalValue {
                    numerator: total_marginal,
                    denominator: total_work,
                })
            },
            if total_work == 0 {
                Some("FEATURE_UTILITY_PER_WORK_UNAVAILABLE")
            } else {
                None
            },
        ),
        metric(
            "feature_churn",
            "count",
            Some(rational(churn_events, 1)?),
            None,
        ),
        metric(
            "feature_dormancy",
            "count",
            Some(rational(dormant, 1)?),
            None,
        ),
        metric(
            "feature_obsolete_protection",
            "count",
            Some(rational(obsolete_protected, 1)?),
            None,
        ),
    ];

    Ok(FeatureMetricTable {
        schema: "bonsai.feature-metric-table/v1".to_owned(),
        metrics,
        classifications,
    })
}

fn metric(
    id: &str,
    unit: &str,
    value: Option<RationalValue>,
    detail_code: Option<&str>,
) -> FeatureMetric {
    FeatureMetric {
        key: MetricKey {
            id: id.to_owned(),
            version: "1.0".to_owned(),
        },
        unit: unit.to_owned(),
        window: "lifetime".to_owned(),
        value,
        detail_code: detail_code.map(str::to_owned),
    }
}

fn rational(numerator: u64, denominator: u64) -> Result<RationalValue, FeatureMetricError> {
    if denominator == 0 {
        return Err(FeatureMetricError::Arithmetic);
    }
    Ok(RationalValue {
        numerator: i64::try_from(numerator).map_err(|_| FeatureMetricError::Arithmetic)?,
        denominator,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        FeatureClass, FeatureEvent, FeatureEventKind, FeatureMetricError, derive_feature_metrics,
    };

    #[allow(clippy::too_many_arguments)]
    fn event(
        step: u64,
        feature_id: &str,
        kind: FeatureEventKind,
        representation_id: &str,
        bytes: u64,
        work_cost: u64,
        consumer_delta: i64,
        marginal_utility: i64,
    ) -> FeatureEvent {
        FeatureEvent {
            step,
            feature_id: feature_id.to_owned(),
            kind,
            representation_id: representation_id.to_owned(),
            bytes,
            work_cost,
            consumer_delta,
            marginal_utility,
        }
    }

    #[test]
    fn useful_redundant_and_dormant_are_distinct() {
        let events = [
            event(0, "f1", FeatureEventKind::Birth, "rep-a", 10, 5, 0, 0),
            event(1, "f2", FeatureEventKind::Birth, "rep-a", 10, 5, 0, 0),
            event(2, "f3", FeatureEventKind::Birth, "rep-b", 8, 4, 0, 0),
            event(3, "f1", FeatureEventKind::Activate, "rep-a", 0, 0, 0, 0),
            event(
                4,
                "f1",
                FeatureEventKind::ConsumerUpdate,
                "rep-a",
                0,
                0,
                1,
                4,
            ),
            event(
                5,
                "f2",
                FeatureEventKind::UtilityUpdate,
                "rep-a",
                0,
                0,
                0,
                -1,
            ),
        ];
        let table = derive_feature_metrics(&events).expect("table");
        let class = |id: &str| {
            table
                .classifications
                .iter()
                .find(|row| row.feature_id == id)
                .expect("row")
                .class
        };
        assert_eq!(class("f1"), FeatureClass::Useful);
        assert_eq!(class("f2"), FeatureClass::Redundant);
        assert_eq!(class("f3"), FeatureClass::Dormant);
    }

    #[test]
    fn empty_trace_fails_closed() {
        assert!(matches!(
            derive_feature_metrics(&[]),
            Err(FeatureMetricError::Trace)
        ));
    }
}
