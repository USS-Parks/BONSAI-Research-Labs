use bonsai_bundle::{
    AccessMode, BundleSchemas, CheckStatus, OverallVerdict, migrate_v0_manifest,
    validate_result_bundle,
};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct Corpus {
    format: String,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    name: String,
    manifest: String,
    expected_verdict: OverallVerdict,
    expected_reasons: Vec<String>,
}

#[test]
fn committed_corpus_has_exact_machine_verdicts() {
    let root = workspace_root();
    let corpus: Corpus = read_json(&root.join("fixtures/bundle-validation/expected-outcomes.json"));
    assert_eq!(corpus.format, "bonsai.bundle-validation-corpus/v1");
    let schemas = schemas(&root);
    let report_schema: Value = read_json(&root.join("schemas/bundle-validation-report-v1.json"));
    let report_validator =
        jsonschema::draft202012::new(&report_schema).expect("report schema must compile");

    for case in corpus.cases {
        let report = validate_result_bundle(&root, &case.manifest, &schemas)
            .unwrap_or_else(|error| panic!("fixture {} failed operationally: {error}", case.name));
        assert_eq!(
            report.verdict, case.expected_verdict,
            "fixture {}",
            case.name
        );
        assert_eq!(
            report.reason_codes, case.expected_reasons,
            "fixture {}",
            case.name
        );
        let value = serde_json::to_value(&report).expect("report must serialize");
        report_validator
            .validate(&value)
            .unwrap_or_else(|error| panic!("fixture {} report failed schema: {error}", case.name));
    }
}

#[test]
fn v0_migration_is_reproducible_and_non_mutating() {
    let root = workspace_root();
    let path = root.join("fixtures/bundle-validation/v0/manifest.json");
    let before = fs::read(&path).expect("v0 fixture must be readable");
    let first = migrate_v0_manifest(&before).expect("v0 must migrate");
    let second = migrate_v0_manifest(&before).expect("same v0 must migrate again");
    assert_eq!(first, second);
    assert_eq!(fs::read(path).expect("v0 fixture must remain"), before);

    let report = validate_result_bundle(
        &root,
        "fixtures/bundle-validation/v0/manifest.json",
        &schemas(&root),
    )
    .expect("v0 validation report");
    assert_eq!(report.verdict, OverallVerdict::Migratable);
    assert_eq!(
        report.migrated_manifest_sha256.as_deref(),
        Some(bonsai_bundle::BlobId::digest(&first).to_hex().as_str())
    );
}

#[test]
fn forward_epoch_is_hash_checked_but_never_interpreted() {
    let root = workspace_root();
    let report = validate_result_bundle(
        &root,
        "fixtures/bundle-validation/forward/manifest.json",
        &schemas(&root),
    )
    .expect("forward report");
    assert_eq!(report.verdict, OverallVerdict::ForwardReadable);
    assert_eq!(report.access_mode, AccessMode::ReadOnly);
    assert_eq!(report.checks.hashes.status, CheckStatus::Pass);
    assert_eq!(report.checks.schema.status, CheckStatus::NotRun);
    assert_eq!(report.checks.track.status, CheckStatus::NotRun);
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn schemas(root: &Path) -> BundleSchemas {
    BundleSchemas {
        bundle_manifest: read_json(&root.join("schemas/bundle-manifest-v1.json")),
        experiment_manifest: read_json(&root.join("schemas/experiment-manifest-v1.json")),
        track_declaration: read_json(&root.join("schemas/track-declaration-v1.json")),
        platform_inventory: read_json(&root.join("schemas/platform-inventory-v1.json")),
        resource_policy: read_json(&root.join("schemas/resource-policy-v1.json")),
        metric_estimate: read_json(&root.join("schemas/metric-estimate-v1.json")),
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> T {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}
