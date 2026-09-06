use super::snapshot::{Snapshot, digest};
use super::{Result, ensure, text};
use serde_json::Value;

// Supported source is part of the verifier binary, not supplied by the bundle.
// This narrowly attests the reference program's declared data flow; it does not
// attest a hostile interpreter, kernel or machine operator.
const SOURCES: &[(&str, &[u8])] = &[
    (
        "scripts/adapter_entrypoint.py",
        include_bytes!("../../../../scripts/adapter_entrypoint.py"),
    ),
    (
        "python/bonsai-reference/src/bonsai_reference/__init__.py",
        include_bytes!("../../../../python/bonsai-reference/src/bonsai_reference/__init__.py"),
    ),
    (
        "python/bonsai-reference/src/bonsai_reference/control.py",
        include_bytes!("../../../../python/bonsai-reference/src/bonsai_reference/control.py"),
    ),
    (
        "python/bonsai-reference/src/bonsai_reference/primitive_adapter.py",
        include_bytes!(
            "../../../../python/bonsai-reference/src/bonsai_reference/primitive_adapter.py"
        ),
    ),
    (
        "python/bonsai-reference/src/bonsai_reference/environment_adapter.py",
        include_bytes!(
            "../../../../python/bonsai-reference/src/bonsai_reference/environment_adapter.py"
        ),
    ),
    (
        "python/bonsai-reference/src/bonsai_reference/adapter_protocol.py",
        include_bytes!(
            "../../../../python/bonsai-reference/src/bonsai_reference/adapter_protocol.py"
        ),
    ),
    (
        "python/bonsai-reference/src/bonsai_reference/adapter_wire.py",
        include_bytes!("../../../../python/bonsai-reference/src/bonsai_reference/adapter_wire.py"),
    ),
    (
        "python/bonsai-reference/src/bonsai_reference/transport.py",
        include_bytes!("../../../../python/bonsai-reference/src/bonsai_reference/transport.py"),
    ),
    (
        "python/bonsai-reference/src/bonsai_reference/scenario.py",
        include_bytes!("../../../../python/bonsai-reference/src/bonsai_reference/scenario.py"),
    ),
    (
        "python/bonsai-reference/src/bonsai/adapter/v1/adapter_pb2.py",
        include_bytes!("../../../../python/bonsai-reference/src/bonsai/adapter/v1/adapter_pb2.py"),
    ),
];

pub(super) fn check(snapshot: &Snapshot, manifest: &Value, identity: &Value) -> Result<()> {
    let current = SOURCES
        .iter()
        .all(|(path, bytes)| identity["source_files"][*path] == digest(bytes));
    let previous: Value = serde_json::from_str(include_str!(
        "../../../../fixtures/adapter-compatibility/v1/historical-source-set.json"
    ))
    .map_err(|_| "RUN_REFERENCE_SOURCE_CATALOG_INVALID")?;
    let historical = SOURCES.iter().all(|(path, _)| {
        previous["source_files"][*path].is_string()
            && identity["source_files"][*path] == previous["source_files"][*path]
    });
    ensure(current || historical, "RUN_REFERENCE_SOURCE_UNSUPPORTED")?;
    let components = snapshot.json("component-identity.json")?;
    for (role, key, id, module) in [
        (
            "agent",
            "adapter",
            "bonsai-primitive-tabular",
            "primitive_adapter",
        ),
        (
            "environment",
            "environment",
            "bonsai-causal-environment",
            "environment_adapter",
        ),
    ] {
        let component = &manifest[key];
        let entry = component["entrypoint"]
            .as_array()
            .ok_or("RUN_ENTRYPOINT_UNSUPPORTED")?;
        ensure(
            component["component_id"] == id
                && component["version"] == "1.0.0"
                && components[role]["component_id"] == id
                && entry.len() == 5,
            "RUN_REFERENCE_COMPONENT_UNSUPPORTED",
        )?;
        let executable = entry[0].as_str().ok_or("RUN_ENTRYPOINT_UNSUPPORTED")?;
        let bootstrap = entry[3].as_str().ok_or("RUN_ENTRYPOINT_UNSUPPORTED")?;
        ensure(
            executable.starts_with('/')
                && !executable.contains("/../")
                && bootstrap.starts_with('/')
                && bootstrap.ends_with("/scripts/adapter_entrypoint.py")
                && entry[1] == "-I"
                && entry[2] == "-B"
                && entry[4] == module,
            "RUN_ENTRYPOINT_UNSUPPORTED",
        )?;
        let files = components[role]["entrypoint_files"]
            .as_object()
            .ok_or("RUN_COMPONENT_FILES_INVALID")?;
        ensure(
            files.len() == 2
                && files.get(bootstrap).and_then(Value::as_str)
                    == identity["source_files"]["scripts/adapter_entrypoint.py"].as_str(),
            "RUN_COMPONENT_FILES_INVALID",
        )?;
        let binary = files
            .get(executable)
            .and_then(Value::as_str)
            .ok_or("RUN_COMPONENT_FILES_INVALID")?;
        ensure(
            super::trace::unhex(binary)?.len() == 32,
            "RUN_COMPONENT_FILES_INVALID",
        )?;
    }
    Ok(())
}

pub(super) fn launch(audit: &Value, manifest: &Value) -> Result<()> {
    let cwd = text(audit, "current_directory")?;
    ensure(
        cwd.starts_with('/') && cwd.ends_with("/agent") && !cwd.contains("/../"),
        "RUN_AGENT_ROOT_INVALID",
    )?;
    let entry = manifest["adapter"]["entrypoint"]
        .as_array()
        .ok_or("RUN_ENTRYPOINT_UNSUPPORTED")?;
    let mut arguments = entry[1..].to_vec();
    arguments.extend([
        Value::from("--bonsai-input"),
        Value::from(format!("configuration={cwd}/inputs/configuration")),
        Value::from("--bonsai-work-dir"),
        Value::from(format!("{cwd}/work")),
    ]);
    ensure(
        audit["arguments"] == Value::Array(arguments)
            && audit["environment_keys"]
                == serde_json::json!([
                    "BONSAI_AGENT_ROOT",
                    "BONSAI_INPUT_ROOT",
                    "BONSAI_WORK_ROOT"
                ])
            && audit["inherited_handles"]
                == serde_json::json!([
                    "stdin:protocol",
                    "stdout:protocol",
                    "stderr:bounded-diagnostic"
                ]),
        "RUN_INFORMATION_BOUNDARY_VIOLATION",
    )
}
