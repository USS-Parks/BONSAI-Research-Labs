use super::snapshot::{Snapshot, digest};
use super::{Result, ensure, number, text};
use crate::{BundleSchemas, CheckStatus};
use bonsai_contracts::accounting::OnlineAccounting;
use serde_json::{Value, json};

pub(super) struct Context {
    pub(super) manifest: Value,
    pub(super) accounting: OnlineAccounting,
    pub(super) steps: u64,
    pub(super) actions: u64,
    pub(super) seed: u64,
}

impl Context {
    pub(super) fn load(snapshot: &Snapshot, schemas: &BundleSchemas) -> Result<Self> {
        for (name, schema) in [
            ("bundle-manifest.json", &schemas.bundle_manifest),
            ("experiment-manifest.json", &schemas.experiment_manifest),
            ("track-declaration.json", &schemas.track_declaration),
            ("platform-inventory.json", &schemas.platform_inventory),
            ("resource-policy.json", &schemas.resource_policy),
            ("metric-estimate.json", &schemas.metric_estimate),
        ] {
            let check = crate::validation::validate_value_schema(
                &snapshot.json(name)?,
                schema,
                "RUN_SCHEMA_INVALID",
            )
            .map_err(|_| "RUN_VERIFIER_SCHEMA_INVALID")?;
            ensure(check.status == CheckStatus::Pass, "RUN_SCHEMA_INVALID")?;
        }
        let manifest = snapshot.json("experiment-manifest.json")?;
        ensure(
            manifest["run_id"] == snapshot.run_id
                && manifest["track"]["declared_track"] == "A"
                && snapshot.json("track-declaration.json")?["declared_track"] == "A",
            "RUN_TRACK_UNSUPPORTED",
        )?;
        let steps = profile(&manifest["resource_profile"])?;
        let actions = number(&manifest["adapter"]["config"], "action_count")?;
        ensure(
            (2..=256).contains(&actions)
                && manifest["scenario"]["config"] == manifest["environment"]["config"]
                && number(&manifest["environment"]["config"], "action_count")? == actions,
            "RUN_CONFIGURATION_UNSUPPORTED",
        )?;
        let seeds = manifest["seeds"].as_array().ok_or("RUN_SEED_INVALID")?;
        ensure(seeds.len() == 1, "RUN_SEED_INVALID")?;
        let seed = text(&seeds[0], "value")?
            .parse::<u64>()
            .map_err(|_| "RUN_SEED_INVALID")?;
        number(&manifest["environment"]["config"], "horizon")?;
        ensure(seed.checked_add(steps).is_some(), "RUN_SEED_OVERFLOW")?;
        let accounting = OnlineAccounting::from_declaration_or_legacy(
            manifest["adapter"].get("accounting_contract"),
        )?;
        let context = Self {
            manifest,
            accounting,
            steps,
            actions,
            seed,
        };
        Self::check_roles(snapshot)?;
        context.check_source(snapshot)?;
        context.check_status(snapshot)?;
        context.check_counters()?;
        Ok(context)
    }

    fn check_roles(snapshot: &Snapshot) -> Result<()> {
        let bundle = snapshot.json("bundle-manifest.json")?;
        ensure(
            bundle["bundle_id"] == snapshot.run_id
                && bundle["format"] == "bonsai.bundle/v1"
                && bundle["epoch"] == 1
                && bundle["minor"] == 0,
            "RUN_BUNDLE_IDENTITY_INVALID",
        )?;
        let files = bundle["files"]
            .as_array()
            .ok_or("RUN_BUNDLE_ROLES_INVALID")?;
        let roles = [
            ("experiment-manifest.json", "experiment_manifest"),
            ("track-declaration.json", "track_declaration"),
            ("platform-inventory.json", "platform_inventory"),
            ("resource-policy.json", "resource_policy"),
            ("failures.json", "failure_log"),
            ("metric-estimate.json", "metric_estimate"),
            (super::trace::SEGMENT, "event_segment"),
        ];
        ensure(files.len() == roles.len(), "RUN_BUNDLE_ROLES_INVALID")?;
        for (path, role) in roles {
            let matching = files
                .iter()
                .filter(|file| file["role"] == role)
                .collect::<Vec<_>>();
            ensure(matching.len() == 1, "RUN_BUNDLE_ROLES_INVALID")?;
            let file = matching[0];
            ensure(
                file["path"] == path
                    && file["required"] == true
                    && file["sha256"] == digest(snapshot.bytes(path)?),
                "RUN_BUNDLE_ROLE_HASH_INVALID",
            )?;
        }
        Ok(())
    }

