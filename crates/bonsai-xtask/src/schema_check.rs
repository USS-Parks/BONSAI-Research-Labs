//! Frozen schema-compatibility conformance suite.

use bonsai_contracts::inventory::{PlatformInventory, sanitize_inventory_json};
use bonsai_contracts::resource::{ResourcePolicy, validate_resource_policy};
use bonsai_contracts::track::{Track, TrackDeclaration, derive_track};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

const CATALOG_SCHEMA: &str = "bonsai.schema-catalog/v1";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    schema: String,
    epoch: u64,
    minor: u64,
    protobuf_messages: Vec<ProtoMessage>,
    json_schemas: Vec<JsonSchema>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtoMessage {
    name: String,
    fields: Vec<ProtoField>,
    #[serde(default)]
    reserved_numbers: Vec<u32>,
    #[serde(default)]
    reserved_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtoField {
    number: u32,
    name: String,
    #[serde(rename = "type")]
    field_type: String,
    presence: String,
    unit: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct JsonSchema {
    name: String,
    uri: String,
    version: Option<String>,
    properties: Vec<JsonProperty>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct JsonProperty {
    name: String,
    #[serde(rename = "type")]
    property_type: String,
    required: bool,
    unit: Option<String>,
}

#[derive(Debug)]
struct CompatibilityError {
    code: &'static str,
    path: String,
    detail: String,
}

struct FixtureCase {
    file: &'static str,
    compatible: bool,
    expected_error: Option<&'static str>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestRejectionFixture {
    fixture_base: String,
    remove: String,
    expected_error: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContractRejectionFixture {
    fixture_base: String,
    remove: String,
    expected_error: String,
    expected_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrackFixture {
    name: String,
    input: Value,
    expected: Track,
}

pub(crate) fn run() -> Result<(), String> {
    let root = workspace_root();
    let fixture_dir = root.join("fixtures/schema-compatibility/v1");
    let baseline = load_catalog(&fixture_dir.join("baseline.json"))?;
    let baseline_errors = validate_catalog(&baseline);
    if !baseline_errors.is_empty() {
        return Err(format_errors(
            "baseline fixture is invalid",
            &baseline_errors,
        ));
    }

    let cases = [
        FixtureCase {
            file: "additive.json",
            compatible: true,
            expected_error: None,
        },
        FixtureCase {
            file: "field-renumbering.json",
            compatible: false,
            expected_error: Some("FIELD_RENUMBERED"),
        },
        FixtureCase {
            file: "field-reuse.json",
            compatible: false,
            expected_error: Some("FIELD_REUSE"),
        },
        FixtureCase {
            file: "silent-unit-change.json",
            compatible: false,
            expected_error: Some("UNIT_CHANGED"),
        },
        FixtureCase {
            file: "unversioned-json.json",
            compatible: false,
            expected_error: Some("JSON_VERSION_MISSING"),
        },
    ];

    for case in cases {
        let path = fixture_dir.join(case.file);
        let raw = fs::read(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
        let candidate = parse_catalog(&path, &raw)?;
        let mut errors = validate_catalog(&candidate);
        errors.extend(compare_catalogs(&baseline, &candidate));
        let observed_compatible = errors.is_empty();
        if observed_compatible != case.compatible {
            return Err(format_errors(
                &format!(
                    "{} expected compatible={} but observed compatible={observed_compatible}",
                    case.file, case.compatible
                ),
                &errors,
            ));
        }
        if let Some(expected) = case.expected_error
            && !errors.iter().any(|error| error.code == expected)
        {
            return Err(format_errors(
                &format!("{} did not produce expected error {expected}", case.file),
                &errors,
            ));
        }
        let digest = Sha256::digest(canonical_json(&raw)?);
        let mut digest_hex = String::with_capacity(64);
        for byte in digest {
            write!(digest_hex, "{byte:02x}")
                .map_err(|error| format!("encode canonical digest: {error}"))?;
        }
        println!(
            "schema fixture {}: {} canonical_sha256={digest_hex}",
            case.file,
            if observed_compatible {
                "compatible"
            } else {
                case.expected_error.unwrap_or("incompatible")
            }
        );
    }

    run_experiment_manifest_suite(&root)?;
    run_platform_inventory_suite(&root)?;
    run_track_suite(&root)?;
    run_resource_policy_suite(&root)?;
    run_metric_contract_suite(&root)?;

    println!(
        "schema check passed: compatibility, manifest, inventory, derived track, resource policy, metric, uncertainty, and claim-result suites"
    );
    Ok(())
}

// Keeping the four schemas and their exact negative matrix in one routine
// makes coverage reviewable as one BC-08 gate rather than distributed setup.
#[allow(clippy::too_many_lines)]
fn run_metric_contract_suite(root: &Path) -> Result<(), String> {
    let fixture_dir = root.join("fixtures/metric-contracts/v1");
    let contracts = [
        (
            "metric-spec",
            "schemas/metric-spec-v1.json",
            "valid-metric-spec.json",
        ),
        (
            "metric-estimate",
            "schemas/metric-estimate-v1.json",
            "valid-metric-estimate.json",
        ),
        (
            "metric-uncertainty",
            "schemas/metric-uncertainty-v1.json",
            "valid-metric-uncertainty.json",
        ),
        (
            "claim-result",
            "schemas/claim-result-v1.json",
            "valid-claim-result.json",
        ),
    ];

    let mut schemas = HashMap::new();
    let mut fixtures = HashMap::new();
    for (name, schema_relative, fixture_name) in contracts {
        let schema_path = root.join(schema_relative);
        let schema_raw = fs::read(&schema_path)
            .map_err(|error| format!("read {}: {error}", schema_path.display()))?;
        let schema: Value = serde_json::from_slice(&schema_raw)
            .map_err(|error| format!("parse {}: {error}", schema_path.display()))?;
        jsonschema::draft202012::meta::validate(&schema)
            .map_err(|error| format!("{name} is not valid Draft 2020-12: {error}"))?;
        reject_schema_defaults(&schema, "")?;
        let validator = jsonschema::draft202012::new(&schema)
            .map_err(|error| format!("compile {name} schema: {error}"))?;

        let fixture_path = fixture_dir.join(fixture_name);
        let fixture_raw = fs::read(&fixture_path)
            .map_err(|error| format!("read {}: {error}", fixture_path.display()))?;
        let fixture: Value = serde_json::from_slice(&fixture_raw)
            .map_err(|error| format!("parse {}: {error}", fixture_path.display()))?;
        validator
            .validate(&fixture)
            .map_err(|error| format!("valid {name} fixture failed: {error}"))?;
        let canonical = canonical_json(&fixture_raw)?;
        let windows_text = String::from_utf8(fixture_raw)
            .map_err(|_| format!("valid {name} fixture is not UTF-8"))?
            .replace('\n', "\r\n");
        if canonical != canonical_json(windows_text.as_bytes())? {
            return Err(format!(
                "{name} canonical bytes differ for LF and CRLF input"
            ));
        }
        println!(
            "{name} {fixture_name}: valid canonical_sha256={} schema_canonical_sha256={}",
            sha256_hex(&canonical),
            sha256_hex(&canonical_json(&schema_raw)?)
        );
        schemas.insert(fixture_name, schema);
        fixtures.insert(fixture_name, fixture);
    }

    validate_metric_fixture_links(&fixtures)?;

    let claim_schema = schemas
        .get("valid-claim-result.json")
        .ok_or_else(|| "claim-result schema fixture mapping missing".to_owned())?;
    let claim_validator = jsonschema::draft202012::new(claim_schema)
        .map_err(|error| format!("compile claim-result schema: {error}"))?;
    let base_claim = fixtures
        .get("valid-claim-result.json")
        .ok_or_else(|| "valid claim-result fixture missing".to_owned())?;
    for verdict in ["pass", "fail", "indeterminate", "not_run"] {
        let mut claim = base_claim.clone();
        claim["verdict"] = Value::String(verdict.to_owned());
        claim_validator
            .validate(&claim)
            .map_err(|error| format!("claim verdict {verdict} failed schema: {error}"))?;
    }

    let rejection_cases = [
        "missing-scalar-unit.json",
        "missing-scalar-provenance.json",
        "missing-claim-criterion.json",
        "missing-claim-evidence.json",
        "missing-claim-reasons.json",
    ];
    for file in rejection_cases {
        let path = fixture_dir.join(file);
        let raw = fs::read(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
        let rejection: ContractRejectionFixture = serde_json::from_slice(&raw)
            .map_err(|error| format!("parse {}: {error}", path.display()))?;
        let base = fixtures
            .get(rejection.fixture_base.as_str())
            .ok_or_else(|| format!("{file} names unknown fixture base"))?;
        let schema = schemas
            .get(rejection.fixture_base.as_str())
            .ok_or_else(|| format!("{file} has no schema mapping"))?;
        let validator = jsonschema::draft202012::new(schema)
            .map_err(|error| format!("compile rejection schema for {file}: {error}"))?;
        let mut candidate = base.clone();
        remove_json_pointer(&mut candidate, &rejection.remove)
            .map_err(|error| format!("apply {file}: {error}"))?;
        let observed_errors = validator
            .iter_errors(&candidate)
            .map(|error| error.to_string())
            .collect::<Vec<_>>();
        if observed_errors.is_empty()
            || !observed_errors.iter().any(|error| {
                error
                    .to_ascii_lowercase()
                    .contains(&rejection.expected_error)
            })
        {
            return Err(format!(
                "{file} expected {} containing {:?}, observed: {}",
                rejection.expected_code,
                rejection.expected_error,
                observed_errors.join("; ")
            ));
        }
        println!("metric/claim fixture {file}: {}", rejection.expected_code);
    }
    Ok(())
}

fn validate_metric_fixture_links(fixtures: &HashMap<&str, Value>) -> Result<(), String> {
    let spec = fixtures
        .get("valid-metric-spec.json")
        .ok_or_else(|| "valid metric specification missing".to_owned())?;
    let estimate = fixtures
        .get("valid-metric-estimate.json")
        .ok_or_else(|| "valid metric estimate missing".to_owned())?;
    let uncertainty = fixtures
        .get("valid-metric-uncertainty.json")
        .ok_or_else(|| "valid metric uncertainty missing".to_owned())?;
    let claim = fixtures
        .get("valid-claim-result.json")
        .ok_or_else(|| "valid claim result missing".to_owned())?;

    let spec_sha256 = sha256_hex(&canonical_json(
        &serde_json::to_vec(spec).map_err(|error| format!("serialize metric spec: {error}"))?,
    )?);
    if estimate
        .pointer("/metric_spec/canonical_sha256")
        .and_then(Value::as_str)
        != Some(spec_sha256.as_str())
        || estimate.pointer("/metric_spec/metric_id") != spec.get("metric_id")
        || estimate.pointer("/metric_spec/metric_version") != spec.get("metric_version")
        || estimate.pointer("/result/unit") != spec.get("unit")
    {
        return Err(
            "metric estimate does not bind the exact specification identity/unit".to_owned(),
        );
    }

    let estimate_id = estimate
        .get("estimate_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "valid estimate identity missing".to_owned())?;
    let uncertainty_id = uncertainty
        .get("uncertainty_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "valid uncertainty identity missing".to_owned())?;
    if uncertainty.get("estimate_id").and_then(Value::as_str) != Some(estimate_id)
        || !estimate
            .get("uncertainty_ids")
            .and_then(Value::as_array)
            .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(uncertainty_id)))
        || uncertainty.pointer("/result/unit") != estimate.pointer("/result/unit")
    {
        return Err("uncertainty record does not bind the estimate identity/unit".to_owned());
    }
    let lower = uncertainty
        .pointer("/result/lower_bound")
        .and_then(Value::as_f64)
        .ok_or_else(|| "valid uncertainty lower bound missing".to_owned())?;
    let upper = uncertainty
        .pointer("/result/upper_bound")
        .and_then(Value::as_f64)
        .ok_or_else(|| "valid uncertainty upper bound missing".to_owned())?;
    let value = estimate
        .pointer("/result/value")
        .and_then(Value::as_f64)
        .ok_or_else(|| "valid metric scalar missing".to_owned())?;
    if lower > value || value > upper {
        return Err("uncertainty interval does not contain the metric estimate".to_owned());
    }

    let evidence_ids = claim
        .get("evidence")
        .and_then(Value::as_array)
        .ok_or_else(|| "valid claim evidence missing".to_owned())?
        .iter()
        .filter_map(|item| item.get("evidence_id").and_then(Value::as_str))
        .collect::<HashSet<_>>();
    if !evidence_ids.contains(estimate_id) || !evidence_ids.contains(uncertainty_id) {
        return Err("claim result does not cite its metric estimate and uncertainty".to_owned());
    }
    Ok(())
}

fn run_resource_policy_suite(root: &Path) -> Result<(), String> {
    let schema_path = root.join("schemas/resource-policy-v1.json");
    let schema_raw = fs::read(&schema_path)
        .map_err(|error| format!("read {}: {error}", schema_path.display()))?;
    let schema: Value = serde_json::from_slice(&schema_raw)
        .map_err(|error| format!("parse {}: {error}", schema_path.display()))?;
    jsonschema::draft202012::meta::validate(&schema)
        .map_err(|error| format!("resource policy is not valid Draft 2020-12: {error}"))?;
    reject_schema_defaults(&schema, "")?;
    let validator = jsonschema::draft202012::new(&schema)
        .map_err(|error| format!("compile resource policy schema: {error}"))?;

    let fixture_path = root.join("fixtures/resource-policy/v1/valid.json");
    let fixture_raw = fs::read(&fixture_path)
        .map_err(|error| format!("read {}: {error}", fixture_path.display()))?;
    let fixture: Value = serde_json::from_slice(&fixture_raw)
        .map_err(|error| format!("parse {}: {error}", fixture_path.display()))?;
    validator
        .validate(&fixture)
        .map_err(|error| format!("valid resource policy fixture failed schema: {error}"))?;
    let policy: ResourcePolicy = serde_json::from_value(fixture.clone())
        .map_err(|error| format!("decode resource policy fixture: {error}"))?;
    validate_resource_policy(&policy)
        .map_err(|error| format!("resource policy semantic validation failed: {error}"))?;

    let canonical = canonical_json(&fixture_raw)?;
    let windows_text = String::from_utf8(fixture_raw.clone())
        .map_err(|_| "valid resource policy fixture is not UTF-8".to_owned())?
        .replace('\n', "\r\n");
    if canonical != canonical_json(windows_text.as_bytes())? {
        return Err("resource policy canonical bytes differ for LF and CRLF input".to_owned());
    }

    let mut invalid_limits = policy.clone();
    invalid_limits.limits[0].soft_limit = invalid_limits.limits[0].hard_limit + 1;
    if validate_resource_policy(&invalid_limits).is_ok() {
        return Err("resource policy accepted a soft limit above its hard limit".to_owned());
    }
    let mut incomplete_allocations = policy.clone();
    incomplete_allocations.work_class_allocations.pop();
    if validate_resource_policy(&incomplete_allocations).is_ok() {
        return Err("resource policy accepted incomplete work-class allocation".to_owned());
    }

    println!(
        "resource policy valid.json: valid scopes=4 work_classes=9 outcomes=5 canonical_sha256={} schema_canonical_sha256={}",
        sha256_hex(&canonical),
        sha256_hex(&canonical_json(&schema_raw)?)
    );
    Ok(())
}

fn run_track_suite(root: &Path) -> Result<(), String> {
    let schema_path = root.join("schemas/track-declaration-v1.json");
    let schema_raw = fs::read(&schema_path)
        .map_err(|error| format!("read {}: {error}", schema_path.display()))?;
    let schema: Value = serde_json::from_slice(&schema_raw)
        .map_err(|error| format!("parse {}: {error}", schema_path.display()))?;
    jsonschema::draft202012::meta::validate(&schema)
        .map_err(|error| format!("track declaration is not valid Draft 2020-12: {error}"))?;
    reject_schema_defaults(&schema, "")?;
    let validator = jsonschema::draft202012::new(&schema)
        .map_err(|error| format!("compile track declaration schema: {error}"))?;
    let path = root.join("fixtures/track-classification/v1/cases.json");
    let raw = fs::read(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let fixtures: Vec<TrackFixture> = serde_json::from_slice(&raw)
        .map_err(|error| format!("parse {}: {error}", path.display()))?;
    for fixture in &fixtures {
        validator
            .validate(&fixture.input)
            .map_err(|error| format!("track fixture {} failed schema: {error}", fixture.name))?;
        let input: TrackDeclaration = serde_json::from_value(fixture.input.clone())
            .map_err(|error| format!("decode track fixture {}: {error}", fixture.name))?;
        let verdict = derive_track(&input);
        if verdict.derived != fixture.expected {
            return Err(format!(
                "track fixture {} expected {:?}, observed {:?}",
                fixture.name, fixture.expected, verdict.derived
            ));
        }
        println!(
            "track fixture {}: {:?} reason={}",
            fixture.name, verdict.derived, verdict.reason_code
        );
    }
    println!(
        "track classification: {} cases schema_canonical_sha256={}",
        fixtures.len(),
        sha256_hex(&canonical_json(&schema_raw)?)
    );
    Ok(())
}

fn run_platform_inventory_suite(root: &Path) -> Result<(), String> {
    let schema_path = root.join("schemas/platform-inventory-v1.json");
    let schema_raw = fs::read(&schema_path)
        .map_err(|error| format!("read {}: {error}", schema_path.display()))?;
    let schema: Value = serde_json::from_slice(&schema_raw)
        .map_err(|error| format!("parse {}: {error}", schema_path.display()))?;
    jsonschema::draft202012::meta::validate(&schema)
        .map_err(|error| format!("platform inventory is not valid Draft 2020-12: {error}"))?;
    reject_schema_defaults(&schema, "")?;
    let validator = jsonschema::draft202012::options()
        .should_validate_formats(true)
        .build(&schema)
        .map_err(|error| format!("compile platform inventory schema: {error}"))?;

    let fixture_dir = root.join("fixtures/platform-inventory/v1");
    let raw_path = fixture_dir.join("raw-sensitive.json");
    let expected_path = fixture_dir.join("sanitized-expected.json");
    let raw: Value = serde_json::from_slice(
        &fs::read(&raw_path).map_err(|error| format!("read {}: {error}", raw_path.display()))?,
    )
    .map_err(|error| format!("parse {}: {error}", raw_path.display()))?;
    let expected_raw = fs::read(&expected_path)
        .map_err(|error| format!("read {}: {error}", expected_path.display()))?;
    let expected: Value = serde_json::from_slice(&expected_raw)
        .map_err(|error| format!("parse {}: {error}", expected_path.display()))?;

    if validator.is_valid(&raw) {
        return Err("raw sensitive inventory unexpectedly satisfies the public schema".to_owned());
    }
    let sanitized = sanitize_inventory_json(&raw);
    if sanitized != expected {
        return Err("sanitized inventory does not match the frozen expected fixture".to_owned());
    }
    validator
        .validate(&sanitized)
        .map_err(|error| format!("sanitized inventory failed schema validation: {error}"))?;
    serde_json::from_value::<PlatformInventory>(sanitized.clone())
        .map_err(|error| format!("sanitized inventory failed Rust contract decoding: {error}"))?;

    let rendered = serde_json::to_string(&sanitized)
        .map_err(|error| format!("serialize sanitized inventory: {error}"))?;
    for secret in [
        "private-build-host",
        "researcher",
        "CPU-SERIAL-SECRET",
        "GPU-SERIAL-SECRET",
        "registry-token-secret",
        "collector-token-secret",
        "api-key-secret",
    ] {
        if rendered.contains(secret) {
            return Err(format!(
                "inventory redaction retained forbidden fixture value: {secret}"
            ));
        }
    }
    for (pointer, expected_value) in [
        ("/os/build", "26100"),
        ("/os/architecture", "x86_64"),
        ("/cpu/model", "Fixture CPU 8C"),
        ("/runtimes/0/version", "3.14.4"),
        ("/compilers/0/version", "1.96.0"),
        ("/collectors/0/status", "available"),
    ] {
        if sanitized.pointer(pointer).and_then(Value::as_str) != Some(expected_value) {
            return Err(format!(
                "inventory redaction lost reproducibility field {pointer}"
            ));
        }
    }
    let lock_hash = sanitized
        .pointer("/dependency_locks/0/sha256")
        .and_then(Value::as_str);
    if lock_hash != Some("f2565497c1c59ebb1c22f88fca096a0d05e1efd9435f99d46c71e4dcfdf17d22") {
        return Err("inventory redaction lost dependency lock identity".to_owned());
    }

    println!(
        "platform inventory redaction: pass canonical_sha256={} schema_canonical_sha256={}",
        sha256_hex(&canonical_json(&expected_raw)?),
        sha256_hex(&canonical_json(&schema_raw)?)
    );
    Ok(())
}

fn run_experiment_manifest_suite(root: &Path) -> Result<(), String> {
    let schema_path = root.join("schemas/experiment-manifest-v1.json");
    let schema_raw = fs::read(&schema_path)
        .map_err(|error| format!("read {}: {error}", schema_path.display()))?;
    let schema: Value = serde_json::from_slice(&schema_raw)
        .map_err(|error| format!("parse {}: {error}", schema_path.display()))?;
    jsonschema::draft202012::meta::validate(&schema)
        .map_err(|error| format!("experiment manifest is not valid Draft 2020-12: {error}"))?;
    reject_schema_defaults(&schema, "")?;
    let validator = jsonschema::draft202012::options()
        .should_validate_formats(true)
        .build(&schema)
        .map_err(|error| format!("compile experiment manifest schema: {error}"))?;

    let fixture_dir = root.join("fixtures/experiment-manifest/v1");
    let valid_path = fixture_dir.join("valid.json");
    let valid_raw =
        fs::read(&valid_path).map_err(|error| format!("read {}: {error}", valid_path.display()))?;
    let valid: Value = serde_json::from_slice(&valid_raw)
        .map_err(|error| format!("parse {}: {error}", valid_path.display()))?;
    validator
        .validate(&valid)
        .map_err(|error| format!("valid experiment manifest fixture failed: {error}"))?;

    let canonical = canonical_json(&valid_raw)?;
    let windows_text = String::from_utf8(valid_raw.clone())
        .map_err(|_| "valid experiment manifest fixture is not UTF-8".to_owned())?
        .replace('\n', "\r\n");
    let windows_canonical = canonical_json(windows_text.as_bytes())?;
    if canonical != windows_canonical {
        return Err("experiment manifest canonical bytes differ for LF and CRLF input".to_owned());
    }
    let manifest_digest = sha256_hex(&canonical);
    let schema_digest = sha256_hex(&canonical_json(&schema_raw)?);
    println!(
        "experiment manifest valid.json: valid canonical_sha256={manifest_digest} schema_canonical_sha256={schema_digest}"
    );

    let cases = [
        ("missing-replay.json", "MANIFEST_REPLAY_REQUIRED"),
        ("missing-resource.json", "MANIFEST_RESOURCE_REQUIRED"),
        ("missing-seeds.json", "MANIFEST_SEEDS_REQUIRED"),
    ];
    for (file, code) in cases {
        let path = fixture_dir.join(file);
        let raw = fs::read(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
        let fixture: ManifestRejectionFixture = serde_json::from_slice(&raw)
            .map_err(|error| format!("parse {}: {error}", path.display()))?;
        if fixture.fixture_base != "valid.json" || fixture.expected_error != "required" {
            return Err(format!(
                "{file} must derive from valid.json and expect a required-property failure"
            ));
        }
        let mut candidate = valid.clone();
        remove_json_pointer(&mut candidate, &fixture.remove)
            .map_err(|error| format!("apply {file}: {error}"))?;
        let observed_errors = validator
            .iter_errors(&candidate)
            .map(|error| error.to_string())
            .collect::<Vec<_>>();
        if observed_errors.is_empty() {
            return Err(format!(
                "{file} expected {code} but the manifest remained valid"
            ));
        }
        if !observed_errors
            .iter()
            .any(|error| error.to_ascii_lowercase().contains(&fixture.expected_error))
        {
            return Err(format!(
                "{file} expected validator error containing {:?}, observed: {}",
                fixture.expected_error,
                observed_errors.join("; ")
            ));
        }
        println!("experiment manifest fixture {file}: {code}");
    }
    Ok(())
}

fn reject_schema_defaults(value: &Value, path: &str) -> Result<(), String> {
    match value {
        Value::Object(properties) => {
            if properties.contains_key("default") {
                return Err(format!(
                    "experiment manifest schema contains prohibited mutable default at {path}"
                ));
            }
            for (key, child) in properties {
                let child_path = format!("{path}/{key}");
                reject_schema_defaults(child, &child_path)?;
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                let child_path = format!("{path}/{index}");
                reject_schema_defaults(child, &child_path)?;
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
    Ok(())
}

fn remove_json_pointer(value: &mut Value, pointer: &str) -> Result<(), String> {
    let segments = pointer
        .strip_prefix('/')
        .ok_or_else(|| format!("JSON pointer must begin with '/': {pointer}"))?
        .split('/')
        .map(|segment| segment.replace("~1", "/").replace("~0", "~"))
        .collect::<Vec<_>>();
    let (last, parents) = segments
        .split_last()
        .ok_or_else(|| "cannot remove the document root".to_owned())?;
    let mut parent = value;
    for segment in parents {
        parent = parent
            .as_object_mut()
            .and_then(|object| object.get_mut(segment))
            .ok_or_else(|| format!("JSON pointer parent does not exist: {pointer}"))?;
    }
    let removed = parent
        .as_object_mut()
        .and_then(|object| object.remove(last));
    if removed.is_none() {
        return Err(format!("JSON pointer target does not exist: {pointer}"));
    }
    Ok(())
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}

fn load_catalog(path: &Path) -> Result<Catalog, String> {
    let raw = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    parse_catalog(path, &raw)
}

fn parse_catalog(path: &Path, raw: &[u8]) -> Result<Catalog, String> {
    serde_json::from_slice(raw).map_err(|error| format!("parse {}: {error}", path.display()))
}

fn validate_catalog(catalog: &Catalog) -> Vec<CompatibilityError> {
    let mut errors = Vec::new();
    if catalog.schema != CATALOG_SCHEMA {
        push_error(
            &mut errors,
            "CATALOG_VERSION_INVALID",
            "schema",
            format!("expected {CATALOG_SCHEMA}"),
        );
    }
    if catalog.epoch == 0 {
        push_error(
            &mut errors,
            "EPOCH_INVALID",
            "epoch",
            "epoch must be at least 1",
        );
    }

    let mut message_names = HashSet::new();
    for message in &catalog.protobuf_messages {
        if !message_names.insert(message.name.as_str()) {
            push_error(
                &mut errors,
                "DUPLICATE_MESSAGE",
                format!("protobuf_messages.{}", message.name),
                "message names must be unique",
            );
        }
        validate_message(message, &mut errors);
    }

    let mut schema_names = HashSet::new();
    for schema in &catalog.json_schemas {
        if !schema_names.insert(schema.name.as_str()) {
            push_error(
                &mut errors,
                "DUPLICATE_JSON_SCHEMA",
                format!("json_schemas.{}", schema.name),
                "JSON schema names must be unique",
            );
        }
        validate_json_schema(catalog, schema, &mut errors);
    }
    errors
}

fn validate_message(message: &ProtoMessage, errors: &mut Vec<CompatibilityError>) {
    let reserved_numbers = message
        .reserved_numbers
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    let reserved_names = message
        .reserved_names
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut numbers = HashSet::new();
    let mut names = HashSet::new();
    for field in &message.fields {
        let path = format!("protobuf_messages.{}.{}", message.name, field.name);
        if field.number == 0 || (19_000..=19_999).contains(&field.number) {
            push_error(
                errors,
                "FIELD_NUMBER_INVALID",
                &path,
                "field number is zero or in the Protobuf implementation-reserved range",
            );
        }
        if !numbers.insert(field.number) || !names.insert(field.name.as_str()) {
            push_error(
                errors,
                "DUPLICATE_FIELD",
                &path,
                "field names and numbers must be unique within a message",
            );
        }
        if reserved_numbers.contains(&field.number) || reserved_names.contains(field.name.as_str())
        {
            push_error(
                errors,
                "FIELD_REUSE",
                &path,
                "field uses a reserved number or name",
            );
        }
        if is_numeric_type(&field.field_type) && field.unit.is_none() {
            push_error(
                errors,
                "UNIT_MISSING",
                &path,
                "numeric fields require an explicit unit",
            );
        }
    }
}

fn validate_json_schema(
    catalog: &Catalog,
    schema: &JsonSchema,
    errors: &mut Vec<CompatibilityError>,
) {
    let path = format!("json_schemas.{}", schema.name);
    let expected_version = format!("{}.{}", catalog.epoch, catalog.minor);
    match &schema.version {
        Some(version) if version == &expected_version => {}
        Some(version) => push_error(
            errors,
            "JSON_VERSION_MISMATCH",
            &path,
            format!("version {version} does not match catalog {expected_version}"),
        ),
        None => push_error(
            errors,
            "JSON_VERSION_MISSING",
            &path,
            "JSON schemas require an explicit epoch.minor version",
        ),
    }
    let epoch_segment = format!("/v{}", catalog.epoch);
    if !schema.uri.ends_with(&epoch_segment) && !schema.uri.contains(&format!("{epoch_segment}/")) {
        push_error(
            errors,
            "JSON_URI_UNVERSIONED",
            &path,
            format!("URI must contain the epoch segment {epoch_segment}"),
        );
    }

    let mut names = HashSet::new();
    for property in &schema.properties {
        let property_path = format!("{path}.{}", property.name);
        if !names.insert(property.name.as_str()) {
            push_error(
                errors,
                "DUPLICATE_JSON_PROPERTY",
                &property_path,
                "property names must be unique",
            );
        }
        if matches!(property.property_type.as_str(), "integer" | "number")
            && property.unit.is_none()
        {
            push_error(
                errors,
                "UNIT_MISSING",
                &property_path,
                "numeric properties require an explicit unit",
            );
        }
    }
}

fn compare_catalogs(baseline: &Catalog, candidate: &Catalog) -> Vec<CompatibilityError> {
    let mut errors = Vec::new();
    if candidate.epoch != baseline.epoch {
        push_error(
            &mut errors,
            "EPOCH_CHANGE_REQUIRES_MIGRATION",
            "epoch",
            "minor compatibility checks require an unchanged epoch",
        );
        return errors;
    }
    if candidate.minor < baseline.minor {
        push_error(
            &mut errors,
            "MINOR_VERSION_REGRESSED",
            "minor",
            "minor version cannot decrease within an epoch",
        );
    }

    let candidate_messages = candidate
        .protobuf_messages
        .iter()
        .map(|message| (message.name.as_str(), message))
        .collect::<HashMap<_, _>>();
    for old_message in &baseline.protobuf_messages {
        let Some(new_message) = candidate_messages.get(old_message.name.as_str()) else {
            push_error(
                &mut errors,
                "MESSAGE_REMOVED",
                format!("protobuf_messages.{}", old_message.name),
                "messages cannot be removed within an epoch",
            );
            continue;
        };
        compare_messages(old_message, new_message, &mut errors);
    }

    let candidate_schemas = candidate
        .json_schemas
        .iter()
        .map(|schema| (schema.name.as_str(), schema))
        .collect::<HashMap<_, _>>();
    for old_schema in &baseline.json_schemas {
        let Some(new_schema) = candidate_schemas.get(old_schema.name.as_str()) else {
            push_error(
                &mut errors,
                "JSON_SCHEMA_REMOVED",
                format!("json_schemas.{}", old_schema.name),
                "JSON schemas cannot be removed within an epoch",
            );
            continue;
        };
        compare_json_schemas(old_schema, new_schema, &mut errors);
    }
    errors
}

fn compare_messages(old: &ProtoMessage, new: &ProtoMessage, errors: &mut Vec<CompatibilityError>) {
    let old_by_name = old
        .fields
        .iter()
        .map(|field| (field.name.as_str(), field))
        .collect::<HashMap<_, _>>();
    let old_by_number = old
        .fields
        .iter()
        .map(|field| (field.number, field))
        .collect::<HashMap<_, _>>();
    let new_by_name = new
        .fields
        .iter()
        .map(|field| (field.name.as_str(), field))
        .collect::<HashMap<_, _>>();

    for field in &new.fields {
        let path = format!("protobuf_messages.{}.{}", new.name, field.name);
        if let Some(prior) = old_by_name.get(field.name.as_str()) {
            if field.number != prior.number {
                push_error(
                    errors,
                    "FIELD_RENUMBERED",
                    &path,
                    format!(
                        "field number changed from {} to {}",
                        prior.number, field.number
                    ),
                );
            }
            compare_field_semantics(prior, field, &path, errors);
        }
        if let Some(prior) = old_by_number.get(&field.number)
            && field.name != prior.name
        {
            push_error(
                errors,
                "FIELD_REUSE",
                &path,
                format!(
                    "field number {} previously belonged to {}",
                    field.number, prior.name
                ),
            );
        }
    }

    for field in &old.fields {
        if !new_by_name.contains_key(field.name.as_str()) {
            let number_reserved = new.reserved_numbers.contains(&field.number);
            let name_reserved = new.reserved_names.contains(&field.name);
            if !number_reserved || !name_reserved {
                push_error(
                    errors,
                    "DELETION_NOT_RESERVED",
                    format!("protobuf_messages.{}.{}", new.name, field.name),
                    "deleted fields must reserve both their number and name",
                );
            }
        }
    }
}

fn compare_field_semantics(
    old: &ProtoField,
    new: &ProtoField,
    path: &str,
    errors: &mut Vec<CompatibilityError>,
) {
    if old.field_type != new.field_type || old.presence != new.presence {
        push_error(
            errors,
            "FIELD_TYPE_CHANGED",
            path,
            "field type or presence changed within an epoch",
        );
    }
    if old.unit != new.unit {
        push_error(
            errors,
            "UNIT_CHANGED",
            path,
            "field unit changed without an epoch migration",
        );
    }
}

fn compare_json_schemas(old: &JsonSchema, new: &JsonSchema, errors: &mut Vec<CompatibilityError>) {
    let old_properties = old
        .properties
        .iter()
        .map(|property| (property.name.as_str(), property))
        .collect::<HashMap<_, _>>();
    let new_properties = new
        .properties
        .iter()
        .map(|property| (property.name.as_str(), property))
        .collect::<HashMap<_, _>>();
    for property in &old.properties {
        let path = format!("json_schemas.{}.{}", old.name, property.name);
        let Some(candidate) = new_properties.get(property.name.as_str()) else {
            push_error(
                errors,
                "JSON_PROPERTY_REMOVED",
                &path,
                "properties cannot be removed within an epoch",
            );
            continue;
        };
        if property.property_type != candidate.property_type {
            push_error(
                errors,
                "JSON_TYPE_CHANGED",
                &path,
                "property type changed within an epoch",
            );
        }
        if !property.required && candidate.required {
            push_error(
                errors,
                "JSON_REQUIRED_ADDED",
                &path,
                "an existing optional property cannot become required",
            );
        }
        if property.unit != candidate.unit {
            push_error(
                errors,
                "UNIT_CHANGED",
                &path,
                "property unit changed without an epoch migration",
            );
        }
    }
    for property in &new.properties {
        if !old_properties.contains_key(property.name.as_str()) && property.required {
            push_error(
                errors,
                "JSON_REQUIRED_ADDED",
                format!("json_schemas.{}.{}", new.name, property.name),
                "new properties must be optional within an epoch",
            );
        }
    }
}

fn is_numeric_type(field_type: &str) -> bool {
    matches!(
        field_type,
        "double"
            | "float"
            | "int32"
            | "int64"
            | "uint32"
            | "uint64"
            | "sint32"
            | "sint64"
            | "fixed32"
            | "fixed64"
            | "sfixed32"
            | "sfixed64"
    )
}

fn canonical_json(raw: &[u8]) -> Result<Vec<u8>, String> {
    let value: Value = serde_json::from_slice(raw)
        .map_err(|error| format!("parse JSON before canonicalization: {error}"))?;
    let mut output = Vec::new();
    write_canonical(&value, &mut output)?;
    Ok(output)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

fn write_canonical(value: &Value, output: &mut Vec<u8>) -> Result<(), String> {
    match value {
        Value::Null => output.extend_from_slice(b"null"),
        Value::Bool(true) => output.extend_from_slice(b"true"),
        Value::Bool(false) => output.extend_from_slice(b"false"),
        Value::Number(number) => output.extend_from_slice(number.to_string().as_bytes()),
        Value::String(text) => output.extend_from_slice(
            serde_json::to_string(text)
                .map_err(|error| format!("canonicalize string: {error}"))?
                .as_bytes(),
        ),
        Value::Array(values) => {
            output.push(b'[');
            for (index, item) in values.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                write_canonical(item, output)?;
            }
            output.push(b']');
        }
        Value::Object(properties) => {
            output.push(b'{');
            let mut keys = properties.keys().collect::<Vec<_>>();
            keys.sort_unstable();
            for (index, key) in keys.into_iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                output.extend_from_slice(
                    serde_json::to_string(key)
                        .map_err(|error| format!("canonicalize object key: {error}"))?
                        .as_bytes(),
                );
                output.push(b':');
                write_canonical(&properties[key], output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn push_error(
    errors: &mut Vec<CompatibilityError>,
    code: &'static str,
    path: impl Into<String>,
    detail: impl Into<String>,
) {
    errors.push(CompatibilityError {
        code,
        path: path.into(),
        detail: detail.into(),
    });
}

fn format_errors(context: &str, errors: &[CompatibilityError]) -> String {
    let mut output = context.to_owned();
    for error in errors {
        let _ = write!(
            output,
            "\n- {} at {}: {}",
            error.code, error.path, error.detail
        );
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{canonical_json, run};

    #[test]
    fn canonical_json_sorts_object_keys_without_reordering_arrays() {
        assert_eq!(
            canonical_json(br#"{ "z": [2, 1], "a": {"b": true, "a": null} }"#)
                .expect("fixture is valid JSON"),
            br#"{"a":{"a":null,"b":true},"z":[2,1]}"#
        );
    }

    #[test]
    fn frozen_compatibility_suite_has_expected_outcomes() {
        run().expect("frozen schema compatibility suite must pass");
    }
}
