//! M4 acceptance harness: threat model, hardening, supply chain, L profile, RC evidence.

#![forbid(unsafe_code)]

use bonsai_contracts::resource::{ResourcePolicy, validate_resource_policy};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub const D21_DISCLAIMER: &str = "not a hostile-native-code sandbox";
pub const L_MIN_STEPS: u64 = 10_000_000;
pub const L_MIN_WALL_NS: u64 = 259_200_000_000_000;
pub const L_MIN_PAIRED_SEEDS: usize = 3;
pub const CI_PROBE_STEPS: u64 = 32;
const HMAC_BLOCK: usize = 64;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreatModel {
    pub schema: String,
    pub disclaimer_id: String,
    pub disclaimer: String,
    pub boundaries: Vec<ThreatBoundary>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreatBoundary {
    pub id: String,
    pub name: String,
    pub trust_boundary: String,
    pub threats: Vec<String>,
    pub controls: Vec<String>,
    pub tests: Vec<String>,
    pub residual_risk: String,
    pub prompts: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrityEnvelope {
    pub payload_sha256: String,
    pub hmac_sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupplyWaiverFile {
    pub schema: String,
    pub policy: String,
    pub accepted: Vec<SupplyWaiver>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupplyWaiver {
    pub id: String,
    pub package: String,
    pub severity: String,
    pub accepted_by: String,
    pub expires_on: String,
    pub residual_risk: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostAttestation {
    pub schema: String,
    pub os_family: String,
    pub architecture: String,
    pub evidence_class: String,
    pub runner_class: String,
    pub physical_acceptance: bool,
    pub long_duration_claim: bool,
    pub profile_id: String,
    pub required_wall_time_ns: u64,
    pub required_steps: u64,
    pub required_paired_seeds: u64,
    pub status: String,
    pub result: String,
    pub reason_codes: Vec<String>,
    pub observed_wall_time_ns: Option<u64>,
    pub observed_steps: Option<u64>,
    pub paired_seeds_completed: u64,
    pub thermal_record: String,
    pub attestor: String,
    pub notes: String,
}

#[derive(Clone, Debug)]
pub struct DurationProbe {
    pub steps: u64,
    pub elapsed_ns: u128,
    pub evidence_class: &'static str,
    pub long_duration_claim: bool,
}

#[derive(Debug)]
pub struct AcceptError(pub String);

impl std::fmt::Display for AcceptError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for AcceptError {}

/// Load the frozen BV-11 machine table.
///
/// # Errors
///
/// Returns a parse or coverage failure when the fixture is incomplete.
pub fn load_threat_model(root: &Path) -> Result<ThreatModel, AcceptError> {
    read_json(&root.join("fixtures/threat-model/v1/boundaries.json"))
}

/// Confirm every trust boundary has controls, tests, residual risk, and prompt mapping.
///
/// # Errors
///
/// Returns the first missing field, missing test path, or document gap.
pub fn verify_threat_model(root: &Path) -> Result<(), AcceptError> {
    let model = load_threat_model(root)?;
    if model.schema != "bonsai.threat-model/v1" || model.disclaimer_id != "D-21" {
        return Err(AcceptError("threat model identity is not D-21 / v1".into()));
    }
    if !model
        .disclaimer
        .to_ascii_lowercase()
        .contains(D21_DISCLAIMER)
    {
        return Err(AcceptError("machine disclaimer is not prominent".into()));
    }
    let doc = fs::read_to_string(root.join("docs/security/THREAT-MODEL.md"))
        .map_err(|error| AcceptError(format!("read threat model: {error}")))?;
    let head: String = doc.lines().take(12).collect::<Vec<_>>().join("\n");
    if !head.to_ascii_lowercase().contains(D21_DISCLAIMER) {
        return Err(AcceptError(
            "D-21 hostile-sandbox disclaimer is not prominent".into(),
        ));
    }
    let required = [
        "TB-01", "TB-02", "TB-03", "TB-04", "TB-05", "TB-06", "TB-07", "TB-08", "TB-09", "TB-10",
    ];
    let seen = model
        .boundaries
        .iter()
        .map(|boundary| boundary.id.as_str())
        .collect::<BTreeSet<_>>();
    if seen != required.into_iter().collect() {
        return Err(AcceptError(format!(
            "threat boundaries must be TB-01..TB-10, observed {seen:?}"
        )));
    }
    for boundary in &model.boundaries {
        if boundary.trust_boundary.is_empty()
            || boundary.threats.is_empty()
            || boundary.controls.is_empty()
            || boundary.tests.is_empty()
            || boundary.residual_risk.is_empty()
            || boundary.prompts.is_empty()
        {
            return Err(AcceptError(format!(
                "{} missing controls, tests, residual risk, or prompt mapping",
                boundary.id
            )));
        }
        if !doc.contains(&boundary.id) {
            return Err(AcceptError(format!(
                "THREAT-MODEL.md omits {}",
                boundary.id
            )));
        }
        for test in &boundary.tests {
            if !root.join(test).exists() {
                return Err(AcceptError(format!(
                    "{} names missing test {test}",
                    boundary.id
                )));
            }
        }
    }
    Ok(())
}

/// HMAC-SHA256 plus payload SHA-256. No external key service.
#[must_use]
pub fn sign_bytes(key: &[u8], payload: &[u8]) -> IntegrityEnvelope {
    IntegrityEnvelope {
        payload_sha256: sha256_hex(payload),
        hmac_sha256: hex_encode(&hmac_sha256(key, payload)),
    }
}

/// Verify a signed payload using only the local operator key.
///
/// # Errors
///
/// Returns `TAMPER_DETECTED` when either digest mismatches.
pub fn verify_bytes(
    key: &[u8],
    payload: &[u8],
    envelope: &IntegrityEnvelope,
) -> Result<(), AcceptError> {
    let expected = sign_bytes(key, payload);
    if expected == *envelope {
        Ok(())
    } else {
        Err(AcceptError("TAMPER_DETECTED".into()))
    }
}

/// Replace every supplied secret literal with `<redacted>`.
#[must_use]
pub fn redact(text: &str, secrets: &[&str]) -> String {
    secrets.iter().fold(text.to_owned(), |value, secret| {
        value.replace(secret, "<redacted>")
    })
}

/// Fail closed when a secret literal remains in published text.
///
/// # Errors
///
/// Returns `SECRET_LEAK` when any listed secret is still present.
pub fn reject_secret_leak(text: &str, secrets: &[&str]) -> Result<(), AcceptError> {
    if secrets.iter().any(|secret| text.contains(secret)) {
        Err(AcceptError("SECRET_LEAK".into()))
    } else {
        Ok(())
    }
}

/// Fail closed when artifact count or lineage depth exceeds the bound.
///
/// # Errors
///
/// Returns a stable bound code.
pub fn enforce_graph_and_artifact_bounds(
    artifact_count: u64,
    graph_depth: u64,
    max_artifacts: u64,
    max_depth: u64,
) -> Result<(), AcceptError> {
    if artifact_count > max_artifacts {
        return Err(AcceptError("ARTIFACT_BOMB_BOUNDED".into()));
    }
    if graph_depth > max_depth {
        return Err(AcceptError("GRAPH_DEPTH_BOUNDED".into()));
    }
    Ok(())
}

/// Load lockfile/SBOM identity and refuse silent critical waivers.
///
/// # Errors
///
/// Returns a policy failure when locks are missing or a critical waiver is incomplete.
pub fn verify_supply_chain(root: &Path) -> Result<(String, String), AcceptError> {
    let cargo = fs::read(root.join("Cargo.lock"))
        .map_err(|error| AcceptError(format!("Cargo.lock: {error}")))?;
    let uv =
        fs::read(root.join("uv.lock")).map_err(|error| AcceptError(format!("uv.lock: {error}")))?;
    let waivers: SupplyWaiverFile = read_json(&root.join("fixtures/supply-chain/v1/waivers.json"))?;
    if waivers.schema != "bonsai.supply-chain-waivers/v1" {
        return Err(AcceptError("waiver schema mismatch".into()));
    }
    for waiver in &waivers.accepted {
        if matches!(waiver.severity.as_str(), "critical" | "high")
            && (waiver.accepted_by.is_empty()
                || waiver.expires_on.is_empty()
                || waiver.residual_risk.is_empty())
        {
            return Err(AcceptError(format!(
                "silent {} waiver {}",
                waiver.severity, waiver.id
            )));
        }
    }
    for required in [
        "docs/governance/DEPENDENCY-POLICY.md",
        "scripts/generate_sbom.py",
        "scripts/offline_restore.py",
        "scripts/package_rc.py",
    ] {
        if !root.join(required).is_file() {
            return Err(AcceptError(format!("missing supply-chain file {required}")));
        }
    }
    Ok((sha256_hex(&cargo), sha256_hex(&uv)))
}

/// Confirm the L manifest meets D-16 minima without treating it as a 72 h pass.
///
/// # Errors
///
/// Returns a profile or schema failure.
pub fn verify_l_manifest(root: &Path) -> Result<(), AcceptError> {
    let manifest: Value = read_json(&root.join("fixtures/l-profile/v1/manifest.json"))?;
    let profile = manifest
        .get("resource_profile")
        .ok_or_else(|| AcceptError("L manifest missing resource_profile".into()))?;
    let profile_id = profile
        .get("profile_id")
        .and_then(Value::as_str)
        .ok_or_else(|| AcceptError("L profile_id missing".into()))?;
    let steps = profile
        .get("step_limit")
        .and_then(Value::as_u64)
        .ok_or_else(|| AcceptError("L step_limit missing".into()))?;
    let wall = profile
        .get("wall_time_limit_ns")
        .and_then(Value::as_u64)
        .ok_or_else(|| AcceptError("L wall_time_limit_ns missing".into()))?;
    if profile_id != "L" || steps < L_MIN_STEPS || wall < L_MIN_WALL_NS {
        return Err(AcceptError("L manifest does not meet D-16 minima".into()));
    }
    let seeds = manifest
        .get("seeds")
        .and_then(Value::as_array)
        .ok_or_else(|| AcceptError("L seeds missing".into()))?;
    if seeds.len() < L_MIN_PAIRED_SEEDS * 2 {
        return Err(AcceptError("L manifest needs three paired seeds".into()));
    }
    let eligibility = manifest
        .pointer("/publication_eligibility/status")
        .and_then(Value::as_str);
    if eligibility != Some("ineligible") {
        return Err(AcceptError(
            "L manifest cannot be publication-eligible without a physical host run".into(),
        ));
    }
    let policy: ResourcePolicy =
        read_json(&root.join("fixtures/l-profile/v1/resource-policy.json"))?;
    if policy.resource_profile_id != "L" {
        return Err(AcceptError("L resource policy profile mismatch".into()));
    }
    validate_resource_policy(&policy).map_err(|error| AcceptError(error.to_string()))?;
    Ok(())
}

/// Confirm each required OS attestation is honest not-run / indeterminate.
///
/// # Errors
///
/// Returns a failure if any attestation claims a 72 h pass or physical acceptance.
pub fn verify_host_attestations(root: &Path) -> Result<Vec<HostAttestation>, AcceptError> {
    let mut rows = Vec::new();
    for name in [
        "windows-not-run.json",
        "macos-not-run.json",
        "linux-not-run.json",
    ] {
        let row: HostAttestation =
            read_json(&root.join("fixtures/host-attestation/v1").join(name))?;
        if row.physical_acceptance
            || row.long_duration_claim
            || row.status == "pass"
            || row.result == "pass"
            || row.evidence_class != "not-run"
        {
            return Err(AcceptError(format!(
                "{name} invents L acceptance or physical pass"
            )));
        }
        if row.required_wall_time_ns < L_MIN_WALL_NS
            || row.required_steps < L_MIN_STEPS
            || row.required_paired_seeds < L_MIN_PAIRED_SEEDS as u64
        {
            return Err(AcceptError(format!("{name} understates D-16 L minima")));
        }
        if row.observed_wall_time_ns.is_some() || row.observed_steps.is_some() {
            return Err(AcceptError(format!(
                "{name} records observed L totals without a host run"
            )));
        }
        rows.push(row);
    }
    let families = rows
        .iter()
        .map(|row| row.os_family.as_str())
        .collect::<HashSet<_>>();
    if families != HashSet::from(["windows", "macos", "linux"]) {
        return Err(AcceptError(
            "host attestations must cover Win/macOS/Linux".into(),
        ));
    }
    Ok(rows)
}

/// Refuse to treat a short CI probe as L acceptance.
///
/// # Errors
///
/// Returns `L_ACCEPTANCE_NOT_CLAIMED` when a caller asks to promote the probe.
pub fn run_ci_duration_probe(claim_as_l: bool) -> Result<DurationProbe, AcceptError> {
    let started = Instant::now();
    let mut acc = 0_u64;
    for step in 0..CI_PROBE_STEPS {
        acc = acc.wrapping_add(step);
    }
    let _ = acc;
    let probe = DurationProbe {
        steps: CI_PROBE_STEPS,
        elapsed_ns: started.elapsed().as_nanos(),
        evidence_class: "hosted-ci",
        long_duration_claim: false,
    };
    if claim_as_l || probe.steps >= L_MIN_STEPS || probe.elapsed_ns >= u128::from(L_MIN_WALL_NS) {
        return Err(AcceptError("L_ACCEPTANCE_NOT_CLAIMED".into()));
    }
    Ok(probe)
}

/// Confirm operator handoff files exist and refuse overclaim language.
///
/// # Errors
///
/// Returns a missing-file or overclaim failure.
pub fn verify_operator_handoff(root: &Path) -> Result<(), AcceptError> {
    let required = [
        "docs/operator/INSTALL.md",
        "docs/operator/RUN.md",
        "docs/operator/ANALYZE.md",
        "docs/operator/ADAPTER.md",
        "docs/operator/METRIC.md",
        "docs/operator/CLAIM.md",
        "docs/operator/PLATFORM.md",
        "docs/operator/LIMITATIONS.md",
        "README.md",
        "docs/experiments/M1-HEARTBEAT.md",
    ];
    for path in required {
        if !root.join(path).is_file() {
            return Err(AcceptError(format!("missing operator file {path}")));
        }
    }
    let limitations = fs::read_to_string(root.join("docs/operator/LIMITATIONS.md"))
        .map_err(|error| AcceptError(error.to_string()))?;
    let readme = fs::read_to_string(root.join("README.md"))
        .map_err(|error| AcceptError(error.to_string()))?;
    for banned in [
        "instrument completion criteria pass",
        "72-hour physical-host pass",
    ] {
        if readme.contains(banned) || limitations.contains(banned) {
            return Err(AcceptError(format!("operator docs overclaim: {banned}")));
        }
    }
    if readme
        .to_ascii_lowercase()
        .contains("instrument is complete")
    {
        return Err(AcceptError(
            "README must not claim the instrument is complete".into(),
        ));
    }
    if !limitations.to_ascii_lowercase().contains(D21_DISCLAIMER) {
        return Err(AcceptError("LIMITATIONS.md must restate D-21".into()));
    }
    let run = fs::read_to_string(root.join("docs/operator/RUN.md"))
        .map_err(|error| AcceptError(error.to_string()))?;
    if !run.contains("bonsai_reference.heartbeat") {
        return Err(AcceptError(
            "RUN.md must show the M1 heartbeat command".into(),
        ));
    }
    if !run.contains("PYTHONPATH=python/bonsai-reference/src") {
        return Err(AcceptError(
            "RUN.md must set PYTHONPATH for the uninstalled package".into(),
        ));
    }
    Ok(())
}

/// Confirm RC notes, hashes, and SBOMs exist without publication actions.
///
/// # Errors
///
/// Returns a missing artifact or unauthorized-publication marker failure.
pub fn verify_release_candidate(root: &Path) -> Result<Vec<(String, String)>, AcceptError> {
    let notes = root.join("docs/releases/RC-0.0.0-NOTES.md");
    let audit = root.join("docs/verification/M4-COMPLETION-AUDIT.md");
    let sums = root.join("evidence/release-candidate/SHA256SUMS");
    let forbid = root.join("evidence/release-candidate/OD-03-NO-UPLOAD.txt");
    for path in [&notes, &audit, &sums, &forbid] {
        if !path.is_file() {
            return Err(AcceptError(format!(
                "missing RC file {}",
                path.strip_prefix(root).unwrap_or(path).display()
            )));
        }
    }
    let notes_text = fs::read_to_string(&notes).map_err(|error| AcceptError(error.to_string()))?;
    let forbid_text =
        fs::read_to_string(&forbid).map_err(|error| AcceptError(error.to_string()))?;
    for marker in [
        "NO git tag",
        "NO crates.io",
        "NO pypi",
        "NO marketing claim",
    ] {
        if !notes_text.contains(marker) && !forbid_text.contains(marker) {
            return Err(AcceptError(format!(
                "RC artifacts missing OD-03 marker {marker}"
            )));
        }
    }
    if notes_text
        .to_ascii_lowercase()
        .contains("instrument is complete")
    {
        return Err(AcceptError(
            "RC notes must not claim instrument completion".into(),
        ));
    }
    let sums_text = fs::read_to_string(&sums).map_err(|error| AcceptError(error.to_string()))?;
    check_listed_hashes(root, &sums_text)
}

fn check_listed_hashes(root: &Path, sums: &str) -> Result<Vec<(String, String)>, AcceptError> {
    let mut hashes = Vec::new();
    for line in sums.lines() {
        let mut parts = line.split_whitespace();
        let (Some(digest), Some(name)) = (parts.next(), parts.next()) else {
            continue;
        };
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(AcceptError(format!(
                "invalid SHA-256 in SHA256SUMS: {name}"
            )));
        }
        let path = root.join(name);
        let bytes = fs::read(&path)
            .map_err(|error| AcceptError(format!("read SHA256SUMS path {name}: {error}")))?;
        let actual = sha256_hex(&bytes);
        if actual != digest {
            return Err(AcceptError(format!("SHA256SUMS mismatch: {name}")));
        }
        hashes.push((name.to_owned(), digest.to_owned()));
    }
    if hashes.is_empty() {
        return Err(AcceptError("SHA256SUMS is empty".into()));
    }
    Ok(hashes)
}

#[must_use]
pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}

#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex_encode(&Sha256::digest(bytes))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, AcceptError> {
    let raw =
        fs::read(path).map_err(|error| AcceptError(format!("read {}: {error}", path.display())))?;
    serde_json::from_slice(&raw)
        .map_err(|error| AcceptError(format!("parse {}: {error}", path.display())))
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut key_block = [0_u8; HMAC_BLOCK];
    if key.len() > HMAC_BLOCK {
        key_block[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0_u8; HMAC_BLOCK];
    let mut opad = [0_u8; HMAC_BLOCK];
    for index in 0..HMAC_BLOCK {
        ipad[index] = key_block[index] ^ 0x36;
        opad[index] = key_block[index] ^ 0x5c;
    }
    let mut inner_input = Vec::with_capacity(HMAC_BLOCK + message.len());
    inner_input.extend_from_slice(&ipad);
    inner_input.extend_from_slice(message);
    let inner = Sha256::digest(&inner_input);
    let mut outer_input = [0_u8; HMAC_BLOCK + 32];
    outer_input[..HMAC_BLOCK].copy_from_slice(&opad);
    outer_input[HMAC_BLOCK..].copy_from_slice(&inner);
    Sha256::digest(outer_input).into()
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::{
        check_listed_hashes, hex_encode, hmac_sha256, redact, sha256_hex, sign_bytes, verify_bytes,
    };
    use std::fs;

    #[test]
    fn hmac_matches_known_rfc_4231_case() {
        let digest = hmac_sha256(b"Jefe", b"what do ya want for nothing?");
        assert_eq!(
            hex_encode(&digest),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn tampered_payload_fails_verify() {
        let envelope = sign_bytes(b"key", b"bundle");
        assert!(verify_bytes(b"key", b"bundle", &envelope).is_ok());
        assert_eq!(
            verify_bytes(b"key", b"tampered", &envelope)
                .expect_err("tamper")
                .0,
            "TAMPER_DETECTED"
        );
    }

    #[test]
    fn redaction_removes_every_literal() {
        assert_eq!(
            redact("token=secret; secret", &["secret"]),
            "token=<redacted>; <redacted>"
        );
    }

    #[test]
    fn listed_hash_mismatch_fails() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("artifact.txt");
        fs::write(&path, b"honest").expect("write");
        let honest = format!("{}  artifact.txt\n", sha256_hex(b"honest"));
        check_listed_hashes(directory.path(), &honest).expect("matching digest");
        let err = check_listed_hashes(
            directory.path(),
            &format!("{}  artifact.txt\n", "0".repeat(64)),
        )
        .expect_err("mismatch");
        assert_eq!(err.0, "SHA256SUMS mismatch: artifact.txt");
    }
}
