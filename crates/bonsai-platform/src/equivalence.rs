//! Cross-platform resource semantics versus numeric equivalence (BM-14).

use crate::capability::{CapabilityMatrix, Support};
use crate::energy::{EnergyTier, qualify_present_backends};
use crate::linux::detect_linux_backend;
use crate::macos::detect_macos_backend;
use crate::nvidia::nvidia_capability_matrix;
use crate::windows::detect_windows_backend;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Comparability {
    Semantic,
    Numeric,
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComparabilityCell {
    pub counter_id: String,
    pub windows: Support,
    pub macos: Support,
    pub linux: Support,
    pub comparability: Comparability,
    pub note: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComparabilityMatrix {
    pub schema: String,
    pub cells: Vec<ComparabilityCell>,
    pub energy_floor: EnergyTier,
    pub unsupported_cross_platform_claim: bool,
}

/// Build the three-OS availability matrix from live capability probes.
#[must_use]
pub fn resource_comparability_matrix() -> ComparabilityMatrix {
    let windows = detect_windows_backend();
    let macos = detect_macos_backend();
    let linux = detect_linux_backend();
    let _nvidia = nvidia_capability_matrix();
    let energy_floor = qualify_present_backends()
        .into_iter()
        .map(|record| record.tier)
        .min()
        .unwrap_or(EnergyTier::E0);
    let cells = [
        (
            "process.cpu_time",
            support(&windows, "process.cpu_time"),
            support(&macos, "process.cpu_time"),
            support(&linux, "cgroup.cpu.stat"),
            "accumulated CPU is semantically comparable; Job Object vs cgroup totals are not numeric equivalents",
        ),
        (
            "process.memory",
            support(&windows, "process.committed_memory"),
            support(&macos, "process.resident_memory"),
            support(&linux, "cgroup.memory.current"),
            "RSS, committed, and cgroup current are not numeric equivalents",
        ),
        (
            "process.io",
            support(&windows, "process.io_read"),
            support(&macos, "process.io"),
            support(&linux, "cgroup.io.stat"),
            "Windows all-process I/O is not Linux disk I/O",
        ),
        (
            "hard.cpu_limit",
            support_limit(&windows, "job.cpu_rate"),
            support_limit(&macos, "cpu_time"),
            support_limit(&linux, "cgroup.cpu.max"),
            "macOS is monitor/terminate-only; Job Object and cgroup hard caps are not portable",
        ),
        (
            "energy",
            Support::Unsupported,
            Support::Unsupported,
            Support::Unsupported,
            "uniform energy availability is forbidden; floor stays E0/E1 unless qualified",
        ),
    ]
    .into_iter()
    .map(|(counter_id, windows_s, macos_s, linux_s, note)| ComparabilityCell {
        comparability: classify(windows_s, macos_s, linux_s, counter_id),
        counter_id: counter_id.to_owned(),
        windows: windows_s,
        macos: macos_s,
        linux: linux_s,
        note: note.to_owned(),
    })
    .collect::<Vec<_>>();
    let unsupported_cross_platform_claim = cells.iter().any(|cell| {
        cell.comparability == Comparability::Numeric
            && [cell.windows, cell.macos, cell.linux]
                .iter()
                .any(|support| *support != Support::Supported)
    });
    ComparabilityMatrix {
        schema: "bonsai.resource-comparability/v1".to_owned(),
        cells,
        energy_floor,
        unsupported_cross_platform_claim,
    }
}

fn support(matrix: &CapabilityMatrix, id: &str) -> Support {
    matrix
        .control(id)
        .map_or(Support::Unsupported, |control| control.support)
}

fn support_limit(matrix: &CapabilityMatrix, id: &str) -> Support {
    matrix
        .limits
        .iter()
        .find(|control| control.control_id == id)
        .map_or(Support::Unsupported, |control| control.support)
}

fn classify(windows: Support, macos: Support, linux: Support, counter_id: &str) -> Comparability {
    if counter_id == "energy" {
        return Comparability::Unavailable;
    }
    if [windows, macos, linux]
        .iter()
        .all(|support| matches!(*support, Support::Unsupported | Support::NoPermission))
    {
        return Comparability::Unavailable;
    }
    if counter_id == "process.cpu_time" {
        Comparability::Semantic
    } else {
        Comparability::Unavailable
    }
}