    fn check_source(&self, snapshot: &Snapshot) -> Result<()> {
        let identity = snapshot.json("source-identity.json")?;
        let source = &self.manifest["source"];
        ensure(
            identity["schema"] == "bonsai.run-source-identity/v1"
                && identity["revision"] == source["revision"]
                && identity["dirty"] == source["dirty"],
            "RUN_SOURCE_IDENTITY_INVALID",
        )?;
        if source["dirty"] == true {
            ensure(
                identity["tracked_patch_sha256"] == source["dirty_patch_sha256"],
                "RUN_SOURCE_PATCH_MISMATCH",
            )?;
        }
        super::trace::unhex(text(&identity, "operator_executable_sha256")?)?
            .len()
            .eq(&32)
            .then_some(())
            .ok_or("RUN_OPERATOR_IDENTITY_INVALID")?;
        if source["dirty"] == false {
            ensure(
                identity["tracked_patch_sha256"] == digest(b""),
                "RUN_CLEAN_SOURCE_PATCH_PRESENT",
            )?;
        }
        let inventory = snapshot.json("platform-inventory.json")?;
        ensure(
            inventory["os"]["family"] == "linux",
            "RUN_CONTROLLER_PLATFORM_UNSUPPORTED",
        )?;
        measured_inventory(&inventory)?;
        let locks = inventory["dependency_locks"]
            .as_array()
            .ok_or("RUN_LOCKS_MISSING")?;
        for name in ["Cargo.lock", "uv.lock"] {
            let matches = locks
                .iter()
                .filter(|lock| lock["lockfile_name"] == name)
                .collect::<Vec<_>>();
            ensure(
                matches.len() == 1
                    && matches[0]["sha256"] == identity["source_files"][name]
                    && identity["source_files"][name]
                        .as_str()
                        .is_some_and(|v| v.len() == 64),
                "RUN_LOCK_IDENTITY_MISMATCH",
            )?;
        }
        super::reference::check(snapshot, &self.manifest, &identity)?;
        Ok(())
    }

    fn check_status(&self, snapshot: &Snapshot) -> Result<()> {
        let status = snapshot.json("run-status.json")?;
        ensure(
            status["status"] == "COMPLETE"
                && status["artifact_index_sha256"] == snapshot.index_sha256
                && snapshot.json("failures.json")? == json!([]),
            "RUN_NOT_COMPLETE",
        )?;
        ensure(
            number(&status, "execution_ns")?
                < number(&self.manifest["resource_profile"], "wall_time_limit_ns")?
                && snapshot.total_bytes
                    <= number(
                        &self.manifest["resource_profile"],
                        "observer_output_limit_bytes",
                    )?,
            "RUN_RESOURCE_BOUND_EXCEEDED",
        )?;
        let records = std::str::from_utf8(snapshot.bytes("lifecycle/lifecycle.jsonl")?)
            .map_err(|_| "RUN_LIFECYCLE_INVALID")?
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).map_err(|_| "RUN_LIFECYCLE_INVALID"))
            .collect::<Result<Vec<_>>>()?;
        ensure(records.len() == 4, "RUN_LIFECYCLE_INVALID")?;
        for (ordinal, state) in ["created", "running", "terminating", "completed"]
            .iter()
            .enumerate()
        {
            ensure(
                records[ordinal]["ordinal"] == ordinal
                    && records[ordinal]["state"] == *state
                    && records[ordinal]["reason_code"].is_null(),
                "RUN_LIFECYCLE_INVALID",
            )?;
        }
        Ok(())
    }

    fn check_counters(&self) -> Result<()> {
        let counters = self.manifest["expected_counters"]
            .as_array()
            .ok_or("RUN_COUNTERS_MISSING")?;
        for counter in counters {
            let unit = match text(counter, "counter_id")? {
                "process_cpu_time" => "ns",
                "agent_rss" | "agent_storage" => "B",
                "environment_steps" | "work_items" => "1",
                _ => return Err("RUN_COUNTER_UNSUPPORTED"),
            };
            ensure(
                counter["unit"] == unit && counter["acceptable_basis"] == "measured",
                "RUN_COUNTER_BASIS_UNSUPPORTED",
            )?;
        }
        ensure(
            self.manifest["metrics"]
                == json!([{
                    "metric_id":"behavior.reward", "version":"1.0", "required":true, "parameters":{}
                }]),
            "RUN_METRIC_UNSUPPORTED",
        )
    }
}

fn profile(value: &Value) -> Result<u64> {
    ensure(
        value["energy_tier"] == "E0"
            && value["energy_budget_uj"].is_null()
            && (value["profile_id"] == "S" || value["profile_id"] == "custom"),
        "RUN_PROFILE_UNSUPPORTED",
    )?;
    for (key, maximum) in [
        ("step_limit", 2000),
        ("wall_time_limit_ns", 120_000_000_000),
        ("agent_rss_limit_bytes", 1_073_741_824),
        ("agent_storage_limit_bytes", 67_108_864),
        ("observer_output_limit_bytes", 536_870_912),
        ("per_step_cpu_time_limit_ns", 10_000_000),
        ("action_deadline_ns", 50_000_000),
    ] {
        let actual = number(value, key)?;
        ensure(
            actual > 0 && actual <= maximum && (value["profile_id"] != "S" || actual == maximum),
            "RUN_PROFILE_LIMIT_UNSUPPORTED",
        )?;
    }
    number(value, "step_limit")
}

fn measured_inventory(value: &Value) -> Result<()> {
    let collectors = value["collectors"]
        .as_array()
        .ok_or("RUN_INVENTORY_UNAVAILABLE")?;
    let collector = collectors
        .iter()
        .find(|v| v["collector_id"] == "linux_proc_inventory")
        .ok_or("RUN_INVENTORY_UNAVAILABLE")?;
    ensure(
        collector["status"] == "available"
            && collector["capabilities"]
                == json!(["cpu_topology", "guest_memory", "monotonic_clock"])
            && number(&value["cpu"], "logical_cores")? > 0
            && number(&value["memory"], "physical_total_bytes")? > 0,
        "RUN_INVENTORY_UNAVAILABLE",
    )?;
    let clocks = value["clocks"].as_array().ok_or("RUN_CLOCK_UNAVAILABLE")?;
    ensure(
        clocks.iter().any(|clock| {
            clock["clock_id"] == "rust_instant"
                && clock["monotonic"] == true
                && clock["resolution_ns"].as_u64().is_some_and(|n| n > 0)
        }),
        "RUN_CLOCK_UNAVAILABLE",
    )
}
