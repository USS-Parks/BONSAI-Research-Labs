//! Continual-learning metrics with forgetting separated from plasticity loss.

use crate::{MetricKey, RationalValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// One ordered continual-learning observation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContinualPoint {
    pub step: u64,
    pub task_id: String,
    pub phase: ContinualPhase,
    pub performance: i64,
    pub agent_age: u64,
}

/// Learning phase relative to task switches.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContinualPhase {
    Train,
    RetainProbe,
    Adapt,
    Relearn,
}

/// One versioned continual metric outcome.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContinualMetric {
    pub key: MetricKey,
    pub unit: String,
    pub window: String,
    pub value: Option<RationalValue>,
    pub detail_code: Option<String>,
}

/// Exact-age continual performance curve.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContinualAgeValue {
    pub age: u64,
    pub mean_performance: RationalValue,
}

/// Deterministic continual metric table.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContinualMetricTable {
    pub schema: String,
    pub metrics: Vec<ContinualMetric>,
    pub age_curve: Vec<ContinualAgeValue>,
}

/// Failures for malformed continual traces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContinualMetricError {
    Trace,
    Arithmetic,
    Coverage,
}

impl fmt::Display for ContinualMetricError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Trace => "CONTINUAL_TRACE_INVALID",
            Self::Arithmetic => "CONTINUAL_METRIC_ARITHMETIC_FAILED",
            Self::Coverage => "CONTINUAL_METRIC_COVERAGE_INVALID",
        })
    }
}

impl Error for ContinualMetricError {}

/// Derive BK-04 continual metrics from one ordered multi-task trace.
///
/// Retention and forgetting are measured on retain probes of earlier tasks.
/// Plasticity and adaptation are measured on new-task adapt phases.
/// These families are independent: a trace may forget while remaining plastic,
/// or lose plasticity while retaining prior competence.
///
/// # Errors
///
/// Rejects empty/non-contiguous traces, empty task IDs, and checked arithmetic failures.
pub fn derive_continual_metrics(
    points: &[ContinualPoint],
) -> Result<ContinualMetricTable, ContinualMetricError> {
    validate_trace(points)?;

    let train_by_task = last_phase_performance(points, ContinualPhase::Train);
    let retain_by_task = last_phase_performance(points, ContinualPhase::RetainProbe);
    let adapt_by_task = last_phase_performance(points, ContinualPhase::Adapt);
    let relearn_by_task = last_phase_performance(points, ContinualPhase::Relearn);

    let retention = mean_ratio_against_baseline(&retain_by_task, &train_by_task)?;
    let forgetting = mean_drop_from_baseline(&retain_by_task, &train_by_task)?;
    let adaptation = mean_absolute(&adapt_by_task)?;
    let plasticity_loss = mean_plasticity_loss(&adapt_by_task, &train_by_task)?;
    let transfer = mean_signed_transfer(&adapt_by_task, &train_by_task)?;
    let interference = mean_interference(&retain_by_task, &train_by_task)?;
    let relearning = mean_relearning(&relearn_by_task, &train_by_task)?;
    let divergence = performance_divergence(points)?;
    let age_curve = derive_age_curve(points)?;

    let mut metrics = vec![
        optional_metric(
            "retention",
            "ratio",
            "retain_probes",
            retention,
            "RETENTION_UNAVAILABLE",
        ),
        optional_metric(
            "forgetting",
            "performance",
            "retain_probes",
            forgetting,
            "FORGETTING_UNAVAILABLE",
        ),
        optional_metric(
            "adaptation",
            "performance",
            "adapt_phases",
            adaptation,
            "ADAPTATION_UNAVAILABLE",
        ),
        optional_metric(
            "plasticity_loss",
            "performance",
            "adapt_phases",
            plasticity_loss,
            "PLASTICITY_UNAVAILABLE",
        ),
        optional_metric(
            "transfer",
            "performance",
            "adapt_vs_train",
            transfer,
            "TRANSFER_UNAVAILABLE",
        ),
        optional_metric(
            "interference",
            "performance",
            "retain_vs_train",
            interference,
            "INTERFERENCE_UNAVAILABLE",
        ),
        optional_metric(
            "relearning",
            "ratio",
            "relearn_phases",
            relearning,
            "RELEARNING_UNAVAILABLE",
        ),
        available_metric("divergence", "performance", "lifetime", divergence),
    ];
    metrics.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(ContinualMetricTable {
        schema: "bonsai.continual-metric-table/v1".to_owned(),
        metrics,
        age_curve,
    })
}

