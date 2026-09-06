use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/governed-run/v1/positive/telemetry/segment-00000000000000000000.bseg")
}
fn invoke(root: &Path, output: &Path, maximum: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bonsai-xtask"))
        .args(["recover-run", "--root"])
        .arg(root)
        .arg("--output")
        .arg(output)
        .arg("--maximum-output-bytes")
        .arg(maximum)
        .output()
        .expect("recover-run")
}
fn interrupted(root: &Path) -> Vec<u8> {
    let bytes = fs::read(fixture()).expect("real governed trace");
    let mut offset = 60_usize;
    for _ in 0..50 {
        assert_eq!(&bytes[offset..offset + 8], b"BNSFRM01");
        let length = u32::from_le_bytes(bytes[offset + 8..offset + 12].try_into().expect("length"));
        offset += 44 + usize::try_from(length).expect("length fits");
    }
    let bytes = bytes[..offset + 20].to_vec();
    fs::create_dir(root).expect("root");
    fs::create_dir(root.join("telemetry")).expect("telemetry");
    fs::write(
        root.join("telemetry/segment-00000000000000000000.open"),
        &bytes,
    )
    .expect("interrupted");
    bytes
}

#[test]
fn real_governed_prefix_is_salvaged_without_becoming_track_a() {
    let temp = tempfile::tempdir().expect("temporary");
    let root = temp.path().join("source");
    let original = interrupted(&root);
    let destination = temp.path().join("recovered");
    let result = invoke(&root, &destination, "16777216");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).expect("report");
    assert_eq!(report["status"], "INCOMPLETE");
    assert_eq!(report["recovered_events"], 50);
    assert_eq!(report["truncated_tail"], true);
    assert_eq!(report["continuation"], true);
    assert_eq!(report["uninterrupted_track_a_eligible"], false);
    assert_eq!(report["agent_state_restored"], false);
    let id =
        bonsai_bundle::BlobId::from_hex(report["source_capture_sha256"].as_str().expect("digest"))
            .expect("id");
    let blob = bonsai_bundle::validate_blob(&destination, id).expect("retained source");
    assert_eq!(
        blob.byte_length,
        u64::try_from(original.len()).expect("length")
    );
    assert_eq!(
        fs::read(root.join("telemetry/segment-00000000000000000000.open")).expect("source"),
        original
    );
    assert_eq!(
        bonsai_bundle::validate_segment(destination.join("segment-00000000000000000000.bseg"))
            .expect("segment")
            .frame_count,
        50
    );
    let verification = Command::new(env!("CARGO_BIN_EXE_bonsai-xtask"))
        .args(["verify-run", "--root"])
        .arg(&destination)
        .arg("--receipt-sha256")
        .arg("0".repeat(64))
        .output()
        .expect("verifier");
    assert!(!verification.status.success());
}

#[test]
fn recovery_rejects_corruption_and_preflights_output_quota() {
    let temp = tempfile::tempdir().expect("temporary");
    let root = temp.path().join("source");
    let mut bytes = interrupted(&root);
    let quota_output = temp.path().join("too-small");
    assert!(!invoke(&root, &quota_output, "4096").status.success());
    assert!(!quota_output.exists());
    bytes[72] ^= 1;
    let source = root.join("telemetry/segment-00000000000000000000.open");
    fs::write(&source, &bytes).expect("corrupt");
    let result = invoke(&root, &temp.path().join("corrupt"), "16777216");
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("CHECKSUM"));
    assert_eq!(fs::read(source).expect("preserved"), bytes);
}
