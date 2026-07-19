use crate::BlobId;
use bonsai_contracts::track::{Track, TrackDeclaration, derive_track};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

const CURRENT_EPOCH: u64 = 1;
const CURRENT_MINOR: u64 = 0;
const CURRENT_FORMAT: &str = "bonsai.bundle/v1";
const MIGRATION_REGISTRY: &str = "bonsai.bundle-migrations/v1";

/// Draft 2020-12 schemas required by whole-bundle conformance validation.
#[derive(Clone, Debug)]
pub struct BundleSchemas {
    pub bundle_manifest: Value,
    pub experiment_manifest: Value,
    pub track_declaration: Value,
    pub platform_inventory: Value,
    pub resource_policy: Value,
    pub metric_estimate: Value,
}

/// Machine-report access posture.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessMode {
    ValidationOnly,
    ReadOnly,
}

/// Whole-bundle machine verdict. This is not a scientific claim verdict.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum OverallVerdict {
    #[serde(rename = "VALID")]
    Valid,
    #[serde(rename = "MIGRATABLE")]
    Migratable,
    #[serde(rename = "FORWARD_READABLE")]
    ForwardReadable,
    #[serde(rename = "INVALID")]
    Invalid,
    #[serde(rename = "INDETERMINATE")]
    Indeterminate,
    #[serde(rename = "VALID_WITH_LIMITATIONS")]
    ValidWithLimitations,
}

/// Status of one independent bundle check.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pass,
    Fail,
    Limited,
    NotRun,
}

/// Stable status and reason codes for one validation dimension.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CheckResult {
    pub status: CheckStatus,
    pub reason_codes: Vec<String>,
}

impl CheckResult {
    fn pass() -> Self {
        Self {
            status: CheckStatus::Pass,
            reason_codes: Vec::new(),
        }
    }

    fn fail(code: &str) -> Self {
        Self::with(CheckStatus::Fail, code)
    }

    fn limited(code: &str) -> Self {
        Self::with(CheckStatus::Limited, code)
    }

    fn not_run(code: &str) -> Self {
        Self::with(CheckStatus::NotRun, code)
    }

    fn with(status: CheckStatus, code: &str) -> Self {
        Self {
            status,
            reason_codes: vec![code.to_owned()],
        }
    }
}

/// All required validation dimensions in a single object.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ValidationChecks {
    pub schema: CheckResult,
    pub hashes: CheckResult,
    pub track: CheckResult,
    pub inventory: CheckResult,
    pub resource_policy: CheckResult,
    pub failures: CheckResult,
    pub metric_provenance: CheckResult,
    pub migrations: CheckResult,
}

/// Stable JSON report returned for every readable manifest epoch.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BundleValidationReport {
    pub schema: String,
    pub bundle_id: Option<String>,
    pub manifest_epoch: Option<u64>,
    pub access_mode: AccessMode,
    pub verdict: OverallVerdict,
    pub checks: ValidationChecks,
    pub reason_codes: Vec<String>,
    pub migrated_manifest_sha256: Option<String>,
}

/// Operational failures that prevent a validation report from being produced.
#[derive(Debug)]
#[non_exhaustive]
pub enum BundleValidationError {
    Io(io::Error),
    ManifestJson(serde_json::Error),
    SchemaCompile(String),
    UnsafePath,
    UnsupportedOldEpoch(u64),
    MigrationInvalid,
}

impl BundleValidationError {
    /// Stable machine-oriented outcome code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Io(_) => "BUNDLE_VALIDATION_IO_ERROR",
            Self::ManifestJson(_) => "BUNDLE_MANIFEST_JSON_INVALID",
            Self::SchemaCompile(_) => "BUNDLE_SCHEMA_COMPILE_ERROR",
            Self::UnsafePath => "BUNDLE_PATH_UNSAFE",
            Self::UnsupportedOldEpoch(_) => "BUNDLE_OLD_EPOCH_UNSUPPORTED",
            Self::MigrationInvalid => "BUNDLE_MIGRATION_INVALID",
        }
    }
}

impl fmt::Display for BundleValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.code())?;
        match self {
            Self::Io(error) => write!(formatter, ": {error}"),
            Self::ManifestJson(error) => write!(formatter, ": {error}"),
            Self::SchemaCompile(error) => write!(formatter, ": {error}"),
            Self::UnsupportedOldEpoch(epoch) => write!(formatter, ": epoch={epoch}"),
            _ => Ok(()),
        }
    }
}

