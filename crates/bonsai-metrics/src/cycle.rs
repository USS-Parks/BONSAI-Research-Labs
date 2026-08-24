//! Discovery-cycle health metrics. Artifact count is not open-endedness.

use crate::RationalValue;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

/// Exact diagnostic class for a controlled cycle trace.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CycleClass {
    Healthy,
    Collapse,
    Runaway,
    Cycling,
    Ossified,
}

/// One discovery-cycle window with forward, backward, and population fields.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CycleTrace {
    pub trace_id: String,
    pub forward_rate: i64,
    pub backward_credit_latency: i64,
    pub backward_credit_magnitude: i64,
    pub useful_count: u64,
    pub total_count: u64,
    pub generations: u64,
    pub bottleneck: bool,
    pub maintenance_cost: i64,
    pub benefit: i64,
    pub population_delta: i64,
    pub unique_lineages: u64,
    pub recycle_rate: i64,
    pub novelty_rate: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CycleRow {
    pub trace_id: String,
    pub class: CycleClass,
    pub forward_rate: i64,
    pub backward_credit_latency: i64,
    pub backward_credit_magnitude: i64,
    pub useful_total_ratio: Option<RationalValue>,
    pub generations: u64,
    pub bottleneck: bool,
    pub maintenance_benefit: Option<RationalValue>,
    pub survival_by_utility: Option<RationalValue>,
    pub open_endedness: Option<RationalValue>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CycleTable {
    pub schema: String,
    pub rows: Vec<CycleRow>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CycleError {
    Identity,
    Trace,
}

impl fmt::Display for CycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "CYCLE_IDENTITY_INVALID",
            Self::Trace => "CYCLE_TRACE_INVALID",
        })
    }
}

impl Error for CycleError {}

/// Derive cycle-health classes without treating artifact count as openness.
///
/// Open-endedness is novelty among useful artifacts. Total population size
/// is retained for runaway/collapse detection only.
///
/// # Errors
///
/// Rejects empty identities or a zero total population.
pub fn derive_cycle_health(traces: &[CycleTrace]) -> Result<CycleTable, CycleError> {
    if traces.is_empty() {
        return Err(CycleError::Trace);
    }
    let mut ids = BTreeSet::new();
    let mut rows = Vec::new();
    for trace in traces {
        if trace.trace_id.is_empty() || !ids.insert(trace.trace_id.as_str()) {
            return Err(CycleError::Identity);
        }
        if trace.total_count == 0 {
            return Err(CycleError::Trace);
        }
        rows.push(CycleRow {
            trace_id: trace.trace_id.clone(),
            class: classify(trace),
            forward_rate: trace.forward_rate,
            backward_credit_latency: trace.backward_credit_latency,
            backward_credit_magnitude: trace.backward_credit_magnitude,
            useful_total_ratio: Some(crate::normalize(RationalValue {
                numerator: i64::try_from(trace.useful_count).unwrap_or(i64::MAX),
                denominator: trace.total_count,
            })),
            generations: trace.generations,
            bottleneck: trace.bottleneck,
            maintenance_benefit: (trace.maintenance_cost != 0).then_some(crate::normalize(
                RationalValue {
                    numerator: trace.benefit,
                    denominator: trace.maintenance_cost.unsigned_abs(),
                },
            )),
            survival_by_utility: Some(crate::normalize(RationalValue {
                numerator: i64::try_from(trace.useful_count).unwrap_or(i64::MAX),
                denominator: trace.total_count,
            })),
            open_endedness: Some(crate::normalize(RationalValue {
                numerator: trace.novelty_rate,
                denominator: trace.total_count,
            })),
        });
    }
    rows.sort_by(|left, right| left.trace_id.cmp(&right.trace_id));
    Ok(CycleTable {
        schema: "bonsai.cycle-health-table/v1".to_owned(),
        rows,
    })
}

