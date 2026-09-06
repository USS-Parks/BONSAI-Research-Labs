//! Thin composition of the existing transport, cgroup authority, governor, and evidence pipeline.
mod evidence;
mod identity;
mod inventory;
mod metadata;
mod peer;
mod quota;
mod watch;

use bonsai_bundle::SegmentWriter;
use bonsai_contracts::bonsai::adapter::v1::{self as wire, adapter_frame::Message as Kind};
use bonsai_contracts::resource::{BudgetScope, WorkClass};
use bonsai_governor::{BudgetAccounts, BudgetLimit, CounterKey, LimitProjection, TypedAmount};
use bonsai_platform::linux_authority::{LinuxAuthority, LinuxLimits};
use bonsai_report::{ReportData, generate_static_report};
use bonsai_runtime::{
    AgentLaunchPolicy, AgentStorageBroker, ChildTransport, IsolatedRunLayout, LifecycleState,
    ProcessCommand, RunSupervisor, StoragePolicy, TransportLimits,
};
use evidence::Evidence;
use peer::{AdapterPeer, hex};
use prost::Message;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

struct Inputs {
    manifest: Value,
    original: Vec<u8>,
    output: PathBuf,
    authority: PathBuf,
    cancel: Option<PathBuf>,
    run_id: [u8; 16],
}

pub(super) fn run(args: &[OsString]) -> Result<i32, String> {
    let began = Instant::now();
    let inputs = Inputs::parse(args)?;
    run_inputs(&inputs, began)
}

fn run_inputs(inputs: &Inputs, began: Instant) -> Result<i32, String> {
    let components = component_snapshot(&inputs.manifest)?;
    let source_identity = identity::source_identity(&inputs.manifest)?;
    let inventory = inventory::collect()?;
    fs::create_dir(&inputs.output).map_err(|e| format!("create new run root: {e}"))?;
    let layout = IsolatedRunLayout::create(&inputs.output).map_err(|e| e.to_string())?;
    write_run_status(
        &layout,
        &json!({"status":"INCOMPLETE","reason_code":"RUN_NOT_FINALIZED"}),
    )?;
    quota::write(
        layout.observer_root(),
        &layout.observer_root().join("platform-inventory.json"),
        &serde_json::to_vec_pretty(&inventory).map_err(|e| e.to_string())?,
    )?;
    let outcome = run_started(inputs, &layout, began, &source_identity, &components);
    if let Err(error) = &outcome {
        write_run_status(
            &layout,
            &json!({"status":"INCOMPLETE","reason_code":if ["RUN_WALL_LIMIT_EXCEEDED","OBSERVER_STORAGE_EXHAUSTED"].contains(&error.as_str()) {error.as_str()} else {"RUN_FINALIZATION_FAILED"},"detail":error.chars().take(256).collect::<String>()}),
        )?;
    }
    outcome
}