fn validate_trace(points: &[ContinualPoint]) -> Result<(), ContinualMetricError> {
    if points.is_empty()
        || points
            .iter()
            .enumerate()
            .any(|(index, point)| point.step != index as u64 || point.task_id.is_empty())
    {
        return Err(ContinualMetricError::Trace);
    }
    Ok(())
}

fn last_phase_performance(
    points: &[ContinualPoint],
    phase: ContinualPhase,
) -> BTreeMap<String, i64> {
    let mut map = BTreeMap::new();
    for point in points.iter().filter(|point| point.phase == phase) {
        map.insert(point.task_id.clone(), point.performance);
    }
    map
}

fn mean_ratio_against_baseline(
    observed: &BTreeMap<String, i64>,
    baseline: &BTreeMap<String, i64>,
) -> Result<Option<RationalValue>, ContinualMetricError> {
    if observed.is_empty() {
        return Ok(None);
    }
    let mut numerator = 0_i64;
    let mut denominator = 0_u64;
    for (task, value) in observed {
        let Some(base) = baseline.get(task) else {
            return Err(ContinualMetricError::Coverage);
        };
        if *base == 0 {
            return Err(ContinualMetricError::Arithmetic);
        }
        numerator = numerator
            .checked_add(*value)
            .ok_or(ContinualMetricError::Arithmetic)?;
        denominator = denominator
            .checked_add(u64::try_from(*base).map_err(|_| ContinualMetricError::Arithmetic)?)
            .ok_or(ContinualMetricError::Arithmetic)?;
    }
    rational(numerator, denominator).map(Some)
}

fn mean_drop_from_baseline(
    observed: &BTreeMap<String, i64>,
    baseline: &BTreeMap<String, i64>,
) -> Result<Option<RationalValue>, ContinualMetricError> {
    if observed.is_empty() {
        return Ok(None);
    }
    let mut total = 0_i64;
    for (task, value) in observed {
        let Some(base) = baseline.get(task) else {
            return Err(ContinualMetricError::Coverage);
        };
        let drop = base
            .checked_sub(*value)
            .ok_or(ContinualMetricError::Arithmetic)?;
        total = total
            .checked_add(drop)
            .ok_or(ContinualMetricError::Arithmetic)?;
    }
    rational(total, observed.len() as u64).map(Some)
}

fn mean_absolute(
    observed: &BTreeMap<String, i64>,
) -> Result<Option<RationalValue>, ContinualMetricError> {
    if observed.is_empty() {
        return Ok(None);
    }
    let total = observed.values().try_fold(0_i64, |sum, value| {
        sum.checked_add(*value)
            .ok_or(ContinualMetricError::Arithmetic)
    })?;
    rational(total, observed.len() as u64).map(Some)
}

fn mean_plasticity_loss(
    adapt: &BTreeMap<String, i64>,
    train: &BTreeMap<String, i64>,
) -> Result<Option<RationalValue>, ContinualMetricError> {
    // Plasticity loss: how far adapt performance falls short of the agent's
    // own prior train competence on any overlapping task identity used as a
    // reference ceiling. Tasks present only in adapt use the mean train ceiling.
    if adapt.is_empty() || train.is_empty() {
        return Ok(None);
    }
    let train_mean = {
        let total = train.values().try_fold(0_i64, |sum, value| {
            sum.checked_add(*value)
                .ok_or(ContinualMetricError::Arithmetic)
        })?;
        // Keep exact rational later; for ceiling use rounded-down integer mean only
        // as a reference when task IDs differ.
        total
            .checked_div(i64::try_from(train.len()).map_err(|_| ContinualMetricError::Arithmetic)?)
            .ok_or(ContinualMetricError::Arithmetic)?
    };
    let mut total_loss = 0_i64;
    for (task, value) in adapt {
        let ceiling = train.get(task).copied().unwrap_or(train_mean);
        let loss = ceiling
            .checked_sub(*value)
            .ok_or(ContinualMetricError::Arithmetic)?
            .max(0);
        total_loss = total_loss
            .checked_add(loss)
            .ok_or(ContinualMetricError::Arithmetic)?;
    }
    rational(total_loss, adapt.len() as u64).map(Some)
}

