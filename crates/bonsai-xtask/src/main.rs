//! Repository-local task runner.

#![forbid(unsafe_code)]

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::env;
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

mod schema_check;

#[derive(Debug)]
struct VerifyArgs {
    prompt: String,
    record_dir: PathBuf,
    evidence_class: String,
    runner_class: String,
    redactions: Vec<String>,
    command: Vec<OsString>,
}

#[derive(Serialize)]
struct ArtifactRecord {
    path: String,
    sha256: String,
    bytes: usize,
}

#[derive(Serialize)]
struct VerificationRecord {
    schema: &'static str,
    record_id: String,
    prompt: String,
    command: Vec<String>,
    working_directory: &'static str,
    source_revision: String,
    dirty_before: bool,
    os: &'static str,
    architecture: &'static str,
    evidence_class: String,
    runner_class: String,
    started_unix_ns_utc: u128,
    ended_unix_ns_utc: u128,
    duration_ns: u128,
    exit_code: i32,
    result: &'static str,
    stdout: ArtifactRecord,
    stderr: ArtifactRecord,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(u8::try_from(code.clamp(0, 255)).unwrap_or(1)),
        Err(error) => {
            eprintln!("bonsai-xtask: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<i32, String> {
    let mut args = env::args_os();
    let _program = args.next();
    match args
        .next()
        .and_then(|arg| arg.into_string().ok())
        .as_deref()
    {
        Some("verify") => {
            let remaining = args.collect::<Vec<_>>();
            verify(parse_verify_args(&remaining)?)
        }
        Some("schema-check") => {
            if args.next().is_some() {
                return Err("schema-check does not accept arguments".to_owned());
            }
            schema_check::run()?;
            Ok(0)
        }
        Some("bundle-check") => {
            let remaining = args.collect::<Vec<_>>();
            bundle_check(&remaining)
        }
        _ => Err(usage()),
    }
}

fn usage() -> String {
    "usage:\n  cargo xtask verify --prompt <ID> [--record-dir <PATH>] \
     [--evidence-class <CLASS>] [--runner-class <CLASS>] [--redact <TEXT>] -- <COMMAND> [ARGS...]\n  \
     cargo xtask schema-check
  cargo xtask bundle-check [--root <PATH>] <MANIFEST>"
        .to_owned()
}

fn bundle_check(args: &[OsString]) -> Result<i32, String> {
    let mut root = PathBuf::from(".");
    let manifest;
    match args {
        [path] => manifest = PathBuf::from(path),
        [option, root_path, path] if option == "--root" => {
            root = PathBuf::from(root_path);
            manifest = PathBuf::from(path);
        }
        _ => return Err(usage()),
    }
    let schemas = bonsai_bundle::BundleSchemas {
        bundle_manifest: embedded_json(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../schemas/bundle-manifest-v1.json"
        )))?,
        experiment_manifest: embedded_json(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../schemas/experiment-manifest-v1.json"
        )))?,
        track_declaration: embedded_json(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../schemas/track-declaration-v1.json"
        )))?,
        platform_inventory: embedded_json(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../schemas/platform-inventory-v1.json"
        )))?,
        resource_policy: embedded_json(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../schemas/resource-policy-v1.json"
        )))?,
        metric_estimate: embedded_json(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../schemas/metric-estimate-v1.json"
        )))?,
    };
    let report = bonsai_bundle::validate_result_bundle(root, manifest, &schemas)
        .map_err(|error| error.to_string())?;
    serde_json::to_writer(io::stdout().lock(), &report)
        .map_err(|error| format!("serialize bundle validation report: {error}"))?;
    println!();
    Ok(match report.verdict {
        bonsai_bundle::OverallVerdict::Invalid | bonsai_bundle::OverallVerdict::Indeterminate => 2,
        bonsai_bundle::OverallVerdict::Valid
        | bonsai_bundle::OverallVerdict::Migratable
        | bonsai_bundle::OverallVerdict::ForwardReadable
        | bonsai_bundle::OverallVerdict::ValidWithLimitations => 0,
    })
}

fn embedded_json(bytes: &[u8]) -> Result<serde_json::Value, String> {
    serde_json::from_slice(bytes).map_err(|error| format!("parse embedded schema: {error}"))
}

fn parse_verify_args(args: &[OsString]) -> Result<VerifyArgs, String> {
    let mut prompt = None;
    let mut record_dir = PathBuf::from("evidence/verification");
    let mut evidence_class = "local".to_owned();
    let mut runner_class = "unknown".to_owned();
    let mut redactions = Vec::new();
    let mut index = 0;

    while index < args.len() {
        if args[index] == "--" {
            let command = args[index + 1..].to_vec();
            if command.is_empty() {
                return Err("verify requires a command after --".to_owned());
            }
            return Ok(VerifyArgs {
                prompt: prompt.ok_or_else(|| "verify requires --prompt".to_owned())?,
                record_dir,
                evidence_class,
                runner_class,
                redactions,
                command,
            });
        }

        let option = args[index]
            .to_str()
            .ok_or_else(|| "verify options must be valid UTF-8".to_owned())?;
        let value = args
            .get(index + 1)
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("{option} requires a UTF-8 value"))?;
        match option {
            "--prompt" => prompt = Some(value.to_owned()),
            "--record-dir" => record_dir = PathBuf::from(value),
            "--evidence-class" => value.clone_into(&mut evidence_class),
            "--runner-class" => value.clone_into(&mut runner_class),
            "--redact" => redactions.push(value.to_owned()),
            _ => return Err(format!("unknown verify option: {option}")),
        }
        index += 2;
    }
    Err("verify requires -- followed by a command".to_owned())
}

