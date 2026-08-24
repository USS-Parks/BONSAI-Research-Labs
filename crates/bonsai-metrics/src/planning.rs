//! Planning metrics and D-19 consequential-backup test.

use crate::RationalValue;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

/// Exact versus labeled approximate influence.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupMethod {
    Exact,
    Approximate,
}

/// One paired backup: observed trajectory versus omit-one counterfactual.
///
/// The pair shares `state_id` and `random_draw`. Value change is recorded but
/// is never sufficient for consequentiality.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BackupRecord {
    pub backup_id: String,
    pub state_id: String,
    pub random_draw: i64,
    pub value_delta: i64,
    pub realized_policy: BTreeMap<String, i64>,
    pub realized_action: String,
    pub counterfactual_policy: BTreeMap<String, i64>,
    pub counterfactual_action: String,
    pub method: BackupMethod,
    pub approximate_influence: Option<i64>,
}

/// One planning episode with search-control and backup fixtures.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlanningTrace {
    pub updates: u64,
    pub states_visited: u64,
    pub options_considered: u64,
    pub search_control_ops: u64,
    pub value_gain: i64,
    pub realized_agreement_hits: u64,
    pub realized_agreement_trials: u64,
    pub primitive_time_depth: u64,
    pub latency: u64,
    pub backups_saved: u64,
    pub exploitation_failure: bool,
    pub unused_plans: u64,
    pub epsilon: i64,
    pub backups: Vec<BackupRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BackupRow {
    pub backup_id: String,
    pub value_changed: bool,
    pub policy_l1: i64,
    pub action_changed: bool,
    pub consequential: bool,
    pub method: BackupMethod,
    pub approximation_error: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlanningMetricTable {
    pub schema: String,
    pub updates: u64,
    pub states_visited: u64,
    pub options_considered: u64,
    pub search_control_ops: u64,
    pub value_gain_per_operation: Option<RationalValue>,
    pub realized_agreement: Option<RationalValue>,
    pub primitive_time_depth: u64,
    pub latency: u64,
    pub backups_saved: u64,
    pub exploitation_failure: bool,
    pub unused_plans: u64,
    pub backups: Vec<BackupRow>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanningMetricError {
    Identity,
    Trace,
    Arithmetic,
}

impl fmt::Display for PlanningMetricError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "PLANNING_IDENTITY_INVALID",
            Self::Trace => "PLANNING_TRACE_INVALID",
            Self::Arithmetic => "PLANNING_METRIC_ARITHMETIC_FAILED",
        })
    }
}

impl Error for PlanningMetricError {}

/// Derive planning metrics and D-19 consequentiality.
///
/// A backup is consequential only when the omit-one paired counterfactual
/// changes later policy mass by more than `epsilon` or changes an action.
/// Value change is recorded and is not sufficient.
///
/// # Errors
///
/// Rejects empty identities, unpaired state/draw fields, or overflow.
pub fn derive_planning_metrics(
    trace: &PlanningTrace,
) -> Result<PlanningMetricTable, PlanningMetricError> {
    validate(trace)?;
    let operations = trace
        .updates
        .saturating_add(trace.search_control_ops)
        .max(1);
    let mut rows = Vec::new();
    for backup in &trace.backups {
        let policy_l1 = policy_l1(&backup.realized_policy, &backup.counterfactual_policy);
        let action_changed = backup.realized_action != backup.counterfactual_action;
        let consequential = policy_l1 > trace.epsilon || action_changed;
        let approximation_error = match backup.method {
            BackupMethod::Exact => None,
            BackupMethod::Approximate => {
                let predicted = backup.approximate_influence.unwrap_or(0);
                let exact = i64::from(consequential);
                Some(predicted.abs_diff(exact).try_into().unwrap_or(i64::MAX))
            }
        };
        rows.push(BackupRow {
            backup_id: backup.backup_id.clone(),
            value_changed: backup.value_delta != 0,
            policy_l1,
            action_changed,
            consequential,
            method: backup.method,
            approximation_error,
        });
    }
    rows.sort_by(|left, right| left.backup_id.cmp(&right.backup_id));
    Ok(PlanningMetricTable {
        schema: "bonsai.planning-metric-table/v1".to_owned(),
        updates: trace.updates,
        states_visited: trace.states_visited,
        options_considered: trace.options_considered,
        search_control_ops: trace.search_control_ops,
        value_gain_per_operation: Some(crate::normalize(RationalValue {
            numerator: trace.value_gain,
            denominator: operations,
        })),
        realized_agreement: ratio(
            trace.realized_agreement_hits,
            trace.realized_agreement_trials,
        )?,
        primitive_time_depth: trace.primitive_time_depth,
        latency: trace.latency,
        backups_saved: trace.backups_saved,
        exploitation_failure: trace.exploitation_failure,
        unused_plans: trace.unused_plans,
        backups: rows,
    })
}

fn validate(trace: &PlanningTrace) -> Result<(), PlanningMetricError> {
    if trace.epsilon < 0
        || trace.realized_agreement_hits > trace.realized_agreement_trials
        || trace.backups.is_empty()
    {
        return Err(PlanningMetricError::Trace);
    }
    let mut ids = BTreeSet::new();
    for backup in &trace.backups {
        if backup.backup_id.is_empty()
            || backup.state_id.is_empty()
            || backup.realized_action.is_empty()
            || backup.counterfactual_action.is_empty()
            || !ids.insert(backup.backup_id.as_str())
        {
            return Err(PlanningMetricError::Identity);
        }
        if backup.method == BackupMethod::Approximate && backup.approximate_influence.is_none() {
            return Err(PlanningMetricError::Trace);
        }
    }
    Ok(())
}

