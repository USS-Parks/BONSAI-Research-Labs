//! Shared OS capability, host-class, and enforcement evidence types.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Support {
    Supported,
    Unsupported,
    NoPermission,
    MonitorTerminateOnly,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HostClass {
    Physical,
    HostedCi,
    Container,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ControlCapability {
    pub control_id: String,
    pub support: Support,
    pub detail_code: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityMatrix {
    pub schema: String,
    pub backend_id: String,
    pub os_family: String,
    pub architecture: String,
    pub host_class: HostClass,
    pub counters: Vec<ControlCapability>,
    pub limits: Vec<ControlCapability>,
    pub nested_process_trees: Support,
    pub privilege_notes: String,
    pub physical_acceptance: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnforcementRecord {
    pub schema: String,
    pub backend_id: String,
    pub limit_id: String,
    pub support: Support,
    pub hard_limit: Option<u64>,
    pub observed: Option<u64>,
    pub overshoot: Option<u64>,
    pub terminated: bool,
    pub cleaned_up: bool,
    pub descendant_escape: bool,
    pub detail_code: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendError {
    Identity,
    Platform,
    Arithmetic,
    Io,
    Unsupported,
    Privilege,
}

impl fmt::Display for BackendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "PLATFORM_BACKEND_IDENTITY_INVALID",
            Self::Platform => "PLATFORM_BACKEND_PLATFORM_MISMATCH",
            Self::Arithmetic => "PLATFORM_BACKEND_ARITHMETIC_FAILED",
            Self::Io => "PLATFORM_BACKEND_IO_FAILED",
            Self::Unsupported => "PLATFORM_BACKEND_UNSUPPORTED",
            Self::Privilege => "PLATFORM_BACKEND_NO_PERMISSION",
        })
    }
}

impl Error for BackendError {}

impl CapabilityMatrix {
    /// Build a capability matrix with required identities.
    ///
    /// # Errors
    ///
    /// Rejects empty backend or OS identities.
    pub fn new(
        backend_id: &str,
        os_family: &str,
        host_class: HostClass,
        counters: Vec<ControlCapability>,
        limits: Vec<ControlCapability>,
        nested_process_trees: Support,
        privilege_notes: &str,
    ) -> Result<Self, BackendError> {
        if backend_id.is_empty() || os_family.is_empty() {
            return Err(BackendError::Identity);
        }
        Ok(Self::assembled(
            backend_id,
            os_family,
            host_class,
            counters,
            limits,
            nested_process_trees,
            privilege_notes,
        ))
    }

    /// Assemble a matrix for a known backend identity.
    #[must_use]
    pub fn assembled(
        backend_id: &str,
        os_family: &str,
        host_class: HostClass,
        counters: Vec<ControlCapability>,
        limits: Vec<ControlCapability>,
        nested_process_trees: Support,
        privilege_notes: &str,
    ) -> Self {
        let physical_acceptance = matches!(host_class, HostClass::Physical)
            && counters
                .iter()
                .any(|counter| counter.support == Support::Supported);
        Self {
            schema: "bonsai.capability-matrix/v1".to_owned(),
            backend_id: backend_id.to_owned(),
            os_family: os_family.to_owned(),
            architecture: std::env::consts::ARCH.to_owned(),
            host_class,
            counters,
            limits,
            nested_process_trees,
            privilege_notes: privilege_notes.to_owned(),
            physical_acceptance,
        }
    }

    #[must_use]
    pub fn control(&self, control_id: &str) -> Option<&ControlCapability> {
        self.counters
            .iter()
            .chain(self.limits.iter())
            .find(|control| control.control_id == control_id)
    }

    #[must_use]
    pub fn hard_limit_supported(&self, limit_id: &str) -> bool {
        self.limits
            .iter()
            .any(|limit| limit.control_id == limit_id && limit.support == Support::Supported)
    }
}

#[must_use]
pub fn control(control_id: &str, support: Support, detail_code: &str) -> ControlCapability {
    ControlCapability {
        control_id: control_id.to_owned(),
        support,
        detail_code: detail_code.to_owned(),
    }
}

/// Record one intentional limit crossing. Unsupported limits stay declared.
///
/// # Errors
///
/// Rejects empty identities or contradictory numeric fields.
#[allow(clippy::too_many_arguments)]
pub fn record_enforcement(
    backend_id: &str,
    limit_id: &str,
    support: Support,
    hard_limit: Option<u64>,
    observed: Option<u64>,
    terminated: bool,
    cleaned_up: bool,
    descendant_escape: bool,
    detail_code: &str,
) -> Result<EnforcementRecord, BackendError> {
    if backend_id.is_empty() || limit_id.is_empty() || detail_code.is_empty() {
        return Err(BackendError::Identity);
    }
    let overshoot = match (support, hard_limit, observed) {
        (Support::Supported | Support::MonitorTerminateOnly, Some(hard), Some(seen)) => {
            Some(seen.saturating_sub(hard))
        }
        (Support::Unsupported | Support::NoPermission, None, None) => None,
        _ => return Err(BackendError::Identity),
    };
    Ok(EnforcementRecord {
        schema: "bonsai.enforcement-record/v1".to_owned(),
        backend_id: backend_id.to_owned(),
        limit_id: limit_id.to_owned(),
        support,
        hard_limit,
        observed,
        overshoot,
        terminated,
        cleaned_up,
        descendant_escape,
        detail_code: detail_code.to_owned(),
    })
}