fn mean_signed_transfer(
    adapt: &BTreeMap<String, i64>,
    train: &BTreeMap<String, i64>,
) -> Result<Option<RationalValue>, ContinualMetricError> {
    if adapt.is_empty() || train.is_empty() {
        return Ok(None);
    }
    let train_mean_num = train.values().try_fold(0_i64, |sum, value| {
        sum.checked_add(*value)
            .ok_or(ContinualMetricError::Arithmetic)
    })?;
    let train_mean = rational(train_mean_num, train.len() as u64)?;
    let adapt_mean_num = adapt.values().try_fold(0_i64, |sum, value| {
        sum.checked_add(*value)
            .ok_or(ContinualMetricError::Arithmetic)
    })?;
    let adapt_mean = rational(adapt_mean_num, adapt.len() as u64)?;
    // transfer = adapt_mean - train_mean as exact rational difference on common denom
    let left = adapt_mean
        .numerator
        .checked_mul(
            i64::try_from(train_mean.denominator).map_err(|_| ContinualMetricError::Arithmetic)?,
        )
        .ok_or(ContinualMetricError::Arithmetic)?;
    let right = train_mean
        .numerator
        .checked_mul(
            i64::try_from(adapt_mean.denominator).map_err(|_| ContinualMetricError::Arithmetic)?,
        )
        .ok_or(ContinualMetricError::Arithmetic)?;
    let numerator = left
        .checked_sub(right)
        .ok_or(ContinualMetricError::Arithmetic)?;
    let denominator = adapt_mean
        .denominator
        .checked_mul(train_mean.denominator)
        .ok_or(ContinualMetricError::Arithmetic)?;
    rational(numerator, denominator).map(Some)
}

fn mean_interference(
    retain: &BTreeMap<String, i64>,
    train: &BTreeMap<String, i64>,
) -> Result<Option<RationalValue>, ContinualMetricError> {
    // Interference equals forgetting magnitude when retain probes exist.
    mean_drop_from_baseline(retain, train)
}

fn mean_relearning(
    relearn: &BTreeMap<String, i64>,
    train: &BTreeMap<String, i64>,
) -> Result<Option<RationalValue>, ContinualMetricError> {
    mean_ratio_against_baseline(relearn, train)
}

fn performance_divergence(
    points: &[ContinualPoint],
) -> Result<RationalValue, ContinualMetricError> {
    let min = points
        .iter()
        .map(|point| point.performance)
        .min()
        .ok_or(ContinualMetricError::Trace)?;
    let max = points
        .iter()
        .map(|point| point.performance)
        .max()
        .ok_or(ContinualMetricError::Trace)?;
    rational(
        max.checked_sub(min)
            .ok_or(ContinualMetricError::Arithmetic)?,
        1,
    )
}

fn derive_age_curve(
    points: &[ContinualPoint],
) -> Result<Vec<ContinualAgeValue>, ContinualMetricError> {
    let mut buckets: BTreeMap<u64, (i64, u64)> = BTreeMap::new();
    for point in points {
        let entry = buckets.entry(point.agent_age).or_insert((0, 0));
        entry.0 = entry
            .0
            .checked_add(point.performance)
            .ok_or(ContinualMetricError::Arithmetic)?;
        entry.1 = entry
            .1
            .checked_add(1)
            .ok_or(ContinualMetricError::Arithmetic)?;
    }
    buckets
        .into_iter()
        .map(|(age, (sum, count))| {
            Ok(ContinualAgeValue {
                age,
                mean_performance: rational(sum, count)?,
            })
        })
        .collect()
}

