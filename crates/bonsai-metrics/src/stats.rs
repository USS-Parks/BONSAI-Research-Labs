//! Paired statistical aggregation with Holm adjustment and undeclared-exclusion detection.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

const LCG_A: u64 = 1_664_525;
const LCG_C: u64 = 1_013_904_223;
const LCG_MASK: u64 = 0xFFFF_FFFF;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SeedPair {
    pub pair_id: String,
    pub outcome_a: Option<i64>,
    pub outcome_b: Option<i64>,
    pub failed_a: bool,
    pub failed_b: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeFamily {
    pub outcome_id: String,
    pub pairs: Vec<SeedPair>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StatisticalPlan {
    pub preregistered_outcomes: Vec<String>,
    pub declared_pairs: Vec<String>,
    pub declared_exclusions: Vec<String>,
    pub bootstrap_resamples: u64,
    pub bootstrap_seed: u64,
    pub alpha_numerator: i64,
    pub alpha_denominator: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeRow {
    pub outcome_id: String,
    pub complete_pairs: u64,
    pub missing_runs: u64,
    pub failed_runs: u64,
    pub excluded_runs: u64,
    pub effect_numerator: Option<i64>,
    pub effect_denominator: Option<u64>,
    pub ci_low_numerator: Option<i64>,
    pub ci_high_numerator: Option<i64>,
    pub ci_denominator: Option<u64>,
    pub p_numerator: Option<u64>,
    pub p_denominator: Option<u64>,
    pub holm_rejected: bool,
    pub sensitivity_min_numerator: Option<i64>,
    pub sensitivity_max_numerator: Option<i64>,
    pub sensitivity_denominator: Option<u64>,
    pub detail_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StatisticalTable {
    pub schema: String,
    pub rows: Vec<OutcomeRow>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatError {
    Plan,
    UndeclaredExclusion,
    UndeclaredMetric,
}

impl fmt::Display for StatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Plan => "STAT_PLAN_INVALID",
            Self::UndeclaredExclusion => "STAT_UNDECLARED_EXCLUSION",
            Self::UndeclaredMetric => "STAT_UNDECLARED_METRIC",
        })
    }
}

impl Error for StatError {}

/// Aggregate preregistered paired outcomes.
///
/// Undeclared pair or metric omissions fail closed. Missing and failed runs
/// are reported and never silently dropped.
///
/// # Errors
///
/// Rejects an invalid plan, undeclared exclusions, or undeclared metrics.
pub fn aggregate(
    plan: &StatisticalPlan,
    families: &[OutcomeFamily],
) -> Result<StatisticalTable, StatError> {
    validate_plan(plan)?;
    let declared = plan.declared_pairs.iter().cloned().collect::<BTreeSet<_>>();
    let excluded = plan
        .declared_exclusions
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if !declared.is_disjoint(&excluded) {
        return Err(StatError::Plan);
    }
    let preregistered = plan
        .preregistered_outcomes
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut rows = Vec::new();
    let mut p_values = Vec::new();
    for family in families {
        if !preregistered.contains(&family.outcome_id) {
            return Err(StatError::UndeclaredMetric);
        }
        let completed = complete_diffs(&family.pairs, &declared, &excluded)?;
        if completed.diffs.is_empty() {
            rows.push(OutcomeRow {
                outcome_id: family.outcome_id.clone(),
                complete_pairs: 0,
                missing_runs: completed.missing,
                failed_runs: completed.failed,
                excluded_runs: completed.excluded,
                effect_numerator: None,
                effect_denominator: None,
                ci_low_numerator: None,
                ci_high_numerator: None,
                ci_denominator: None,
                p_numerator: None,
                p_denominator: None,
                holm_rejected: false,
                sensitivity_min_numerator: None,
                sensitivity_max_numerator: None,
                sensitivity_denominator: None,
                detail_code: Some("STAT_PAIRS_UNAVAILABLE".to_owned()),
            });
            continue;
        }
        let n = u64::try_from(completed.diffs.len()).unwrap_or(u64::MAX);
        let effect = completed.diffs.iter().sum::<i64>();
        let (ci_low, ci_high) = bootstrap_ci(
            &completed.diffs,
            plan.bootstrap_resamples,
            plan.bootstrap_seed,
        );
        let (p_num, p_den) = bootstrap_p(
            &completed.diffs,
            plan.bootstrap_resamples,
            plan.bootstrap_seed,
        );
        let (sens_min, sens_max, sens_den) = sensitivity(&completed.diffs);
        rows.push(OutcomeRow {
            outcome_id: family.outcome_id.clone(),
            complete_pairs: n,
            missing_runs: completed.missing,
            failed_runs: completed.failed,
            excluded_runs: completed.excluded,
            effect_numerator: Some(effect),
            effect_denominator: Some(n),
            ci_low_numerator: Some(ci_low),
            ci_high_numerator: Some(ci_high),
            ci_denominator: Some(n),
            p_numerator: Some(p_num),
            p_denominator: Some(p_den),
            holm_rejected: false,
            sensitivity_min_numerator: sens_min,
            sensitivity_max_numerator: sens_max,
            sensitivity_denominator: sens_den,
            detail_code: None,
        });
        p_values.push((family.outcome_id.clone(), p_num, p_den));
    }
    let rejected = holm(&p_values, plan.alpha_numerator, plan.alpha_denominator);
    for row in &mut rows {
        row.holm_rejected = rejected.contains(&row.outcome_id);
    }
    rows.sort_by(|left, right| left.outcome_id.cmp(&right.outcome_id));
    Ok(StatisticalTable {
        schema: "bonsai.statistical-table/v1".to_owned(),
        rows,
    })
}

struct Completed {
    diffs: Vec<i64>,
    missing: u64,
    failed: u64,
    excluded: u64,
}

fn validate_plan(plan: &StatisticalPlan) -> Result<(), StatError> {
    if plan.preregistered_outcomes.is_empty()
        || plan.declared_pairs.is_empty()
        || plan.bootstrap_resamples < 2
        || plan.alpha_numerator <= 0
        || plan.alpha_denominator == 0
        || plan
            .preregistered_outcomes
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != plan.preregistered_outcomes.len()
        || plan.declared_pairs.iter().collect::<BTreeSet<_>>().len() != plan.declared_pairs.len()
        || plan
            .declared_exclusions
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != plan.declared_exclusions.len()
    {
        return Err(StatError::Plan);
    }
    Ok(())
}

fn complete_diffs(
    pairs: &[SeedPair],
    declared: &BTreeSet<String>,
    excluded: &BTreeSet<String>,
) -> Result<Completed, StatError> {
    let mut ordered = pairs.to_vec();
    ordered.sort_by(|left, right| left.pair_id.cmp(&right.pair_id));
    let mut diffs = Vec::new();
    let mut missing = 0;
    let mut failed = 0;
    let mut excluded_runs = 0;
    for pair in ordered {
        if !declared.contains(&pair.pair_id) && !excluded.contains(&pair.pair_id) {
            return Err(StatError::UndeclaredExclusion);
        }
        if excluded.contains(&pair.pair_id) {
            excluded_runs += 1;
            continue;
        }
        if pair.failed_a || pair.failed_b {
            failed += 1;
            continue;
        }
        match (pair.outcome_a, pair.outcome_b) {
            (Some(left), Some(right)) => diffs.push(left.saturating_sub(right)),
            _ => missing += 1,
        }
    }
    Ok(Completed {
        diffs,
        missing,
        failed,
        excluded: excluded_runs,
    })
}

fn lcg(state: u64) -> u64 {
    state.wrapping_mul(LCG_A).wrapping_add(LCG_C) & LCG_MASK
}

fn bootstrap_sums(diffs: &[i64], resamples: u64, seed: u64) -> Vec<i64> {
    let mut state = seed & LCG_MASK;
    let n = diffs.len();
    let mut sums = Vec::new();
    for _ in 0..resamples {
        let mut total = 0_i64;
        for _ in 0..n {
            state = lcg(state);
            let index = usize::try_from(state % u64::try_from(n).unwrap_or(1)).unwrap_or(0);
            total = total.saturating_add(diffs[index]);
        }
        sums.push(total);
    }
    sums
}

fn bootstrap_ci(diffs: &[i64], resamples: u64, seed: u64) -> (i64, i64) {
    let mut sums = bootstrap_sums(diffs, resamples, seed);
    sums.sort_unstable();
    let last = resamples.saturating_sub(1);
    let low = usize::try_from((25 * last) / 1000).unwrap_or(0);
    let high = usize::try_from((975 * last) / 1000).unwrap_or(sums.len().saturating_sub(1));
    (sums[low], sums[high])
}

fn bootstrap_p(diffs: &[i64], resamples: u64, seed: u64) -> (u64, u64) {
    let observed = diffs.iter().sum::<i64>();
    let count = bootstrap_sums(diffs, resamples, seed)
        .into_iter()
        .filter(|total| {
            observed == 0 || (observed > 0 && *total <= 0) || (observed < 0 && *total >= 0)
        })
        .count();
    (
        u64::try_from(count).unwrap_or(u64::MAX).saturating_add(1),
        resamples.saturating_add(1),
    )
}

fn sensitivity(diffs: &[i64]) -> (Option<i64>, Option<i64>, Option<u64>) {
    if diffs.len() < 2 {
        return (None, None, None);
    }
    let total = diffs.iter().sum::<i64>();
    let leave = diffs
        .iter()
        .map(|value| total.saturating_sub(*value))
        .collect::<Vec<_>>();
    (
        leave.iter().copied().min(),
        leave.iter().copied().max(),
        Some(u64::try_from(diffs.len().saturating_sub(1)).unwrap_or(u64::MAX)),
    )
}

fn holm(p_values: &[(String, u64, u64)], alpha_num: i64, alpha_den: u64) -> BTreeSet<String> {
    let mut ordered = p_values.to_vec();
    ordered.sort_by(|left, right| {
        let left_key = left.1.saturating_mul(10_000) / left.2.max(1);
        let right_key = right.1.saturating_mul(10_000) / right.2.max(1);
        left_key.cmp(&right_key).then_with(|| left.0.cmp(&right.0))
    });
    let count = ordered.len();
    let mut rejected = BTreeSet::new();
    for (index, (outcome_id, p_num, p_den)) in ordered.into_iter().enumerate() {
        let remaining = u64::try_from(count.saturating_sub(index)).unwrap_or(1);
        let left = p_num.saturating_mul(alpha_den).saturating_mul(remaining);
        let right = p_den.saturating_mul(u64::try_from(alpha_num).unwrap_or(0));
        if left <= right {
            rejected.insert(outcome_id);
        } else {
            break;
        }
    }
    rejected
}

#[cfg(test)]
mod tests {
    use super::{OutcomeFamily, SeedPair, StatError, StatisticalPlan, aggregate};
    use std::collections::BTreeMap;

    fn diagnostic_plan() -> StatisticalPlan {
        StatisticalPlan {
            preregistered_outcomes: vec!["latency".to_owned(), "reward".to_owned()],
            declared_pairs: vec![
                "p1".to_owned(),
                "p2".to_owned(),
                "p3".to_owned(),
                "p4".to_owned(),
            ],
            declared_exclusions: vec!["p_skip".to_owned()],
            bootstrap_resamples: 20,
            bootstrap_seed: 1,
            alpha_numerator: 5,
            alpha_denominator: 100,
        }
    }

    fn pair(
        id: &str,
        left: Option<i64>,
        right: Option<i64>,
        failed_a: bool,
        failed_b: bool,
    ) -> SeedPair {
        SeedPair {
            pair_id: id.to_owned(),
            outcome_a: left,
            outcome_b: right,
            failed_a,
            failed_b,
        }
    }

    fn diagnostic_families() -> Vec<OutcomeFamily> {
        vec![
            OutcomeFamily {
                outcome_id: "latency".to_owned(),
                pairs: vec![
                    pair("p1", Some(3), Some(4), false, false),
                    pair("p2", Some(2), Some(5), false, false),
                    pair("p3", Some(1), Some(6), false, false),
                    pair("p4", None, Some(2), true, false),
                    pair("p_skip", Some(0), Some(50), false, false),
                ],
            },
            OutcomeFamily {
                outcome_id: "reward".to_owned(),
                pairs: vec![
                    pair("p1", Some(10), Some(6), false, false),
                    pair("p2", Some(8), Some(7), false, false),
                    pair("p3", Some(12), Some(5), false, false),
                    pair("p4", Some(9), None, false, true),
                    pair("p_skip", Some(100), Some(0), false, false),
                ],
            },
        ]
    }

    #[test]
    fn undeclared_exclusion_fails_closed() {
        let mut families = diagnostic_families();
        families[0]
            .pairs
            .push(pair("sneak", Some(1), Some(0), false, false));
        assert_eq!(
            aggregate(&diagnostic_plan(), &families).err(),
            Some(StatError::UndeclaredExclusion)
        );
    }

    #[test]
    fn undeclared_metric_fails_closed() {
        let mut families = diagnostic_families();
        families.push(OutcomeFamily {
            outcome_id: "posthoc".to_owned(),
            pairs: vec![pair("p1", Some(1), Some(0), false, false)],
        });
        assert_eq!(
            aggregate(&diagnostic_plan(), &families).err(),
            Some(StatError::UndeclaredMetric)
        );
    }

    #[test]
    fn committed_corpus_matches_independent_python_reference() {
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/statistical-aggregation/v1/expected-outcomes.json"
        ))
        .expect("fixture");
        let table = aggregate(&diagnostic_plan(), &diagnostic_families()).expect("stats");
        let rows = table
            .rows
            .iter()
            .map(|row| {
                (
                    row.outcome_id.clone(),
                    serde_json::to_value(row).expect("row"),
                )
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            serde_json::json!({"schema":"bonsai.statistical-outcomes/v1","rows":rows}),
            expected
        );
    }
}
