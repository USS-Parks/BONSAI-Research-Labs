use std::fs;
use std::path::Path;

pub(super) const METADATA_RESERVE: u64 = 128 * 1024;
const CONTROL_RESERVE: u64 = 8 * 1024;

// Events have a separate byte-exact allowance. Reserve fixed space for the
// bounded lifecycle journal and durable status, so exhaustion cannot erase them.
pub(super) fn write(root: &Path, path: &Path, bytes: &[u8]) -> Result<(), String> {
    if !path.starts_with(root) {
        return Err("OBSERVER_PATH_INVALID".into());
    }
    let used = size(root, root, true, 0)?;
    let previous = match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_file() && !meta.file_type().is_symlink() => meta.len(),
        Ok(_) => return Err("OBSERVER_OUTPUT_NOT_REGULAR".into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
        Err(error) => return Err(error.to_string()),
    };
    let next = used
        .checked_sub(previous)
        .and_then(|n| n.checked_add(u64::try_from(bytes.len()).ok()?))
        .ok_or("OBSERVER_SIZE_OVERFLOW")?;
    if next > METADATA_RESERVE - CONTROL_RESERVE {
        return Err("OBSERVER_STORAGE_EXHAUSTED".into());
    }
    fs::write(path, bytes).map_err(|e| e.to_string())
}

pub(super) fn final_check(root: &Path, maximum: u64) -> Result<u64, String> {
    let used = size(root, root, false, 0)?;
    if used.checked_add(1024).is_none_or(|next| next > maximum) {
        return Err("OBSERVER_STORAGE_EXHAUSTED".into());
    }
    Ok(used)
}

fn size(root: &Path, directory: &Path, metadata_only: bool, depth: u8) -> Result<u64, String> {
    if depth > 16 {
        return Err("OBSERVER_DEPTH_EXCEEDED".into());
    }
    let mut total = 0_u64;
    for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if metadata_only
            && directory == root
            && [
                "telemetry",
                "lifecycle",
                "run-status.json",
                "run-status.pending",
            ]
            .iter()
            .any(|name| entry.file_name() == *name)
        {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        let bytes = if metadata.file_type().is_symlink() {
            return Err("OBSERVER_SYMLINK_REJECTED".into());
        } else if metadata.is_dir() {
            size(root, &path, metadata_only, depth + 1)?
        } else if metadata.is_file() {
            metadata.len()
        } else {
            return Err("OBSERVER_OUTPUT_NOT_REGULAR".into());
        };
        total = total.checked_add(bytes).ok_or("OBSERVER_SIZE_OVERFLOW")?;
    }
    Ok(total)
}
