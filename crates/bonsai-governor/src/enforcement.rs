//! Fail-closed platform enforcement preflight (BQ-07).

use bonsai_platform::capability::CapabilityMatrix;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HardControl {
    pub control_id: String,
    pub required: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PreflightDecision {
    pub schema: String,
    pub admitted: bool,
    pub rejected_controls: Vec<String>,
    pub reason_code: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnforcementBridgeError {
    Identity,
    UnsupportedHard,
}

impl fmt::Display for EnforcementBridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "ENFORCEMENT_BRIDGE_IDENTITY_INVALID",
            Self::UnsupportedHard => "ENFORCEMENT_BRIDGE_HARD_CONTROL_UNSUPPORTED",
        })
    }
}

impl Error for EnforcementBridgeError {}

/// Select only supported hard controls. Unsupported hard policy refuses start.
///
/// Soft or monitor-only controls cannot silently satisfy a required hard
/// control. Track A does not begin when preflight rejects.
///
/// # Errors
///
/// Rejects empty control identities.
pub fn preflight_hard_controls(
    matrix: &CapabilityMatrix,
    controls: &[HardControl],
) -> Result<PreflightDecision, EnforcementBridgeError> {
    if controls.iter().any(|control| control.control_id.is_empty()) {
        return Err(EnforcementBridgeError::Identity);
    }
    let mut rejected = Vec::new();
    for control in controls {
        if !control.required {
            continue;
        }
        if !matrix.hard_limit_supported(&control.control_id) {
            rejected.push(control.control_id.clone());
        }
    }
    if rejected.is_empty() {
        Ok(PreflightDecision {
            schema: "bonsai.enforcement-preflight/v1".to_owned(),
            admitted: true,
            rejected_controls: Vec::new(),
            reason_code: "ENFORCEMENT_PREFLIGHT_OK".to_owned(),
        })
    } else {
        Ok(PreflightDecision {
            schema: "bonsai.enforcement-preflight/v1".to_owned(),
            admitted: false,
            rejected_controls: rejected,
            reason_code: "ENFORCEMENT_HARD_CONTROL_UNSUPPORTED".to_owned(),
        })
    }
}
