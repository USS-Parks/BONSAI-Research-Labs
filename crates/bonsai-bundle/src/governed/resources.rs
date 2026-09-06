use super::context::Context;
use super::snapshot::Snapshot;
use super::{Result, ensure, number, text};
use serde_json::{Value, json};

pub(super) fn controls(snapshot: &Snapshot, context: &Context, event: &Value) -> Result<()> {
    for (key, path) in [
        ("policy_sha256", "resource-policy.json"),
        ("resolved_policy_sha256", "resolved-execution-policy.json"),
        ("inventory_sha256", "platform-inventory.json"),
    ] {
        super::trace::hash_matches(event, key, snapshot.bytes(path)?)?;
    }
    let resolved = snapshot.json("resolved-execution-policy.json")?;
    let profile = &context.manifest["resource_profile"];
    ensure(
        resolved["format"] == "bonsai.resolved-execution-policy/v1"
            && resolved["policy_id"] == context.manifest["manifest_id"]
            && resolved["agent_storage"] == json!({"maximum_files":4096,"replay_allowed":false})
            && resolved["observer_metadata_reserve_bytes"] == 131_072,
        "RUN_RESOLVED_POLICY_INVALID",
    )?;
    let suffix = snapshot.run_id.replace('-', "");
    let mut parent = None;
    for (name, maximum) in [
        ("agent", number(profile, "agent_rss_limit_bytes")?),
        ("environment", 134_217_728),
    ] {
        let recorded = &event[name];
        let expected = json!({"cpu_quota_usec":100_000,"cpu_period_usec":100_000,
            "memory_max_bytes":maximum,"pids_max":32});
        ensure(
            recorded["limits"] == expected
                && resolved[format!("{name}_linux_controls")] == expected
                && recorded["cpu_max"] == "100000 100000"
                && recorded["memory_max"] == maximum
                && recorded["memory_swap_max"] == 0
                && recorded["memory_oom_group"] == 1
                && recorded["pids_max"] == 32,
            "RUN_CONTROLLER_MISSING_OR_CHANGED",
        )?;
        let path = text(recorded, "cgroup_path")?;
        let ending = format!("/bonsai-{name}-{suffix}");
        let root = path
            .strip_suffix(&ending)
            .ok_or("RUN_CONTROLLER_IDENTITY_INVALID")?;
        ensure(
            root.starts_with("/sys/fs/cgroup/") && !root.contains("/../"),
            "RUN_CONTROLLER_IDENTITY_INVALID",
        )?;
        if let Some(prior) = &parent {
            ensure(prior == root, "RUN_CONTROLLER_PARENT_MISMATCH")?;
        }
        parent = Some(root.to_owned());
    }
    policy(snapshot, context)
}

fn policy(snapshot: &Snapshot, context: &Context) -> Result<()> {
    let value = snapshot.json("resource-policy.json")?;
    let typed = serde_json::from_value(value.clone()).map_err(|_| "RUN_RESOURCE_POLICY_INVALID")?;
    bonsai_contracts::resource::validate_resource_policy(&typed)
        .map_err(|_| "RUN_RESOURCE_POLICY_INVALID")?;
    let profile = &context.manifest["resource_profile"];
    ensure(
        value["policy_id"] == context.manifest["manifest_id"]
            && value["policy_version"] == "1.0"
            && value["resource_profile_id"] == profile["profile_id"],
        "RUN_RESOURCE_POLICY_IDENTITY_INVALID",
    )?;
    let limits = value["limits"]
        .as_array()
        .ok_or("RUN_RESOURCE_POLICY_INVALID")?;
    ensure(limits.len() == 11, "RUN_RESOURCE_POLICY_UNSUPPORTED")?;
    let specifications = specifications(context)?;
    for (id, class, scope, counter, unit, maximum) in specifications {
        let matching = limits
            .iter()
            .filter(|limit| limit["limit_id"] == id)
            .collect::<Vec<_>>();
        ensure(matching.len() == 1, "RUN_POLICY_LIMIT_MISSING")?;
        let rolling = if scope == "rolling_window" {
            json!({"duration_ns":number(profile,"wall_time_limit_ns")?})
        } else {
            Value::Null
        };
        ensure(
            *matching[0]
                == json!({"limit_id":id,"work_class":class,"scope":scope,
            "counter_id":counter,"unit":unit,"soft_limit":maximum,"hard_limit":maximum,
            "basis_requirement":"measured","rolling_window":rolling}),
            "RUN_POLICY_LIMIT_SUBSTITUTED",
        )?;
    }
    Ok(())
}

