//! Primary behavior metrics over deterministic analytical traces.

use crate::{MetricKey, RationalValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BehaviorPoint {
    pub step: u64,
    pub reward: i64,
    pub performance: i64,
    pub comparator_reward: Option<i64>,
    pub competent: bool,
    pub change_point: bool,
    pub agent_age: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BehaviorMetric {
    pub key: MetricKey,
    pub unit: String,
    pub window: String,
    pub value: Option<RationalValue>,
    pub detail_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgeConditionedValue {
    pub age: u64,
    pub reward_rate: RationalValue,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BehaviorMetricTable {
    pub schema: String,
    pub metrics: Vec<BehaviorMetric>,
    pub age_conditioned_reward: Vec<AgeConditionedValue>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BehaviorMetricError {
    Trace,
    Window,
    Arithmetic,
}

impl fmt::Display for BehaviorMetricError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Trace => "BEHAVIOR_TRACE_INVALID",
            Self::Window => "BEHAVIOR_WINDOW_INVALID",
            Self::Arithmetic => "BEHAVIOR_METRIC_ARITHMETIC_FAILED",
        })
    }
}

impl Error for BehaviorMetricError {}

/// Derive the BK-02 primary metric set from one ordered trace.
///
/// Regret is available only when every point carries a defensible comparator.
/// Recovery is available only when every declared change point later reaches
/// competency. No missing outcome is converted to zero.
///
/// # Errors
///
/// Rejects empty/non-contiguous traces, invalid windows, and checked arithmetic
/// failures.
pub fn derive_behavior_metrics(
    points: &[BehaviorPoint],
    worst_window_steps: usize,
) -> Result<BehaviorMetricTable, BehaviorMetricError> {
    validate_trace(points, worst_window_steps)?;

    let cumulative_reward = checked_sum(points.iter().map(|point| point.reward))?;
    let reward_rate = rational(cumulative_reward, points.len() as u64)?;
    let regret = points
        .iter()
        .map(|point| {
            point
                .comparator_reward
                .and_then(|comparator| comparator.checked_sub(point.reward))
        })
        .collect::<Option<Vec<_>>>()
        .map(|values| checked_sum(values.into_iter()))
        .transpose()?;
    let lifelong_auc_twice = points.windows(2).try_fold(0_i64, |total, pair| {
        pair[0]
            .performance
            .checked_add(pair[1].performance)
            .and_then(|interval| total.checked_add(interval))
            .ok_or(BehaviorMetricError::Arithmetic)
    })?;
    let competency_time = points
        .iter()
        .find(|point| point.competent)
        .map(|point| i64::try_from(point.step).map_err(|_| BehaviorMetricError::Arithmetic))
        .transpose()?;

    let (mean_recovery, worst_recovery) = derive_recovery(points)?;

    let worst_window = points
        .windows(worst_window_steps)
        .map(|window| checked_sum(window.iter().map(|point| point.reward)))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .min()
        .ok_or(BehaviorMetricError::Window)?;

    let age_conditioned_reward = derive_age_conditioned(points)?;

    let mut metrics = vec![
        available(
            "cumulative_reward",
            "reward",
            "lifetime",
            rational(cumulative_reward, 1)?,
        ),
        available("reward_rate", "reward/step", "lifetime", reward_rate),
        optional(
            "regret",
            "reward",
            "lifetime",
            regret.map(|value| rational(value, 1)).transpose()?,
            "COMPARATOR_UNAVAILABLE",
        ),
        available(
            "lifelong_auc",
            "performance_step",
            "lifetime",
            rational(lifelong_auc_twice, 2)?,
        ),
        optional(
            "competency_time",
            "step",
            "first_competent",
            competency_time
                .map(|value| rational(value, 1))
                .transpose()?,
            "COMPETENCY_NOT_REACHED",
        ),
        optional(
            "mean_recovery_steps",
            "step",
            "all_change_points",
            mean_recovery,
            "RECOVERY_UNAVAILABLE",
        ),
        optional(
            "worst_recovery_steps",
            "step",
            "all_change_points",
            worst_recovery,
            "RECOVERY_UNAVAILABLE",
        ),
        available(
            "worst_window_reward_rate",
            "reward/step",
            &format!("rolling_{worst_window_steps}_steps"),
            rational(worst_window, worst_window_steps as u64)?,
        ),
    ];
    metrics.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(BehaviorMetricTable {
        schema: "bonsai.behavior-metric-table/v1".to_owned(),
        metrics,
        age_conditioned_reward,
    })
}

fn validate_trace(
    points: &[BehaviorPoint],
    worst_window_steps: usize,
) -> Result<(), BehaviorMetricError> {
    if points.is_empty()
        || points
            .iter()
            .enumerate()
            .any(|(index, point)| point.step != index as u64)
    {
        return Err(BehaviorMetricError::Trace);
    }
    if worst_window_steps == 0 || worst_window_steps > points.len() {
        return Err(BehaviorMetricError::Window);
    }
    Ok(())
}

fn derive_recovery(
    points: &[BehaviorPoint],
) -> Result<(Option<RationalValue>, Option<RationalValue>), BehaviorMetricError> {
    let recovery_durations = points
        .iter()
        .filter(|point| point.change_point)
        .map(|change| {
            points
                .iter()
                .skip_while(|point| point.step < change.step)
                .find(|point| point.competent)
                .map(|point| point.step - change.step)
        })
        .collect::<Option<Vec<_>>>();
    let recovery = recovery_durations
        .as_ref()
        .filter(|durations| !durations.is_empty());
    let mean = recovery
        .map(|durations| {
            let total = durations.iter().try_fold(0_u64, |sum, duration| {
                sum.checked_add(*duration)
                    .ok_or(BehaviorMetricError::Arithmetic)
            })?;
            rational(
                i64::try_from(total).map_err(|_| BehaviorMetricError::Arithmetic)?,
                durations.len() as u64,
            )
        })
        .transpose()?;
    let worst = recovery
        .and_then(|durations| durations.iter().copied().max())
        .map(|duration| {
            rational(
                i64::try_from(duration).map_err(|_| BehaviorMetricError::Arithmetic)?,
                1,
            )
        })
        .transpose()?;
    Ok((mean, worst))
}

fn derive_age_conditioned(
    points: &[BehaviorPoint],
) -> Result<Vec<AgeConditionedValue>, BehaviorMetricError> {
    let mut by_age: BTreeMap<u64, (i64, u64)> = BTreeMap::new();
    for point in points {
        let entry = by_age.entry(point.agent_age).or_insert((0, 0));
        entry.0 = entry
            .0
            .checked_add(point.reward)
            .ok_or(BehaviorMetricError::Arithmetic)?;
        entry.1 = entry
            .1
            .checked_add(1)
            .ok_or(BehaviorMetricError::Arithmetic)?;
    }
    by_age
        .into_iter()
        .map(|(age, (reward, count))| {
            Ok(AgeConditionedValue {
                age,
                reward_rate: rational(reward, count)?,
            })
        })
        .collect()
}

fn key(id: &str) -> MetricKey {
    MetricKey {
        id: id.to_owned(),
        version: "1.0".to_owned(),
    }
}

fn available(id: &str, unit: &str, window: &str, value: RationalValue) -> BehaviorMetric {
    BehaviorMetric {
        key: key(id),
        unit: unit.to_owned(),
        window: window.to_owned(),
        value: Some(value),
        detail_code: None,
    }
}

fn optional(
    id: &str,
    unit: &str,
    window: &str,
    value: Option<RationalValue>,
    detail: &str,
) -> BehaviorMetric {
    BehaviorMetric {
        key: key(id),
        unit: unit.to_owned(),
        window: window.to_owned(),
        detail_code: value.is_none().then(|| detail.to_owned()),
        value,
    }
}

fn checked_sum(mut values: impl Iterator<Item = i64>) -> Result<i64, BehaviorMetricError> {
    values.try_fold(0_i64, |sum, value| {
        sum.checked_add(value)
            .ok_or(BehaviorMetricError::Arithmetic)
    })
}

fn rational(numerator: i64, denominator: u64) -> Result<RationalValue, BehaviorMetricError> {
    if denominator == 0 {
        return Err(BehaviorMetricError::Arithmetic);
    }
    Ok(super::normalize(RationalValue {
        numerator,
        denominator,
    }))
}

#[cfg(test)]
mod tests {
    use super::{BehaviorPoint, derive_behavior_metrics};
    use crate::RationalValue;

    fn curve() -> Vec<BehaviorPoint> {
        vec![
            BehaviorPoint {
                step: 0,
                reward: 0,
                performance: 0,
                comparator_reward: Some(2),
                competent: false,
                change_point: false,
                agent_age: 0,
            },
            BehaviorPoint {
                step: 1,
                reward: 2,
                performance: 2,
                comparator_reward: Some(2),
                competent: true,
                change_point: false,
                agent_age: 1,
            },
            BehaviorPoint {
                step: 2,
                reward: 0,
                performance: 0,
                comparator_reward: Some(2),
                competent: false,
                change_point: true,
                agent_age: 2,
            },
            BehaviorPoint {
                step: 3,
                reward: 2,
                performance: 2,
                comparator_reward: Some(2),
                competent: true,
                change_point: false,
                agent_age: 3,
            },
        ]
    }

    fn value(table: &super::BehaviorMetricTable, id: &str) -> Option<RationalValue> {
        table
            .metrics
            .iter()
            .find(|metric| metric.key.id == id)
            .and_then(|metric| metric.value.clone())
    }

    #[test]
    fn analytical_curve_has_exact_primary_metrics_and_units() {
        let table = derive_behavior_metrics(&curve(), 2).expect("metrics");
        assert_eq!(
            value(&table, "cumulative_reward"),
            Some(RationalValue {
                numerator: 4,
                denominator: 1
            })
        );
        assert_eq!(
            value(&table, "reward_rate"),
            Some(RationalValue {
                numerator: 1,
                denominator: 1
            })
        );
        assert_eq!(
            value(&table, "regret"),
            Some(RationalValue {
                numerator: 4,
                denominator: 1
            })
        );
        assert_eq!(
            value(&table, "lifelong_auc"),
            Some(RationalValue {
                numerator: 3,
                denominator: 1
            })
        );
        assert_eq!(
            value(&table, "competency_time"),
            Some(RationalValue {
                numerator: 1,
                denominator: 1
            })
        );
        assert_eq!(
            value(&table, "mean_recovery_steps"),
            Some(RationalValue {
                numerator: 1,
                denominator: 1
            })
        );
        assert_eq!(
            value(&table, "worst_window_reward_rate"),
            Some(RationalValue {
                numerator: 1,
                denominator: 1
            })
        );
        assert_eq!(table.age_conditioned_reward.len(), 4);
        assert_eq!(
            table
                .metrics
                .iter()
                .find(|metric| metric.key.id == "worst_window_reward_rate")
                .expect("metric")
                .window,
            "rolling_2_steps"
        );
    }

    #[test]
    fn indefensible_regret_and_unrecovered_change_are_unavailable() {
        let mut points = curve();
        points[1].comparator_reward = None;
        points[3].competent = false;
        let table = derive_behavior_metrics(&points, 2).expect("metrics");
        assert_eq!(value(&table, "regret"), None);
        assert_eq!(value(&table, "mean_recovery_steps"), None);
        assert_eq!(value(&table, "worst_recovery_steps"), None);
        assert_eq!(
            table
                .metrics
                .iter()
                .find(|metric| metric.key.id == "regret")
                .expect("regret")
                .detail_code
                .as_deref(),
            Some("COMPARATOR_UNAVAILABLE")
        );
    }

    #[test]
    fn trace_and_window_contracts_fail_closed() {
        assert!(derive_behavior_metrics(&[], 1).is_err());
        let mut points = curve();
        points[2].step = 9;
        assert!(derive_behavior_metrics(&points, 2).is_err());
        assert!(derive_behavior_metrics(&curve(), 5).is_err());
    }
}