fn verify(args: VerifyArgs) -> Result<i32, String> {
    validate_label("prompt", &args.prompt)?;
    validate_label("evidence class", &args.evidence_class)?;
    validate_label("runner class", &args.runner_class)?;
    if args.redactions.iter().any(String::is_empty) {
        return Err("redaction text cannot be empty".to_owned());
    }

    let source_revision = git_stdout(["rev-parse", "HEAD"])?;
    let dirty_before = !git_stdout(["status", "--porcelain"])?.is_empty();
    let started_unix_ns_utc = unix_ns()?;
    let timer = Instant::now();
    let output = Command::new(&args.command[0])
        .args(&args.command[1..])
        .output()
        .map_err(|error| format!("failed to start command: {error}"))?;
    let duration_ns = timer.elapsed().as_nanos();
    let ended_unix_ns_utc = unix_ns()?;
    let exit_code = output.status.code().unwrap_or(-1);
    let result = if output.status.success() {
        "pass"
    } else {
        "fail"
    };

    let record_id = format!("{}-{started_unix_ns_utc}", sanitize_id(&args.prompt));
    let artifact_dir = args.record_dir.join("artifacts");
    fs::create_dir_all(&artifact_dir).map_err(io_error("create artifact directory"))?;
    let stdout = redact_bytes(&output.stdout, &args.redactions);
    let stderr = redact_bytes(&output.stderr, &args.redactions);
    let stdout_record = write_artifact(&artifact_dir, &record_id, "stdout", &stdout)?;
    let stderr_record = write_artifact(&artifact_dir, &record_id, "stderr", &stderr)?;

    let command = args
        .command
        .iter()
        .map(|part| redact(&part.to_string_lossy(), &args.redactions))
        .collect();
    let record = VerificationRecord {
        schema: "bonsai.verification-record/v1",
        record_id,
        prompt: args.prompt,
        command,
        working_directory: "<repository-root>",
        source_revision,
        dirty_before,
        os: env::consts::OS,
        architecture: env::consts::ARCH,
        evidence_class: args.evidence_class,
        runner_class: args.runner_class,
        started_unix_ns_utc,
        ended_unix_ns_utc,
        duration_ns,
        exit_code,
        result,
        stdout: stdout_record,
        stderr: stderr_record,
    };
    append_record(&args.record_dir, &record)?;
    println!(
        "verification record {}: {} (exit {exit_code})",
        record.record_id, record.result
    );
    Ok(exit_code)
}

fn validate_label(label: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_.".contains(character))
    {
        return Err(format!(
            "{label} must use only ASCII letters, digits, hyphen, underscore, or period"
        ));
    }
    Ok(())
}

fn unix_ns() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .map_err(|error| format!("system clock precedes Unix epoch: {error}"))
}

fn git_stdout<const N: usize>(args: [&str; N]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|error| format!("failed to start git: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git command failed with exit code {:?}",
            output.status.code()
        ));
    }
    String::from_utf8(output.stdout)
        .map(|text| text.trim().to_owned())
        .map_err(|_| "git output was not UTF-8".to_owned())
}

fn sanitize_id(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect()
}

fn redact_bytes(bytes: &[u8], redactions: &[String]) -> Vec<u8> {
    redact(&String::from_utf8_lossy(bytes), redactions).into_bytes()
}

fn redact(text: &str, redactions: &[String]) -> String {
    redactions.iter().fold(text.to_owned(), |value, secret| {
        value.replace(secret, "<redacted>")
    })
}

fn write_artifact(
    directory: &Path,
    record_id: &str,
    stream: &str,
    bytes: &[u8],
) -> Result<ArtifactRecord, String> {
    let file_name = format!("{record_id}.{stream}.txt");
    let path = directory.join(&file_name);
    fs::write(&path, bytes).map_err(io_error("write output artifact"))?;
    Ok(ArtifactRecord {
        path: format!("artifacts/{file_name}"),
        sha256: sha256_hex(bytes),
        bytes: bytes.len(),
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn append_record(directory: &Path, record: &VerificationRecord) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(io_error("create record directory"))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join("records.jsonl"))
        .map_err(io_error("open append-only verification record"))?;
    serde_json::to_writer(&mut file, record)
        .map_err(|error| format!("serialize verification record: {error}"))?;
    file.write_all(b"\n")
        .map_err(io_error("append verification record"))?;
    file.sync_all()
        .map_err(io_error("sync verification record"))
}

fn io_error(context: &'static str) -> impl FnOnce(io::Error) -> String {
    move |error| format!("{context}: {error}")
}

#[cfg(test)]
mod tests {
    use super::{redact, sanitize_id};

    #[test]
    fn redacts_every_literal_occurrence() {
        assert_eq!(
            redact("token=secret; secret", &["secret".to_owned()]),
            "token=<redacted>; <redacted>"
        );
    }

    #[test]
    fn creates_portable_record_ids() {
        assert_eq!(sanitize_id("BG-06.fixture"), "BG-06-fixture");
    }
}
