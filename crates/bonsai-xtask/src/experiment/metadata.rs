use super::identity::file_hash;
use super::peer::hex;
use super::{Inputs, RunSummary, number};
use bonsai_runtime::IsolatedRunLayout;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub(super) fn prepare(inputs: &Inputs, layout: &IsolatedRunLayout) -> Result<(), String> {
    let [agent, environment] = super::execution_limits(inputs)?;
    let resolved = json!({"format":"bonsai.resolved-execution-policy/v1",
        "policy_id":inputs.manifest["manifest_id"],"agent_linux_controls":agent,"environment_linux_controls":environment,
        "agent_storage":{"maximum_files":4096,"replay_allowed":false},
        "observer_metadata_reserve_bytes":super::quota::METADATA_RESERVE,
        "unsupported_work_classes":["feature_generation","option_learning","model_learning","planning","curation"],
        "policy_scope":"fixed primitive runner; unsupported work is never dispatched",
        "decision_coverage":"v1 requires all five outcomes; equal soft/hard limits make soft-only states unreachable"});
    write(
        layout
            .observer_root()
            .join("resolved-execution-policy.json"),
        &resolved,
    )?;
    let policy = resource_policy(inputs)?;
    let typed: bonsai_contracts::resource::ResourcePolicy =
        serde_json::from_value(policy.clone()).map_err(|e| e.to_string())?;
    bonsai_contracts::resource::validate_resource_policy(&typed).map_err(|e| e.to_string())?;
    checked(
        layout.observer_root(),
        "resource-policy.json",
        "resource-policy-v1.json",
        &policy,
    )
}

