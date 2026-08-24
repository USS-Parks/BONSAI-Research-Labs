//! macOS measurement and monitor/terminate enforcement (BM-07, BM-08).

use crate::capability::{
    BackendError, CapabilityMatrix, EnforcementRecord, HostClass, Support, control,
    record_enforcement,
};
use serde::{Deserialize, Serialize};

const BACKEND: &str = "macos-process";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MacosWorkload {
    pub workload_id: String,
    pub cpu_time_ns: u64,
    pub resident_memory_bytes: u64,
    pub process_count: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MacosObservation {
    pub cpu_time_ns: u64,
    pub resident_memory_bytes: u64,
    pub process_count: u64,
    pub thermal_available: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MacosReconciliation {
    pub schema: String,
    pub workload_id: String,
    pub cpu_delta: u64,
    pub memory_delta: u64,
    pub process_count_match: bool,
    pub thermal_status: Support,
    pub claim_ready: bool,
    pub detail_code: String,
}

/// Detect macOS public-interface measurement support.
#[must_use]
pub fn detect_macos_backend() -> CapabilityMatrix {
    let on_macos = cfg!(target_os = "macos");
    let apple_silicon = on_macos && cfg!(target_arch = "aarch64");
    let (portable, portable_detail, notes) = if on_macos {
        (
            Support::Supported,
            "PORTABLE_PROCESS_TREE",
            if apple_silicon {
                "Apple-silicon portable counters available; thermal and private energy APIs stay explicit"
            } else {
                "Intel macOS portable counters available; thermal and private energy APIs stay explicit"
            },
        )
    } else {
        (
            Support::Unsupported,
            "NOT_MACOS_HOST",
            "macOS backends are unavailable off Darwin; privilege and thermal fields stay unsupported",
        )
    };
    let counters = [
        ("process.cpu_time", portable, portable_detail),
        ("process.resident_memory", portable, portable_detail),
        ("process.io", portable, portable_detail),
        ("process.count", portable, portable_detail),
        ("process.tree", portable, portable_detail),
        (
            "thermal.pressure",
            if on_macos {
                Support::NoPermission
            } else {
                Support::Unsupported
            },
            if on_macos {
                "THERMAL_PRIVILEGE_REQUIRED"
            } else {
                "NOT_MACOS_HOST"
            },
        ),
    ]
    .into_iter()
    .map(|(id, support, detail)| control(id, support, detail))
    .collect();
    let limits = ["cpu_time", "memory", "storage", "process_tree", "deadline"]
        .into_iter()
        .map(|id| {
            control(
                id,
                if on_macos {
                    Support::MonitorTerminateOnly
                } else {
                    Support::Unsupported
                },
                if on_macos {
                    "MACOS_MONITOR_TERMINATE_ONLY"
                } else {
                    "NOT_MACOS_HOST"
                },
            )
        })
        .collect();
    CapabilityMatrix::assembled(
        BACKEND,
        "macos",
        HostClass::Unknown,
        counters,
        limits,
        if on_macos {
            Support::Supported
        } else {
            Support::Unsupported
        },
        notes,
    )
}

/// Compare a controlled macOS workload with observed portable counters.
///
/// # Errors
///
/// Rejects empty workload identity.
pub fn reconcile_macos(
    workload: &MacosWorkload,
    observed: &MacosObservation,
) -> Result<MacosReconciliation, BackendError> {
    if workload.workload_id.is_empty() {
        return Err(BackendError::Identity);
    }
    let cpu_delta = workload.cpu_time_ns.abs_diff(observed.cpu_time_ns);
    let memory_delta = workload
        .resident_memory_bytes
        .abs_diff(observed.resident_memory_bytes);
    let process_count_match = workload.process_count == observed.process_count;
    let thermal_status = if observed.thermal_available {
        Support::Supported
    } else {
        Support::NoPermission
    };
    let claim_ready = process_count_match && cpu_delta <= 5_000_000 && memory_delta == 0;
    Ok(MacosReconciliation {
        schema: "bonsai.macos-reconciliation/v1".to_owned(),
        workload_id: workload.workload_id.clone(),
        cpu_delta,
        memory_delta,
        process_count_match,
        thermal_status,
        claim_ready,
        detail_code: if claim_ready {
            "MACOS_RECONCILED".to_owned()
        } else {
            "MACOS_DELTA_OUTSIDE_TOLERANCE".to_owned()
        },
    })
}

/// Record monitor/terminate enforcement with measured overshoot.
///
/// # Errors
///
/// Rejects contradictory support/value combinations.
pub fn macos_enforcement(
    limit_id: &str,
    hard_limit: u64,
    observed: u64,
    terminated: bool,
) -> Result<EnforcementRecord, BackendError> {
    record_enforcement(
        BACKEND,
        limit_id,
        Support::MonitorTerminateOnly,
        Some(hard_limit),
        Some(observed),
        terminated,
        true,
        false,
        "MACOS_MONITOR_TERMINATE_OVERSHOOT",
    )
}