fn run_started(
    inputs: &Inputs,
    layout: &IsolatedRunLayout,
    began: Instant,
    source_identity: &Value,
    components: &Value,
) -> Result<i32, String> {
    quota::write(
        layout.observer_root(),
        &layout.observer_root().join("experiment-manifest.json"),
        &inputs.original,
    )?;
    quota::write(
        layout.observer_root(),
        &layout.observer_root().join("source-identity.json"),
        &serde_json::to_vec_pretty(&source_identity).map_err(|e| e.to_string())?,
    )?;
    quota::write(
        layout.observer_root(),
        &layout.observer_root().join("component-identity.json"),
        &serde_json::to_vec_pretty(&components).map_err(|e| e.to_string())?,
    )?;
    let mut supervisor = RunSupervisor::create(layout.observer_root().join("lifecycle"))
        .map_err(|e| e.to_string())?;
    supervisor
        .transition_to(LifecycleState::Running, None)
        .map_err(|e| e.to_string())?;
    let profile = &inputs.manifest["resource_profile"];
    let maximum = number(profile, "observer_output_limit_bytes")?;
    metadata::prepare(inputs, layout)?;
    let mut writer =
        SegmentWriter::create(layout.telemetry_root(), 0, 256 * 1024).map_err(|e| e.to_string())?;
    // Reserve room for immutable inputs, final status, and static report in addition to event frames.
    let mut log = Evidence::new(
        &mut writer,
        inputs.run_id,
        maximum
            .checked_sub(quota::METADATA_RESERVE)
            .ok_or("OBSERVER_BUDGET_TOO_SMALL")?,
    )?;
    let mut result = execute(inputs, layout, &mut log, began);
    if began.elapsed().as_nanos() > u128::from(number(profile, "wall_time_limit_ns")?) {
        result = Err("RUN_WALL_LIMIT_EXCEEDED".into());
    }
    if identity::source_identity(&inputs.manifest).as_ref() != Ok(source_identity) {
        result = Err("SOURCE_CHANGED_DURING_RUN".into());
    }
    if component_snapshot(&inputs.manifest).as_ref() != Ok(components) {
        result = Err("COMPONENT_CHANGED_DURING_RUN".into());
    }
    let (complete, details) = match result {
        Ok(value) => (true, value),
        Err(error) => (false, json!({"reason_code":error})),
    };
    log.append(
        "run.status",
        &json!({"status_scope":"execution_only","status":if complete {"COMPLETE"} else {"INCOMPLETE"},"details":details}),
    )?;
    let evidence = log.finish()?;
    let (events, event_bytes) = (evidence.events, evidence.bytes);
    let segment = writer.finalize().map_err(|e| e.to_string())?;
    supervisor
        .transition_to(LifecycleState::Terminating, None)
        .map_err(|e| e.to_string())?;

    let execution_ns = began.elapsed().as_nanos();
    let summary = RunSummary {
        complete,
        details: &details,
        events,
        event_bytes,
        execution_ns,
        rewards: evidence.rewards,
        reward_sum: evidence.reward_sum,
    };
    metadata::assemble(inputs, layout, &summary)?;
    write_report(inputs, layout, &summary, &segment.checksum)?;
    supervisor
        .transition_to(
            if complete {
                LifecycleState::Completed
            } else {
                LifecycleState::Failed
            },
            if complete {
                None
            } else {
                Some("RUN_INCOMPLETE")
            },
        )
        .map_err(|e| e.to_string())?;
    let artifact_index_sha256 = metadata::artifact_index(layout)?;
    let observer_bytes = quota::final_check(layout.observer_root(), maximum)?;
    if complete && began.elapsed().as_nanos() >= u128::from(number(profile, "wall_time_limit_ns")?)
    {
        return Err("RUN_WALL_LIMIT_EXCEEDED".into());
    }
    write_run_status(
        layout,
        &json!({
            "status":if complete {"COMPLETE"} else {"INCOMPLETE"},"details":details,"events":events,
            "execution_ns":execution_ns,"observer_bytes_before_status":observer_bytes,"artifact_index_sha256":artifact_index_sha256,"report_generation_outside_action_deadline":true,
        }),
    )?;
    println!(
        "{}",
        json!({"status":if complete {"COMPLETE"} else {"INCOMPLETE"},"output":layout.root(),"events":events})
    );
    Ok(if complete { 0 } else { 2 })
}

fn write_run_status(layout: &IsolatedRunLayout, value: &Value) -> Result<(), String> {
    use std::io::Write;
    let pending = layout.observer_root().join("run-status.pending");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&pending)
        .map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| e.to_string())?;
    drop(file);
    fs::rename(&pending, layout.observer_root().join("run-status.json"))
        .map_err(|e| e.to_string())?;
    fs::File::open(layout.observer_root())
        .and_then(|file| file.sync_all())
        .map_err(|e| e.to_string())
}

fn component_snapshot(manifest: &Value) -> Result<Value, String> {
    Ok(
        json!({"agent":identity::component_identity(&manifest["adapter"])?,
        "environment":identity::component_identity(&manifest["environment"])?}),
    )
}

struct RunSummary<'a> {
    complete: bool,
    details: &'a Value,
    events: u64,
    event_bytes: u64,
    execution_ns: u128,
    rewards: u64,
    reward_sum: i64,
}

