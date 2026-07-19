use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn cli_emits_one_report_and_uses_verdict_exit_codes() {
    let valid = run("fixtures/bundle-validation/v1/manifest.json");
    assert!(valid.status.success());
    let valid_report: Value = serde_json::from_slice(&valid.stdout).expect("valid report JSON");
    assert_eq!(valid_report["verdict"], "VALID");
    assert!(valid.stderr.is_empty());

    let invalid = run("fixtures/bundle-validation/tampered/manifest.json");
    assert_eq!(invalid.status.code(), Some(2));
    let invalid_report: Value =
        serde_json::from_slice(&invalid.stdout).expect("invalid report JSON");
    assert_eq!(invalid_report["verdict"], "INVALID");
    assert_eq!(
        invalid_report["reason_codes"],
        serde_json::json!(["FILE_HASH_MISMATCH"])
    );
    assert!(invalid.stderr.is_empty());
}

fn run(manifest: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bonsai-xtask"))
        .args(["bundle-check", "--root"])
        .arg(workspace_root())
        .arg(manifest)
        .output()
        .expect("bundle-check must start")
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
