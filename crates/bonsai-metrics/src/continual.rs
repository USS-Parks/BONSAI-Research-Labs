//! Continual-learning metrics that separate forgetting from plasticity loss.

use crate::{MetricKey, RationalValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// Role of one labeled diagnostic phase.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContinualPhaseRole {
    First,
    Intervening,
    Return,
}

/// One ordered performance sample.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContinualPoint {
    pub step: u64,
    pub task_id: String,
    pub performance: i64,
    pub competent: bool,
    pub agent_age: u64,
}

/// One contiguous labeled phase used to separate retention from adaptation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContinualPhase {
    pub phase_id: String,
    pub task_id: String,
    pub role: ContinualPhaseRole,
    pub start_step: u64,
    pub end_step: u64,
    pub attainable: i64,
}

/// One diagnostic continual-learning trace.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContinualTrace {
    pub points: Vec<ContinualPoint>,
    pub phases: Vec<ContinualPhase>,
    pub baseline_intervening_without_prior: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContinualMetric {
    pub key: MetricKey,
    pub unit: String,
    pub window: String,
    pub value: Option<RationalValue>,
    pub detail_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgeConditionedPerformance {
    pub age: u64,
    pub performance: RationalValue,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContinualMetricTable {
    pub schema: String,
    pub metrics: Vec<ContinualMetric>,
    pub age_conditioned_performance: Vec<AgeConditionedPerformance>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContinualMetricError {
    Trace,
    Phase,
    Arithmetic,
}

impl fmt::Display for ContinualMetricError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Trace => "CONTINUAL_TRACE_INVALID",
            Self::Phase => "CONTINUAL_PHASE_INVALID",
            Self::Arithmetic => "CONTINUAL_METRIC_ARITHMETIC_FAILED",
        })
    }
}

impl Error for ContinualMetricError {}

/// Derive retention, adaptation, forgetting, and plasticity-loss metrics.
///
/// Forgetting is the drop on the first task after the intervening task.
/// Plasticity loss is the shortfall of intervening performance versus that
/// phase's attainable score. The two quantities are independent.
///
/// # Errors
///
/// Rejects empty/non-contiguous traces, missing labeled phases, and overflow.
#[allow(clippy::too_many_lines)]
pub fn derive_continual_metrics(
    trace: &ContinualTrace,
) -> Result<ContinualMetricTable, ContinualMetricError> {
    validate_trace(trace)?;
    let first = phase_mean(trace, ContinualPhaseRole::First)?;
    let intervening = phase_mean(trace, ContinualPhaseRole::Intervening)?;
    let returning = phase_mean(trace, ContinualPhaseRole::Return)?;
    let first_phase = role(trace, ContinualPhaseRole::First)?;
    let intervening_phase = role(trace, ContinualPhaseRole::Intervening)?;
    let return_phase = role(trace, ContinualPhaseRole::Return)?;

    let forgetting = first
        .checked_sub(returning)
        .ok_or(ContinualMetricError::Arithmetic)?;
    let plasticity_loss = intervening_phase
        .attainable
        .checked_sub(intervening)
        .ok_or(ContinualMetricError::Arithmetic)?;
    let retention = returning;
    let adaptation = intervening;
    let interference = forgetting;
    let transfer = match trace.baseline_intervening_without_prior {
        Some(baseline) => Some(
            intervening
                .checked_sub(baseline)
                .ok_or(ContinualMetricError::Arithmetic)?,
        ),
        None => None,
    };
    let relearning = relearning_steps(trace, return_phase)?;
    let divergence = first
        .checked_sub(returning)
        .and_then(i64::checked_abs)
        .and_then(|left| {
            first
                .checked_sub(intervening)
                .and_then(i64::checked_abs)
                .and_then(|right| left.checked_add(right))
        })
        .ok_or(ContinualMetricError::Arithmetic)?;

    let mut metrics = vec![
        available(
            "first_task_performance",
            "performance",
            &first_phase.phase_id,
            first,
        )?,
        available(
            "intervening_task_performance",
            "performance",
            &intervening_phase.phase_id,
            intervening,
        )?,
        available(
            "return_task_performance",
            "performance",
            &return_phase.phase_id,
            returning,
        )?,
        available("retention", "performance", "return_vs_first", retention)?,
        available(
            "forgetting",
            "performance",
            "first_minus_return",
            forgetting,
        )?,
        available("adaptation", "performance", "intervening", adaptation)?,
        available(
            "plasticity_loss",
            "performance",
            "intervening_attainable_minus_actual",
            plasticity_loss,
        )?,
        available(
            "interference",
            "performance",
            "first_minus_return",
            interference,
        )?,
        optional(
            "transfer",
            "performance",
            "intervening_minus_no_prior_baseline",
            transfer.map(|value| rational(value, 1)).transpose()?,
            "TRANSFER_BASELINE_UNAVAILABLE",
        ),
        optional(
            "relearning_steps",
            "step",
            "return_until_competent",
            relearning.map(|value| rational(value, 1)).transpose()?,
            "RELEARNING_UNAVAILABLE",
        ),
        available(
            "divergence",
            "performance",
            "phase_abs_delta_sum",
            divergence,
        )?,
    ];
    metrics.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(ContinualMetricTable {
        schema: "bonsai.continual-metric-table/v1".to_owned(),
        metrics,
        age_conditioned_performance: age_curves(&trace.points)?,
    })
}

