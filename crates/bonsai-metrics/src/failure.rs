//! Charter §14 failure-criteria detectors with ternary verdicts.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

/// One charter section 14 failure criterion.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureId {
    UsefulBehaviorNotExceedingControls,
    AbstractionCountWithoutUtility,
    OptionModelsIncreaseErrorWithoutGain,
    StaleCreditProtectsOld,
    FeaturePopulationCollapses,
    AdaptationDegradesToBaseline,
    HiddenReplayOrUndeclaredCompute,
    ResourceComplianceOmittedMeasurements,
    Unreproducible,
    OpenEndednessVanishesWhenHeldFixed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureVerdict {
    Pass,
    Fail,
    Indeterminate,
}

/// Observed window for one criterion. Missing required input is indeterminate.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FailureObservation {
    pub case_id: String,
    pub criterion: FailureId,
    pub window: u64,
    pub tolerance: i64,
    pub comparator_id: Option<String>,
    pub evidence_available: bool,
    pub primary: Option<i64>,
    pub secondary: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FailureRow {
    pub case_id: String,
    pub criterion: FailureId,
    pub window: u64,
    pub tolerance: i64,
    pub comparator_id: Option<String>,
    pub verdict: FailureVerdict,
    pub detail_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FailureTable {
    pub schema: String,
    pub rows: Vec<FailureRow>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureError {
    Identity,
    Trace,
}

impl fmt::Display for FailureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "FAILURE_IDENTITY_INVALID",
            Self::Trace => "FAILURE_TRACE_INVALID",
        })
    }
}

impl Error for FailureError {}

const ALL_CRITERIA: [FailureId; 10] = [
    FailureId::UsefulBehaviorNotExceedingControls,
    FailureId::AbstractionCountWithoutUtility,
    FailureId::OptionModelsIncreaseErrorWithoutGain,
    FailureId::StaleCreditProtectsOld,
    FailureId::FeaturePopulationCollapses,
    FailureId::AdaptationDegradesToBaseline,
    FailureId::HiddenReplayOrUndeclaredCompute,
    FailureId::ResourceComplianceOmittedMeasurements,
    FailureId::Unreproducible,
    FailureId::OpenEndednessVanishesWhenHeldFixed,
];

/// Detect every charter §14 failure. Unavailable input is never a pass.
///
/// # Errors
///
/// Rejects empty identities or a zero window.
pub fn detect_failures(observations: &[FailureObservation]) -> Result<FailureTable, FailureError> {
    if observations.is_empty() {
        return Err(FailureError::Trace);
    }
    let mut ids = BTreeSet::new();
    let mut rows = Vec::new();
    for observation in observations {
        if observation.case_id.is_empty() || !ids.insert(observation.case_id.as_str()) {
            return Err(FailureError::Identity);
        }
        if observation.window == 0 {
            return Err(FailureError::Trace);
        }
        rows.push(evaluate(observation));
    }
    rows.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    Ok(FailureTable {
        schema: "bonsai.failure-table/v1".to_owned(),
        rows,
    })
}

/// Stable ordered list of the ten charter §14 criteria.
#[must_use]
pub const fn charter_failure_ids() -> [FailureId; 10] {
    ALL_CRITERIA
}

fn evaluate(observation: &FailureObservation) -> FailureRow {
    let (verdict, detail) = if !observation.evidence_available {
        (
            FailureVerdict::Indeterminate,
            Some("FAILURE_EVIDENCE_UNAVAILABLE"),
        )
    } else if needs_comparator(observation.criterion)
        && observation
            .comparator_id
            .as_deref()
            .is_none_or(str::is_empty)
    {
        (
            FailureVerdict::Indeterminate,
            Some("FAILURE_COMPARATOR_UNAVAILABLE"),
        )
    } else if observation.primary.is_none()
        || (needs_secondary(observation.criterion) && observation.secondary.is_none())
    {
        (
            FailureVerdict::Indeterminate,
            Some("FAILURE_INPUT_UNAVAILABLE"),
        )
    } else if failed(observation) {
        (FailureVerdict::Fail, Some("FAILURE_CRITERION_MET"))
    } else {
        (FailureVerdict::Pass, None)
    };
    FailureRow {
        case_id: observation.case_id.clone(),
        criterion: observation.criterion,
        window: observation.window,
        tolerance: observation.tolerance,
        comparator_id: observation.comparator_id.clone(),
        verdict,
        detail_code: detail.map(str::to_owned),
    }
}

fn needs_comparator(criterion: FailureId) -> bool {
    matches!(
        criterion,
        FailureId::UsefulBehaviorNotExceedingControls
            | FailureId::AdaptationDegradesToBaseline
            | FailureId::Unreproducible
    )
}

fn needs_secondary(criterion: FailureId) -> bool {
    matches!(
        criterion,
        FailureId::AbstractionCountWithoutUtility
            | FailureId::OptionModelsIncreaseErrorWithoutGain
            | FailureId::StaleCreditProtectsOld
    )
}