fn classify(trace: &CycleTrace) -> CycleClass {
    if trace.useful_count == 0
        || (trace.population_delta < 0
            && trace.benefit <= 0
            && trace.useful_count * 2 < trace.total_count)
    {
        CycleClass::Collapse
    } else if trace.population_delta > 0
        && trace.useful_count.saturating_mul(4) < trace.total_count
        && trace.benefit <= trace.maintenance_cost
    {
        CycleClass::Runaway
    } else if trace.recycle_rate > trace.forward_rate
        && trace.novelty_rate == 0
        && trace.unique_lineages <= 1
    {
        CycleClass::Cycling
    } else if trace.generations >= 3
        && trace.novelty_rate == 0
        && (trace.bottleneck || trace.forward_rate == 0)
    {
        CycleClass::Ossified
    } else {
        CycleClass::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::{CycleClass, CycleTrace, derive_cycle_health};

    #[allow(clippy::too_many_arguments)]
    fn trace(
        id: &str,
        useful: u64,
        total: u64,
        benefit: i64,
        maintenance: i64,
        pop: i64,
        novelty: i64,
        forward: i64,
        recycle: i64,
        lineages: u64,
        generations: u64,
        bottleneck: bool,
    ) -> CycleTrace {
        CycleTrace {
            trace_id: id.to_owned(),
            forward_rate: forward,
            backward_credit_latency: 2,
            backward_credit_magnitude: 3,
            useful_count: useful,
            total_count: total,
            generations,
            bottleneck,
            maintenance_cost: maintenance,
            benefit,
            population_delta: pop,
            unique_lineages: lineages,
            recycle_rate: recycle,
            novelty_rate: novelty,
        }
    }

    fn diagnostic() -> Vec<CycleTrace> {
        vec![
            trace("healthy", 4, 5, 10, 2, 1, 3, 2, 0, 3, 2, false),
            trace("collapse", 0, 5, -1, 2, -3, 0, 0, 0, 1, 2, false),
            trace("runaway", 1, 20, 0, 8, 10, 0, 1, 0, 2, 2, false),
            trace("cycling", 3, 4, 4, 2, 0, 0, 1, 8, 1, 2, false),
            trace("ossified", 2, 2, 4, 1, 0, 0, 0, 0, 1, 8, true),
        ]
    }

    #[test]
    fn synthetic_traces_separate_health_classes() {
        let table = derive_cycle_health(&diagnostic()).expect("cycle");
        let class = |id: &str| {
            table
                .rows
                .iter()
                .find(|row| row.trace_id == id)
                .expect("row")
                .class
        };
        assert_eq!(class("healthy"), CycleClass::Healthy);
        assert_eq!(class("collapse"), CycleClass::Collapse);
        assert_eq!(class("runaway"), CycleClass::Runaway);
        assert_eq!(class("cycling"), CycleClass::Cycling);
        assert_eq!(class("ossified"), CycleClass::Ossified);
    }

    #[test]
    fn open_endedness_is_not_artifact_count() {
        let table = derive_cycle_health(&diagnostic()).expect("cycle");
        let runaway = table
            .rows
            .iter()
            .find(|row| row.trace_id == "runaway")
            .expect("row");
        assert_eq!(
            runaway.open_endedness,
            Some(crate::normalize(crate::RationalValue {
                numerator: 0,
                denominator: 20,
            }))
        );
        assert_ne!(
            runaway.open_endedness.as_ref().map(|value| value.numerator),
            Some(i64::from(20))
        );
    }

    #[test]
    fn committed_cycle_classes_match_fixture() {
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/cycle-health/v1/expected-outcomes.json"
        ))
        .expect("fixture");
        let table = derive_cycle_health(&diagnostic()).expect("cycle");
        let classes = table
            .rows
            .iter()
            .map(|row| {
                (
                    row.trace_id.clone(),
                    serde_json::to_value(row.class).expect("class"),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            serde_json::json!({"schema":"bonsai.cycle-health-outcomes/v1","classes":classes}),
            expected
        );
    }
}
