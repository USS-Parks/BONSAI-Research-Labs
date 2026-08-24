//! Concrete C2 and C3 adjudication under D-15 and D-20 rules.

use crate::ClaimVerdict;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(clippy::struct_excessive_bools)]
#[serde(deny_unknown_fields)]
pub struct AdaptationEvidence {
    pub case_id: String,
    pub c1: ClaimVerdict,
    pub first_phase_return: Option<i64>,
    pub return_phase_return: Option<i64>,
    pub final_score_only: bool,
    pub paired_seeds: u64,
    pub holm_rejected: bool,
    pub effect_positive: bool,
    pub exact_or_matched_utility: bool,
    pub proxy_only: bool,
    pub declared_utility: Option<i64>,
    pub c3_eligible: bool,
    pub track_leakage: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct C2C3Row {
    pub case_id: String,
    pub c2: ClaimVerdict,
    pub c3: ClaimVerdict,
    pub c2_detail: Option<String>,
    pub c3_detail: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct C2C3Table {
    pub schema: String,
    pub rows: Vec<C2C3Row>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdaptationError {
    Identity,
    Trace,
}

impl fmt::Display for AdaptationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "CLAIM_ADAPTATION_IDENTITY_INVALID",
            Self::Trace => "CLAIM_ADAPTATION_TRACE_INVALID",
        })
    }
}

impl Error for AdaptationError {}

const C2_MIN_PAIRED_SEEDS: u64 = 20;

/// Adjudicate C2 continual adaptation and C3 abstraction utility.
///
/// Proxy-only utility, final-score-only adaptation, and comparator-track
/// leakage cannot produce a pass. Missing prerequisites stay indeterminate.
///
/// # Errors
///
/// Rejects empty identities.
pub fn adjudicate_c2_c3(records: &[AdaptationEvidence]) -> Result<C2C3Table, AdaptationError> {
    if records.is_empty() {
        return Err(AdaptationError::Trace);
    }
    let mut ids = BTreeSet::new();
    let mut rows = Vec::new();
    for record in records {
        if record.case_id.is_empty() || !ids.insert(record.case_id.as_str()) {
            return Err(AdaptationError::Identity);
        }
        let (c2, c2_detail) = c2_verdict(record);
        let (c3, c3_detail) = c3_verdict(record, c2);
        rows.push(C2C3Row {
            case_id: record.case_id.clone(),
            c2,
            c3,
            c2_detail,
            c3_detail,
        });
    }
    rows.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    Ok(C2C3Table {
        schema: "bonsai.c2-c3-table/v1".to_owned(),
        rows,
    })
}

fn c2_verdict(record: &AdaptationEvidence) -> (ClaimVerdict, Option<String>) {
    if record.c1 != ClaimVerdict::Pass {
        return (
            ClaimVerdict::Indeterminate,
            Some("C2_PREREQUISITE_C1".to_owned()),
        );
    }
    if record.final_score_only {
        return (ClaimVerdict::Fail, Some("C2_FINAL_SCORE_ONLY".to_owned()));
    }
    if record.first_phase_return.is_none() || record.return_phase_return.is_none() {
        return (
            ClaimVerdict::Indeterminate,
            Some("C2_PHASE_UNAVAILABLE".to_owned()),
        );
    }
    if record.paired_seeds < C2_MIN_PAIRED_SEEDS {
        return (
            ClaimVerdict::Indeterminate,
            Some("C2_SEED_COUNT".to_owned()),
        );
    }
    if record.holm_rejected && record.effect_positive {
        (ClaimVerdict::Pass, None)
    } else {
        (ClaimVerdict::Fail, Some("C2_ADAPTATION_ABSENT".to_owned()))
    }
}

