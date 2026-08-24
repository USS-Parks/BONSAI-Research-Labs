//! Concrete C0 and C1 adjudication for controlled bundle fixtures.

use crate::ClaimVerdict;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

/// Controlled diagnostic class for a C0/C1 bundle.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BundleClass {
    Compliant,
    SoftDegraded,
    HardViolating,
    Unavailable,
    Tampered,
    AmbiguousTrack,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(clippy::struct_excessive_bools)]
#[serde(deny_unknown_fields)]
pub struct BundleEvidence {
    pub case_id: String,
    pub provenance_valid: bool,
    pub events_valid: bool,
    pub resource_evidence: bool,
    pub budget_compliant: bool,
    pub soft_degraded: bool,
    pub hard_violation: bool,
    pub hard_counter_declared: bool,
    pub hard_counter_available: bool,
    pub tampered: bool,
    pub track_ambiguous: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimRow {
    pub case_id: String,
    pub class: BundleClass,
    pub c0: ClaimVerdict,
    pub c1: ClaimVerdict,
    pub detail_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct C0C1Table {
    pub schema: String,
    pub rows: Vec<ClaimRow>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdjudicationError {
    Identity,
    Trace,
}

impl fmt::Display for AdjudicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "CLAIM_BUNDLE_IDENTITY_INVALID",
            Self::Trace => "CLAIM_BUNDLE_TRACE_INVALID",
        })
    }
}

impl Error for AdjudicationError {}

/// Adjudicate C0 provenance/event/resource evidence and C1 budget compliance.
///
/// C1 cannot pass when a declared hard counter is unavailable.
///
/// # Errors
///
/// Rejects empty identities.
pub fn adjudicate_c0_c1(bundles: &[BundleEvidence]) -> Result<C0C1Table, AdjudicationError> {
    if bundles.is_empty() {
        return Err(AdjudicationError::Trace);
    }
    let mut ids = BTreeSet::new();
    let mut rows = Vec::new();
    for bundle in bundles {
        if bundle.case_id.is_empty() || !ids.insert(bundle.case_id.as_str()) {
            return Err(AdjudicationError::Identity);
        }
        rows.push(evaluate(bundle));
    }
    rows.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    Ok(C0C1Table {
        schema: "bonsai.c0-c1-table/v1".to_owned(),
        rows,
    })
}

fn evaluate(bundle: &BundleEvidence) -> ClaimRow {
    let class = classify(bundle);
    let (c0, c0_detail) = c0_verdict(bundle);
    let (c1, c1_detail) = c1_verdict(bundle, c0);
    ClaimRow {
        case_id: bundle.case_id.clone(),
        class,
        c0,
        c1,
        detail_code: c1_detail.or(c0_detail),
    }
}

fn classify(bundle: &BundleEvidence) -> BundleClass {
    if bundle.tampered {
        BundleClass::Tampered
    } else if bundle.track_ambiguous {
        BundleClass::AmbiguousTrack
    } else if bundle.hard_violation {
        BundleClass::HardViolating
    } else if !bundle.resource_evidence
        || (bundle.hard_counter_declared && !bundle.hard_counter_available)
    {
        BundleClass::Unavailable
    } else if bundle.soft_degraded {
        BundleClass::SoftDegraded
    } else {
        BundleClass::Compliant
    }
}

fn c0_verdict(bundle: &BundleEvidence) -> (ClaimVerdict, Option<String>) {
    if bundle.tampered || !bundle.provenance_valid || !bundle.events_valid {
        (ClaimVerdict::Fail, Some("C0_EVIDENCE_INVALID".to_owned()))
    } else if !bundle.resource_evidence || bundle.track_ambiguous {
        (
            ClaimVerdict::Indeterminate,
            Some("C0_EVIDENCE_UNAVAILABLE".to_owned()),
        )
    } else {
        (ClaimVerdict::Pass, None)
    }
}

fn c1_verdict(bundle: &BundleEvidence, c0: ClaimVerdict) -> (ClaimVerdict, Option<String>) {
    if c0 != ClaimVerdict::Pass {
        return (ClaimVerdict::Indeterminate.min_fail(c0), None);
    }
    if bundle.hard_counter_declared && !bundle.hard_counter_available {
        return (
            ClaimVerdict::Indeterminate,
            Some("C1_HARD_COUNTER_UNAVAILABLE".to_owned()),
        );
    }
    if bundle.hard_violation || !bundle.budget_compliant {
        (ClaimVerdict::Fail, Some("C1_BUDGET_VIOLATION".to_owned()))
    } else {
        (ClaimVerdict::Pass, None)
    }
}

