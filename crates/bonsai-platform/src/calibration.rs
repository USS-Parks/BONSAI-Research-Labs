//! Measurement calibration records and controlled workload helpers.

use crate::portable::{OperationKind, OperationLedger, collect_process_tree};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs::OpenOptions;
use std::hint::black_box;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationCoverage {
    Measured,
    Unavailable,
    Unstable,
    Error,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CounterCalibration {
    pub counter_id: String,
    pub unit: String,
    pub expected: Option<u64>,
    pub observed: Option<u64>,
    pub absolute_error: Option<u64>,
    pub error_parts_per_million: Option<u64>,
    pub resolution: Option<u64>,
    pub tolerance_parts_per_million: u64,
    pub coverage: CalibrationCoverage,
    pub qualification: String,
    pub dependent_claim_ready: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CounterCalibrationInput {
    pub counter_id: String,
    pub unit: String,
    pub expected: Option<u64>,
    pub observed: Option<u64>,
    pub resolution: Option<u64>,
    pub tolerance_parts_per_million: u64,
    pub coverage: CalibrationCoverage,
    pub qualification: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationReport {
    pub schema: String,
    pub workload_id: String,
    pub observer_wall_time_ns: u64,
    pub counters: Vec<CounterCalibration>,
    pub verdict: CalibrationVerdict,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationVerdict {
    Pass,
    Fail,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalibrationError {
    Identity,
    Unit,
    Resolution,
    ContradictoryCoverage,
    Arithmetic,
    Process,
}

impl fmt::Display for CalibrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "CALIBRATION_IDENTITY_INVALID",
            Self::Unit => "CALIBRATION_UNIT_INVALID",
            Self::Resolution => "CALIBRATION_RESOLUTION_INVALID",
            Self::ContradictoryCoverage => "CALIBRATION_COVERAGE_CONTRADICTORY",
            Self::Arithmetic => "CALIBRATION_ARITHMETIC_FAILED",
            Self::Process => "CALIBRATION_PROCESS_COLLECTION_FAILED",
        })
    }
}

impl Error for CalibrationError {}

/// Adjudicate one counter against a declared known load and tolerance.
///
/// # Errors
///
/// Returns a stable error for malformed identities/units/resolution,
/// contradictory coverage/value fields, or arithmetic overflow.
pub fn calibrate_counter(
    input: CounterCalibrationInput,
) -> Result<CounterCalibration, CalibrationError> {
    let CounterCalibrationInput {
        counter_id,
        unit,
        expected,
        observed,
        resolution,
        tolerance_parts_per_million,
        coverage,
        qualification,
    } = input;
    if counter_id.is_empty() || qualification.is_empty() {
        return Err(CalibrationError::Identity);
    }
    if unit.is_empty() {
        return Err(CalibrationError::Unit);
    }
    if resolution == Some(0) {
        return Err(CalibrationError::Resolution);
    }
    let measured = coverage == CalibrationCoverage::Measured;
    if measured != (expected.is_some() && observed.is_some() && resolution.is_some()) {
        return Err(CalibrationError::ContradictoryCoverage);
    }
    if !measured && (expected.is_some() || observed.is_some() || resolution.is_some()) {
        return Err(CalibrationError::ContradictoryCoverage);
    }

    let (absolute_error, error_parts_per_million) = match (expected, observed) {
        (Some(expected), Some(observed)) => {
            let error = expected.abs_diff(observed);
            let ppm = if expected == 0 {
                u64::from(error != 0) * 1_000_000
            } else {
                u64::try_from(
                    u128::from(error)
                        .checked_mul(1_000_000)
                        .ok_or(CalibrationError::Arithmetic)?
                        / u128::from(expected),
                )
                .map_err(|_| CalibrationError::Arithmetic)?
            };
            (Some(error), Some(ppm))
        }
        (None, None) => (None, None),
        _ => return Err(CalibrationError::ContradictoryCoverage),
    };
    let dependent_claim_ready = measured
        && error_parts_per_million.is_some_and(|error| error <= tolerance_parts_per_million);
    Ok(CounterCalibration {
        counter_id,
        unit,
        expected,
        observed,
        absolute_error,
        error_parts_per_million,
        resolution,
        tolerance_parts_per_million,
        coverage,
        qualification,
        dependent_claim_ready,
    })
}

/// Assemble a versioned workload report and fail its verdict when any declared
/// counter is unavailable, unstable, erroneous, or outside tolerance.
///
/// # Errors
///
/// Returns a stable error for an empty identity/counter set or zero observer
/// cost, because those cannot support a calibration record.
pub fn calibration_report(
    workload_id: impl Into<String>,
    observer_wall_time_ns: u64,
    counters: Vec<CounterCalibration>,
) -> Result<CalibrationReport, CalibrationError> {
    let workload_id = workload_id.into();
    if workload_id.is_empty() || counters.is_empty() {
        return Err(CalibrationError::Identity);
    }
    if observer_wall_time_ns == 0 {
        return Err(CalibrationError::Resolution);
    }
    let verdict = if counters.iter().all(|counter| counter.dependent_claim_ready) {
        CalibrationVerdict::Pass
    } else {
        CalibrationVerdict::Fail
    };
    Ok(CalibrationReport {
        schema: "bonsai.measurement-calibration/v1".to_owned(),
        workload_id,
        observer_wall_time_ns,
        counters,
        verdict,
    })
}

/// Run a process-local CPU load and compare accumulated CPU time to monotonic
/// wall duration under a deliberately broad portable calibration tolerance.
///
/// # Errors
///
/// Returns a stable error when live process collection or arithmetic fails.
pub fn calibrate_live_cpu(
    minimum_duration: Duration,
) -> Result<CounterCalibration, CalibrationError> {
    if minimum_duration.is_zero() {
        return Err(CalibrationError::Resolution);
    }
    let process_id = std::process::id();
    let before = collect_process_tree(process_id).map_err(|_| CalibrationError::Process)?;
    let started = Instant::now();
    let mut value = 1_u64;
    while started.elapsed() < minimum_duration {
        value = black_box(value.rotate_left(7).wrapping_mul(6_364_136_223_846_793_005));
    }
    black_box(value);
    let elapsed_ns =
        u64::try_from(started.elapsed().as_nanos()).map_err(|_| CalibrationError::Arithmetic)?;
    let after = collect_process_tree(process_id).map_err(|_| CalibrationError::Process)?;
    let observed = after
        .accumulated_cpu_time_ns
        .saturating_sub(before.accumulated_cpu_time_ns);
    calibrate_counter(CounterCalibrationInput {
        counter_id: "process_tree.cpu_time".to_owned(),
        unit: "ns".to_owned(),
        expected: Some(elapsed_ns),
        observed: Some(observed),
        resolution: Some(1_000_000),
        tolerance_parts_per_million: 750_000,
        coverage: CalibrationCoverage::Measured,
        qualification: "single_busy_thread_cpu_time_compared_with_monotonic_elapsed_time"
            .to_owned(),
    })
}

/// Allocate and touch a known number of bytes, recording the live RSS delta.
///
/// # Errors
///
/// Returns a stable error for a zero allocation, live process collection
/// failure, or arithmetic overflow.
pub fn calibrate_allocation_load(bytes: usize) -> Result<CounterCalibration, CalibrationError> {
    if bytes == 0 {
        return Err(CalibrationError::Resolution);
    }
    let process_id = std::process::id();
    let before = collect_process_tree(process_id).map_err(|_| CalibrationError::Process)?;
    let mut allocation = vec![0_u8; bytes];
    for offset in (0..bytes).step_by(4096) {
        allocation[offset] = 1;
    }
    black_box(&allocation);
    let after = collect_process_tree(process_id).map_err(|_| CalibrationError::Process)?;
    let expected = u64::try_from(bytes).map_err(|_| CalibrationError::Arithmetic)?;
    let observed = after
        .resident_memory_bytes
        .saturating_sub(before.resident_memory_bytes);
    calibrate_counter(CounterCalibrationInput {
        counter_id: "process_tree.rss_allocation_delta".to_owned(),
        unit: "B".to_owned(),
        expected: Some(expected),
        observed: Some(observed),
        resolution: Some(4096),
        tolerance_parts_per_million: 1_000_000,
        coverage: CalibrationCoverage::Measured,
        qualification: "rss_snapshot_delta_not_portable_committed_memory".to_owned(),
    })
}

/// Write and synchronize a known byte count and compare the process I/O delta.
///
/// # Errors
///
/// Returns a stable error for an empty load, file/process failure, or
/// arithmetic overflow.
pub fn calibrate_file_io(
    path: &Path,
    bytes: usize,
) -> Result<CounterCalibration, CalibrationError> {
    if bytes == 0 {
        return Err(CalibrationError::Resolution);
    }
    let process_id = std::process::id();
    let before = collect_process_tree(process_id).map_err(|_| CalibrationError::Process)?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|_| CalibrationError::Process)?;
    let payload = vec![0xA5_u8; bytes];
    file.write_all(&payload)
        .and_then(|()| file.sync_all())
        .map_err(|_| CalibrationError::Process)?;
    let after = collect_process_tree(process_id).map_err(|_| CalibrationError::Process)?;
    let expected = u64::try_from(bytes).map_err(|_| CalibrationError::Arithmetic)?;
    let observed = after
        .total_io_written_bytes
        .saturating_sub(before.total_io_written_bytes);
    calibrate_counter(CounterCalibrationInput {
        counter_id: "process_tree.io_write".to_owned(),
        unit: "B".to_owned(),
        expected: Some(expected),
        observed: Some(observed),
        resolution: Some(1),
        tolerance_parts_per_million: 2_000_000,
        coverage: CalibrationCoverage::Measured,
        qualification: if cfg!(windows) {
            "all_process_io_bytes"
        } else {
            "disk_io_bytes_subject_to_cache"
        }
        .to_owned(),
    })
}