fn write_report(
    inputs: &Inputs,
    layout: &IsolatedRunLayout,
    summary: &RunSummary<'_>,
    checksum: &[u8; 32],
) -> Result<(), String> {
    let RunSummary {
        complete,
        details,
        events,
        event_bytes,
        execution_ns,
        rewards,
        reward_sum,
    } = *summary;
    let report=generate_static_report(&ReportData {
        schema:"bonsai.static-report/v1".into(),title:"BONSAI governed online experiment".into(),
        manifest:inputs.manifest.clone(),platform:serde_json::from_slice(&fs::read(layout.observer_root().join("platform-inventory.json")).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?,
        track:json!({"declared":inputs.manifest["track"],"adjudication":"pending_BX_06"}),
        resources:json!({"execution_ns":execution_ns,"event_bytes":event_bytes}),
        overhead:json!({"report_generated_after_execution":true,"overhead_claim":"not_measured"}),
        behavior:json!({"status_scope":"execution_only","bundle_status_path":"../run-status.json","status":if complete {"COMPLETE"} else {"INCOMPLETE"},"events":events,"details":details,"observed_rewards":rewards,"reward_sum":reward_sum}),
        failures:if complete {json!([])} else {json!([details])},
        claims:json!({"C0":"not_adjudicated","C1":"not_adjudicated","energy":"unavailable_E0"}),
        limitations:vec!["Linux delegated controls; physical-host acceptance is separate.".into(),
            "Launch-path isolation is not a hostile-code filesystem sandbox.".into(),
            "Inventory machine identity is run-local; memory is guest-visible; physical acceptance is not established. Accelerator, driver, energy, and elevation availability are not probed; this run grants no elevation.".into()],
        hashes:BTreeMap::from([("experiment-manifest.json".into(),hex(&Sha256::digest(&inputs.original))),
            ("event-segment-content".into(),hex(checksum))]),
    }).map_err(|e|e.to_string())?;
    quota::write(
        layout.observer_root(),
        &layout.report_root().join("report.json"),
        report.machine_json.as_bytes(),
    )?;
    quota::write(
        layout.observer_root(),
        &layout.report_root().join("index.html"),
        report.html.as_bytes(),
    )?;
    Ok(())
}

impl Inputs {
    fn parse(args: &[OsString]) -> Result<Self, String> {
        let mut manifest = None;
        let mut output = None;
        let mut authority = None;
        let mut cancel = None;
        if !args.len().is_multiple_of(2) {
            return Err("run requires --manifest PATH --output NEW_PATH --authority DELEGATED_ROOT [--cancel-file PATH]".into());
        }
        for pair in args.chunks_exact(2) {
            let target = match pair[0].to_str() {
                Some("--manifest") => &mut manifest,
                Some("--output") => &mut output,
                Some("--authority") => &mut authority,
                Some("--cancel-file") => &mut cancel,
                _ => return Err("unknown run option".into()),
            };
            if target.replace(PathBuf::from(&pair[1])).is_some() {
                return Err("duplicate run option".into());
            }
        }
        let path = manifest.ok_or("missing --manifest")?;
        if fs::metadata(&path).map_err(|e| e.to_string())?.len() > 1024 * 1024 {
            return Err("MANIFEST_TOO_LARGE".into());
        }
        let original = fs::read(path).map_err(|e| e.to_string())?;
        let manifest: Value = serde_json::from_slice(&original).map_err(|e| e.to_string())?;
        let schema: Value =
            serde_json::from_str(include_str!("../../../schemas/experiment-manifest-v1.json"))
                .map_err(|e| e.to_string())?;
        jsonschema::validator_for(&schema)
            .map_err(|e| e.to_string())?
            .validate(&manifest)
            .map_err(|e| e.to_string())?;
        validate_manifest_support(&manifest)?;
        let id = manifest["run_id"]
            .as_str()
            .ok_or("RUN_ID_INVALID")?
            .replace('-', "");
        let mut run_id = [0_u8; 16];
        for (i, byte) in run_id.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&id[i * 2..i * 2 + 2], 16).map_err(|e| e.to_string())?;
        }
        if run_id == [0; 16] {
            return Err("RUN_ID_INVALID".into());
        }
        Ok(Self {
            manifest,
            original,
            output: output.ok_or("missing --output")?,
            authority: authority.ok_or("missing --authority")?,
            cancel,
            run_id,
        })
    }
}

fn validate_manifest_support(manifest: &Value) -> Result<(), String> {
    let profile = &manifest["resource_profile"];
    if profile["energy_tier"] != "E0"
        || profile["profile_id"] != "S" && profile["profile_id"] != "custom"
    {
        return Err("RUN_PROFILE_UNSUPPORTED".into());
    }
    if number(profile, "step_limit")? > 2000
        || number(profile, "wall_time_limit_ns")? > 120_000_000_000
    {
        return Err("RUN_IMPLEMENTATION_BOUND_EXCEEDED".into());
    }
    if manifest["track"]["declared_track"] != "A" {
        return Err("RUN_TRACK_UNSUPPORTED".into());
    }
    if manifest["seeds"]
        .as_array()
        .is_none_or(|seeds| seeds.len() != 1)
    {
        return Err("RUN_REQUIRES_ONE_BASE_SEED".into());
    }
    if manifest["scenario"]["config"] != manifest["environment"]["config"] {
        return Err("SCENARIO_CONFIGURATION_MISMATCH".into());
    }
    for (key, ceiling) in [
        ("agent_rss_limit_bytes", 1_073_741_824_u64),
        ("agent_storage_limit_bytes", 67_108_864),
        ("observer_output_limit_bytes", 536_870_912),
        ("action_deadline_ns", 50_000_000),
        ("per_step_cpu_time_limit_ns", 10_000_000),
    ] {
        if number(profile, key)? > ceiling {
            return Err(format!("RUN_IMPLEMENTATION_BOUND_EXCEEDED:{key}"));
        }
        if profile["profile_id"] == "S" && number(profile, key)? != ceiling {
            return Err("S_PROFILE_LIMIT_MISMATCH".into());
        }
    }
    if profile["profile_id"] == "S"
        && (number(profile, "step_limit")? != 2000
            || number(profile, "wall_time_limit_ns")? != 120_000_000_000)
    {
        return Err("S_PROFILE_LIMIT_MISMATCH".into());
    }
    for counter in manifest["expected_counters"]
        .as_array()
        .ok_or("EXPECTED_COUNTERS_MISSING")?
    {
        if counter["required_for_run"] == true
            && ![
                "process_cpu_time",
                "agent_rss",
                "environment_steps",
                "agent_storage",
                "work_items",
            ]
            .iter()
            .any(|id| counter["counter_id"] == *id)
        {
            return Err("REQUIRED_COUNTER_UNSUPPORTED".into());
        }
    }
    for metric in manifest["metrics"].as_array().ok_or("METRICS_MISSING")? {
        if metric["required"] == true && metric["metric_id"] != "behavior.reward" {
            return Err("REQUIRED_METRIC_UNSUPPORTED".into());
        }
    }
    let track = &manifest["track"];
    if track["batch_size"] != 1
        || track["replay"]["enabled"] != false
        || track["offline_updates"] != false
        || track["observer_data_access"] != false
        || track["privileged_inputs"] != false
        || track["human_labels"] != false
    {
        return Err("RUN_TRACK_CONFIGURATION_UNSUPPORTED".into());
    }
    Ok(())
}