fn validate_trace(trace: &ContinualTrace) -> Result<(), ContinualMetricError> {
    if trace.points.is_empty()
        || trace
            .points
            .iter()
            .enumerate()
            .any(|(index, point)| point.step != index as u64 || point.task_id.is_empty())
    {
        return Err(ContinualMetricError::Trace);
    }
    let mut seen = [false; 3];
    for phase in &trace.phases {
        if phase.phase_id.is_empty()
            || phase.task_id.is_empty()
            || phase.end_step <= phase.start_step
            || phase.end_step > trace.points.len() as u64
        {
            return Err(ContinualMetricError::Phase);
        }
        let index = match phase.role {
            ContinualPhaseRole::First => 0,
            ContinualPhaseRole::Intervening => 1,
            ContinualPhaseRole::Return => 2,
        };
        if seen[index] {
            return Err(ContinualMetricError::Phase);
        }
        seen[index] = true;
        let start = usize_step(phase.start_step)?;
        let end = usize_step(phase.end_step)?;
        if trace.points[start..end]
            .iter()
            .any(|point| point.task_id != phase.task_id)
        {
            return Err(ContinualMetricError::Phase);
        }
    }
    if seen.iter().any(|present| !present) {
        return Err(ContinualMetricError::Phase);
    }
    Ok(())
}

fn role(
    trace: &ContinualTrace,
    wanted: ContinualPhaseRole,
) -> Result<&ContinualPhase, ContinualMetricError> {
    trace
        .phases
        .iter()
        .find(|phase| phase.role == wanted)
        .ok_or(ContinualMetricError::Phase)
}

fn phase_mean(
    trace: &ContinualTrace,
    wanted: ContinualPhaseRole,
) -> Result<i64, ContinualMetricError> {
    let phase = role(trace, wanted)?;
    let window = &trace.points[usize_step(phase.start_step)?..usize_step(phase.end_step)?];
    let total = window.iter().try_fold(0_i64, |sum, point| {
        sum.checked_add(point.performance)
            .ok_or(ContinualMetricError::Arithmetic)
    })?;
    Ok(total / i64::try_from(window.len()).map_err(|_| ContinualMetricError::Arithmetic)?)
}

