//! Linux cgroup v2 measurement and fail-closed enforcement (BM-09, BM-10).

use crate::capability::{
    BackendError, CapabilityMatrix, EnforcementRecord, HostClass, Support, control,
    record_enforcement,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const BACKEND: &str = "linux-cgroup-v2";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CgroupSnapshot {
    pub path: String,
    pub cpu_usage_usec: Option<u64>,
    pub memory_current_bytes: Option<u64>,
    pub io_read_bytes: Option<u64>,
    pub io_write_bytes: Option<u64>,
    pub pids_current: Option<u64>,
    pub controllers: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CgroupWorkload {
    pub workload_id: String,
    pub cpu_usage_usec: u64,
    pub memory_current_bytes: u64,
    pub pids_current: u64,
    pub nested_child_pids: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CgroupReconciliation {
    pub schema: String,
    pub workload_id: String,
    pub cpu_delta: u64,
    pub memory_delta: u64,
    pub pid_delta: u64,
    pub nested_covered: bool,
    pub claim_ready: bool,
    pub detail_code: String,
}

/// Detect cgroup v2 and classify live versus no-permission enforcement.
#[must_use]
pub fn detect_linux_backend() -> CapabilityMatrix {
    if !cfg!(target_os = "linux") {
        return CapabilityMatrix::assembled(
            BACKEND,
            "linux",
            HostClass::Unknown,
            vec![control("cgroup.v2", Support::Unsupported, "NOT_LINUX_HOST")],
            vec![control(
                "cgroup.cpu.max",
                Support::Unsupported,
                "NOT_LINUX_HOST",
            )],
            Support::Unsupported,
            "cgroup v2 backends are unavailable off Linux",
        );
    }
    let v2 = Path::new("/sys/fs/cgroup/cgroup.controllers").is_file();
    let delegated = v2 && can_create_child_cgroup();
    let measure = if v2 {
        Support::Supported
    } else {
        Support::Unsupported
    };
    let enforce = if delegated {
        Support::Supported
    } else if v2 {
        Support::NoPermission
    } else {
        Support::Unsupported
    };
    let detail = if v2 {
        if delegated {
            "CGROUP_V2_DELEGATED"
        } else {
            "CGROUP_V2_NOT_DELEGATED"
        }
    } else {
        "CGROUP_V1_OR_ABSENT"
    };
    let counters = [
        "cgroup.cpu.stat",
        "cgroup.memory.current",
        "cgroup.io.stat",
        "cgroup.pids.current",
        "cgroup.pressure",
    ]
    .into_iter()
    .map(|id| control(id, measure, detail))
    .collect();
    let limits = [
        "cgroup.cpu.max",
        "cgroup.memory.max",
        "cgroup.io.max",
        "cgroup.pids.max",
    ]
    .into_iter()
    .map(|id| control(id, enforce, detail))
    .collect();
    CapabilityMatrix::assembled(
        BACKEND,
        "linux",
        HostClass::Container,
        counters,
        limits,
        measure,
        if delegated {
            "cgroup v2 controllers are delegated; hard limits may be applied in a child tree"
        } else if v2 {
            "cgroup v2 is readable; controller writes are not delegated and fail closed before Track A"
        } else {
            "cgroup v2 is absent; cgroup v1 is not a release requirement"
        },
    )
}

/// Read the current process cgroup v2 snapshot when the files exist.
///
/// # Errors
///
/// Returns a stable error when the platform is not Linux or the cgroup path
/// cannot be parsed.
pub fn collect_current_cgroup() -> Result<CgroupSnapshot, BackendError> {
    if !cfg!(target_os = "linux") {
        return Err(BackendError::Platform);
    }
    let relative = current_cgroup_relative()?;
    let root = PathBuf::from("/sys/fs/cgroup").join(relative.trim_start_matches('/'));
    if !root.is_dir() {
        return Err(BackendError::Unsupported);
    }
    Ok(CgroupSnapshot {
        path: root.display().to_string(),
        cpu_usage_usec: read_stat_field(&root.join("cpu.stat"), "usage_usec"),
        memory_current_bytes: read_u64_file(&root.join("memory.current")),
        io_read_bytes: None,
        io_write_bytes: None,
        pids_current: read_u64_file(&root.join("pids.current")),
        controllers: read_controllers(&root.join("cgroup.controllers")),
    })
}

/// Reconcile controlled cgroup totals, including nested PID coverage.
///
/// # Errors
///
/// Rejects empty workload identity.
pub fn reconcile_cgroup(
    workload: &CgroupWorkload,
    observed: &CgroupSnapshot,
) -> Result<CgroupReconciliation, BackendError> {
    if workload.workload_id.is_empty() {
        return Err(BackendError::Identity);
    }
    let cpu_delta = observed
        .cpu_usage_usec
        .map_or(u64::MAX, |value| value.abs_diff(workload.cpu_usage_usec));
    let memory_delta = observed.memory_current_bytes.map_or(u64::MAX, |value| {
        value.abs_diff(workload.memory_current_bytes)
    });
    let pid_delta = observed
        .pids_current
        .map_or(u64::MAX, |value| value.abs_diff(workload.pids_current));
    let nested_covered = observed
        .pids_current
        .is_some_and(|pids| pids >= workload.nested_child_pids);
    let claim_ready = nested_covered && cpu_delta <= 1_000 && memory_delta == 0 && pid_delta == 0;
    Ok(CgroupReconciliation {
        schema: "bonsai.cgroup-reconciliation/v1".to_owned(),
        workload_id: workload.workload_id.clone(),
        cpu_delta,
        memory_delta,
        pid_delta,
        nested_covered,
        claim_ready,
        detail_code: if claim_ready {
            "CGROUP_RECONCILED".to_owned()
        } else if !nested_covered {
            "CGROUP_NESTED_TREE_UNCOVERED".to_owned()
        } else {
            "CGROUP_DELTA_OUTSIDE_TOLERANCE".to_owned()
        },
    })
}

/// Apply a cgroup limit or fail closed when the controller is not writable.
///
/// # Errors
///
/// Rejects empty identities. Privilege failures become explicit records, not
/// silent downgrades.
pub fn linux_enforcement(
    limit_id: &str,
    hard_limit: u64,
    observed: Option<u64>,
    apply: impl FnOnce() -> Result<(), BackendError>,
) -> Result<EnforcementRecord, BackendError> {
    match apply() {
        Ok(()) => record_enforcement(
            BACKEND,
            limit_id,
            Support::Supported,
            Some(hard_limit),
            observed,
            observed.is_some_and(|value| value >= hard_limit),
            true,
            false,
            "CGROUP_CONTROLLER_APPLIED",
        ),
        Err(BackendError::Privilege) => record_enforcement(
            BACKEND,
            limit_id,
            Support::NoPermission,
            None,
            None,
            false,
            true,
            false,
            "CGROUP_CONTROLLER_NOT_DELEGATED",
        ),
        Err(error) => Err(error),
    }
}

/// Attempt to create a probe child cgroup in the current tree.
///
/// # Errors
///
/// Returns privilege or I/O errors without claiming a hard limit applied.
pub fn try_apply_probe_limit() -> Result<(), BackendError> {
    let relative = current_cgroup_relative()?;
    let child = PathBuf::from("/sys/fs/cgroup")
        .join(relative.trim_start_matches('/'))
        .join("bonsai-enforce-probe");
    match fs::create_dir(&child) {
        Ok(()) => {
            let _ = fs::remove_dir(&child);
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            Err(BackendError::Privilege)
        }
        Err(_) => Err(BackendError::Io),
    }
}

fn can_create_child_cgroup() -> bool {
    try_apply_probe_limit().is_ok()
}

fn current_cgroup_relative() -> Result<String, BackendError> {
    let text = fs::read_to_string("/proc/self/cgroup").map_err(|_| BackendError::Io)?;
    text.lines()
        .find_map(|line| line.strip_prefix("0::"))
        .map(ToOwned::to_owned)
        .ok_or(BackendError::Unsupported)
}

fn read_u64_file(path: &Path) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn read_stat_field(path: &Path, field: &str) -> Option<u64> {
    fs::read_to_string(path).ok()?.lines().find_map(|line| {
        let (name, value) = line.split_once(' ')?;
        (name == field).then(|| value.parse().ok()).flatten()
    })
}

fn read_controllers(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .map(|text| text.split_whitespace().map(ToOwned::to_owned).collect())
        .unwrap_or_default()
}