trait VerdictExt {
    fn min_fail(self, other: ClaimVerdict) -> ClaimVerdict;
}

impl VerdictExt for ClaimVerdict {
    fn min_fail(self, other: ClaimVerdict) -> ClaimVerdict {
        match (self, other) {
            (ClaimVerdict::Fail, _) | (_, ClaimVerdict::Fail) => ClaimVerdict::Fail,
            (ClaimVerdict::Indeterminate, _) | (_, ClaimVerdict::Indeterminate) => {
                ClaimVerdict::Indeterminate
            }
            _ => ClaimVerdict::Pass,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BundleEvidence, ClaimVerdict, adjudicate_c0_c1};

    #[allow(clippy::fn_params_excessive_bools, clippy::too_many_arguments)]
    fn bundle(
        id: &str,
        provenance: bool,
        events: bool,
        resource: bool,
        budget: bool,
        soft: bool,
        hard: bool,
        hard_declared: bool,
        hard_available: bool,
        tampered: bool,
        ambiguous: bool,
    ) -> BundleEvidence {
        BundleEvidence {
            case_id: id.to_owned(),
            provenance_valid: provenance,
            events_valid: events,
            resource_evidence: resource,
            budget_compliant: budget,
            soft_degraded: soft,
            hard_violation: hard,
            hard_counter_declared: hard_declared,
            hard_counter_available: hard_available,
            tampered,
            track_ambiguous: ambiguous,
        }
    }

    fn corpus() -> Vec<BundleEvidence> {
        vec![
            bundle(
                "compliant",
                true,
                true,
                true,
                true,
                false,
                false,
                true,
                true,
                false,
                false,
            ),
            bundle(
                "soft_degraded",
                true,
                true,
                true,
                true,
                true,
                false,
                true,
                true,
                false,
                false,
            ),
            bundle(
                "hard_violating",
                true,
                true,
                true,
                false,
                false,
                true,
                true,
                true,
                false,
                false,
            ),
            bundle(
                "unavailable",
                true,
                true,
                false,
                true,
                false,
                false,
                true,
                false,
                false,
                false,
            ),
            bundle(
                "tampered", false, false, true, true, false, false, true, true, true, false,
            ),
            bundle(
                "ambiguous_track",
                true,
                true,
                true,
                true,
                false,
                false,
                true,
                true,
                false,
                true,
            ),
        ]
    }

    #[test]
    fn six_bundle_classes_receive_exact_verdicts() {
        let table = adjudicate_c0_c1(&corpus()).expect("claims");
        let row = |id: &str| {
            table
                .rows
                .iter()
                .find(|row| row.case_id == id)
                .expect("row")
        };
        assert_eq!(row("compliant").c0, ClaimVerdict::Pass);
        assert_eq!(row("compliant").c1, ClaimVerdict::Pass);
        assert_eq!(row("soft_degraded").c1, ClaimVerdict::Pass);
        assert_eq!(row("hard_violating").c1, ClaimVerdict::Fail);
        assert_eq!(row("unavailable").c0, ClaimVerdict::Indeterminate);
        assert_eq!(row("unavailable").c1, ClaimVerdict::Indeterminate);
        assert_eq!(row("tampered").c0, ClaimVerdict::Fail);
        assert_eq!(row("ambiguous_track").c0, ClaimVerdict::Indeterminate);
    }

    #[test]
    fn c1_cannot_pass_when_declared_hard_counter_is_unavailable() {
        let table = adjudicate_c0_c1(&[bundle(
            "hard_counter",
            true,
            true,
            true,
            true,
            false,
            false,
            true,
            false,
            false,
            false,
        )])
        .expect("claims");
        assert_eq!(table.rows[0].c0, ClaimVerdict::Pass);
        assert_eq!(table.rows[0].c1, ClaimVerdict::Indeterminate);
        assert_eq!(
            table.rows[0].detail_code.as_deref(),
            Some("C1_HARD_COUNTER_UNAVAILABLE")
        );
    }

    #[test]
    fn committed_c0_c1_verdicts_match_fixture() {
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/c0-c1-adjudication/v1/expected-outcomes.json"
        ))
        .expect("fixture");
        let table = adjudicate_c0_c1(&corpus()).expect("claims");
        let rows = table
            .rows
            .iter()
            .map(|row| {
                (
                    row.case_id.clone(),
                    serde_json::json!({
                        "class": row.class,
                        "c0": row.c0,
                        "c1": row.c1,
                        "detail_code": row.detail_code,
                    }),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            serde_json::json!({"schema":"bonsai.c0-c1-outcomes/v1","rows":rows}),
            expected
        );
    }
}
