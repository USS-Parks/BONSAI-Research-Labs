use super::peer::hex;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn source_identity(manifest: &Value) -> Result<Value, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let revision = git(&root, &["rev-parse", "HEAD"])?;
    if manifest["source"]["revision"] != String::from_utf8_lossy(&revision).trim() {
        return Err("SOURCE_REVISION_MISMATCH".into());
    }
    let dirty = !git(&root, &["status", "--porcelain"])?.is_empty();
    if manifest["source"]["dirty"] != dirty {
        return Err("SOURCE_DIRTY_STATE_MISMATCH".into());
    }
    let tracked_delta = git(&root, &["diff", "HEAD", "--binary"])?;
    let patch_hash = hex(&Sha256::digest(&tracked_delta));
    if dirty && manifest["source"]["dirty_patch_sha256"] != patch_hash {
        return Err("SOURCE_PATCH_MISMATCH".into());
    }
    let mut sources = BTreeMap::new();
    for relative in [
        "crates",
        "proto",
        "schemas",
        "scripts",
        "python/bonsai-reference/src",
    ] {
        collect(&root, &root.join(relative), &mut sources, 0)?;
    }
    for relative in [
        "Cargo.toml",
        "Cargo.lock",
        "pyproject.toml",
        "uv.lock",
        "rust-toolchain.toml",
    ] {
        let path = root.join(relative);
        if path.is_file() {
            sources.insert(relative.to_owned(), file_hash(&path)?);
        }
    }
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    Ok(
        json!({"schema":"bonsai.run-source-identity/v1","revision":manifest["source"]["revision"],
        "dirty":dirty,"tracked_patch_sha256":patch_hash,"source_files":sources,
        "operator_executable_sha256":file_hash(&executable)?,
        "scope":"all Rust, Python, protocol, schema, and build/lock source files; includes untracked implementation files"}),
    )
}

fn collect(
    root: &Path,
    path: &Path,
    output: &mut BTreeMap<String, String>,
    depth: usize,
) -> Result<(), String> {
    if depth > 16 || output.len() > 10_000 {
        return Err("SOURCE_INVENTORY_BOUND_EXCEEDED".into());
    }
    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let metadata = entry.file_type().map_err(|e| e.to_string())?;
        let path = entry.path();
        if metadata.is_symlink() {
            return Err("SOURCE_SYMLINK_UNSUPPORTED".into());
        }
        if metadata.is_dir() {
            if entry.file_name() != "__pycache__" {
                collect(root, &path, output, depth + 1)?;
            }
        } else if metadata.is_file()
            && path
                .extension()
                .and_then(|v| v.to_str())
                .is_some_and(|v| ["rs", "py", "pyi", "proto", "json", "toml", "lock"].contains(&v))
        {
            if entry.metadata().map_err(|e| e.to_string())?.len() > 16 * 1024 * 1024 {
                return Err("SOURCE_FILE_TOO_LARGE".into());
            }
            let relative = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            output.insert(relative, file_hash(&path)?);
        }
    }
    Ok(())
}

pub(super) fn file_hash(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 256 * 1024];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(hex(&hasher.finalize()))
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("SOURCE_GIT_QUERY_FAILED".into());
    }
    Ok(output.stdout)
}

pub(super) fn component_identity(component: &Value) -> Result<Value, String> {
    let entry = component["entrypoint"]
        .as_array()
        .ok_or("ENTRYPOINT_MISSING")?;
    let executable = entry
        .first()
        .and_then(Value::as_str)
        .map(Path::new)
        .ok_or("ABSOLUTE_COMPONENT_EXECUTABLE_REQUIRED")?;
    if !executable.is_absolute() || !executable.is_file() {
        return Err("ABSOLUTE_COMPONENT_EXECUTABLE_REQUIRED".into());
    }
    let mut files = BTreeMap::new();
    for argument in entry {
        let text = argument.as_str().ok_or("ENTRYPOINT_INVALID")?;
        let path = PathBuf::from(text);
        if path.is_absolute() && path.is_file() {
            files.insert(text.to_owned(), file_hash(&path)?);
        }
    }
    if files.is_empty() {
        return Err("ABSOLUTE_COMPONENT_EXECUTABLE_REQUIRED".into());
    }
    Ok(json!({"component_id":component["component_id"],"entrypoint_files":files}))
}
