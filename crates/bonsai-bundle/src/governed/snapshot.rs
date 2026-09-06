use crate::BlobId;
use crate::validation::{checked_root, resolve_path};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

type Result<T> = std::result::Result<T, &'static str>;
const MAXIMUM_BYTES: u64 = 536_870_912;
const SPECIAL: [&str; 3] = ["artifact-index.json", "run-status.json", "run-receipt.json"];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    format: String,
    run_id: String,
    index_sha256: String,
    status_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Index {
    format: String,
    files: Vec<Entry>,
    excluded: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: String,
    bytes: u64,
    sha256: String,
}

pub(super) struct Snapshot {
    files: BTreeMap<String, Vec<u8>>,
    pub(super) run_id: String,
    pub(super) index_sha256: String,
    pub(super) total_bytes: u64,
}

impl Snapshot {
    pub(super) fn load(root: &Path, expected_receipt: &str) -> Result<Self> {
        canonical_digest(expected_receipt)?;
        let root = checked_root(root).map_err(|_| "RUN_ROOT_UNSAFE")?;
        let receipt_bytes = read(&root, "run-receipt.json", 4096)?;
        if digest(&receipt_bytes) != expected_receipt {
            return Err("RUN_RECEIPT_HASH_MISMATCH");
        }
        let receipt: Receipt =
            serde_json::from_slice(&receipt_bytes).map_err(|_| "RUN_RECEIPT_INVALID")?;
        if receipt.format != "bonsai.run-receipt/v1" {
            return Err("RUN_RECEIPT_VERSION_UNSUPPORTED");
        }
        canonical_digest(&receipt.index_sha256)?;
        canonical_digest(&receipt.status_sha256)?;
        let index_bytes = read(&root, "artifact-index.json", 1024 * 1024)?;
        let status_bytes = read(&root, "run-status.json", 4096)?;
        if digest(&index_bytes) != receipt.index_sha256
            || digest(&status_bytes) != receipt.status_sha256
        {
            return Err("RUN_COMPLETION_HASH_MISMATCH");
        }
        let index: Index = serde_json::from_slice(&index_bytes).map_err(|_| "RUN_INDEX_INVALID")?;
        validate_index(&index)?;
        let mut files = BTreeMap::new();
        let mut total_bytes =
            u64::try_from(receipt_bytes.len() + index_bytes.len() + status_bytes.len())
                .map_err(|_| "RUN_SIZE_OVERFLOW")?;
        for entry in index.files {
            safe_name(&entry.path)?;
            canonical_digest(&entry.sha256)?;
            if SPECIAL.contains(&entry.path.as_str()) || entry.path == "run-status.pending" {
                return Err("RUN_INDEX_ROLE_INVALID");
            }
            total_bytes = total_bytes
                .checked_add(entry.bytes)
                .ok_or("RUN_SIZE_OVERFLOW")?;
            if total_bytes > MAXIMUM_BYTES {
                return Err("RUN_VERIFICATION_BOUND_EXCEEDED");
            }
            let bytes = read(&root, &entry.path, entry.bytes)?;
            if u64::try_from(bytes.len()).map_err(|_| "RUN_SIZE_OVERFLOW")? != entry.bytes
                || digest(&bytes) != entry.sha256
            {
                return Err("RUN_FILE_HASH_MISMATCH");
            }
            if files.insert(entry.path, bytes).is_some() {
                return Err("RUN_DUPLICATE_FILE");
            }
        }
        files.insert("artifact-index.json".into(), index_bytes);
        files.insert("run-status.json".into(), status_bytes);
        files.insert("run-receipt.json".into(), receipt_bytes);
        let actual = inventory(&root, &root, 0, &mut 0)?;
        if actual != files.keys().cloned().collect() {
            return Err("RUN_UNINDEXED_FILE");
        }
        Ok(Self {
            files,
            run_id: receipt.run_id,
            index_sha256: receipt.index_sha256,
            total_bytes,
        })
    }

    pub(super) fn bytes(&self, name: &str) -> Result<&[u8]> {
        self.files
            .get(name)
            .map(Vec::as_slice)
            .ok_or("RUN_REQUIRED_FILE_MISSING")
    }

    pub(super) fn json(&self, name: &str) -> Result<Value> {
        serde_json::from_slice(self.bytes(name)?).map_err(|_| "RUN_JSON_INVALID")
    }
}

fn validate_index(index: &Index) -> Result<()> {
    if index.format != "bonsai.run-artifact-index/v1"
        || index.files.is_empty()
        || index.files.len() > 128
    {
        return Err("RUN_INDEX_INVALID");
    }
    let expected = BTreeSet::from([
        "artifact-index.json",
        "run-status.json",
        "run-status.pending",
        "run-receipt.json",
    ]);
    let actual = index
        .excluded
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if actual != expected || actual.len() != index.excluded.len() {
        return Err("RUN_INDEX_EXCLUSIONS_INVALID");
    }
    Ok(())
}

pub(super) fn digest(bytes: &[u8]) -> String {
    BlobId::digest(bytes).to_hex()
}

fn canonical_digest(value: &str) -> Result<()> {
    if BlobId::from_hex(value)
        .map_err(|_| "RUN_DIGEST_INVALID")?
        .to_hex()
        != value
    {
        return Err("RUN_DIGEST_INVALID");
    }
    Ok(())
}

