//! Clock probing and calibration with monotonic duration authority.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClockProbe {
    pub call_start_monotonic_ns: u64,
    pub reading_monotonic_ns: u64,
    pub call_end_monotonic_ns: u64,
    pub wall_unix_ns: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClockCalibrationPolicy {
    pub maximum_resolution_ns: u64,
    pub maximum_call_overhead_ns: u64,
    pub suspend_gap_ns: u64,
}

impl Default for ClockCalibrationPolicy {
    fn default() -> Self {
        Self {
            maximum_resolution_ns: 1_000_000_000,
            maximum_call_overhead_ns: 1_000_000_000,
            suspend_gap_ns: 1_000_000_000,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClockAnnotation {
    MonotonicRegression {
        previous_ns: u64,
        current_ns: u64,
    },
    WallClockRegression {
        previous_ns: u64,
        current_ns: u64,
    },
    SuspendOrPause {
        monotonic_delta_ns: u64,
        wall_delta_ns: u64,
    },
    WallClockUnavailable,
    CrossProcessComparisonUnqualified,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationVerdict {
    Pass,
    Fail,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClockCalibrationReport {
    pub schema: String,
    pub probe_count: usize,
    pub effective_resolution_ns: u64,
    pub maximum_call_overhead_ns: u64,
    pub verdict: CalibrationVerdict,
    pub annotations: Vec<ClockAnnotation>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockCalibrationError {
    Policy,
    InsufficientProbes,
    ProbeBounds,
    NoPositiveResolution,
    SystemClock,
    Arithmetic,
}

impl fmt::Display for ClockCalibrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Policy => "CLOCK_CALIBRATION_POLICY_INVALID",
            Self::InsufficientProbes => "CLOCK_CALIBRATION_PROBES_INSUFFICIENT",
            Self::ProbeBounds => "CLOCK_CALIBRATION_PROBE_BOUNDS_INVALID",
            Self::NoPositiveResolution => "CLOCK_CALIBRATION_RESOLUTION_UNOBSERVED",
            Self::SystemClock => "CLOCK_CALIBRATION_SYSTEM_CLOCK_FAILED",
            Self::Arithmetic => "CLOCK_CALIBRATION_ARITHMETIC_FAILED",
        })
    }
}

impl Error for ClockCalibrationError {}

/// Calibrate a clock from immutable probes.
///
/// Durations and call overhead use only monotonic values. Optional wall-clock
/// values may reveal regressions or suspend/pause gaps but never replace the
/// monotonic duration basis.
///
/// # Errors
///
/// Returns a stable error for invalid policy, insufficient/malformed probes,
/// absent positive resolution, or overflow.
pub fn calibrate_clock(
    probes: &[ClockProbe],
    policy: ClockCalibrationPolicy,
) -> Result<ClockCalibrationReport, ClockCalibrationError> {
    if policy.maximum_resolution_ns == 0
        || policy.maximum_call_overhead_ns == 0
        || policy.suspend_gap_ns == 0
    {
        return Err(ClockCalibrationError::Policy);
    }
    if probes.len() < 2 {
        return Err(ClockCalibrationError::InsufficientProbes);
    }
    if probes.iter().any(|probe| {
        probe.call_start_monotonic_ns > probe.reading_monotonic_ns
            || probe.reading_monotonic_ns > probe.call_end_monotonic_ns
    }) {
        return Err(ClockCalibrationError::ProbeBounds);
    }

    let maximum_call_overhead_ns = probes
        .iter()
        .map(|probe| probe.call_end_monotonic_ns - probe.call_start_monotonic_ns)
        .max()
        .ok_or(ClockCalibrationError::InsufficientProbes)?;
    let effective_resolution_ns = probes
        .windows(2)
        .filter_map(|pair| {
            pair[1]
                .reading_monotonic_ns
                .checked_sub(pair[0].reading_monotonic_ns)
                .filter(|delta| *delta > 0)
        })
        .min()
        .ok_or(ClockCalibrationError::NoPositiveResolution)?;

    let mut annotations = vec![ClockAnnotation::CrossProcessComparisonUnqualified];
    if probes.iter().all(|probe| probe.wall_unix_ns.is_none()) {
        annotations.push(ClockAnnotation::WallClockUnavailable);
    }
    for pair in probes.windows(2) {
        let previous = pair[0];
        let current = pair[1];
        if current.reading_monotonic_ns < previous.reading_monotonic_ns {
            annotations.push(ClockAnnotation::MonotonicRegression {
                previous_ns: previous.reading_monotonic_ns,
                current_ns: current.reading_monotonic_ns,
            });
        }
        if let (Some(previous_wall), Some(current_wall)) =
            (previous.wall_unix_ns, current.wall_unix_ns)
        {
            if current_wall < previous_wall {
                annotations.push(ClockAnnotation::WallClockRegression {
                    previous_ns: previous_wall,
                    current_ns: current_wall,
                });
                continue;
            }
            let monotonic_delta = current
                .reading_monotonic_ns
                .saturating_sub(previous.reading_monotonic_ns);
            let wall_delta = current_wall - previous_wall;
            if wall_delta.saturating_sub(monotonic_delta) >= policy.suspend_gap_ns {
                annotations.push(ClockAnnotation::SuspendOrPause {
                    monotonic_delta_ns: monotonic_delta,
                    wall_delta_ns: wall_delta,
                });
            }
        }
    }
    let regression = annotations.iter().any(|annotation| {
        matches!(
            annotation,
            ClockAnnotation::MonotonicRegression { .. }
                | ClockAnnotation::WallClockRegression { .. }
        )
    });
    let verdict = if regression
        || effective_resolution_ns > policy.maximum_resolution_ns
        || maximum_call_overhead_ns > policy.maximum_call_overhead_ns
    {
        CalibrationVerdict::Fail
    } else {
        CalibrationVerdict::Pass
    };
    Ok(ClockCalibrationReport {
        schema: "bonsai.clock-calibration/v1".to_owned(),
        probe_count: probes.len(),
        effective_resolution_ns,
        maximum_call_overhead_ns,
        verdict,
        annotations,
    })
}

/// Capture system clock probes using one process-local `Instant` epoch.
///
/// # Errors
///
/// Returns a stable error when fewer than two probes are requested, monotonic
/// arithmetic overflows, or wall time predates the Unix epoch.
pub fn capture_system_probes(count: usize) -> Result<Vec<ClockProbe>, ClockCalibrationError> {
    if count < 2 {
        return Err(ClockCalibrationError::InsufficientProbes);
    }
    let epoch = Instant::now();
    let mut probes = Vec::with_capacity(count);
    for _ in 0..count {
        let start = elapsed_ns(epoch)?;
        let reading = elapsed_ns(epoch)?;
        let wall = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ClockCalibrationError::SystemClock)?;
        let end = elapsed_ns(epoch)?;
        probes.push(ClockProbe {
            call_start_monotonic_ns: start,
            reading_monotonic_ns: reading,
            call_end_monotonic_ns: end,
            wall_unix_ns: Some(
                u64::try_from(wall.as_nanos()).map_err(|_| ClockCalibrationError::Arithmetic)?,
            ),
        });
    }
    Ok(probes)
}

fn elapsed_ns(epoch: Instant) -> Result<u64, ClockCalibrationError> {
    u64::try_from(epoch.elapsed().as_nanos()).map_err(|_| ClockCalibrationError::Arithmetic)
}

#[cfg(test)]
mod tests {
    use super::{
        CalibrationVerdict, ClockAnnotation, ClockCalibrationPolicy, ClockProbe, calibrate_clock,
        capture_system_probes,
    };