/// Produce exact externally charged operation/event load counters.
///
/// # Errors
///
/// Returns a stable error if the operation ledger or calibration arithmetic
/// fails.
pub fn calibrate_operation_load(count: u64) -> Result<Vec<CounterCalibration>, CalibrationError> {
    let mut ledger = OperationLedger::default();
    ledger
        .charge(OperationKind::EnvironmentStep, count)
        .map_err(|_| CalibrationError::Arithmetic)?;
    ledger
        .charge(OperationKind::WorkItem, count)
        .map_err(|_| CalibrationError::Arithmetic)?;
    let snapshot = ledger.snapshot();
    [
        (
            "operations.environment_steps",
            OperationKind::EnvironmentStep,
        ),
        ("operations.work_items", OperationKind::WorkItem),
    ]
    .into_iter()
    .map(|(counter_id, kind)| {
        calibrate_counter(CounterCalibrationInput {
            counter_id: counter_id.to_owned(),
            unit: "1".to_owned(),
            expected: Some(count),
            observed: Some(snapshot.0.get(&kind).copied().unwrap_or_default()),
            resolution: Some(1),
            tolerance_parts_per_million: 0,
            coverage: CalibrationCoverage::Measured,
            qualification: "externally_charged_exact_counter".to_owned(),
        })
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        CalibrationCoverage, CalibrationVerdict, CounterCalibrationInput,
        calibrate_allocation_load, calibrate_counter, calibrate_file_io, calibrate_live_cpu,
        calibrate_operation_load, calibration_report,
    };
    use std::time::{Duration, Instant};

    #[test]
    fn exact_operation_and_event_loads_have_zero_error() {
        let counters = calibrate_operation_load(10_000).expect("operation calibration");
        assert!(counters.iter().all(|counter| {
            counter.absolute_error == Some(0)
                && counter.error_parts_per_million == Some(0)
                && counter.dependent_claim_ready
        }));
    }

    #[test]
    fn unavailable_and_unstable_counters_fail_dependent_claims() {
        for coverage in [
            CalibrationCoverage::Unavailable,
            CalibrationCoverage::Unstable,
            CalibrationCoverage::Error,
        ] {
            let counter = calibrate_counter(CounterCalibrationInput {
                counter_id: "accelerator.energy".to_owned(),
                unit: "uJ".to_owned(),
                expected: None,
                observed: None,
                resolution: None,
                tolerance_parts_per_million: 100_000,
                coverage,
                qualification: "not_qualified".to_owned(),
            })
            .expect("explicit non-measured result");
            assert!(!counter.dependent_claim_ready);
            let report = calibration_report("unavailable", 1, vec![counter]).expect("report");
            assert_eq!(report.verdict, CalibrationVerdict::Fail);
        }
    }

    #[test]
    fn live_workloads_record_error_resolution_coverage_and_observer_cost() {
        let observer_started = Instant::now();
        let cpu = calibrate_live_cpu(Duration::from_millis(80)).expect("live CPU calibration");
        let allocation =
            calibrate_allocation_load(8 * 1024 * 1024).expect("allocation calibration");
        let directory = tempfile::tempdir().expect("calibration directory");
        let io = calibrate_file_io(&directory.path().join("io.bin"), 64 * 1024)
            .expect("I/O calibration");
        let observer_wall_time_ns =
            u64::try_from(observer_started.elapsed().as_nanos()).expect("observer duration fits");
        let report = calibration_report(
            "live_portable_loads",
            observer_wall_time_ns,
            vec![cpu, allocation, io],
        )
        .expect("calibration report");
        for counter in &report.counters {
            assert_eq!(counter.coverage, CalibrationCoverage::Measured);
            assert!(counter.expected.is_some());
            assert!(counter.observed.is_some());
            assert!(counter.absolute_error.is_some());
            assert!(counter.error_parts_per_million.is_some());
            assert!(counter.resolution.is_some());
        }
        assert!(matches!(
            report.verdict,
            CalibrationVerdict::Pass | CalibrationVerdict::Fail
        ));
        assert!(report.observer_wall_time_ns >= 80_000_000);
    }
}