fn safe_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > 512
        || name.contains(['\\', ':'])
        || name
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err("RUN_FILE_PATH_UNSAFE");
    }
    Ok(())
}

fn read(root: &Path, name: &str, maximum: u64) -> Result<Vec<u8>> {
    safe_name(name)?;
    let path = resolve_path(root, Path::new(name)).map_err(|_| "RUN_FILE_PATH_UNSAFE")?;
    let file = File::open(path).map_err(|_| "RUN_FILE_UNREADABLE")?;
    let mut bytes = Vec::new();
    file.take(maximum.checked_add(1).ok_or("RUN_SIZE_OVERFLOW")?)
        .read_to_end(&mut bytes)
        .map_err(|_| "RUN_FILE_UNREADABLE")?;
    if u64::try_from(bytes.len()).map_err(|_| "RUN_SIZE_OVERFLOW")? > maximum {
        return Err("RUN_FILE_SIZE_INVALID");
    }
    Ok(bytes)
}

fn inventory(
    root: &Path,
    directory: &Path,
    depth: u8,
    count: &mut u64,
) -> Result<BTreeSet<String>> {
    if depth > 16 {
        return Err("RUN_DIRECTORY_DEPTH_EXCEEDED");
    }
    let mut files = BTreeSet::new();
    for entry in fs::read_dir(directory).map_err(|_| "RUN_DIRECTORY_UNREADABLE")? {
        *count += 1;
        if *count > 256 {
            return Err("RUN_DIRECTORY_BOUND_EXCEEDED");
        }
        let entry = entry.map_err(|_| "RUN_DIRECTORY_UNREADABLE")?;
        let path = entry.path();
        let name = entry.file_name();
        safe_name(name.to_str().ok_or("RUN_FILE_PATH_UNSAFE")?)?;
        let kind = entry.file_type().map_err(|_| "RUN_FILE_UNREADABLE")?;
        if kind.is_symlink() {
            return Err("RUN_FILE_PATH_UNSAFE");
        }
        if kind.is_dir() {
            files.extend(inventory(root, &path, depth + 1, count)?);
        } else if kind.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| "RUN_FILE_PATH_UNSAFE")?
                .to_str()
                .ok_or("RUN_FILE_PATH_UNSAFE")?
                .replace('\\', "/");
            if !files.insert(relative) {
                return Err("RUN_DUPLICATE_FILE");
            }
        } else {
            return Err("RUN_FILE_PATH_UNSAFE");
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::{Snapshot, digest};
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    fn fixture() -> (TempDir, String) {
        let root = tempfile::tempdir().expect("temporary run");
        fs::write(root.path().join("data.json"), b"{}").expect("data");
        let index = serde_json::to_vec(&json!({
            "format":"bonsai.run-artifact-index/v1",
            "files":[{"path":"data.json","bytes":2,"sha256":digest(b"{}")}],
            "excluded":["artifact-index.json","run-status.json","run-status.pending","run-receipt.json"]
        })).expect("index");
        let status = b"{\"status\":\"COMPLETE\"}";
        fs::write(root.path().join("artifact-index.json"), &index).expect("index");
        fs::write(root.path().join("run-status.json"), status).expect("status");
        let receipt = serde_json::to_vec(&json!({
            "format":"bonsai.run-receipt/v1","run_id":"test",
            "index_sha256":digest(&index),"status_sha256":digest(status)
        }))
        .expect("receipt");
        fs::write(root.path().join("run-receipt.json"), &receipt).expect("receipt");
        (root, digest(&receipt))
    }

    #[test]
    fn snapshot_keeps_verified_bytes_after_external_mutation() {
        let (root, pin) = fixture();
        let snapshot = Snapshot::load(root.path(), &pin).expect("verified bytes");
        fs::write(root.path().join("data.json"), b"[]").expect("mutation");
        assert_eq!(snapshot.bytes("data.json").expect("retained"), b"{}");
        assert!(matches!(
            Snapshot::load(root.path(), &pin),
            Err("RUN_FILE_HASH_MISMATCH")
        ));
    }

    #[test]
    fn rewritten_receipt_cannot_replace_external_pin() {
        let (root, pin) = fixture();
        fs::write(root.path().join("run-receipt.json"), b"{}").expect("mutation");
        assert!(matches!(
            Snapshot::load(root.path(), &pin),
            Err("RUN_RECEIPT_HASH_MISMATCH")
        ));
    }

    #[test]
    fn completion_marker_and_unindexed_files_are_bound() {
        let (root, pin) = fixture();
        fs::write(root.path().join("run-status.json"), b"{}").expect("mutation");
        assert!(matches!(
            Snapshot::load(root.path(), &pin),
            Err("RUN_COMPLETION_HASH_MISMATCH")
        ));
        let (root, pin) = fixture();
        fs::write(root.path().join("hidden.json"), b"{}").expect("extra");
        assert!(matches!(
            Snapshot::load(root.path(), &pin),
            Err("RUN_UNINDEXED_FILE")
        ));
    }

    #[test]
    fn missing_file_and_traversal_never_enter_the_snapshot() {
        let (root, pin) = fixture();
        fs::remove_file(root.path().join("data.json")).expect("remove");
        assert!(Snapshot::load(root.path(), &pin).is_err());
        for name in [
            "../file",
            "/file",
            "nested/../file",
            "file\\other",
            "C:/file",
            "file//other",
        ] {
            assert!(super::safe_name(name).is_err(), "{name}");
        }
    }
}