pub(super) fn step(context: &Context, event: &Value, total: u64, prior_cpu: u64) -> Result<u64> {
    ensure(
        number(event, "total_step")? == total,
        "RUN_RESOURCE_STEP_MISMATCH",
    )?;
    let profile = &context.manifest["resource_profile"];
    let usage = &event["usage"];
    let before = number(event, "cpu_usage_before_usec")?;
    let after = number(usage, "cpu_usage_usec")?;
    let cpu = after
        .checked_sub(before)
        .and_then(|delta| delta.checked_mul(1000))
        .ok_or("RUN_CPU_COUNTER_INVALID")?;
    ensure(
        before >= prior_cpu
            && cpu == number(event, "cpu_time_ns")?
            && cpu <= number(profile, "per_step_cpu_time_limit_ns")?
            && number(event, "agent_rss_bytes")? > 0
            && number(event, "agent_rss_bytes")? <= number(profile, "agent_rss_limit_bytes")?
            && number(event, "agent_storage_bytes")?
                <= number(profile, "agent_storage_limit_bytes")?
            && number(event, "agent_storage_objects")? <= 4096,
        "RUN_RESOURCE_VIOLATION",
    )?;
    healthy_usage(usage)?;
    ensure(
        usage["populated"] == true
            && usage["member_pids"]
                .as_array()
                .is_some_and(|pids| !pids.is_empty())
            && number(usage, "pids_current")? > 0
            && number(usage, "memory_current_bytes")? <= number(profile, "agent_rss_limit_bytes")?,
        "RUN_RESOURCE_COUNTER_UNAVAILABLE",
    )?;
    Ok(after)
}

pub(super) fn healthy_usage(usage: &Value) -> Result<()> {
    for counter in ["memory_oom_kills", "memory_max_events", "pids_max_events"] {
        ensure(
            number(usage, counter)? == 0,
            "RUN_KERNEL_RESOURCE_VIOLATION",
        )?;
    }
    ensure(
        number(usage, "pids_current")? <= 32,
        "RUN_PIDS_LIMIT_VIOLATION",
    )
}

pub(super) fn work(context: &Context, event: &Value, total: u64, acting: bool) -> Result<()> {
    let (class, amount) = if acting {
        ("acting", context.actions)
    } else {
        ("learning", 1)
    };
    ensure(
        event["work_class"] == class && number(event, "total_step")? == total,
        "RUN_WORK_ORDER_INVALID",
    )?;
    let mut projection = vec![json!({"consumed_before":0,"hard_limit":amount,
        "limit_id":format!("{class}.work_items"),"projected":amount,"requested":amount,
        "scope":"per_step","soft_limit":amount,"state":"within_soft"})];
    if acting {
        projection.push(json!({"consumed_before":total * context.actions,"hard_limit":context.steps * context.actions,
            "limit_id":"acting.rolling_work","projected":(total+1)*context.actions,"requested":context.actions,
            "scope":"rolling_window","soft_limit":context.steps*context.actions,"state":"within_soft"}));
    }
    ensure(
        event["projection"] == Value::Array(projection),
        "RUN_WORK_PROJECTION_INVALID",
    )
}

type LimitSpec = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    u64,
);
fn specifications(context: &Context) -> Result<[LimitSpec; 11]> {
    let profile = &context.manifest["resource_profile"];
    Ok([
        (
            "acting.work_items",
            "acting",
            "per_step",
            "work_items",
            "1",
            context.actions,
        ),
        (
            "learning.work_items",
            "learning",
            "per_step",
            "work_items",
            "1",
            1,
        ),
        (
            "feature_generation.unsupported_work_requests",
            "feature_generation",
            "lifetime",
            "unsupported_work_requests",
            "1",
            1,
        ),
        (
            "option_learning.unsupported_work_requests",
            "option_learning",
            "lifetime",
            "unsupported_work_requests",
            "1",
            1,
        ),
        (
            "model_learning.unsupported_work_requests",
            "model_learning",
            "lifetime",
            "unsupported_work_requests",
            "1",
            1,
        ),
        (
            "planning.unsupported_work_requests",
            "planning",
            "lifetime",
            "unsupported_work_requests",
            "1",
            1,
        ),
        (
            "curation.unsupported_work_requests",
            "curation",
            "lifetime",
            "unsupported_work_requests",
            "1",
            1,
        ),
        (
            "environment.exchange_wall_time",
            "environment",
            "per_event",
            "exchange_wall_time",
            "ns",
            1_000_000_000,
        ),
        (
            "observer.observer_output_bytes",
            "observer",
            "lifetime",
            "observer_output_bytes",
            "B",
            number(profile, "observer_output_limit_bytes")?,
        ),
        (
            "acting.cpu",
            "acting",
            "per_step",
            "cpu_time_ns",
            "ns",
            number(profile, "per_step_cpu_time_limit_ns")?,
        ),
        (
            "acting.rolling_work",
            "acting",
            "rolling_window",
            "work_items",
            "1",
            context.actions * context.steps,
        ),
    ])
}