fn failed(observation: &FailureObservation) -> bool {
    let primary = observation.primary.unwrap_or(0);
    let secondary = observation.secondary.unwrap_or(0);
    match observation.criterion {
        FailureId::UsefulBehaviorNotExceedingControls
        | FailureId::FeaturePopulationCollapses
        | FailureId::AdaptationDegradesToBaseline
        | FailureId::OpenEndednessVanishesWhenHeldFixed => primary <= observation.tolerance,
        FailureId::AbstractionCountWithoutUtility => {
            primary > 0 && secondary <= observation.tolerance
        }
        FailureId::OptionModelsIncreaseErrorWithoutGain => {
            primary > observation.tolerance && secondary <= 0
        }
        FailureId::StaleCreditProtectsOld => primary > 0 && secondary > 0,
        FailureId::HiddenReplayOrUndeclaredCompute
        | FailureId::ResourceComplianceOmittedMeasurements => primary > 0,
        FailureId::Unreproducible => primary > observation.tolerance,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FailureId, FailureObservation, FailureVerdict, charter_failure_ids, detect_failures,
    };

    #[allow(clippy::too_many_arguments)]
    fn observation(
        criterion: FailureId,
        polarity: &str,
        evidence: bool,
        primary: Option<i64>,
        secondary: Option<i64>,
        comparator: Option<&str>,
        tolerance: i64,
    ) -> FailureObservation {
        let name = serde_json::to_value(criterion)
            .ok()
            .and_then(|value| value.as_str().map(str::to_owned))
            .unwrap_or_else(|| "criterion".to_owned());
        FailureObservation {
            case_id: format!("{name}.{polarity}"),
            criterion,
            window: 8,
            tolerance,
            comparator_id: comparator.map(ToOwned::to_owned),
            evidence_available: evidence,
            primary,
            secondary,
        }
    }

    fn corpus() -> Vec<FailureObservation> {
        let mut cases = Vec::new();
        for criterion in charter_failure_ids() {
            let (fail_primary, fail_secondary, pass_primary, pass_secondary, tolerance) =
                signals(criterion);
            let comparator = needs_named_comparator(criterion).then_some("control");
            cases.push(observation(
                criterion,
                "positive",
                true,
                Some(fail_primary),
                fail_secondary,
                comparator,
                tolerance,
            ));
            cases.push(observation(
                criterion,
                "negative",
                true,
                Some(pass_primary),
                pass_secondary,
                comparator,
                tolerance,
            ));
            cases.push(observation(
                criterion,
                "unavailable",
                false,
                None,
                None,
                comparator,
                tolerance,
            ));
        }
        cases
    }

    fn needs_named_comparator(criterion: FailureId) -> bool {
        matches!(
            criterion,
            FailureId::UsefulBehaviorNotExceedingControls
                | FailureId::AdaptationDegradesToBaseline
                | FailureId::Unreproducible
        )
    }

    fn signals(criterion: FailureId) -> (i64, Option<i64>, i64, Option<i64>, i64) {
        match criterion {
            FailureId::UsefulBehaviorNotExceedingControls
            | FailureId::OpenEndednessVanishesWhenHeldFixed => (0, None, 4, None, 1),
            FailureId::AbstractionCountWithoutUtility => (3, Some(0), 3, Some(5), 0),
            FailureId::OptionModelsIncreaseErrorWithoutGain => (4, Some(0), 0, Some(2), 1),
            FailureId::StaleCreditProtectsOld => (1, Some(1), 0, Some(0), 0),
            FailureId::FeaturePopulationCollapses => (0, None, 3, None, 1),
            FailureId::AdaptationDegradesToBaseline => (-2, None, 3, None, 0),
            FailureId::HiddenReplayOrUndeclaredCompute
            | FailureId::ResourceComplianceOmittedMeasurements => (1, None, 0, None, 0),
            FailureId::Unreproducible => (5, None, 0, None, 1),
        }
    }

    #[test]
    fn every_criterion_has_positive_negative_and_unavailable() {
        let table = detect_failures(&corpus()).expect("failures");
        for criterion in charter_failure_ids() {
            let name = serde_json::to_value(criterion)
                .ok()
                .and_then(|value| value.as_str().map(str::to_owned))
                .expect("name");
            let verdict = |polarity: &str| {
                table
                    .rows
                    .iter()
                    .find(|row| row.case_id == format!("{name}.{polarity}"))
                    .expect("row")
                    .verdict
            };
            assert_eq!(verdict("positive"), FailureVerdict::Fail);
            assert_eq!(verdict("negative"), FailureVerdict::Pass);
            assert_eq!(verdict("unavailable"), FailureVerdict::Indeterminate);
        }
    }

    #[test]
    fn unavailable_input_is_never_a_pass() {
        let table = detect_failures(&[observation(
            FailureId::UsefulBehaviorNotExceedingControls,
            "missing_primary",
            true,
            None,
            None,
            Some("control"),
            1,
        )])
        .expect("failures");
        assert_eq!(table.rows[0].verdict, FailureVerdict::Indeterminate);
        assert_eq!(
            table.rows[0].detail_code.as_deref(),
            Some("FAILURE_INPUT_UNAVAILABLE")
        );
    }

    #[test]
    fn committed_failure_verdicts_match_fixture() {
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/failure-criteria/v1/expected-outcomes.json"
        ))
        .expect("fixture");
        let table = detect_failures(&corpus()).expect("failures");
        let verdicts = table
            .rows
            .iter()
            .map(|row| {
                (
                    row.case_id.clone(),
                    serde_json::to_value(row.verdict).expect("verdict"),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            serde_json::json!({"schema":"bonsai.failure-outcomes/v1","verdicts":verdicts}),
            expected
        );
    }
}