fn relearning_steps(
    trace: &ContinualTrace,
    phase: &ContinualPhase,
) -> Result<Option<i64>, ContinualMetricError> {
    let recovered = trace.points[usize_step(phase.start_step)?..usize_step(phase.end_step)?]
        .iter()
        .find(|point| point.competent)
        .map(|point| point.step.checked_sub(phase.start_step));
    recovered
        .map(|steps| {
            steps
                .ok_or(ContinualMetricError::Arithmetic)
                .and_then(|value| {
                    i64::try_from(value).map_err(|_| ContinualMetricError::Arithmetic)
                })
        })
        .transpose()
}

fn age_curves(
    points: &[ContinualPoint],
) -> Result<Vec<AgeConditionedPerformance>, ContinualMetricError> {
    let mut by_age: BTreeMap<u64, (i64, u64)> = BTreeMap::new();
    for point in points {
        let entry = by_age.entry(point.agent_age).or_insert((0, 0));
        entry.0 = entry
            .0
            .checked_add(point.performance)
            .ok_or(ContinualMetricError::Arithmetic)?;
        entry.1 = entry
            .1
            .checked_add(1)
            .ok_or(ContinualMetricError::Arithmetic)?;
    }
    by_age
        .into_iter()
        .map(|(age, (performance, count))| {
            Ok(AgeConditionedPerformance {
                age,
                performance: rational(performance, count)?,
            })
        })
        .collect()
}

fn usize_step(step: u64) -> Result<usize, ContinualMetricError> {
    usize::try_from(step).map_err(|_| ContinualMetricError::Arithmetic)
}

fn key(id: &str) -> MetricKey {
    MetricKey {
        id: id.to_owned(),
        version: "1.0".to_owned(),
    }
}

fn available(
    id: &str,
    unit: &str,
    window: &str,
    value: i64,
) -> Result<ContinualMetric, ContinualMetricError> {
    Ok(ContinualMetric {
        key: key(id),
        unit: unit.to_owned(),
        window: window.to_owned(),
        value: Some(rational(value, 1)?),
        detail_code: None,
    })
}

fn optional(
    id: &str,
    unit: &str,
    window: &str,
    value: Option<RationalValue>,
    detail: &str,
) -> ContinualMetric {
    ContinualMetric {
        key: key(id),
        unit: unit.to_owned(),
        window: window.to_owned(),
        detail_code: value.is_none().then(|| detail.to_owned()),
        value,
    }
}

fn rational(numerator: i64, denominator: u64) -> Result<RationalValue, ContinualMetricError> {
    if denominator == 0 {
        return Err(ContinualMetricError::Arithmetic);
    }
    Ok(super::normalize(RationalValue {
        numerator,
        denominator,
    }))
}

#[cfg(test)]
mod tests {
    use super::{
        ContinualPhase, ContinualPhaseRole, ContinualPoint, ContinualTrace,
        derive_continual_metrics,
    };
    use crate::RationalValue;

    fn point(step: u64, task: &str, performance: i64, competent: bool) -> ContinualPoint {
        ContinualPoint {
            step,
            task_id: task.to_owned(),
            performance,
            competent,
            agent_age: step,
        }
    }

    fn phase(
        id: &str,
        task: &str,
        role: ContinualPhaseRole,
        start: u64,
        end: u64,
        attainable: i64,
    ) -> ContinualPhase {
        ContinualPhase {
            phase_id: id.to_owned(),
            task_id: task.to_owned(),
            role,
            start_step: start,
            end_step: end,
            attainable,
        }
    }

    fn phases(attainable_b: i64) -> Vec<ContinualPhase> {
        vec![
            phase("A0", "A", ContinualPhaseRole::First, 0, 2, 10),
            phase(
                "B0",
                "B",
                ContinualPhaseRole::Intervening,
                2,
                4,
                attainable_b,
            ),
            phase("A1", "A", ContinualPhaseRole::Return, 4, 6, 10),
        ]
    }

