//! Windows Job Object measurement and enforcement contracts (BM-05, BM-06).

use crate::capability::{
    BackendError, CapabilityMatrix, ControlCapability, EnforcementRecord, HostClass, Support,
    control, record_enforcement,
};
use serde::{Deserialize, Serialize};

const BACKEND: &str = "windows-job-object";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobObjectAccounting {
    pub job_id: String,
    pub process_ids: Vec<u32>,
    pub cpu_time_ns: u64,
    pub committed_memory_bytes: u64,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
    pub process_count: u64,
    pub kill_on_close: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ControlledWorkload {
    pub workload_id: String,
    pub process_ids: Vec<u32>,
    pub cpu_time_ns: u64,
    pub committed_memory_bytes: u64,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobReconciliation {
    pub schema: String,
    pub workload_id: String,
    pub child_escape: bool,
    pub cpu_delta: u64,
    pub memory_delta: u64,
    pub io_read_delta: u64,
    pub io_write_delta: u64,
    pub claim_ready: bool,
    pub detail_code: String,
}

/// Detect Windows Job Object measurement support without inventing a live pass.
#[must_use]
pub fn detect_windows_backend() -> CapabilityMatrix {
    let on_windows = cfg!(windows);
    let (counter_support, counter_detail, limit_support, limit_detail, notes) = if on_windows {
        (
            Support::Supported,
            "PORTABLE_PROCESS_TREE",
            Support::Unsupported,
            "NO_SAFE_JOB_OBJECT_BINDING",
            "portable process-tree counters are available; Job Object attach requires a privileged host probe and is not bound in this crate (unsafe_code forbid)",
        )
    } else {
        (
            Support::Unsupported,
            "NOT_WINDOWS_HOST",
            Support::Unsupported,
            "NOT_WINDOWS_HOST",
            "Job Object measurement and enforcement are unavailable off Windows; records stay unsupported",
        )
    };
    let counters = [
        "process.cpu_time",
        "process.committed_memory",
        "process.io_read",
        "process.io_write",
        "process.count",
        "job.cpu_time",
        "job.committed_memory",
        "job.io_read",
        "job.io_write",
        "job.process_count",
    ]
    .into_iter()
    .map(|id| {
        let job = id.starts_with("job.");
        control(
            id,
            if job { limit_support } else { counter_support },
            if job { limit_detail } else { counter_detail },
        )
    })
    .collect::<Vec<ControlCapability>>();
    let limits = [
        "job.cpu_rate",
        "job.memory",
        "job.process_count",
        "job.io",
        "job.kill_on_close",
    ]
    .into_iter()
    .map(|id| control(id, limit_support, limit_detail))
    .collect();
    CapabilityMatrix::assembled(
        BACKEND,
        "windows",
        HostClass::Unknown,
        counters,
        limits,
        if on_windows {
            Support::Supported
        } else {
            Support::Unsupported
        },
        notes,
    )
}

/// Compare a controlled workload with supplied Job Object accounting.
///
/// # Errors
///
/// Rejects empty identities or empty process sets.
pub fn reconcile_job_object(
    workload: &ControlledWorkload,
    job: &JobObjectAccounting,
) -> Result<JobReconciliation, BackendError> {
    if workload.workload_id.is_empty() || job.job_id.is_empty() || workload.process_ids.is_empty() {
        return Err(BackendError::Identity);
    }
    let child_escape = workload
        .process_ids
        .iter()
        .any(|pid| !job.process_ids.contains(pid));
    let cpu_delta = workload.cpu_time_ns.abs_diff(job.cpu_time_ns);
    let memory_delta = workload
        .committed_memory_bytes
        .abs_diff(job.committed_memory_bytes);
    let io_read_delta = workload.io_read_bytes.abs_diff(job.io_read_bytes);
    let io_write_delta = workload.io_write_bytes.abs_diff(job.io_write_bytes);
    let claim_ready = !child_escape
        && cpu_delta <= 1_000_000
        && memory_delta == 0
        && io_read_delta == 0
        && io_write_delta == 0;
    Ok(JobReconciliation {
        schema: "bonsai.job-object-reconciliation/v1".to_owned(),
        workload_id: workload.workload_id.clone(),
        child_escape,
        cpu_delta,
        memory_delta,
        io_read_delta,
        io_write_delta,
        claim_ready,
        detail_code: if child_escape {
            "JOB_OBJECT_CHILD_ESCAPE".to_owned()
        } else if claim_ready {
            "JOB_OBJECT_RECONCILED".to_owned()
        } else {
            "JOB_OBJECT_DELTA_OUTSIDE_TOLERANCE".to_owned()
        },
    })
}

/// Record an intentional Windows hard-limit crossing or declare it unsupported.
///
/// # Errors
///
/// Rejects contradictory support/value combinations.
pub fn windows_enforcement(
    limit_id: &str,
    supported: bool,
    hard_limit: Option<u64>,
    observed: Option<u64>,
    terminated: bool,
) -> Result<EnforcementRecord, BackendError> {
    if supported {
        record_enforcement(
            BACKEND,
            limit_id,
            Support::Supported,
            hard_limit,
            observed,
            terminated,
            true,
            false,
            "JOB_OBJECT_HARD_LIMIT_CROSSED",
        )
    } else {
        record_enforcement(
            BACKEND,
            limit_id,
            Support::Unsupported,
            None,
            None,
            false,
            true,
            false,
            "JOB_OBJECT_LIMIT_UNSUPPORTED",
        )
    }
}
