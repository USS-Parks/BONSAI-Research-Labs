//! Resource, budget-headroom, calibration, and paired-overhead metrics.

use bonsai_platform::calibration::{CalibrationCoverage, CounterCalibration};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceCategory {
    Cpu,
    Wall,
    Accelerator,
    Memory,
    Storage,
    Io,
    Work,
    Energy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceObservation {
    pub counter_id: String,
    pub category: ResourceCategory,
    pub unit: String,
    pub semantic_scope: String,
    pub platform: String,
    pub total: Option<u64>,
    pub hard_limit: Option<u64>,
    pub violation_count: u64,
    pub detail_code: Option<String>,
    pub calibration: Option<CounterCalibration>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceMetricRow {
    pub counter_id: String,
    pub category: ResourceCategory,
    pub unit: String,
    pub semantic_scope: String,
    pub platform: String,
    pub total: Option<u64>,
    pub hard_limit: Option<u64>,
    pub headroom: Option<u64>,
    pub violation_count: u64,
    pub absolute_sampling_error: Option<u64>,
    pub error_parts_per_million: Option<u64>,
    pub comparable_only_with_same_semantics: bool,
    pub dependent_claim_ready: bool,
    pub detail_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PairedOverheadInput {
    pub baseline_throughput: u64,
    pub instrumented_throughput: u64,
    pub throughput_overhead_ci_upper_ppm: u64,
    pub baseline_p95_latency_ns: u64,
    pub instrumented_p95_latency_ns: u64,
    pub latency_overhead_ci_upper_ppm: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum D11Verdict {
    Pass,
    Fail,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PairedOverheadMetric {
    pub throughput_point_overhead_ppm: u64,
    pub throughput_overhead_ci_upper_ppm: u64,
    pub p95_latency_point_overhead_ppm: u64,
    pub latency_overhead_ci_upper_ppm: u64,
    pub d11_throughput_ceiling_ppm: u64,
    pub d11_latency_ceiling_ppm: u64,
    pub verdict: D11Verdict,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceMetricTable {
    pub schema: String,
    pub rows: Vec<ResourceMetricRow>,
    pub overhead: PairedOverheadMetric,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceMetricError {
    Identity,
    Duplicate,
    Availability,
    Calibration,
    Overhead,
    Arithmetic,
}

impl fmt::Display for ResourceMetricError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "RESOURCE_METRIC_IDENTITY_INVALID",
            Self::Duplicate => "RESOURCE_METRIC_DUPLICATE_COUNTER",
            Self::Availability => "RESOURCE_METRIC_AVAILABILITY_INVALID",
            Self::Calibration => "RESOURCE_METRIC_CALIBRATION_INVALID",
            Self::Overhead => "RESOURCE_METRIC_OVERHEAD_INVALID",
            Self::Arithmetic => "RESOURCE_METRIC_ARITHMETIC_FAILED",
        })
    }
}

impl Error for ResourceMetricError {}

/// Derive sorted, platform-qualified resource rows and the D-11 overhead gate.
///
/// Rows remain separate even when their categories match. Consumers may compare
/// rows only when unit and `semantic_scope` are identical; this function never
/// aggregates fields across platforms.
///
/// # Errors
///
/// Rejects duplicate/malformed counters, contradictory missingness/calibration,
/// and invalid paired-overhead evidence.
pub fn derive_resource_metrics(
    mut observations: Vec<ResourceObservation>,
    overhead: &PairedOverheadInput,
) -> Result<ResourceMetricTable, ResourceMetricError> {
    observations.sort_by(|left, right| left.counter_id.cmp(&right.counter_id));
    if observations.is_empty() {
        return Err(ResourceMetricError::Identity);
    }
    let mut previous: Option<String> = None;
    let mut rows = Vec::with_capacity(observations.len());
    for observation in observations {
        if observation.counter_id.is_empty()
            || observation.unit.is_empty()
            || observation.semantic_scope.is_empty()
            || observation.platform.is_empty()
        {
            return Err(ResourceMetricError::Identity);
        }
        if previous.as_deref() == Some(observation.counter_id.as_str()) {
            return Err(ResourceMetricError::Duplicate);
        }
        previous = Some(observation.counter_id.as_str().to_owned());
        rows.push(derive_row(observation)?);
    }
    Ok(ResourceMetricTable {
        schema: "bonsai.resource-metric-table/v1".to_owned(),
        rows,
        overhead: derive_overhead(overhead)?,
    })
}

fn derive_row(observation: ResourceObservation) -> Result<ResourceMetricRow, ResourceMetricError> {
    if observation.total.is_some() == observation.detail_code.is_some() {
        return Err(ResourceMetricError::Availability);
    }
    let headroom = observation
        .total
        .zip(observation.hard_limit)
        .map(|(total, limit)| limit.saturating_sub(total));
    let (absolute_sampling_error, error_parts_per_million, calibration_ready) =
        match observation.calibration.as_ref() {
            Some(calibration)
                if calibration.counter_id == observation.counter_id
                    && calibration.unit == observation.unit
                    && calibration.coverage == CalibrationCoverage::Measured =>
            {
                (
                    calibration.absolute_error,
                    calibration.error_parts_per_million,
                    calibration.dependent_claim_ready,
                )
            }
            Some(_) => return Err(ResourceMetricError::Calibration),
            None => (None, None, false),
        };
    let dependent_claim_ready = observation.total.is_some()
        && calibration_ready
        && observation.violation_count == 0
        && observation
            .total
            .zip(observation.hard_limit)
            .is_none_or(|(total, limit)| total <= limit);
    Ok(ResourceMetricRow {
        counter_id: observation.counter_id,
        category: observation.category,
        unit: observation.unit,
        semantic_scope: observation.semantic_scope,
        platform: observation.platform,
        total: observation.total,
        hard_limit: observation.hard_limit,
        headroom,
        violation_count: observation.violation_count,
        absolute_sampling_error,
        error_parts_per_million,
        comparable_only_with_same_semantics: true,
        dependent_claim_ready,
        detail_code: observation.detail_code,
    })
}

fn derive_overhead(
    input: &PairedOverheadInput,
) -> Result<PairedOverheadMetric, ResourceMetricError> {
    if input.baseline_throughput == 0 || input.baseline_p95_latency_ns == 0 {
        return Err(ResourceMetricError::Overhead);
    }
    let throughput_point = ppm_ratio(
        input
            .baseline_throughput
            .saturating_sub(input.instrumented_throughput),
        input.baseline_throughput,
    )?;
    let latency_point = ppm_ratio(
        input
            .instrumented_p95_latency_ns
            .saturating_sub(input.baseline_p95_latency_ns),
        input.baseline_p95_latency_ns,
    )?;
    if input.throughput_overhead_ci_upper_ppm < throughput_point
        || input.latency_overhead_ci_upper_ppm < latency_point
    {
        return Err(ResourceMetricError::Overhead);
    }
    let verdict = if input.throughput_overhead_ci_upper_ppm <= 50_000
        && input.latency_overhead_ci_upper_ppm <= 100_000
    {
        D11Verdict::Pass
    } else {
        D11Verdict::Fail
    };
    Ok(PairedOverheadMetric {
        throughput_point_overhead_ppm: throughput_point,
        throughput_overhead_ci_upper_ppm: input.throughput_overhead_ci_upper_ppm,
        p95_latency_point_overhead_ppm: latency_point,
        latency_overhead_ci_upper_ppm: input.latency_overhead_ci_upper_ppm,
        d11_throughput_ceiling_ppm: 50_000,
        d11_latency_ceiling_ppm: 100_000,
        verdict,
    })
}

fn ppm_ratio(numerator: u64, denominator: u64) -> Result<u64, ResourceMetricError> {
    u64::try_from(
        u128::from(numerator)
            .checked_mul(1_000_000)
            .ok_or(ResourceMetricError::Arithmetic)?
            / u128::from(denominator),
    )
    .map_err(|_| ResourceMetricError::Arithmetic)
}

#[cfg(test)]
mod tests {
    use super::{
        D11Verdict, PairedOverheadInput, ResourceCategory, ResourceMetricError,
        ResourceObservation, derive_resource_metrics,
    };
    use bonsai_platform::calibration::{CalibrationCoverage, CounterCalibration};

    fn calibration(id: &str, unit: &str, expected: u64, observed: u64) -> CounterCalibration {
        let error = expected.abs_diff(observed);
        CounterCalibration {
            counter_id: id.to_owned(),
            unit: unit.to_owned(),
            expected: Some(expected),
            observed: Some(observed),
            absolute_error: Some(error),
            error_parts_per_million: Some(error * 1_000_000 / expected),
            resolution: Some(1),
            tolerance_parts_per_million: 100_000,
            coverage: CalibrationCoverage::Measured,
            qualification: "known_load".to_owned(),
            dependent_claim_ready: error * 1_000_000 / expected <= 100_000,
        }
    }

    fn overhead(throughput_upper: u64, latency_upper: u64) -> PairedOverheadInput {
        PairedOverheadInput {
            baseline_throughput: 1_000,
            instrumented_throughput: 980,
            throughput_overhead_ci_upper_ppm: throughput_upper,
            baseline_p95_latency_ns: 1_000,
            instrumented_p95_latency_ns: 1_050,
            latency_overhead_ci_upper_ppm: latency_upper,
        }
    }

    fn observation(
        id: &str,
        category: ResourceCategory,
        unit: &str,
        scope: &str,
        expected: u64,
        observed: u64,
    ) -> ResourceObservation {
        ResourceObservation {
            counter_id: id.to_owned(),
            category,
            unit: unit.to_owned(),
            semantic_scope: scope.to_owned(),
            platform: "fixture".to_owned(),
            total: Some(observed),
            hard_limit: Some(expected * 2),
            violation_count: 0,
            detail_code: None,
            calibration: Some(calibration(id, unit, expected, observed)),
        }
    }

    #[test]
    fn calibrated_known_loads_reproduce_totals_and_headroom() {
        let table = derive_resource_metrics(
            vec![
                observation(
                    "cpu",
                    ResourceCategory::Cpu,
                    "ns",
                    "process_tree_cpu",
                    1_000,
                    1_050,
                ),
                observation(
                    "storage",
                    ResourceCategory::Storage,
                    "byte",
                    "agent_storage",
                    500,
                    500,
                ),
                observation(
                    "work",
                    ResourceCategory::Work,
                    "count",
                    "environment_steps",
                    100,
                    100,
                ),
            ],
            &overhead(30_000, 80_000),
        )
        .expect("metrics");
        assert_eq!(table.rows[0].absolute_sampling_error, Some(50));
        assert_eq!(table.rows[0].headroom, Some(950));
        assert!(table.rows.iter().all(|row| row.dependent_claim_ready));
        assert_eq!(table.overhead.verdict, D11Verdict::Pass);
    }

    #[test]
    fn d11_ceiling_uses_upper_confidence_bounds() {
        assert_eq!(
            derive_resource_metrics(
                vec![observation(
                    "cpu",
                    ResourceCategory::Cpu,
                    "ns",
                    "process_tree_cpu",
                    100,
                    100
                )],
                &overhead(50_001, 100_000),
            )
            .expect("metrics")
            .overhead
            .verdict,
            D11Verdict::Fail
        );
    }

    #[test]
    fn unavailable_and_incomparable_fields_are_never_aggregated() {
        let unavailable = ResourceObservation {
            counter_id: "energy".to_owned(),
            category: ResourceCategory::Energy,
            unit: "uj".to_owned(),
            semantic_scope: "device_shared".to_owned(),
            platform: "fixture".to_owned(),
            total: None,
            hard_limit: None,
            violation_count: 0,
            detail_code: Some("COUNTER_UNAVAILABLE".to_owned()),
            calibration: None,
        };
        let table = derive_resource_metrics(
            vec![
                unavailable,
                observation(
                    "rss",
                    ResourceCategory::Memory,
                    "byte",
                    "resident_set",
                    100,
                    100,
                ),
                observation(
                    "virtual",
                    ResourceCategory::Memory,
                    "byte",
                    "virtual_memory",
                    200,
                    200,
                ),
            ],
            &overhead(30_000, 80_000),
        )
        .expect("metrics");
        assert_eq!(table.rows.len(), 3);
        assert!(table.rows[0].total.is_none());
        assert_ne!(table.rows[1].semantic_scope, table.rows[2].semantic_scope);
    }

    #[test]
    fn contradictory_calibration_and_confidence_bounds_fail_closed() {
        let mut wrong = observation("cpu", ResourceCategory::Cpu, "ns", "cpu", 100, 100);
        wrong.calibration.as_mut().expect("calibration").unit = "ms".to_owned();
        assert_eq!(
            derive_resource_metrics(vec![wrong], &overhead(30_000, 80_000)),
            Err(ResourceMetricError::Calibration)
        );
        assert_eq!(
            derive_resource_metrics(
                vec![observation(
                    "cpu",
                    ResourceCategory::Cpu,
                    "ns",
                    "cpu",
                    100,
                    100
                )],
                &overhead(10_000, 40_000),
            ),
            Err(ResourceMetricError::Overhead)
        );
    }
}