    fn trace(
        a0: i64,
        b: i64,
        a1: i64,
        attainable_b: i64,
        competent_return: bool,
    ) -> ContinualTrace {
        ContinualTrace {
            points: vec![
                point(0, "A", a0, true),
                point(1, "A", a0, true),
                point(2, "B", b, b >= attainable_b),
                point(3, "B", b, b >= attainable_b),
                point(4, "A", a1, competent_return),
                point(5, "A", a1, competent_return),
            ],
            phases: phases(attainable_b),
            baseline_intervening_without_prior: Some(0),
        }
    }

    fn value(table: &super::ContinualMetricTable, id: &str) -> Option<i64> {
        table
            .metrics
            .iter()
            .find(|metric| metric.key.id == id)
            .and_then(|metric| metric.value.clone())
            .map(|value| {
                assert_eq!(
                    value,
                    RationalValue {
                        numerator: value.numerator,
                        denominator: 1
                    }
                );
                value.numerator
            })
    }

    #[test]
    fn forgetting_and_plasticity_are_independently_manipulated() {
        let forget_learn = derive_continual_metrics(&trace(10, 10, 0, 10, false)).expect("forget");
        let retain_rigid = derive_continual_metrics(&trace(10, 0, 10, 10, true)).expect("rigid");
        let retain_adapt = derive_continual_metrics(&trace(10, 10, 10, 10, true)).expect("adapt");

        assert_eq!(value(&forget_learn, "forgetting"), Some(10));
        assert_eq!(value(&forget_learn, "plasticity_loss"), Some(0));
        assert_eq!(value(&retain_rigid, "forgetting"), Some(0));
        assert_eq!(value(&retain_rigid, "plasticity_loss"), Some(10));
        assert_eq!(value(&retain_adapt, "forgetting"), Some(0));
        assert_eq!(value(&retain_adapt, "plasticity_loss"), Some(0));
        assert_ne!(
            value(&forget_learn, "forgetting"),
            value(&retain_rigid, "forgetting")
        );
        assert_ne!(
            value(&forget_learn, "plasticity_loss"),
            value(&retain_rigid, "plasticity_loss")
        );
    }

    #[test]
    fn transfer_and_relearning_are_unavailable_without_evidence() {
        let mut missing = trace(10, 8, 6, 10, false);
        missing.baseline_intervening_without_prior = None;
        missing.points[4].competent = false;
        missing.points[5].competent = false;
        let table = derive_continual_metrics(&missing).expect("metrics");
        assert_eq!(value(&table, "transfer"), None);
        assert_eq!(value(&table, "relearning_steps"), None);
        assert_eq!(
            table
                .metrics
                .iter()
                .find(|metric| metric.key.id == "transfer")
                .expect("transfer")
                .detail_code
                .as_deref(),
            Some("TRANSFER_BASELINE_UNAVAILABLE")
        );
    }

    #[test]
    fn committed_forgetting_and_plasticity_fixtures_are_exact() {
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/continual-metrics/v1/expected-outcomes.json"
        ))
        .expect("fixture");
        let observed = serde_json::json!({
            "schema": "bonsai.continual-metric-outcomes/v1",
            "forget_learn": {
                "forgetting": value(&derive_continual_metrics(&trace(10, 10, 0, 10, false)).expect("forget"), "forgetting"),
                "plasticity_loss": value(&derive_continual_metrics(&trace(10, 10, 0, 10, false)).expect("forget"), "plasticity_loss"),
            },
            "retain_rigid": {
                "forgetting": value(&derive_continual_metrics(&trace(10, 0, 10, 10, true)).expect("rigid"), "forgetting"),
                "plasticity_loss": value(&derive_continual_metrics(&trace(10, 0, 10, 10, true)).expect("rigid"), "plasticity_loss"),
            },
            "retain_adapt": {
                "forgetting": value(&derive_continual_metrics(&trace(10, 10, 10, 10, true)).expect("adapt"), "forgetting"),
                "plasticity_loss": value(&derive_continual_metrics(&trace(10, 10, 10, 10, true)).expect("adapt"), "plasticity_loss"),
            },
        });
        assert_eq!(observed, expected);
    }
}
