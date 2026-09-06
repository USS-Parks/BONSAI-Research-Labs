use super::snapshot::{Snapshot, digest};
use super::{Result, ensure, number, text};
use bonsai_contracts::accounting::OnlineAccounting;
use serde_json::Value;

// Supported source is part of the verifier binary, not supplied by the bundle.
// This narrowly attests the reference program's declared data flow; it does not
// attest a hostile interpreter, kernel or machine operator.
const SOURCES: &[(&str, &[u8])] = &[
    (
        "python/bonsai-reference/src/bonsai_reference/brdc1.py",
        include_bytes!("../../../../python/bonsai-reference/src/bonsai_reference/brdc1.py"),
    ),
    (
        "python/bonsai-reference/src/bonsai_reference/feature_discovery.py",
        include_bytes!(
            "../../../../python/bonsai-reference/src/bonsai_reference/feature_discovery.py"
        ),
    ),
    (
        "python/bonsai-reference/src/bonsai_reference/option_learning.py",
        include_bytes!(
            "../../../../python/bonsai-reference/src/bonsai_reference/option_learning.py"
        ),
    ),
    (
        "python/bonsai-reference/src/bonsai_reference/option_adapter.py",
        include_bytes!(
            "../../../../python/bonsai-reference/src/bonsai_reference/option_adapter.py"
        ),
    ),
    (
        "python/bonsai-reference/src/bonsai_reference/chain_adapter.py",
        include_bytes!("../../../../python/bonsai-reference/src/bonsai_reference/chain_adapter.py"),
    ),
    (
        "python/bonsai-reference/src/bonsai_reference/gymnasium_session.py",
        include_bytes!(
            "../../../../python/bonsai-reference/src/bonsai_reference/gymnasium_session.py"
        ),
    ),
    (
        "python/bonsai-reference/src/bonsai_reference/gymnasium_adapter.py",
        include_bytes!(
            "../../../../python/bonsai-reference/src/bonsai_reference/gymnasium_adapter.py"
        ),
    ),
    (
        "python/bonsai-reference/src/bonsai_reference/online_adapter.py",
        include_bytes!(
            "../../../../python/bonsai-reference/src/bonsai_reference/online_adapter.py"
        ),
    ),
    (
        "python/bonsai-reference/src/bonsai_reference/linear_adapter.py",
        include_bytes!(
            "../../../../python/bonsai-reference/src/bonsai_reference/linear_adapter.py"
        ),
    ),
    (
        "python/bonsai-reference/src/bonsai_reference/linear_control.py",
        include_bytes!(
            "../../../../python/bonsai-reference/src/bonsai_reference/linear_control.py"
        ),
    ),
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
    let historical = [
        include_str!("../../../../fixtures/adapter-compatibility/v1/historical-source-set.json"),
        include_str!("../../../../fixtures/second-learner/v1/bx09-source-set.json"),
    ]
    .into_iter()
    .any(|raw| historical_matches(raw, identity, 10));
    let bx10 = historical_matches(
        include_str!("../../../../fixtures/external-environment/v1/bx10-source-set.json"),
        identity,
        13,
    );
    let bx12 = historical_matches(
        include_str!("../../../../fixtures/online-options/v1/bx12-source-set.json"),
        identity,
        15,
    );
    ensure(
        current || historical || bx10 || bx12,
        "RUN_REFERENCE_SOURCE_UNSUPPORTED",
    )?;
    let (agent_id, agent_module, agent_version) =
        agent_contract(manifest, current || bx10 || bx12, current)?;
    let (environment_id, environment_module) =
        environment_contract(manifest, current || bx12, current)?;
    let components = snapshot.json("component-identity.json")?;
    for (role, key, id, module, version) in [
        ("agent", "adapter", agent_id, agent_module, agent_version),
        (
            "environment",
            "environment",
            environment_id,
            environment_module,
            "1.0.0",
        ),
    ] {
        let component = &manifest[key];
        let entry = component["entrypoint"]
            .as_array()
            .ok_or("RUN_ENTRYPOINT_UNSUPPORTED")?;
        ensure(
            component["component_id"] == id
                && component["version"] == version
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

fn historical_matches(raw: &str, identity: &Value, expected: usize) -> bool {
    let Ok(previous) = serde_json::from_str::<Value>(raw) else {
        return false;
    };
    previous["source_files"].as_object().is_some_and(|pins| {
        pins.len() == expected
            && pins
                .iter()
                .all(|(path, hash)| hash.is_string() && identity["source_files"][path] == *hash)
    })
}

fn environment_contract(
    manifest: &Value,
    current: bool,
    online: bool,
) -> Result<(&'static str, &'static str)> {
    let component = &manifest["environment"];
    match text(component, "component_id")? {
        "bonsai-causal-environment" => Ok(("bonsai-causal-environment", "environment_adapter")),
        "bonsai-gymnasium-frozen-lake" if current => {
            let config = &component["config"];
            let seed = number(config, "seed")?;
            let horizon = number(config, "horizon")?;
            ensure(
                (1..=1000).contains(&horizon)
                    && *config
                        == serde_json::json!({
                            "scenario_id":"gymnasium-frozen-lake", "version":"1.0", "seed":seed,
                            "horizon":horizon, "action_count":4, "observation_width":1,
                            "environment_id":"FrozenLake-v1", "gymnasium_version":"1.3.0",
                            "map_name":"4x4", "is_slippery":false
                        }),
                "RUN_CONFIGURATION_UNSUPPORTED",
            )?;
            Ok(("bonsai-gymnasium-frozen-lake", "gymnasium_adapter"))
        }
        "bonsai-feature-chain" if online => {
            let config = &component["config"];
            let seed = number(config, "seed")?;
            let horizon = number(config, "horizon")?;
            let terminal = config["goal_terminates"]
                .as_bool()
                .ok_or("RUN_CONFIGURATION_UNSUPPORTED")?;
            ensure(
                (1..=2000).contains(&horizon)
                    && *config
                        == serde_json::json!({
                            "scenario_id":"feature-attainment-chain", "version":"1.0", "seed":seed,
                            "horizon":horizon, "action_count":2, "observation_width":1, "size":5,
                            "reward_state":4, "goal_terminates":terminal,
                        }),
                "RUN_CONFIGURATION_UNSUPPORTED",
            )?;
            Ok(("bonsai-feature-chain", "chain_adapter"))
        }
        _ => Err("RUN_REFERENCE_COMPONENT_UNSUPPORTED"),
    }
}

fn agent_contract(
    manifest: &Value,
    current: bool,
    online: bool,
) -> Result<(&'static str, &'static str, &'static str)> {
    let component = &manifest["adapter"];
    let actions = number(&component["config"], "action_count")?;
    let accounting =
        OnlineAccounting::from_declaration_or_legacy(component.get("accounting_contract"))?;
    match text(component, "component_id")? {
        "bonsai-primitive-tabular" => {
            ensure(
                component["config"] == serde_json::json!({"action_count": actions})
                    && accounting.touches_per_update() == 2
                    && (!current || component.get("accounting_contract").is_some()),
                "RUN_CONFIGURATION_UNSUPPORTED",
            )?;
            Ok((
                "bonsai-primitive-tabular",
                "primitive_adapter",
                if current { "1.1.0" } else { "1.0.0" },
            ))
        }
        "bonsai-linear-nlms" if current => {
            let width = number(&component["config"], "observation_width")?;
            ensure(
                (1..=256).contains(&width)
                    && component["config"]
                        == serde_json::json!({"action_count":actions,"observation_width":width})
                    && number(&manifest["environment"]["config"], "observation_width")? == width
                    && component.get("accounting_contract").is_some()
                    && accounting.touches_per_update() == width + 1,
                "RUN_CONFIGURATION_UNSUPPORTED",
            )?;
            Ok(("bonsai-linear-nlms", "linear_adapter", "1.0.0"))
        }
        "bonsai-online-options" if online => {
            use bonsai_contracts::resource::WorkClass;
            let config = &component["config"];
            let width = number(config, "observation_width")?;
            let seed = number(config, "seed")?;
            let retained = number(config, "retained_state_limit_bytes")?;
            let serialized = number(config, "serialized_state_limit_bytes")?;
            let enabled = config["options_enabled"]
                .as_bool()
                .ok_or("RUN_CONFIGURATION_UNSUPPORTED")?;
            let mode = text(config, "reward_mode")?;
            ensure(
                (2..=8).contains(&actions)
                    && (1..=8).contains(&width)
                    && matches!(mode, "respecting" | "oblivious")
                    && number(&manifest["environment"]["config"], "observation_width")? == width
                    && *config
                        == serde_json::json!({"action_count":actions,"observation_width":width,"seed":seed,
                    "options_enabled":enabled,"reward_mode":mode,"retained_state_limit_bytes":retained,
                    "serialized_state_limit_bytes":serialized})
                    && accounting.is_transition_feedback()
                    && accounting.maximum_parameter_touches_per_update() == Some(1024)
                    && accounting.retained_state_limit_bytes() == Some(retained)
                    && accounting.serialized_state_limit_bytes() == Some(serialized)
                    && accounting.work_per_step(actions)
                        == vec![
                            (WorkClass::Acting, actions + 4),
                            (WorkClass::Learning, 1),
                            (WorkClass::FeatureGeneration, 168 + 4 * width),
                            (
                                WorkClass::OptionLearning,
                                16 + 4 * (8 + 2 * actions + 2 * width),
                            ),
                        ],
                "RUN_CONFIGURATION_UNSUPPORTED",
            )?;
            Ok(("bonsai-online-options", "option_adapter", "1.0.0"))
        }
        _ => Err("RUN_REFERENCE_COMPONENT_UNSUPPORTED"),
    }
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