fn number(value: &Value, key: &str) -> Result<u64, String> {
    value[key]
        .as_u64()
        .ok_or_else(|| format!("MANIFEST_FIELD_INVALID:{key}"))
}

fn execution_limits(inputs: &Inputs) -> Result<[LinuxLimits; 2], String> {
    let agent = LinuxLimits {
        cpu_quota_usec: 100_000,
        cpu_period_usec: 100_000,
        memory_max_bytes: number(
            &inputs.manifest["resource_profile"],
            "agent_rss_limit_bytes",
        )?,
        pids_max: 32,
    };
    Ok([
        agent,
        LinuxLimits {
            memory_max_bytes: 128 * 1024 * 1024,
            ..agent
        },
    ])
}

fn execute(
    inputs: &Inputs,
    layout: &IsolatedRunLayout,
    log: &mut Evidence<'_>,
    began: Instant,
) -> Result<Value, String> {
    let [limits, environment_limits] = execution_limits(inputs)?;
    let suffix = hex(&inputs.run_id);
    let mut agent_authority =
        LinuxAuthority::create(&inputs.authority, &format!("bonsai-agent-{suffix}"), limits)
            .map_err(|e| e.to_string())?;
    let mut environment_authority = match LinuxAuthority::create(
        &inputs.authority,
        &format!("bonsai-environment-{suffix}"),
        environment_limits,
    ) {
        Ok(authority) => authority,
        Err(error) => {
            agent_authority
                .cleanup(Duration::from_secs(1))
                .map_err(|e| e.to_string())?;
            return Err(error.to_string());
        }
    };
    let mut agent = None;
    let mut environment = None;
    let operation = watch::operation(
        inputs,
        began,
        &agent_authority,
        &environment_authority,
        || {
            log.append("run.resource",&json!({"phase":"controls_before_launch","agent":agent_authority.controls().map_err(|e|e.to_string())?,
            "environment":environment_authority.controls().map_err(|e|e.to_string())?}))?;
            let (launched, agent_config) = launch_agent(inputs, layout, &agent_authority, log)?;
            agent = Some(launched);
            let (launched, environment_config) =
                launch_environment(inputs, layout, &environment_authority)?;
            environment = Some(launched);
            let agent = agent.as_mut().ok_or("AGENT_MISSING")?;
            let environment = environment.as_mut().ok_or("ENVIRONMENT_MISSING")?;
            let seed = inputs.manifest["seeds"][0]["value"]
                .as_str()
                .ok_or("SEED_MISSING")?
                .parse::<u64>()
                .map_err(|e| e.to_string())?;
            agent.initialize(inputs.run_id, seed, &agent_config, log)?;
            environment.initialize(inputs.run_id, seed, &environment_config, log)?;
            let result = online_loop(
                inputs,
                layout,
                agent,
                environment,
                &agent_authority,
                log,
                began,
            )?;
            for peer in [agent, environment] {
                peer.exchange(
                    Kind::Stop(wire::Stop {
                        reason_code: "NORMAL_COMPLETION".into(),
                        deadline_monotonic_ns: 0,
                    }),
                    Duration::from_secs(1),
                    log,
                )?;
            }
            Ok(result)
        },
    );
    cleanup_children(
        agent,
        environment,
        &mut agent_authority,
        &mut environment_authority,
        operation.is_ok(),
        log,
    )?;
    operation
}