pub(super) fn assemble(
    inputs: &Inputs,
    layout: &IsolatedRunLayout,
    summary: &RunSummary<'_>,
) -> Result<(), String> {
    let root = layout.observer_root();
    let track = json!({"schema_version":"1.0","declared_track":"A","runtime_facts_complete":false,
        "batch_size":1,"transition_access":"single_pass","replay_capacity_transitions":0,"offline_updates":false,
        "observer_data_access":false,"privileged_state":false,"human_labels":false,"domain_feature_targets":false,
        "update_schedule":"event_driven","fixed_external_budgets":true});
    checked(
        root,
        "track-declaration.json",
        "track-declaration-v1.json",
        &track,
    )?;
    let policy = resource_policy(inputs)?;
    let typed: bonsai_contracts::resource::ResourcePolicy =
        serde_json::from_value(policy.clone()).map_err(|e| e.to_string())?;
    bonsai_contracts::resource::validate_resource_policy(&typed).map_err(|e| e.to_string())?;
    checked(
        root,
        "resource-policy.json",
        "resource-policy-v1.json",
        &policy,
    )?;
    let failures = if summary.complete {
        json!([])
    } else {
        json!([{"code":summary.details["reason_code"],"fatal":true}])
    };
    write(root.join("failures.json"), &failures)?;
    let segment = "telemetry/segment-00000000000000000000.bseg";
    let digest = file_hash(&root.join(segment))?;
    metric(inputs, root, summary, &digest)?;
    let roles = [
        ("experiment-manifest.json", "experiment_manifest"),
        ("track-declaration.json", "track_declaration"),
        ("platform-inventory.json", "platform_inventory"),
        ("resource-policy.json", "resource_policy"),
        ("failures.json", "failure_log"),
        ("metric-estimate.json", "metric_estimate"),
        (segment, "event_segment"),
    ];
    let files=roles.into_iter().map(|(path,role)|Ok(json!({"path":path,"sha256":file_hash(&root.join(path))?,"role":role,"required":true})))
        .collect::<Result<Vec<_>,String>>()?;
    let bundle = json!({"format":"bonsai.bundle/v1","epoch":1,"minor":0,"bundle_id":inputs.manifest["run_id"],"files":files,
        "migration":{"status":"current","source_epoch":1,"registry_id":"bonsai.bundle-migrations/v1"}});
    checked(
        root,
        "bundle-manifest.json",
        "bundle-manifest-v1.json",
        &bundle,
    )?;
    let inventory: Value = serde_json::from_slice(
        &fs::read(root.join("platform-inventory.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    validate("platform-inventory-v1.json", &inventory)
}

fn resource_policy(inputs: &Inputs) -> Result<Value, String> {
    let profile = &inputs.manifest["resource_profile"];
    let mut limits = Vec::new();
    let mut allocations = Vec::new();
    for (class, counter, scope, unit, maximum) in [
        (
            "acting",
            "work_items",
            "per_step",
            "1",
            number(&inputs.manifest["adapter"]["config"], "action_count")?,
        ),
        ("learning", "work_items", "per_step", "1", 1),
        (
            "feature_generation",
            "unsupported_work_requests",
            "lifetime",
            "1",
            1,
        ),
        (
            "option_learning",
            "unsupported_work_requests",
            "lifetime",
            "1",
            1,
        ),
        (
            "model_learning",
            "unsupported_work_requests",
            "lifetime",
            "1",
            1,
        ),
        ("planning", "unsupported_work_requests", "lifetime", "1", 1),
        ("curation", "unsupported_work_requests", "lifetime", "1", 1),
        (
            "environment",
            "exchange_wall_time",
            "per_event",
            "ns",
            1_000_000_000,
        ),
        (
            "observer",
            "observer_output_bytes",
            "lifetime",
            "B",
            number(profile, "observer_output_limit_bytes")?,
        ),
    ] {
        let id = format!("{class}.{counter}");
        limits.push(json!({"limit_id":id,"work_class":class,"scope":scope,"counter_id":counter,"unit":unit,
            "soft_limit":maximum,"hard_limit":maximum,"basis_requirement":"measured","rolling_window":null}));
        allocations.push(json!({"work_class":class,"allocation_weight":1,"limit_ids":[id]}));
    }
    limits.push(json!({"limit_id":"acting.cpu","work_class":"acting","scope":"per_step","counter_id":"cpu_time_ns","unit":"ns",
        "soft_limit":number(profile,"per_step_cpu_time_limit_ns")?,"hard_limit":number(profile,"per_step_cpu_time_limit_ns")?,
        "basis_requirement":"measured","rolling_window":null}));
    allocations[0]["limit_ids"]
        .as_array_mut()
        .ok_or("POLICY_ALLOCATION_INVALID")?
        .push(json!("acting.cpu"));
    let maximum = number(&inputs.manifest["adapter"]["config"], "action_count")?
        .checked_mul(number(profile, "step_limit")?)
        .ok_or("WORK_OVERFLOW")?;
    limits.push(json!({"limit_id":"acting.rolling_work","work_class":"acting","scope":"rolling_window",
        "counter_id":"work_items","unit":"1","soft_limit":maximum,"hard_limit":maximum,
        "basis_requirement":"measured","rolling_window":{"duration_ns":number(profile,"wall_time_limit_ns")?}}));
    allocations[0]["limit_ids"]
        .as_array_mut()
        .ok_or("POLICY_ALLOCATION_INVALID")?
        .push(json!("acting.rolling_work"));
    Ok(
        json!({"schema_version":"1.0","policy_id":inputs.manifest["manifest_id"],"policy_version":"1.0",
        "resource_profile_id":profile["profile_id"],"limits":limits,"work_class_allocations":allocations,
        "decision_rules":[{"reason_code":"WITHIN_SOFT_LIMITS","limit_state":"within_soft","outcome":"admit"},
            {"reason_code":"SOFT_LIMIT_DEFER","limit_state":"soft_exceeded","outcome":"defer"},
            {"reason_code":"SOFT_LIMIT_THROTTLE","limit_state":"soft_exceeded","outcome":"throttle"},
            {"reason_code":"HARD_LIMIT_REJECT","limit_state":"hard_exceeded","outcome":"reject"},
            {"reason_code":"MEASUREMENT_UNAVAILABLE","limit_state":"basis_unavailable","outcome":"terminate"}]}),
    )
}

fn metric(
    inputs: &Inputs,
    root: &Path,
    summary: &RunSummary<'_>,
    segment_digest: &str,
) -> Result<(), String> {
    let expected = number(&inputs.manifest["resource_profile"], "step_limit")?;
    let observed = summary.rewards;
    if observed > expected {
        return Err("METRIC_POPULATION_INVALID".into());
    }
    let spec = json!({"metric_id":"behavior.reward","version":"1.0","aggregation":"sum","population":"accepted_reward_events"});
    let spec_hash = hex(&Sha256::digest(
        serde_json::to_vec(&spec).map_err(|e| e.to_string())?,
    ));
    write(root.join("reward-metric-definition.json"), &spec)?;
    let estimate_id = inputs.manifest["run_id"].clone();
    let uncertainty_id = inputs.manifest["manifest_id"].clone();
    let empty_hash = hex(&Sha256::digest(b"{}"));
    let coverage = f64::from(u32::try_from(observed).map_err(|e| e.to_string())?)
        / f64::from(u32::try_from(expected).map_err(|e| e.to_string())?);
    let result = if observed > 0 {
        json!({"kind":"scalar","value":summary.reward_sum,"unit":inputs.manifest["scenario"]["reward_unit_id"],"availability":"measured"})
    } else {
        json!({"kind":"unavailable","availability":"unavailable","reason_code":"no_accepted_rewards"})
    };
    let estimate = json!({"schema_version":"1.0","estimate_id":estimate_id,"run_id":inputs.manifest["run_id"],
        "metric_spec":{"metric_id":"behavior.reward","metric_version":"1.0","canonical_sha256":spec_hash},
        "population":{"population_id":"accepted_reward_events","eligible_count":expected,"observed_count":observed,"selection_sha256":segment_digest},
        "window":{"basis":"step_count","start":0,"end":expected},"result":result,
        "estimator":{"estimator_id":"exact_integer_sum","estimator_version":"1.0","parameters_sha256":empty_hash},
        "missingness":{"missing_count":expected-observed,"invalid_count":0,"coverage_ratio":coverage,
            "disposition":if observed==expected {"complete"} else if observed==0 {"unavailable"} else {"partial"},
            "reason_codes":if observed==expected {json!([])} else {json!(["incomplete_run"])}},
        "precision":{"representation":"int64","significant_bits":64,"absolute_tolerance":0,"relative_tolerance":0},
        "uncertainty_ids":[uncertainty_id],"inputs":[{"evidence_id":"event-segment-0","evidence_type":"event","sha256":segment_digest,"role":"accepted reward events"}]});
    checked(
        root,
        "metric-estimate.json",
        "metric-estimate-v1.json",
        &estimate,
    )?;
    let uncertainty = json!({"schema_version":"1.0","uncertainty_id":uncertainty_id,"estimate_id":estimate_id,
        "method":{"method_id":"descriptive_single_run","method_version":"1.0","parameters_sha256":empty_hash},
        "confidence_level":null,"sample_count":u64::from(observed>0),"result":{"kind":"unavailable","reason_code":"single_run_no_inference"},
        "inputs":[{"evidence_id":"event-segment-0","sha256":segment_digest,"role":"single run observations"}]});
    checked(
        root,
        "metric-uncertainty.json",
        "metric-uncertainty-v1.json",
        &uncertainty,
    )
}

fn checked(root: &Path, name: &str, schema: &str, value: &Value) -> Result<(), String> {
    validate(schema, value)?;
    write(root.join(name), value)
}
fn validate(schema: &str, value: &Value) -> Result<(), String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../schemas")
        .join(schema);
    let schema: Value = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    jsonschema::validator_for(&schema)
        .map_err(|e| e.to_string())?
        .validate(value)
        .map_err(|e| e.to_string())
}
fn write(path: impl AsRef<Path>, value: &Value) -> Result<(), String> {
    let path = path.as_ref();
    super::quota::write(
        path.parent().ok_or("OBSERVER_PATH_INVALID")?,
        path,
        &serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
}

// The v1 bundle role enum covers scientific inputs. This supplemental index
// also binds reports, lifecycle, identities, and uncertainty without inventing
// additional v1 roles. Durable run-status binds this index after finalization.
pub(super) fn artifact_index(layout: &IsolatedRunLayout) -> Result<String, String> {
    let root = layout.observer_root();
    let mut files = Vec::new();
    index_directory(root, root, &mut files, 0)?;
    files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    let value = json!({"format":"bonsai.run-artifact-index/v1","files":files,
        "excluded":["artifact-index.json","run-status.json","run-status.pending","run-receipt.json"]});
    let path = root.join("artifact-index.json");
    write(&path, &value)?;
    fs::File::open(&path)
        .and_then(|file| file.sync_all())
        .map_err(|e| e.to_string())?;
    fs::File::open(root)
        .and_then(|file| file.sync_all())
        .map_err(|e| e.to_string())?;
    file_hash(&path)
}

fn index_directory(
    root: &Path,
    directory: &Path,
    files: &mut Vec<Value>,
    depth: u8,
) -> Result<(), String> {
    if depth > 16 {
        return Err("OBSERVER_DEPTH_EXCEEDED".into());
    }
    for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if directory == root
            && [
                "artifact-index.json",
                "run-status.json",
                "run-status.pending",
                "run-receipt.json",
            ]
            .iter()
            .any(|name| entry.file_name() == *name)
        {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err("OBSERVER_SYMLINK_REJECTED".into());
        }
        if metadata.is_dir() {
            index_directory(root, &path, files, depth + 1)?;
        } else if metadata.is_file() {
            if files.len() >= 128 {
                return Err("OBSERVER_FILE_LIMIT_EXCEEDED".into());
            }
            files.push(json!({"path":path.strip_prefix(root).map_err(|e|e.to_string())?.to_str().ok_or("OBSERVER_PATH_INVALID")?,
                "bytes":metadata.len(),"sha256":file_hash(&path)?}));
        } else {
            return Err("OBSERVER_OUTPUT_NOT_REGULAR".into());
        }
    }
    Ok(())
}

/// The caller retains the printed digest outside the mutable result directory.
pub(super) fn receipt(inputs: &Inputs, layout: &IsolatedRunLayout) -> Result<String, String> {
    let root = layout.observer_root();
    let value = json!({"format":"bonsai.run-receipt/v1","run_id":inputs.manifest["run_id"],
        "index_sha256":file_hash(&root.join("artifact-index.json"))?,
        "status_sha256":file_hash(&root.join("run-status.json"))?});
    let path = root.join("run-receipt.json");
    write(&path, &value)?;
    file_hash(&path)
}
