use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture_directory() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("test clock must follow Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("bonsai-bg06-fixture-{nonce}"))
}

#[test]
fn passing_and_failing_commands_produce_distinct_sanitized_records() {
    let directory = fixture_directory();
    let binary = env!("CARGO_BIN_EXE_bonsai-xtask");
    let secret = "BONSAI_FIXTURE_SECRET_MUST_NOT_APPEAR";

    let pass = Command::new(binary)
        .args([
            "verify",
            "--prompt",
            "BG-06-PASS",
            "--record-dir",
            directory.to_str().expect("temporary path must be UTF-8"),
            "--redact",
            secret,
            "--",
            "rustc",
            "--version",
        ])
        .env("BONSAI_FIXTURE_SECRET", secret)
        .status()
        .expect("passing fixture must start");
    assert!(pass.success());

    let fail = Command::new(binary)
        .args([
            "verify",
            "--prompt",
            "BG-06-FAIL",
            "--record-dir",
            directory.to_str().expect("temporary path must be UTF-8"),
            "--redact",
            secret,
            "--",
            "rustc",
            "--definitely-invalid-bonsai-option",
        ])
        .env("BONSAI_FIXTURE_SECRET", secret)
        .status()
        .expect("failing fixture must start");
    assert!(!fail.success());

    let records = fs::read_to_string(directory.join("records.jsonl")).expect("records must exist");
    let parsed: Vec<Value> = records
        .lines()
        .map(|line| serde_json::from_str(line).expect("record must be JSON"))
        .collect();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0]["result"], "pass");
    assert_eq!(parsed[1]["result"], "fail");
    assert_ne!(parsed[0]["exit_code"], parsed[1]["exit_code"]);
    assert!(
        parsed
            .iter()
            .all(|record| record["working_directory"] == "<repository-root>")
    );

    let retained = fs::read_dir(&directory)
        .expect("fixture directory must be readable")
        .flat_map(|entry| {
            let path = entry.expect("directory entry must be readable").path();
            if path.is_dir() {
                fs::read_dir(path)
                    .expect("artifact directory must be readable")
                    .map(|child| child.expect("artifact entry must be readable").path())
                    .collect::<Vec<_>>()
            } else {
                vec![path]
            }
        })
        .map(|path| fs::read_to_string(path).expect("fixture artifacts must be UTF-8"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!retained.contains(secret));

    fs::remove_dir_all(&directory).expect("fixture directory must be removable");
}