fn cleanup_children(
    agent: Option<AdapterPeer>,
    environment: Option<AdapterPeer>,
    agent_authority: &mut LinuxAuthority,
    environment_authority: &mut LinuxAuthority,
    successful: bool,
    log: &mut Evidence<'_>,
) -> Result<(), String> {
    let mut cleanup_errors = Vec::new();
    let agent_usage = agent_authority.sample().map_err(|e| e.to_string());
    let environment_usage = environment_authority.sample().map_err(|e| e.to_string());
    if !successful {
        for authority in [&*agent_authority, &*environment_authority] {
            if let Err(error) = authority.terminate() {
                cleanup_errors.push(error.to_string());
            }
        }
    }
    for peer in [agent, environment].into_iter().flatten() {
        match peer.transport.shutdown(Duration::from_secs(1)) {
            Ok(outcome) => {
                if successful && outcome.exit_code != Some(0) {
                    cleanup_errors.push("ADAPTER_EXIT_FAILED".into());
                }
                if let Err(error)=log.append("run.status",&json!({"phase":"child_reaped","adapter":peer.name,"exit_code":outcome.exit_code,
                    "transport_failures":outcome.failures.iter().map(|failure|failure.code).collect::<Vec<_>>(),
                    "stderr_hex":hex(&outcome.stderr.retained),"stderr_total":outcome.stderr.total_bytes,"stderr_truncated":outcome.stderr.truncated})) {cleanup_errors.push(error);}
            }
            Err(error) => cleanup_errors.push(error.to_string()),
        }
    }
    for (name, authority) in [
        ("agent", &mut *agent_authority),
        ("environment", &mut *environment_authority),
    ] {
        match authority.cleanup(Duration::from_secs(1)) {
            Ok(usage) => {
                if let Err(error) = log.append(
                    "run.resource",
                    &json!({"phase":"cleanup","adapter":name,"usage":usage}),
                ) {
                    cleanup_errors.push(error);
                }
            }
            Err(error) => cleanup_errors.push(error.to_string()),
        }
    }
    let agent_usage = agent_usage?;
    let environment_usage = environment_usage?;
    log.append(
        "run.resource",
        &json!({"phase":"before_cleanup","agent":agent_usage,"environment":environment_usage}),
    )?;
    if !cleanup_errors.is_empty() {
        return Err(format!("RUN_CLEANUP_FAILED:{}", cleanup_errors.join(";")));
    }
    if agent_usage.memory_oom_kills > 0
        || agent_usage.memory_max_events > 0
        || agent_usage.pids_max_events > 0
    {
        return Err("AGENT_RESOURCE_VIOLATION".into());
    }
    if environment_usage.memory_oom_kills > 0 || environment_usage.pids_max_events > 0 {
        return Err("ENVIRONMENT_RESOURCE_VIOLATION".into());
    }
    Ok(())
}

fn launch_agent(
    inputs: &Inputs,
    layout: &IsolatedRunLayout,
    authority: &LinuxAuthority,
    log: &mut Evidence<'_>,
) -> Result<(AdapterPeer, Vec<u8>), String> {
    let agent_config =
        serde_json::to_vec(&inputs.manifest["adapter"]["config"]).map_err(|e| e.to_string())?;
    let config_path = layout.observer_root().join("agent-configuration.json");
    quota::write(layout.observer_root(), &config_path, &agent_config)?;
    let grant = layout
        .grant_input(
            "configuration",
            &config_path,
            &hex(&Sha256::digest(&agent_config)),
        )
        .map_err(|e| e.to_string())?;
    let agent_entry = entrypoint(&inputs.manifest["adapter"])?;
    let policy = AgentLaunchPolicy::new(layout.clone());
    let launch = policy
        .build_command(
            &agent_entry[0],
            agent_entry[1..].iter().map(OsString::from),
            &[grant],
        )
        .map_err(|e| e.to_string())?;
    log.append(
        "run.status",
        &json!({"phase":"agent_launch_policy","audit":launch.audit}),
    )?;
    let peer = AdapterPeer::new(
        "agent",
        ChildTransport::spawn_governed(
            &launch.command,
            TransportLimits {
                maximum_frame_bytes: 65_536,
                pending_frame_capacity: 8,
                retained_stderr_bytes: 4096,
            },
            authority,
        )
        .map_err(|e| e.to_string())?,
        Some(policy),
    );
    Ok((peer, agent_config))
}

fn launch_environment(
    inputs: &Inputs,
    layout: &IsolatedRunLayout,
    authority: &LinuxAuthority,
) -> Result<(AdapterPeer, Vec<u8>), String> {
    let environment_config =
        serde_json::to_vec(&inputs.manifest["environment"]["config"]).map_err(|e| e.to_string())?;
    let spec_path = layout
        .observer_root()
        .join("environment-configuration.json");
    quota::write(layout.observer_root(), &spec_path, &environment_config)?;
    let env_entry = entrypoint(&inputs.manifest["environment"])?;
    let mut command = ProcessCommand::new(&env_entry[0])
        .clear_environment()
        .current_directory(layout.observer_root());
    for argument in &env_entry[1..] {
        command = command.argument(argument);
    }
    command = command
        .argument("--spec")
        .argument(spec_path.into_os_string());
    let peer = AdapterPeer::new(
        "environment",
        ChildTransport::spawn_governed(
            &command,
            TransportLimits {
                maximum_frame_bytes: 65_536,
                pending_frame_capacity: 8,
                retained_stderr_bytes: 4096,
            },
            authority,
        )
        .map_err(|e| e.to_string())?,
        None,
    );
    Ok((peer, environment_config))
}

