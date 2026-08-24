//! Concrete C4 and C5 adjudication. Pass is not required for M3 close.

use crate::ClaimVerdict;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(clippy::struct_excessive_bools)]
#[serde(deny_unknown_fields)]
pub struct CycleEvidence {
    pub case_id: String,
    pub c3: ClaimVerdict,
    pub families: u64,
    pub forward_construction: bool,
    pub backward_credit: bool,
    pub one_score_only: bool,
    pub one_seed_only: bool,
    pub one_family_only: bool,
    pub artifact_count_only: bool,
    pub scaled_compute: bool,
    pub multiplicity_corrected: bool,
    pub comparator_mixed: bool,
    pub uncontrolled_growth: bool,
    pub energy_tier: u8,
    pub generations: u64,
    pub net_gain: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct C4C5Row {
    pub case_id: String,
    pub c4: ClaimVerdict,
    pub c5: ClaimVerdict,
    pub c4_detail: Option<String>,
    pub c5_detail: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct C4C5Table {
    pub schema: String,
    pub rows: Vec<C4C5Row>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CycleError {
    Identity,
    Trace,
}

impl fmt::Display for CycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "CLAIM_CYCLE_IDENTITY_INVALID",
            Self::Trace => "CLAIM_CYCLE_TRACE_INVALID",
        })
    }
}

impl Error for CycleError {}

const C4_MIN_FAMILIES: u64 = 3;
const C5_MIN_GENERATIONS: u64 = 2;
const C5_MIN_ENERGY: u8 = 2;

/// Adjudicate C4 construction/credit and C5 multigenerational gain.
///
/// Missing energy, uncorrected multiplicity, comparator mixing, and
/// uncontrolled growth cannot pass. M3 does not require a pass verdict.
///
/// # Errors
///
/// Rejects empty identities.
pub fn adjudicate_c4_c5(records: &[CycleEvidence]) -> Result<C4C5Table, CycleError> {
    if records.is_empty() {
        return Err(CycleError::Trace);
    }
    let mut ids = BTreeSet::new();
    let mut rows = Vec::new();
    for record in records {
        if record.case_id.is_empty() || !ids.insert(record.case_id.as_str()) {
            return Err(CycleError::Identity);
        }
        let (c4, c4_detail) = c4_verdict(record);
        let (c5, c5_detail) = c5_verdict(record, c4);
        rows.push(C4C5Row {
            case_id: record.case_id.clone(),
            c4,
            c5,
            c4_detail,
            c5_detail,
        });
    }
    rows.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    Ok(C4C5Table {
        schema: "bonsai.c4-c5-table/v1".to_owned(),
        rows,
    })
}

fn c4_verdict(record: &CycleEvidence) -> (ClaimVerdict, Option<String>) {
    if record.c3 != ClaimVerdict::Pass {
        return (
            ClaimVerdict::Indeterminate,
            Some("C4_PREREQUISITE_C3".to_owned()),
        );
    }
    if record.one_score_only
        || record.one_seed_only
        || record.one_family_only
        || record.artifact_count_only
        || record.scaled_compute
    {
        return (ClaimVerdict::Fail, Some("C4_SHORTCUT".to_owned()));
    }
    if record.comparator_mixed {
        return (ClaimVerdict::Fail, Some("C4_COMPARATOR_MIXED".to_owned()));
    }
    if !record.multiplicity_corrected {
        return (
            ClaimVerdict::Fail,
            Some("C4_MULTIPLICITY_UNCORRECTED".to_owned()),
        );
    }
    if record.families < C4_MIN_FAMILIES || !record.forward_construction || !record.backward_credit
    {
        return (
            ClaimVerdict::Indeterminate,
            Some("C4_FAMILY_OR_CREDIT_UNAVAILABLE".to_owned()),
        );
    }
    (ClaimVerdict::Pass, None)
}

fn c5_verdict(record: &CycleEvidence, c4: ClaimVerdict) -> (ClaimVerdict, Option<String>) {
    if c4 != ClaimVerdict::Pass {
        return (
            ClaimVerdict::Indeterminate,
            Some("C5_PREREQUISITE_C4".to_owned()),
        );
    }
    if record.uncontrolled_growth {
        return (
            ClaimVerdict::Fail,
            Some("C5_UNCONTROLLED_GROWTH".to_owned()),
        );
    }
    if record.energy_tier < C5_MIN_ENERGY {
        return (ClaimVerdict::Fail, Some("C5_ENERGY_BELOW_E2".to_owned()));
    }
    if record.generations < C5_MIN_GENERATIONS {
        return (
            ClaimVerdict::Indeterminate,
            Some("C5_GENERATIONS_UNAVAILABLE".to_owned()),
        );
    }
    match record.net_gain {
        None => (
            ClaimVerdict::Indeterminate,
            Some("C5_GAIN_UNAVAILABLE".to_owned()),
        ),
        Some(gain) if gain > 0 => (ClaimVerdict::Pass, None),
        Some(_) => (ClaimVerdict::Fail, Some("C5_GAIN_NONPOSITIVE".to_owned())),
    }
}