fn policy_l1(left: &BTreeMap<String, i64>, right: &BTreeMap<String, i64>) -> i64 {
    let mut keys = BTreeSet::new();
    keys.extend(left.keys());
    keys.extend(right.keys());
    keys.into_iter()
        .map(|key| {
            left.get(key)
                .copied()
                .unwrap_or(0)
                .abs_diff(right.get(key).copied().unwrap_or(0))
        })
        .fold(0_i64, |total, delta| {
            total.saturating_add(i64::try_from(delta).unwrap_or(i64::MAX))
        })
}

fn ratio(hits: u64, trials: u64) -> Result<Option<RationalValue>, PlanningMetricError> {
    if trials == 0 {
        return Ok(None);
    }
    Ok(Some(crate::normalize(RationalValue {
        numerator: i64::try_from(hits).map_err(|_| PlanningMetricError::Arithmetic)?,
        denominator: trials,
    })))
}

#[cfg(test)]
mod tests {
    use super::{BackupMethod, BackupRecord, PlanningTrace, derive_planning_metrics};
    use std::collections::BTreeMap;

    fn policy(pairs: &[(&str, i64)]) -> BTreeMap<String, i64> {
        pairs
            .iter()
            .map(|(action, mass)| ((*action).to_owned(), *mass))
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    fn backup(
        id: &str,
        value_delta: i64,
        realized: &[(&str, i64)],
        realized_action: &str,
        counterfactual: &[(&str, i64)],
        counterfactual_action: &str,
        method: BackupMethod,
        approximate_influence: Option<i64>,
    ) -> BackupRecord {
        BackupRecord {
            backup_id: id.to_owned(),
            state_id: "s0".to_owned(),
            random_draw: 7,
            value_delta,
            realized_policy: policy(realized),
            realized_action: realized_action.to_owned(),
            counterfactual_policy: policy(counterfactual),
            counterfactual_action: counterfactual_action.to_owned(),
            method,
            approximate_influence,
        }
    }

    fn diagnostic_trace() -> PlanningTrace {
        PlanningTrace {
            updates: 4,
            states_visited: 3,
            options_considered: 2,
            search_control_ops: 1,
            value_gain: 10,
            realized_agreement_hits: 3,
            realized_agreement_trials: 4,
            primitive_time_depth: 6,
            latency: 9,
            backups_saved: 1,
            exploitation_failure: false,
            unused_plans: 1,
            epsilon: 2,
            backups: vec![
                backup(
                    "value_only",
                    8,
                    &[("left", 10), ("right", 0)],
                    "left",
                    &[("left", 10), ("right", 0)],
                    "left",
                    BackupMethod::Exact,
                    None,
                ),
                backup(
                    "policy_shift",
                    1,
                    &[("left", 8), ("right", 2)],
                    "left",
                    &[("left", 4), ("right", 6)],
                    "left",
                    BackupMethod::Exact,
                    None,
                ),
                backup(
                    "action_change",
                    0,
                    &[("left", 6), ("right", 4)],
                    "left",
                    &[("left", 6), ("right", 4)],
                    "right",
                    BackupMethod::Exact,
                    None,
                ),
                backup(
                    "approx_wrong",
                    5,
                    &[("left", 9), ("right", 1)],
                    "left",
                    &[("left", 8), ("right", 2)],
                    "left",
                    BackupMethod::Approximate,
                    Some(1),
                ),
            ],
        }
    }

    #[test]
    fn value_change_is_not_sufficient_for_consequentiality() {
        let table = derive_planning_metrics(&diagnostic_trace()).expect("metrics");
        let row = |id: &str| {
            table
                .backups
                .iter()
                .find(|row| row.backup_id == id)
                .expect("row")
        };
        assert!(row("value_only").value_changed);
        assert!(!row("value_only").consequential);
        assert!(row("policy_shift").consequential);
        assert!(row("action_change").consequential);
        assert!(!row("action_change").value_changed);
    }

    #[test]
    fn approximate_influence_reports_error_against_exact_counterfactual() {
        let table = derive_planning_metrics(&diagnostic_trace()).expect("metrics");
        let approx = table
            .backups
            .iter()
            .find(|row| row.backup_id == "approx_wrong")
            .expect("row");
        assert!(!approx.consequential);
        assert_eq!(approx.approximation_error, Some(1));
    }

    #[test]
    fn committed_consequentiality_matches_fixture() {
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/planning-metrics/v1/expected-outcomes.json"
        ))
        .expect("fixture");
        let table = derive_planning_metrics(&diagnostic_trace()).expect("metrics");
        let rows = table
            .backups
            .iter()
            .map(|row| {
                (
                    row.backup_id.clone(),
                    serde_json::json!({
                        "consequential": row.consequential,
                        "value_changed": row.value_changed,
                        "approximation_error": row.approximation_error,
                    }),
                )
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            serde_json::json!({"schema":"bonsai.planning-metric-outcomes/v1","backups":rows}),
            expected
        );
    }
}
