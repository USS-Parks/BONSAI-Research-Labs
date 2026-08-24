//! Cross-platform semantic equivalence, not numeric identity (BV-09).

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(clippy::struct_excessive_bools)]
#[serde(deny_unknown_fields)]
pub struct PlatformRecord {
    pub os_family: String,
    pub schema_ok: bool,
    pub manifest_ok: bool,
    pub track: String,
    pub ordering_ok: bool,
    pub metric_ok: bool,
    pub bundle_ok: bool,
    pub claim_ok: bool,
    pub wall_time_ns: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EquivalenceMatrix {
    pub schema: String,
    pub semantic_equivalent: bool,
    pub performance_delta_ns: u64,
    pub failures: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EquivalenceError {
    Coverage,
}

impl fmt::Display for EquivalenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("EQUIVALENCE_PLATFORM_COVERAGE_INVALID")
    }
}

impl Error for EquivalenceError {}

/// Compare schema/manifest/track/ordering/metric/bundle/claim semantics.
///
/// Performance differences are reported separately and never required to match.
///
/// # Errors
///
/// Rejects a matrix that omits windows, macos, or linux.
pub fn semantic_equivalence(
    records: &[PlatformRecord],
) -> Result<EquivalenceMatrix, EquivalenceError> {
    let families = records
        .iter()
        .map(|record| record.os_family.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if !families.contains("windows") || !families.contains("macos") || !families.contains("linux") {
        return Err(EquivalenceError::Coverage);
    }
    let mut failures = Vec::new();
    for record in records {
        if !(record.schema_ok
            && record.manifest_ok
            && record.ordering_ok
            && record.metric_ok
            && record.bundle_ok
            && record.claim_ok)
        {
            failures.push(record.os_family.clone());
        }
    }
    let tracks = records
        .iter()
        .map(|record| record.track.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if tracks.len() != 1 {
        failures.push("track-mismatch".to_owned());
    }
    let times = records.iter().map(|record| record.wall_time_ns);
    let performance_delta_ns = times.clone().max().unwrap_or(0) - times.min().unwrap_or(0);
    Ok(EquivalenceMatrix {
        schema: "bonsai.cross-platform-equivalence/v1".to_owned(),
        semantic_equivalent: failures.is_empty(),
        performance_delta_ns,
        failures,
    })
}
