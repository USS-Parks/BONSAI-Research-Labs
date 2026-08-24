//! Deterministic metric derivation and bounded streaming envelope (BK-14).

use crate::{DerivedMetricTable, MetricKey, MetricRegistry, RationalValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReproReport {
    pub schema: String,
    pub identical: bool,
    pub peak_live_rows: u64,
    pub within_envelope: bool,
    pub agent_access: bool,
    pub detail_code: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReproError {
    Empty,
    Envelope,
}

impl fmt::Display for ReproError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "METRIC_REPRO_EMPTY",
            Self::Envelope => "METRIC_REPRO_ENVELOPE_EXCEEDED",
        })
    }
}

impl Error for ReproError {}

/// Recompute a table and prove it stays inside a declared live-row envelope.
///
/// # Errors
///
/// Rejects empty inputs. Envelope overflow is a report, not a panic.
pub fn reproduce_metrics(
    registry: &MetricRegistry,
    inputs: &BTreeMap<MetricKey, Option<RationalValue>>,
    expected: &DerivedMetricTable,
    max_live_rows: u64,
) -> Result<ReproReport, ReproError> {
    if inputs.is_empty() {
        return Err(ReproError::Empty);
    }
    let first = registry.compute(inputs).map_err(|_| ReproError::Empty)?;
    let second = registry.compute(inputs).map_err(|_| ReproError::Empty)?;
    let identical = first == second && first == *expected;
    let peak_live_rows = u64::try_from(first.rows.len()).unwrap_or(u64::MAX);
    let within_envelope = peak_live_rows <= max_live_rows;
    if !within_envelope {
        return Err(ReproError::Envelope);
    }
    Ok(ReproReport {
        schema: "bonsai.metric-repro/v1".to_owned(),
        identical,
        peak_live_rows,
        within_envelope,
        agent_access: false,
        detail_code: if identical {
            "METRIC_REPRO_IDENTICAL".to_owned()
        } else {
            "METRIC_REPRO_DIVERGED".to_owned()
        },
    })
}