fn entrypoint(component: &Value) -> Result<Vec<String>, String> {
    component["entrypoint"]
        .as_array()
        .ok_or("ENTRYPOINT_MISSING")?
        .iter()
        .map(|part| {
            part.as_str()
                .map(str::to_owned)
                .ok_or_else(|| "ENTRYPOINT_INVALID".into())
        })
        .collect()
}

fn online_loop(
    inputs: &Inputs,
    layout: &IsolatedRunLayout,
    agent: &mut AdapterPeer,
    environment: &mut AdapterPeer,
    authority: &LinuxAuthority,
    log: &mut Evidence<'_>,
    began: Instant,
) -> Result<Value, String> {
    let seed = inputs.manifest["seeds"][0]["value"]
        .as_str()
        .ok_or("SEED_MISSING")?
        .parse::<u64>()
        .map_err(|e| e.to_string())?;
    let profile = &inputs.manifest["resource_profile"];
    let step_limit = number(profile, "step_limit")?;
    let wall = Duration::from_nanos(number(profile, "wall_time_limit_ns")?);
    let action_timeout = Duration::from_nanos(number(profile, "action_deadline_ns")?);
    let storage_limit = number(profile, "agent_storage_limit_bytes")?;
    let action_count = number(&inputs.manifest["adapter"]["config"], "action_count")?;
    let storage = AgentStorageBroker::new(
        layout.clone(),
        StoragePolicy {
            policy_id: "run-agent-storage-v1".into(),
            max_bytes: storage_limit,
            max_files: 4096,
            max_file_bytes: storage_limit,
            allow_replay: false,
        },
    )
    .map_err(|e| e.to_string())?;
    let mut work = WorkMeter::new(action_count, step_limit, wall)?;
    let (mut episode, mut index, mut query) = (0_u64, 0_u64, 0_u64);
    let mut observation = None;
    let mut reward_sum = 0_i64;
    for total in 0..step_limit {
        if began.elapsed() >= wall {
            return Err("RUN_WALL_LIMIT_EXCEEDED".into());
        }
        if inputs.cancel.as_ref().is_some_and(|path| path.exists()) {
            return Err("RUN_CANCELLED".into());
        }
        authority.controls().map_err(|e| e.to_string())?;
        if observation.is_none() {
            let episode_seed = seed.checked_add(episode).ok_or("EPISODE_SEED_OVERFLOW")?;
            agent.reset(episode, episode_seed, log)?;
            environment.reset(episode, episode_seed, log)?;
            index = 0;
            query = 0;
            let bytes = environment.step(
                query,
                "bonsai.environment.observe/v1",
                Vec::new(),
                Duration::from_secs(1),
                log,
            )?;
            query += 1;
            observation =
                Some(wire::CausalObservation::decode(bytes.as_slice()).map_err(|e| e.to_string())?);
        }
        let observed = observation.as_ref().ok_or("OBSERVATION_MISSING")?;
        if observed.step != index {
            return Err("OBSERVATION_INDEX_MISMATCH".into());
        }
        work.admit(total, began.elapsed(), log)?;
        let before = authority.sample().map_err(|e| e.to_string())?;
        let transition = action_transition(
            agent,
            environment,
            observed,
            StepInfo {
                total,
                episode,
                index,
                query,
                action_timeout,
            },
            log,
        )?;
        query += 1;
        let measured =
            feedback_accounting(agent, total, index, transition.reward, action_timeout, log)?;
        work.charge(total, &measured, began.elapsed(), log)?;
        validate_resources(profile, authority, &storage, &before, total, log)?;
        reward_sum = reward_sum
            .checked_add(transition.reward)
            .ok_or("REWARD_OVERFLOW")?;
        let next = transition.next.ok_or("NEXT_OBSERVATION_MISSING")?;
        if next.step != index + 1 {
            return Err("NEXT_OBSERVATION_INDEX_INVALID".into());
        }
        if transition.terminated || transition.truncated {
            if !next.allowed_actions.is_empty() {
                return Err("TERMINAL_ACTIONS_INVALID".into());
            }
            episode += 1;
            observation = None;
        } else {
            observation = Some(next);
            index += 1;
        }
    }
    if began.elapsed() > wall {
        return Err("RUN_WALL_LIMIT_EXCEEDED".into());
    }
    Ok(
        json!({"environment_steps":step_limit,"reward_sum":reward_sum,"work_items":work.prior_work,"finished_episodes":episode}),
    )
}

struct WorkMeter {
    accounts: BudgetAccounts,
    key: CounterKey,
    per_step: u64,
    limits: Vec<BudgetLimit>,
    prior_work: u64,
}

