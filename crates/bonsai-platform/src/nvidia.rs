//! Optional NVIDIA NVML collector with explicit absence states (BM-11).

use crate::capability::{CapabilityMatrix, HostClass, Support, control};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

const BACKEND: &str = "nvidia-nvml";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NvidiaPresence {
    Supported,
    NotSupported,
    NoPermission,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NvidiaDetection {
    pub schema: String,
    pub presence: NvidiaPresence,
    pub driver_version: Option<String>,
    pub device_count: u64,
    pub process_attribution: Support,
    pub board_energy_process_exclusive: bool,
    pub detail_code: String,
}

/// Feature-detect NVIDIA without breaking CPU-only hosts.
#[must_use]
pub fn detect_nvidia() -> NvidiaDetection {
    if !(library_present() || tool_present()) {
        return NvidiaDetection {
            schema: "bonsai.nvidia-detection/v1".to_owned(),
            presence: NvidiaPresence::NotSupported,
            driver_version: None,
            device_count: 0,
            process_attribution: Support::Unsupported,
            board_energy_process_exclusive: false,
            detail_code: "NVIDIA_ABSENT".to_owned(),
        };
    }
    match query_smi() {
        Some((version, count)) => NvidiaDetection {
            schema: "bonsai.nvidia-detection/v1".to_owned(),
            presence: NvidiaPresence::Supported,
            driver_version: Some(version),
            device_count: count,
            process_attribution: Support::Unsupported,
            board_energy_process_exclusive: false,
            detail_code: "NVIDIA_PRESENT_BOARD_NOT_PROCESS_EXCLUSIVE".to_owned(),
        },
        None => NvidiaDetection {
            schema: "bonsai.nvidia-detection/v1".to_owned(),
            presence: NvidiaPresence::NoPermission,
            driver_version: None,
            device_count: 0,
            process_attribution: Support::NoPermission,
            board_energy_process_exclusive: false,
            detail_code: "NVIDIA_NO_PERMISSION".to_owned(),
        },
    }
}

/// Capability matrix used by CPU-only and GPU hosts alike.
#[must_use]
pub fn nvidia_capability_matrix() -> CapabilityMatrix {
    let detection = detect_nvidia();
    let support = match detection.presence {
        NvidiaPresence::Supported => Support::Supported,
        NvidiaPresence::NotSupported => Support::Unsupported,
        NvidiaPresence::NoPermission => Support::NoPermission,
    };
    CapabilityMatrix::assembled(
        BACKEND,
        std::env::consts::OS,
        HostClass::Unknown,
        vec![
            control("gpu.identity", support, &detection.detail_code),
            control(
                "gpu.utilization",
                Support::Unsupported,
                "GPU_UTILIZATION_UNCOLLECTED",
            ),
            control("gpu.memory", Support::Unsupported, "GPU_MEMORY_UNCOLLECTED"),
            control(
                "gpu.energy",
                if detection.presence == NvidiaPresence::Supported {
                    Support::Unsupported
                } else {
                    support
                },
                "BOARD_ENERGY_NOT_PROCESS_EXCLUSIVE",
            ),
        ],
        Vec::new(),
        Support::Unsupported,
        "absence never breaks CPU-only collection; board energy is not process-exclusive",
    )
}

fn library_present() -> bool {
    [
        "/usr/lib/x86_64-linux-gnu/libnvidia-ml.so.1",
        "/usr/lib64/libnvidia-ml.so.1",
        "/usr/lib/libnvidia-ml.so.1",
    ]
    .into_iter()
    .any(|path| Path::new(path).exists())
}

fn tool_present() -> bool {
    Command::new("nvidia-smi")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn query_smi() -> Option<(String, u64)> {
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=driver_version", "--format=csv,noheader"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    let version = lines.first()?.to_string();
    Some((version, u64::try_from(lines.len()).ok()?))
}