impl Error for BundleValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::ManifestJson(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for BundleValidationError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for BundleValidationError {
    fn from(error: serde_json::Error) -> Self {
        Self::ManifestJson(error)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BundleManifest {
    format: String,
    epoch: u64,
    minor: u64,
    bundle_id: String,
    files: Vec<BundleFile>,
    migration: MigrationRecord,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BundleFile {
    path: String,
    sha256: String,
    role: FileRole,
    required: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
enum FileRole {
    ExperimentManifest,
    TrackDeclaration,
    PlatformInventory,
    ResourcePolicy,
    FailureLog,
    MetricEstimate,
    EventSegment,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MigrationRecord {
    status: MigrationStatus,
    source_epoch: u64,
    registry_id: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum MigrationStatus {
    Current,
    Migrated,
}

#[derive(Debug, Deserialize)]
struct ForwardManifest {
    bundle_id: String,
    epoch: u64,
    files: Vec<ForwardFile>,
}

#[derive(Debug, Deserialize)]
struct ForwardFile {
    path: String,
    sha256: String,
    required: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FailureRecord {
    code: String,
    fatal: bool,
}

/// Deterministically migrate the supported epoch-0 manifest to epoch 1 in memory.
///
/// # Errors
///
/// Returns an explicit error for malformed JSON, a nonzero source epoch, or a
/// migrated result that does not conform to the current typed manifest.
pub fn migrate_v0_manifest(raw: &[u8]) -> Result<Vec<u8>, BundleValidationError> {
    let mut value: Value = serde_json::from_slice(raw)?;
    let object = value
        .as_object_mut()
        .ok_or(BundleValidationError::MigrationInvalid)?;
    if object.get("epoch").and_then(Value::as_u64) != Some(0)
        || object.get("format").and_then(Value::as_str) != Some("bonsai.bundle/v0")
    {
        return Err(BundleValidationError::MigrationInvalid);
    }
    object.insert(
        "format".to_owned(),
        Value::String(CURRENT_FORMAT.to_owned()),
    );
    object.insert("epoch".to_owned(), Value::from(CURRENT_EPOCH));
    object.insert("minor".to_owned(), Value::from(CURRENT_MINOR));
    object.insert(
        "migration".to_owned(),
        json!({
            "status": "migrated",
            "source_epoch": 0,
            "registry_id": MIGRATION_REGISTRY
        }),
    );
    let manifest: BundleManifest =
        serde_json::from_value(value).map_err(|_| BundleValidationError::MigrationInvalid)?;
    serde_json::to_vec(&manifest).map_err(BundleValidationError::ManifestJson)
}

/// Produce the single machine-readable conformance report for a result bundle.
///
/// `root` is the trusted containment boundary. Every manifest path must be a
/// canonical relative path to a regular nonsymlink file within that root.
///
/// # Errors
///
/// Returns only operational failures that prevent a report, including an unreadable
/// manifest/root, schema compiler failure, unsafe path, or unsupported pre-v0 epoch.
#[allow(clippy::too_many_lines)]
pub fn validate_result_bundle(
    root: impl AsRef<Path>,
    manifest_path: impl AsRef<Path>,
    schemas: &BundleSchemas,
) -> Result<BundleValidationReport, BundleValidationError> {
    let root = checked_root(root.as_ref())?;
    let manifest_path = resolve_path(&root, manifest_path.as_ref())?;
    let raw = fs::read(&manifest_path)?;
    let value: Value = serde_json::from_slice(&raw)?;
    let epoch = value.get("epoch").and_then(Value::as_u64);
    let bundle_id = value
        .get("bundle_id")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let Some(epoch) = epoch else {
        return Ok(invalid_header_report(
            bundle_id,
            None,
            "MANIFEST_EPOCH_MISSING",
        ));
    };
    if epoch > CURRENT_EPOCH {
        return validate_forward(&root, value, schemas);
    }
    if epoch < CURRENT_EPOCH && epoch != 0 {
        return Err(BundleValidationError::UnsupportedOldEpoch(epoch));
    }

    let (manifest_value, manifest, migrated_digest) = if epoch == 0 {
        let migrated = migrate_v0_manifest(&raw)?;
        let digest = BlobId::digest(&migrated).to_hex();
        let migrated_value: Value = serde_json::from_slice(&migrated)?;
        let manifest: BundleManifest = serde_json::from_slice(&migrated)?;
        (migrated_value, manifest, Some(digest))
    } else {
        let manifest: BundleManifest = match serde_json::from_value(value.clone()) {
            Ok(manifest) => manifest,
            Err(_) => {
                return Ok(invalid_header_report(
                    bundle_id,
                    Some(epoch),
                    "MANIFEST_SCHEMA_INVALID",
                ));
            }
        };
        (value, manifest, None)
    };

    let mut checks = ValidationChecks {
        schema: validate_value_schema(
            &manifest_value,
            &schemas.bundle_manifest,
            "MANIFEST_SCHEMA_INVALID",
        )?,
        hashes: CheckResult::pass(),
        track: CheckResult::pass(),
        inventory: CheckResult::pass(),
        resource_policy: CheckResult::pass(),
        failures: CheckResult::pass(),
        metric_provenance: CheckResult::pass(),
        migrations: if epoch == 0 {
            CheckResult::with(CheckStatus::Pass, "MIGRATION_V0_TO_V1_AVAILABLE")
        } else {
            CheckResult::pass()
        },
    };

    let roles = group_roles(&manifest.files);
    if !required_roles_present(&roles) {
        checks.schema = CheckResult::fail("REQUIRED_BUNDLE_ROLE_MISSING");
    }
    let mut file_bytes = BTreeMap::new();
    let mut content_hashes = BTreeSet::new();
    for entry in &manifest.files {
        if BlobId::from_hex(&entry.sha256).is_err() {
            checks.hashes = CheckResult::fail("FILE_HASH_INVALID");
            continue;
        }
        let path = match resolve_path(&root, Path::new(&entry.path)) {
            Ok(path) => path,
            Err(BundleValidationError::UnsafePath) => {
                checks.hashes = CheckResult::fail("FILE_PATH_UNSAFE");
                continue;
            }
            Err(BundleValidationError::Io(error)) if error.kind() == io::ErrorKind::NotFound => {
                if entry.required {
                    checks.hashes = CheckResult::fail("REQUIRED_FILE_MISSING");
                }
                continue;
            }
            Err(error) => return Err(error),
        };
        let bytes = fs::read(path)?;
        let actual = BlobId::digest(&bytes).to_hex();
        if actual != entry.sha256 {
            checks.hashes = CheckResult::fail("FILE_HASH_MISMATCH");
        }
        content_hashes.insert(actual);
        file_bytes.insert(entry.path.clone(), bytes);
    }

    validate_component_schemas(&manifest, &file_bytes, schemas, &mut checks)?;
    checks.track = validate_track(&manifest, &file_bytes, schemas)?;
    checks.inventory = validate_inventory(&manifest, &file_bytes, schemas)?;
    checks.resource_policy = validate_role_schema(
        &manifest,
        &file_bytes,
        FileRole::ResourcePolicy,
        &schemas.resource_policy,
        "RESOURCE_POLICY_SCHEMA_INVALID",
    )?;
    checks.failures = validate_failures(&manifest, &file_bytes);
    checks.metric_provenance =
        validate_metric_provenance(&manifest, &file_bytes, schemas, &content_hashes)?;

    let verdict = aggregate_verdict(&checks, epoch == 0);
    Ok(finalize_report(
        Some(manifest.bundle_id),
        Some(epoch),
        AccessMode::ValidationOnly,
        verdict,
        checks,
        migrated_digest,
    ))
}

fn validate_forward(
    root: &Path,
    value: Value,
    schemas: &BundleSchemas,
) -> Result<BundleValidationReport, BundleValidationError> {
    let forward: ForwardManifest = match serde_json::from_value(value) {
        Ok(manifest) => manifest,
        Err(_) => {
            return Ok(invalid_header_report(None, None, "FORWARD_HEADER_INVALID"));
        }
    };
    let _ = compile_schema(&schemas.bundle_manifest)?;
    let mut hashes = CheckResult::pass();
    for entry in &forward.files {
        let path = match resolve_path(root, Path::new(&entry.path)) {
            Ok(path) => path,
            Err(BundleValidationError::UnsafePath) => {
                hashes = CheckResult::fail("FILE_PATH_UNSAFE");
                continue;
            }
            Err(BundleValidationError::Io(error)) if error.kind() == io::ErrorKind::NotFound => {
                if entry.required {
                    hashes = CheckResult::fail("REQUIRED_FILE_MISSING");
                }
                continue;
            }
            Err(error) => return Err(error),
        };
        if BlobId::digest(&fs::read(path)?).to_hex() != entry.sha256 {
            hashes = CheckResult::fail("FILE_HASH_MISMATCH");
        }
    }
    let not_run = CheckResult::not_run("FUTURE_EPOCH_NOT_INTERPRETED");
    let checks = ValidationChecks {
        schema: not_run.clone(),
        hashes: hashes.clone(),
        track: not_run.clone(),
        inventory: not_run.clone(),
        resource_policy: not_run.clone(),
        failures: not_run.clone(),
        metric_provenance: not_run,
        migrations: CheckResult::with(CheckStatus::Pass, "FUTURE_EPOCH_READ_ONLY"),
    };
    let verdict = if hashes.status == CheckStatus::Pass {
        OverallVerdict::ForwardReadable
    } else {
        OverallVerdict::Invalid
    };
    Ok(finalize_report(
        Some(forward.bundle_id),
        Some(forward.epoch),
        AccessMode::ReadOnly,
        verdict,
        checks,
        None,
    ))
}

fn validate_component_schemas(
    manifest: &BundleManifest,
    files: &BTreeMap<String, Vec<u8>>,
    schemas: &BundleSchemas,
    checks: &mut ValidationChecks,
) -> Result<(), BundleValidationError> {
    let components = [
        validate_role_schema(
            manifest,
            files,
            FileRole::ExperimentManifest,
            &schemas.experiment_manifest,
            "EXPERIMENT_SCHEMA_INVALID",
        )?,
        validate_role_schema(
            manifest,
            files,
            FileRole::TrackDeclaration,
            &schemas.track_declaration,
            "TRACK_SCHEMA_INVALID",
        )?,
        validate_role_schema(
            manifest,
            files,
            FileRole::PlatformInventory,
            &schemas.platform_inventory,
            "INVENTORY_SCHEMA_INVALID",
        )?,
    ];
    if components
        .iter()
        .any(|check| check.status == CheckStatus::Fail)
    {
        checks.schema = CheckResult {
            status: CheckStatus::Fail,
            reason_codes: components
                .into_iter()
                .flat_map(|check| check.reason_codes)
                .collect(),
        };
    }
    Ok(())
}

fn validate_role_schema(
    manifest: &BundleManifest,
    files: &BTreeMap<String, Vec<u8>>,
    role: FileRole,
    schema: &Value,
    error_code: &str,
) -> Result<CheckResult, BundleValidationError> {
    let Some(bytes) = role_bytes(manifest, files, role) else {
        return Ok(CheckResult::fail("REQUIRED_BUNDLE_ROLE_MISSING"));
    };
    let value: Value = match serde_json::from_slice(bytes) {
        Ok(value) => value,
        Err(_) => return Ok(CheckResult::fail(error_code)),
    };
    validate_value_schema(&value, schema, error_code)
}

fn validate_track(
    manifest: &BundleManifest,
    files: &BTreeMap<String, Vec<u8>>,
    schemas: &BundleSchemas,
) -> Result<CheckResult, BundleValidationError> {
    let schema_check = validate_role_schema(
        manifest,
        files,
        FileRole::TrackDeclaration,
        &schemas.track_declaration,
        "TRACK_SCHEMA_INVALID",
    )?;
    if schema_check.status == CheckStatus::Fail {
        return Ok(schema_check);
    }
    let bytes = role_bytes(manifest, files, FileRole::TrackDeclaration)
        .ok_or(BundleValidationError::MigrationInvalid)?;
    let declaration: TrackDeclaration = match serde_json::from_slice(bytes) {
        Ok(declaration) => declaration,
        Err(_) => return Ok(CheckResult::fail("TRACK_SCHEMA_INVALID")),
    };
    let verdict = derive_track(&declaration);
    if verdict.derived == Track::Indeterminate || !verdict.declaration_matches {
        return Ok(CheckResult::limited(verdict.reason_code));
    }
    Ok(CheckResult::pass())
}

fn validate_inventory(
    manifest: &BundleManifest,
    files: &BTreeMap<String, Vec<u8>>,
    schemas: &BundleSchemas,
) -> Result<CheckResult, BundleValidationError> {
    let schema_check = validate_role_schema(
        manifest,
        files,
        FileRole::PlatformInventory,
        &schemas.platform_inventory,
        "INVENTORY_SCHEMA_INVALID",
    )?;
    if schema_check.status == CheckStatus::Fail {
        return Ok(schema_check);
    }
    let bytes = role_bytes(manifest, files, FileRole::PlatformInventory)
        .ok_or(BundleValidationError::MigrationInvalid)?;
    let value: Value = serde_json::from_slice(bytes)?;
    let unavailable = value
        .get("collectors")
        .and_then(Value::as_array)
        .is_some_and(|collectors| {
            collectors.iter().any(|collector| {
                collector.get("status").and_then(Value::as_str) != Some("available")
            })
        });
    if unavailable {
        Ok(CheckResult::limited("COUNTER_UNAVAILABLE"))
    } else {
        Ok(CheckResult::pass())
    }
}

fn validate_failures(manifest: &BundleManifest, files: &BTreeMap<String, Vec<u8>>) -> CheckResult {
    let Some(bytes) = role_bytes(manifest, files, FileRole::FailureLog) else {
        return CheckResult::fail("REQUIRED_BUNDLE_ROLE_MISSING");
    };
    let records: Vec<FailureRecord> = match serde_json::from_slice(bytes) {
        Ok(records) => records,
        Err(_) => return CheckResult::fail("FAILURE_LOG_INVALID"),
    };
    if records.iter().any(|record| record.code.is_empty()) {
        return CheckResult::fail("FAILURE_LOG_INVALID");
    }
    if records.iter().any(|record| record.fatal) {
        return CheckResult::fail("RECORDED_FATAL_FAILURE");
    }
    if records.is_empty() {
        CheckResult::pass()
    } else {
        CheckResult::limited("RECORDED_NONFATAL_FAILURE")
    }
}

fn validate_metric_provenance(
    manifest: &BundleManifest,
    files: &BTreeMap<String, Vec<u8>>,
    schemas: &BundleSchemas,
    declared_hashes: &BTreeSet<String>,
) -> Result<CheckResult, BundleValidationError> {
    let entries = manifest
        .files
        .iter()
        .filter(|entry| entry.role == FileRole::MetricEstimate)
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return Ok(CheckResult::fail("METRIC_ESTIMATE_MISSING"));
    }
    let validator = compile_schema(&schemas.metric_estimate)?;
    for entry in entries {
        let Some(bytes) = files.get(&entry.path) else {
            return Ok(CheckResult::fail("METRIC_ESTIMATE_MISSING"));
        };
        let value: Value = match serde_json::from_slice(bytes) {
            Ok(value) => value,
            Err(_) => return Ok(CheckResult::fail("METRIC_SCHEMA_INVALID")),
        };
        if !validator.is_valid(&value) {
            return Ok(CheckResult::fail("METRIC_SCHEMA_INVALID"));
        }
        let Some(inputs) = value.get("inputs").and_then(Value::as_array) else {
            return Ok(CheckResult::fail("METRIC_PROVENANCE_MISSING"));
        };
        if inputs.iter().any(|input| {
            input
                .get("sha256")
                .and_then(Value::as_str)
                .is_none_or(|hash| !declared_hashes.contains(hash))
        }) {
            return Ok(CheckResult::fail("METRIC_PROVENANCE_UNBOUND"));
        }
    }
    Ok(CheckResult::pass())
}

fn validate_value_schema(
    value: &Value,
    schema: &Value,
    code: &str,
) -> Result<CheckResult, BundleValidationError> {
    let validator = compile_schema(schema)?;
    if validator.is_valid(value) {
        Ok(CheckResult::pass())
    } else {
        Ok(CheckResult::fail(code))
    }
}

fn compile_schema(schema: &Value) -> Result<jsonschema::Validator, BundleValidationError> {
    jsonschema::draft202012::new(schema)
        .map_err(|error| BundleValidationError::SchemaCompile(error.to_string()))
}

fn group_roles(files: &[BundleFile]) -> BTreeMap<FileRole, usize> {
    let mut roles = BTreeMap::new();
    for file in files {
        *roles.entry(file.role).or_insert(0) += 1;
    }
    roles
}

fn required_roles_present(roles: &BTreeMap<FileRole, usize>) -> bool {
    [
        FileRole::ExperimentManifest,
        FileRole::TrackDeclaration,
        FileRole::PlatformInventory,
        FileRole::ResourcePolicy,
        FileRole::FailureLog,
        FileRole::MetricEstimate,
        FileRole::EventSegment,
    ]
    .iter()
    .all(|role| roles.get(role).is_some_and(|count| *count >= 1))
}

fn role_bytes<'a>(
    manifest: &BundleManifest,
    files: &'a BTreeMap<String, Vec<u8>>,
    role: FileRole,
) -> Option<&'a [u8]> {
    manifest
        .files
        .iter()
        .find(|entry| entry.role == role)
        .and_then(|entry| files.get(&entry.path))
        .map(Vec::as_slice)
}

fn aggregate_verdict(checks: &ValidationChecks, migrated: bool) -> OverallVerdict {
    let statuses = [
        checks.schema.status,
        checks.hashes.status,
        checks.track.status,
        checks.inventory.status,
        checks.resource_policy.status,
        checks.failures.status,
        checks.metric_provenance.status,
        checks.migrations.status,
    ];
    if statuses.contains(&CheckStatus::Fail) {
        OverallVerdict::Invalid
    } else if checks.track.status == CheckStatus::Limited {
        OverallVerdict::Indeterminate
    } else if migrated {
        OverallVerdict::Migratable
    } else if statuses.contains(&CheckStatus::Limited) {
        OverallVerdict::ValidWithLimitations
    } else {
        OverallVerdict::Valid
    }
}

fn invalid_header_report(
    bundle_id: Option<String>,
    epoch: Option<u64>,
    code: &str,
) -> BundleValidationReport {
    let failed = CheckResult::fail(code);
    let not_run = CheckResult::not_run("MANIFEST_INVALID");
    finalize_report(
        bundle_id,
        epoch,
        AccessMode::ValidationOnly,
        OverallVerdict::Invalid,
        ValidationChecks {
            schema: failed,
            hashes: not_run.clone(),
            track: not_run.clone(),
            inventory: not_run.clone(),
            resource_policy: not_run.clone(),
            failures: not_run.clone(),
            metric_provenance: not_run.clone(),
            migrations: not_run,
        },
        None,
    )
}

fn finalize_report(
    bundle_id: Option<String>,
    epoch: Option<u64>,
    access_mode: AccessMode,
    verdict: OverallVerdict,
    checks: ValidationChecks,
    migrated_manifest_sha256: Option<String>,
) -> BundleValidationReport {
    let reason_codes = [
        &checks.schema,
        &checks.hashes,
        &checks.track,
        &checks.inventory,
        &checks.resource_policy,
        &checks.failures,
        &checks.metric_provenance,
        &checks.migrations,
    ]
    .into_iter()
    .flat_map(|check| check.reason_codes.iter().cloned())
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect();
    BundleValidationReport {
        schema: "bonsai.bundle-validation-report/v1".to_owned(),
        bundle_id,
        manifest_epoch: epoch,
        access_mode,
        verdict,
        checks,
        reason_codes,
        migrated_manifest_sha256,
    }
}

fn checked_root(path: &Path) -> Result<PathBuf, BundleValidationError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(BundleValidationError::UnsafePath);
    }
    Ok(fs::canonicalize(path)?)
}

fn resolve_path(root: &Path, relative: &Path) -> Result<PathBuf, BundleValidationError> {
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(BundleValidationError::UnsafePath);
    }
    let path = root.join(relative);
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(BundleValidationError::UnsafePath);
            }
            Ok(_) => {}
            Err(error) => return Err(error.into()),
        }
    }
    let canonical = fs::canonicalize(path)?;
    if !canonical.starts_with(root) || !canonical.is_file() {
        return Err(BundleValidationError::UnsafePath);
    }
    Ok(canonical)
}