impl WorkMeter {
    fn new(action_count: u64, step_limit: u64, wall: Duration) -> Result<Self, String> {
        let accounts = BudgetAccounts::default();
        let key = CounterKey {
            counter_id: "work_items".into(),
            unit: "1".into(),
        };
        let per_step = action_count.checked_add(1).ok_or("WORK_OVERFLOW")?;
        let mut limits = [
            (WorkClass::Acting, action_count, "acting.work_items"),
            (WorkClass::Learning, 1, "learning.work_items"),
        ]
        .map(|(work_class, maximum, id)| BudgetLimit {
            limit_id: id.into(),
            work_class,
            scope: BudgetScope::PerStep,
            key: key.clone(),
            soft_limit: maximum,
            hard_limit: maximum,
            rolling_window_ns: None,
        })
        .to_vec();
        let maximum = action_count
            .checked_mul(step_limit)
            .ok_or("WORK_OVERFLOW")?;
        limits.push(BudgetLimit {
            limit_id: "acting.rolling_work".into(),
            work_class: WorkClass::Acting,
            scope: BudgetScope::RollingWindow,
            key: key.clone(),
            soft_limit: maximum,
            hard_limit: maximum,
            rolling_window_ns: Some(u64::try_from(wall.as_nanos()).map_err(|e| e.to_string())?),
        });
        Ok(Self {
            accounts,
            key,
            per_step,
            limits,
            prior_work: 0,
        })
    }

    fn admit(
        &mut self,
        total: u64,
        elapsed: Duration,
        log: &mut Evidence<'_>,
    ) -> Result<(), String> {
        self.accounts.begin_step();
        let admission_time = u64::try_from(elapsed.as_nanos()).map_err(|e| e.to_string())?;
        for limit in &self.limits[..2] {
            let planned = TypedAmount {
                key: self.key.clone(),
                amount: limit.hard_limit,
            };
            let projection = self
                .accounts
                .project(
                    limit.work_class,
                    &planned,
                    admission_time,
                    &self.limits,
                    true,
                )
                .map_err(|e| e.to_string())?;
            log.append("run.work",&json!({"phase":"admission","total_step":total,"work_class":limit.work_class,"projection":projection}))?;
            if projection
                .iter()
                .any(|p| p.state != LimitProjection::WithinSoft)
            {
                return Err("WORK_ADMISSION_REJECTED".into());
            }
        }
        Ok(())
    }

