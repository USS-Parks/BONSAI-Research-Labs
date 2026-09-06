//! Minor-version extension requirements never grant additional authority.
use super::ProtocolViolation;
use crate::bonsai::adapter::v1::CapabilityDeclaration;
use std::collections::BTreeSet;

pub(super) fn validate(caps: &CapabilityDeclaration, minor: u32) -> Result<(), ProtocolViolation> {
    if minor == 0
        && (!caps.required_capabilities.is_empty() || !caps.optional_capabilities.is_empty())
    {
        return Err(ProtocolViolation::VersionMismatch);
    }
    let mut names = BTreeSet::new();
    for name in caps
        .required_capabilities
        .iter()
        .chain(&caps.optional_capabilities)
    {
        if names.len() >= 32
            || name.is_empty()
            || name.len() > 128
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"./_-".contains(&byte))
            || !names.insert(name)
        {
            return Err(ProtocolViolation::CapabilityDeclaration);
        }
    }
    for required in &caps.required_capabilities {
        let supported = match required.as_str() {
            "bonsai.observation/v1" | "bonsai.action/v1" => true,
            "bonsai.work/v1" => caps.work == Some(true),
            "bonsai.feedback/v1" => caps.feedback == Some(true),
            "bonsai.artifact-events/v1" => {
                caps.asynchronous_events == Some(true)
                    && caps
                        .emitted_event_types
                        .iter()
                        .any(|name| name == "bonsai.artifact.v1.ArtifactLifecycleEvent")
            }
            // Checkpoint restore is not an adapter operation in this epoch.
            _ => false,
        };
        if !supported {
            return Err(ProtocolViolation::RequiredCapabilityUnsupported);
        }
    }
    Ok(())
}
