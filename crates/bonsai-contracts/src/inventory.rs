//! Platform/dependency inventory contracts and boundary sanitization.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Keys that collector output may contain internally but that never cross the
/// sanitized inventory boundary.
pub const FORBIDDEN_INVENTORY_KEYS: &[&str] = &[
    "access_token",
    "api_key",
    "device_serial",
    "device_serials",
    "host_name",
    "hostname",
    "registry_token",
    "secret",
    "serial_number",
    "source_path",
    "token",
    "user_path",
    "user_paths",
];

/// Remove forbidden identity, path, and credential fields recursively.
///
/// The caller must still deserialize and validate the result against the
/// versioned inventory contract. This function never hashes a secret into the
/// published inventory; forbidden values are removed rather than transformed.
#[must_use]
pub fn sanitize_inventory_json(value: &Value) -> Value {
    match value {
        Value::Object(properties) => Value::Object(
            properties
                .iter()
                .filter(|(key, _)| !is_forbidden_key(key))
                .map(|(key, child)| (key.clone(), sanitize_inventory_json(child)))
                .collect::<Map<_, _>>(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(sanitize_inventory_json).collect()),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => value.clone(),
    }
}

fn is_forbidden_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();
    FORBIDDEN_INVENTORY_KEYS.contains(&normalized.as_str())
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformInventory {
    pub schema_version: String,
    pub inventory_id: String,
    pub machine_identity_id: String,
    pub os: OsInventory,
    pub cpu: CpuInventory,
    pub accelerators: Vec<AcceleratorInventory>,
    pub memory: MemoryInventory,
    pub clocks: Vec<ClockInventory>,
    pub drivers: Vec<VersionedComponent>,
    pub runtimes: Vec<VersionedComponent>,
    pub compilers: Vec<VersionedComponent>,
    pub dependency_locks: Vec<DependencyLock>,
    pub privilege: PrivilegeInventory,
    pub collectors: Vec<CollectorDescriptor>,
    pub thermal_power: ThermalPowerInventory,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OsInventory {
    pub family: String,
    pub version: String,
    pub build: String,
    pub kernel: String,
    pub architecture: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CpuInventory {
    pub vendor: String,
    pub model: String,
    pub architecture: String,
    pub logical_cores: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AcceleratorInventory {
    pub accelerator_id: String,
    pub vendor: String,
    pub model: String,
    pub device_class: String,
    pub memory_bytes: Option<u64>,
    pub driver: Option<VersionedComponent>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryInventory {
    pub physical_total_bytes: u64,
    pub page_size_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClockInventory {
    pub clock_id: String,
    pub kind: ClockKind,
    pub resolution_ns: u64,
    pub monotonic: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClockKind {
    Monotonic,
    Wall,
    ProcessCpu,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionedComponent {
    pub component_id: String,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyLock {
    pub ecosystem: String,
    pub lockfile_name: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrivilegeInventory {
    pub process_level: PrivilegeLevel,
    pub elevation_available: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivilegeLevel {
    StandardUser,
    Elevated,
    ContainerRestricted,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CollectorDescriptor {
    pub collector_id: String,
    pub version: String,
    pub status: CollectorStatus,
    pub privilege_requirement: PrivilegeRequirement,
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectorStatus {
    Available,
    Unavailable,
    Unsupported,
    PermissionDenied,
    Error,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivilegeRequirement {
    None,
    Optional,
    Required,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ThermalPowerInventory {
    pub thermal_state: ThermalState,
    pub power_source: PowerSource,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThermalState {
    Unavailable,
    Nominal,
    Fair,
    Serious,
    Critical,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerSource {
    Unknown,
    Ac,
    Battery,
}

#[cfg(test)]
mod tests {
    use super::{PlatformInventory, sanitize_inventory_json};
    use serde_json::json;

    #[test]
    fn sanitizer_removes_sensitive_fields_without_hashing_values() {
        let raw = json!({
            "machine_identity_id": "11111111-1111-4111-8111-111111111111",
            "hostname": "private-host",
            "nested": {"device_serial": "serial-secret", "kept": "value"},
            "token": "credential-secret"
        });
        let sanitized = sanitize_inventory_json(&raw);
        let rendered = serde_json::to_string(&sanitized).expect("sanitized JSON serializes");
        assert_eq!(sanitized["nested"]["kept"], "value");
        assert!(!rendered.contains("private-host"));
        assert!(!rendered.contains("serial-secret"));
        assert!(!rendered.contains("credential-secret"));
    }

    #[test]
    fn public_inventory_type_rejects_unknown_fields() {
        let fixture = json!({"schema_version": "1.0", "unexpected": true});
        assert!(serde_json::from_value::<PlatformInventory>(fixture).is_err());
    }
}