    fn charge(
        &mut self,
        total: u64,
        measured: &wire::PrimitiveAccounting,
        elapsed: Duration,
        log: &mut Evidence<'_>,
    ) -> Result<(), String> {
        if measured.environment_steps != total + 1
            || measured.updates != total + 1
            || measured.parameter_touches != 2 * (total + 1)
            || measured.replay_items_retained != 0
        {
            return Err("ACCOUNTING_INCONSISTENT".into());
        }
        let delta = measured
            .work_items
            .checked_sub(self.prior_work)
            .ok_or("ACCOUNTING_REGRESSION")?;
        if delta != self.per_step {
            return Err("WORK_ACCOUNTING_INCONSISTENT".into());
        }
        let now = u64::try_from(elapsed.as_nanos()).map_err(|e| e.to_string())?;
        for (work_class, amount) in [(WorkClass::Acting, delta - 1), (WorkClass::Learning, 1)] {
            let charge = TypedAmount {
                key: self.key.clone(),
                amount,
            };
            let projection = self
                .accounts
                .project(work_class, &charge, now, &self.limits, true)
                .map_err(|e| e.to_string())?;
            log.append(
                "run.work",
                &json!({"phase":"measured_charge","total_step":total,"work_class":work_class,
                "accounting_hex":hex(&measured.encode_to_vec()),"projection":projection}),
            )?;
            if projection
                .iter()
                .any(|p| p.state != LimitProjection::WithinSoft)
            {
                return Err("WORK_LIMIT_EXCEEDED".into());
            }
            self.accounts
                .commit(work_class, &charge, now)
                .map_err(|e| e.to_string())?;
        }
        self.prior_work = measured.work_items;
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct StepInfo {
    total: u64,
    episode: u64,
    index: u64,
    query: u64,
    action_timeout: Duration,
}

fn action_transition(
    agent: &mut AdapterPeer,
    environment: &mut AdapterPeer,
    observed: &wire::CausalObservation,
    info: StepInfo,
    log: &mut Evidence<'_>,
) -> Result<wire::CausalTransition, String> {
    let StepInfo {
        total,
        episode,
        index,
        query,
        action_timeout,
    } = info;
    let action_bytes = agent.step(
        index,
        "bonsai.agent.observation/v1",
        observed.encode_to_vec(),
        action_timeout,
        log,
    )?;
    let action = wire::CausalAction::decode(action_bytes.as_slice()).map_err(|e| e.to_string())?;
    if action.step != index || !observed.allowed_actions.contains(&action.action) {
        return Err("ACTION_INVALID".into());
    }
    log.append(
        "run.action",
        &json!({"total_step":total,"episode":episode,"step":index,"action":action.action,
            "observation_hex":hex(&observed.encode_to_vec())}),
    )?;
    let transition_bytes = environment.step(
        query,
        "bonsai.environment.action/v1",
        action_bytes,
        Duration::from_secs(1),
        log,
    )?;
    let transition =
        wire::CausalTransition::decode(transition_bytes.as_slice()).map_err(|e| e.to_string())?;
    if transition.step != index
        || transition.action != action.action
        || transition.terminated && transition.truncated
    {
        return Err("TRANSITION_INVALID".into());
    }
    log.append("run.reward",&json!({"total_step":total,"episode":episode,"step":index,"reward":transition.reward,
            "terminated":transition.terminated,"truncated":transition.truncated,"transition_hex":hex(&transition_bytes)}))?;
    Ok(transition)
}

fn feedback_accounting(
    agent: &mut AdapterPeer,
    total: u64,
    index: u64,
    reward_value: i64,
    action_timeout: Duration,
    log: &mut Evidence<'_>,
) -> Result<wire::PrimitiveAccounting, String> {
    let reward = wire::PrimitiveReward {
        step: index,
        reward: reward_value,
    }
    .encode_to_vec();
    let mut id = [1_u8; 16];
    id[..8].copy_from_slice(&total.to_le_bytes());
    agent.exchange(
        Kind::Feedback(wire::Feedback {
            feedback_id: id.to_vec(),
            signal_type: "bonsai.agent.reward/v1".into(),
            signal_sha256: Sha256::digest(&reward).to_vec(),
            signal: reward,
            deadline_monotonic_ns: 0,
        }),
        action_timeout,
        log,
    )?;
    let response = agent.exchange(
        Kind::Work(wire::Work {
            work_item_id: id.to_vec(),
            work_class: "bonsai.agent.accounting/v1".into(),
            payload: Vec::new(),
            payload_sha256: Sha256::digest([]).to_vec(),
            deadline_monotonic_ns: 0,
        }),
        action_timeout,
        log,
    )?;
    let Kind::WorkResult(result) = response else {
        return Err("ACCOUNTING_REQUIRED".into());
    };
    if result.work_item_id != id || result.outcome != "MEASURED" {
        return Err("ACCOUNTING_IDENTITY_INVALID".into());
    }
    let measured =
        wire::PrimitiveAccounting::decode(result.result.as_slice()).map_err(|e| e.to_string())?;
    Ok(measured)
}

fn validate_resources(
    profile: &Value,
    authority: &LinuxAuthority,
    storage: &AgentStorageBroker,
    before: &bonsai_platform::linux_authority::LinuxUsage,
    total: u64,
    log: &mut Evidence<'_>,
) -> Result<(), String> {
    let after = authority.sample().map_err(|e| e.to_string())?;
    let cpu_ns = after
        .cpu_usage_usec
        .checked_sub(before.cpu_usage_usec)
        .and_then(|v| v.checked_mul(1000))
        .ok_or("CPU_COUNTER_INVALID")?;
    let rss_bytes = resident_bytes(&after.member_pids)?;
    let objects = storage.inspect().map_err(|e| e.to_string())?;
    let bytes = objects
        .iter()
        .try_fold(0_u64, |sum, item| sum.checked_add(item.bytes))
        .ok_or("STORAGE_COUNTER_OVERFLOW")?;
    log.append(
            "run.resource",
            &json!({"phase":"step","total_step":total,"cpu_time_ns":cpu_ns,
            "usage":after,"agent_rss_bytes":rss_bytes,"agent_storage_bytes":bytes,"agent_storage_objects":objects.len()}),
        )?;
    if rss_bytes > number(profile, "agent_rss_limit_bytes")?
        || cpu_ns > number(profile, "per_step_cpu_time_limit_ns")?
        || after.memory_oom_kills > 0
        || after.memory_max_events > 0
        || after.pids_max_events > 0
    {
        return Err("AGENT_RESOURCE_VIOLATION".into());
    }
    if bytes > number(profile, "agent_storage_limit_bytes")? || objects.len() > 4096 {
        return Err("AGENT_STORAGE_EXHAUSTED".into());
    }
    Ok(())
}

fn resident_bytes(pids: &[u32]) -> Result<u64, String> {
    pids.iter().try_fold(0_u64, |total, pid| {
        let text = fs::read_to_string(format!("/proc/{pid}/status"))
            .map_err(|e| format!("RSS_UNAVAILABLE:{e}"))?;
        let line = text
            .lines()
            .find_map(|line| line.strip_prefix("VmRSS:"))
            .ok_or("RSS_UNAVAILABLE")?;
        let mut fields = line.split_whitespace();
        let kib = fields
            .next()
            .ok_or("RSS_INVALID")?
            .parse::<u64>()
            .map_err(|e| e.to_string())?;
        if fields.next() != Some("kB") {
            return Err("RSS_UNIT_INVALID".into());
        }
        kib.checked_mul(1024)
            .and_then(|bytes| total.checked_add(bytes))
            .ok_or_else(|| "RSS_OVERFLOW".into())
    })
}