fn rational(numerator: i64, denominator: u64) -> Result<RationalValue, ContinualMetricError> {
    if denominator == 0 {
        return Err(ContinualMetricError::Arithmetic);
    }
    Ok(RationalValue {
        numerator,
        denominator,
    })
}

fn available_metric(id: &str, unit: &str, window: &str, value: RationalValue) -> ContinualMetric {
    ContinualMetric {
        key: MetricKey {
            id: id.to_owned(),
            version: "1.0.0".to_owned(),
        },
        unit: unit.to_owned(),
        window: window.to_owned(),
        value: Some(value),
        detail_code: None,
    }
}

fn optional_metric(
    id: &str,
    unit: &str,
    window: &str,
    value: Option<RationalValue>,
    missing: &str,
) -> ContinualMetric {
    match value {
        Some(value) => ContinualMetric {
            key: MetricKey {
                id: id.to_owned(),
                version: "1.0.0".to_owned(),
            },
            unit: unit.to_owned(),
            window: window.to_owned(),
            value: Some(value),
            detail_code: None,
        },
        None => ContinualMetric {
            key: MetricKey {
                id: id.to_owned(),
                version: "1.0.0".to_owned(),
            },
            unit: unit.to_owned(),
            window: window.to_owned(),
            value: None,
            detail_code: Some(missing.to_owned()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{ContinualPhase, ContinualPoint, derive_continual_metrics};

    fn point(
        step: u64,
        task_id: &str,
        phase: ContinualPhase,
        performance: i64,
        agent_age: u64,
    ) -> ContinualPoint {
        ContinualPoint {
            step,
            task_id: task_id.to_owned(),
            phase,
            performance,
            agent_age,
        }
    }

    #[test]
    fn forgetting_and_plasticity_are_independently_manipulated() {
        // High forgetting on task A, but strong plasticity on task B.
        let forgetting_trace = vec![
            point(0, "A", ContinualPhase::Train, 10, 0),
            point(1, "B", ContinualPhase::Adapt, 10, 1),
            point(2, "A", ContinualPhase::RetainProbe, 2, 2),
        ];
        let forgetting = derive_continual_metrics(&forgetting_trace).expect("forgetting");
        let forget = forgetting
            .metrics
            .iter()
            .find(|metric| metric.key.id == "forgetting")
            .expect("forgetting metric");
        let plasticity = forgetting
            .metrics
            .iter()
            .find(|metric| metric.key.id == "plasticity_loss")
            .expect("plasticity");
        assert_eq!(forget.value.as_ref().expect("value").numerator, 8);
        assert_eq!(plasticity.value.as_ref().expect("value").numerator, 0);

        // Low forgetting, high plasticity loss on the new task.
        let plasticity_trace = vec![
            point(0, "A", ContinualPhase::Train, 10, 0),
            point(1, "B", ContinualPhase::Adapt, 1, 1),
            point(2, "A", ContinualPhase::RetainProbe, 10, 2),
        ];
        let plasticity_table = derive_continual_metrics(&plasticity_trace).expect("plasticity");
        let forget2 = plasticity_table
            .metrics
            .iter()
            .find(|metric| metric.key.id == "forgetting")
            .expect("forgetting");
        let plasticity2 = plasticity_table
            .metrics
            .iter()
            .find(|metric| metric.key.id == "plasticity_loss")
            .expect("plasticity");
        assert_eq!(forget2.value.as_ref().expect("value").numerator, 0);
        assert!(plasticity2.value.as_ref().expect("value").numerator > 0);
        assert_ne!(
            forget.value.as_ref().expect("v"),
            forget2.value.as_ref().expect("v")
        );
        assert_ne!(
            plasticity.value.as_ref().expect("v"),
            plasticity2.value.as_ref().expect("v")
        );
    }
}