fn c3_verdict(record: &AdaptationEvidence, c2: ClaimVerdict) -> (ClaimVerdict, Option<String>) {
    if c2 != ClaimVerdict::Pass {
        return (
            ClaimVerdict::Indeterminate,
            Some("C3_PREREQUISITE_C2".to_owned()),
        );
    }
    if record.track_leakage {
        return (ClaimVerdict::Fail, Some("C3_TRACK_LEAKAGE".to_owned()));
    }
    if record.proxy_only || !record.exact_or_matched_utility || !record.c3_eligible {
        return (ClaimVerdict::Fail, Some("C3_PROXY_ONLY".to_owned()));
    }
    match record.declared_utility {
        None => (
            ClaimVerdict::Indeterminate,
            Some("C3_UTILITY_UNAVAILABLE".to_owned()),
        ),
        Some(utility) if utility > 0 => (ClaimVerdict::Pass, None),
        Some(_) => (
            ClaimVerdict::Fail,
            Some("C3_UTILITY_NONPOSITIVE".to_owned()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{AdaptationEvidence, ClaimVerdict, adjudicate_c2_c3};

    #[allow(clippy::fn_params_excessive_bools, clippy::too_many_arguments)]
    fn record(
        id: &str,
        c1: ClaimVerdict,
        first: Option<i64>,
        ret: Option<i64>,
        final_only: bool,
        seeds: u64,
        holm: bool,
        effect: bool,
        exact: bool,
        proxy: bool,
        utility: Option<i64>,
        eligible: bool,
        leak: bool,
    ) -> AdaptationEvidence {
        AdaptationEvidence {
            case_id: id.to_owned(),
            c1,
            first_phase_return: first,
            return_phase_return: ret,
            final_score_only: final_only,
            paired_seeds: seeds,
            holm_rejected: holm,
            effect_positive: effect,
            exact_or_matched_utility: exact,
            proxy_only: proxy,
            declared_utility: utility,
            c3_eligible: eligible,
            track_leakage: leak,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn corpus() -> Vec<AdaptationEvidence> {
        vec![
            record(
                "pass",
                ClaimVerdict::Pass,
                Some(4),
                Some(5),
                false,
                20,
                true,
                true,
                true,
                false,
                Some(3),
                true,
                false,
            ),
            record(
                "c1_missing",
                ClaimVerdict::Indeterminate,
                Some(4),
                Some(5),
                false,
                20,
                true,
                true,
                true,
                false,
                Some(3),
                true,
                false,
            ),
            record(
                "final_score_only",
                ClaimVerdict::Pass,
                Some(4),
                Some(5),
                true,
                20,
                true,
                true,
                true,
                false,
                Some(3),
                true,
                false,
            ),
            record(
                "phase_unavailable",
                ClaimVerdict::Pass,
                Some(4),
                None,
                false,
                20,
                true,
                true,
                true,
                false,
                Some(3),
                true,
                false,
            ),
            record(
                "proxy_only",
                ClaimVerdict::Pass,
                Some(4),
                Some(5),
                false,
                20,
                true,
                true,
                false,
                true,
                Some(3),
                false,
                false,
            ),
            record(
                "track_leakage",
                ClaimVerdict::Pass,
                Some(4),
                Some(5),
                false,
                20,
                true,
                true,
                true,
                false,
                Some(3),
                true,
                true,
            ),
            record(
                "utility_unavailable",
                ClaimVerdict::Pass,
                Some(4),
                Some(5),
                false,
                20,
                true,
                true,
                true,
                false,
                None,
                true,
                false,
            ),
        ]
    }

    #[test]
    fn every_prerequisite_has_pass_fail_and_indeterminate() {
        let table = adjudicate_c2_c3(&corpus()).expect("claims");
        let row = |id: &str| {
            table
                .rows
                .iter()
                .find(|row| row.case_id == id)
                .expect("row")
        };
        assert_eq!(row("pass").c2, ClaimVerdict::Pass);
        assert_eq!(row("pass").c3, ClaimVerdict::Pass);
        assert_eq!(row("c1_missing").c2, ClaimVerdict::Indeterminate);
        assert_eq!(row("final_score_only").c2, ClaimVerdict::Fail);
        assert_eq!(row("phase_unavailable").c2, ClaimVerdict::Indeterminate);
        assert_eq!(row("proxy_only").c3, ClaimVerdict::Fail);
        assert_eq!(row("track_leakage").c3, ClaimVerdict::Fail);
        assert_eq!(row("utility_unavailable").c3, ClaimVerdict::Indeterminate);
        assert_ne!(row("proxy_only").c3, ClaimVerdict::Pass);
    }

    #[test]
    fn committed_c2_c3_verdicts_match_fixture() {
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/c2-c3-adjudication/v1/expected-outcomes.json"
        ))
        .expect("fixture");
        let table = adjudicate_c2_c3(&corpus()).expect("claims");
        let rows = table
            .rows
            .iter()
            .map(|row| {
                (
                    row.case_id.clone(),
                    serde_json::json!({
                        "c2": row.c2,
                        "c3": row.c3,
                        "c2_detail": row.c2_detail,
                        "c3_detail": row.c3_detail,
                    }),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            serde_json::json!({"schema":"bonsai.c2-c3-outcomes/v1","rows":rows}),
            expected
        );
    }
}
