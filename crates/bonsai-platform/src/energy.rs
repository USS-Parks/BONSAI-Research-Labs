//! Energy evidence tiers E0–E3 and qualification records (BM-12, BM-13).

use crate::capability::{BackendError, Support};
use crate::nvidia::{NvidiaPresence, detect_nvidia};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnergyTier {
    E0,
    E1,
    E2,
    E3,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(clippy::struct_excessive_bools)]
#[serde(deny_unknown_fields)]
pub struct EnergyEvidence {
    pub case_id: String,
    pub collector_id: String,
    pub counter_wrap: bool,
    pub missing_sample: bool,
    pub clock_skew: bool,
    pub privilege_failure: bool,
    pub shared_device: bool,
    pub calibrated: bool,
    pub process_exclusive: bool,
    pub laboratory_probe: bool,
    pub estimated_zero: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyAdjudication {
    pub case_id: String,
    pub tier: EnergyTier,
    pub detail_code: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QualificationRecord {
    pub schema: String,
    pub backend_id: String,
    pub host_fingerprint: String,
    pub calibrated: bool,
    pub uncertainty_ppm: Option<u32>,
    pub counter_scope: String,
    pub sampling_period_ns: Option<u64>,
    pub tier: EnergyTier,
    pub detail_code: String,
}

/// Adjudicate E0–E3. Missing or shared evidence never becomes estimated zero.
///
/// # Errors
///
/// Rejects empty identities or invented estimated-zero rows.
pub fn adjudicate_energy(
    records: &[EnergyEvidence],
) -> Result<Vec<EnergyAdjudication>, BackendError> {
    if records.is_empty() {
        return Err(BackendError::Identity);
    }
    let mut ids = BTreeSet::new();
    let mut rows = Vec::new();
    for record in records {
        if record.case_id.is_empty()
            || record.collector_id.is_empty()
            || !ids.insert(record.case_id.as_str())
        {
            return Err(BackendError::Identity);
        }
        if record.estimated_zero {
            return Err(BackendError::Unsupported);
        }
        rows.push(tier_for(record));
    }
    Ok(rows)
}

fn tier_for(record: &EnergyEvidence) -> EnergyAdjudication {
    if record.missing_sample
        || record.privilege_failure
        || record.counter_wrap
        || record.clock_skew
        || record.shared_device
        || !record.calibrated
        || !record.process_exclusive
    {
        let detail = if record.missing_sample {
            "ENERGY_MISSING_SAMPLE"
        } else if record.privilege_failure {
            "ENERGY_PRIVILEGE_FAILURE"
        } else if record.counter_wrap {
            "ENERGY_COUNTER_WRAP"
        } else if record.clock_skew {
            "ENERGY_CLOCK_SKEW"
        } else if record.shared_device {
            "ENERGY_SHARED_DEVICE"
        } else if !record.calibrated {
            "ENERGY_UNCALIBRATED"
        } else {
            "ENERGY_NOT_PROCESS_EXCLUSIVE"
        };
        return EnergyAdjudication {
            case_id: record.case_id.clone(),
            tier: EnergyTier::E0,
            detail_code: detail.to_owned(),
        };
    }
    if record.laboratory_probe {
        return EnergyAdjudication {
            case_id: record.case_id.clone(),
            tier: EnergyTier::E3,
            detail_code: "ENERGY_LABORATORY_RESERVED".to_owned(),
        };
    }
    EnergyAdjudication {
        case_id: record.case_id.clone(),
        tier: EnergyTier::E2,
        detail_code: "ENERGY_QUALIFIED_COMPARABLE".to_owned(),
    }
}

/// Qualify the collectors actually present on this host. Absence stays E0/E1.
#[must_use]
pub fn qualify_present_backends() -> Vec<QualificationRecord> {
    let nvidia = detect_nvidia();
    let nvidia_tier = match nvidia.presence {
        NvidiaPresence::Supported => EnergyTier::E1,
        NvidiaPresence::NotSupported | NvidiaPresence::NoPermission => EnergyTier::E0,
    };
    let linux_powercap =
        cfg!(target_os = "linux") && std::path::Path::new("/sys/class/powercap").is_dir();
    vec![
        QualificationRecord {
            schema: "bonsai.energy-qualification/v1".to_owned(),
            backend_id: "linux-powercap".to_owned(),
            host_fingerprint: host_fingerprint(),
            calibrated: false,
            uncertainty_ppm: None,
            counter_scope: "package_or_absent".to_owned(),
            sampling_period_ns: None,
            tier: if linux_powercap {
                EnergyTier::E1
            } else {
                EnergyTier::E0
            },
            detail_code: if linux_powercap {
                "POWERCAP_UNQUALIFIED_ESTIMATE".to_owned()
            } else {
                "POWERCAP_ABSENT".to_owned()
            },
        },
        QualificationRecord {
            schema: "bonsai.energy-qualification/v1".to_owned(),
            backend_id: "nvidia-nvml".to_owned(),
            host_fingerprint: host_fingerprint(),
            calibrated: false,
            uncertainty_ppm: None,
            counter_scope: "board_not_process_exclusive".to_owned(),
            sampling_period_ns: None,
            tier: nvidia_tier,
            detail_code: nvidia.detail_code,
        },
        QualificationRecord {
            schema: "bonsai.energy-qualification/v1".to_owned(),
            backend_id: "windows-energy".to_owned(),
            host_fingerprint: host_fingerprint(),
            calibrated: false,
            uncertainty_ppm: None,
            counter_scope: "undocumented_apis_excluded".to_owned(),
            sampling_period_ns: None,
            tier: EnergyTier::E0,
            detail_code: "WINDOWS_ENERGY_UNQUALIFIED".to_owned(),
        },
        QualificationRecord {
            schema: "bonsai.energy-qualification/v1".to_owned(),
            backend_id: "macos-energy".to_owned(),
            host_fingerprint: host_fingerprint(),
            calibrated: false,
            uncertainty_ppm: None,
            counter_scope: "private_api_excluded".to_owned(),
            sampling_period_ns: None,
            tier: EnergyTier::E0,
            detail_code: "MACOS_ENERGY_UNQUALIFIED".to_owned(),
        },
    ]
}

#[must_use]
pub fn support_for_tier(tier: EnergyTier) -> Support {
    match tier {
        EnergyTier::E0 => Support::Unsupported,
        EnergyTier::E1 => Support::MonitorTerminateOnly,
        EnergyTier::E2 | EnergyTier::E3 => Support::Supported,
    }
}

fn host_fingerprint() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}