    fn probe(start: u64, reading: u64, end: u64, wall: u64) -> ClockProbe {
        ClockProbe {
            call_start_monotonic_ns: start,
            reading_monotonic_ns: reading,
            call_end_monotonic_ns: end,
            wall_unix_ns: Some(wall),
        }
    }

    #[test]
    fn deterministic_probes_report_resolution_and_overhead() {
        let report = calibrate_clock(
            &[
                probe(10, 12, 15, 100),
                probe(20, 22, 24, 110),
                probe(30, 35, 39, 123),
            ],
            ClockCalibrationPolicy {
                maximum_resolution_ns: 20,
                maximum_call_overhead_ns: 10,
                suspend_gap_ns: 100,
            },
        )
        .expect("calibration");
        assert_eq!(report.effective_resolution_ns, 10);
        assert_eq!(report.maximum_call_overhead_ns, 9);
        assert_eq!(report.verdict, CalibrationVerdict::Pass);
    }

    #[test]
    fn regressions_and_suspend_gaps_are_explicit() {
        let regression = calibrate_clock(
            &[probe(1, 2, 3, 100), probe(2, 4, 5, 110), probe(2, 3, 4, 90)],
            ClockCalibrationPolicy::default(),
        )
        .expect("regression report");
        assert_eq!(regression.verdict, CalibrationVerdict::Fail);
        assert!(regression.annotations.iter().any(|annotation| matches!(
            annotation,
            ClockAnnotation::MonotonicRegression { .. }
                | ClockAnnotation::WallClockRegression { .. }
        )));

        let report = calibrate_clock(
            &[
                probe(1, 2, 3, 100),
                probe(2, 4, 5, 2_000_000_104),
                probe(5, 7, 8, 2_000_000_107),
            ],
            ClockCalibrationPolicy::default(),
        )
        .expect("annotated suspend");
        assert!(
            report
                .annotations
                .iter()
                .any(|annotation| matches!(annotation, ClockAnnotation::SuspendOrPause { .. }))
        );
        assert_eq!(report.verdict, CalibrationVerdict::Pass);
    }

    #[test]
    fn live_system_clock_meets_hosted_ci_safe_bounds() {
        let probes = capture_system_probes(256).expect("system probes");
        let report = calibrate_clock(&probes, ClockCalibrationPolicy::default())
            .expect("system calibration");
        assert_eq!(report.verdict, CalibrationVerdict::Pass);
        assert!(report.effective_resolution_ns > 0);
        assert!(report.maximum_call_overhead_ns > 0);
    }
}
