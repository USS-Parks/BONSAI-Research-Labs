use super::index::{sql, unsigned};
use super::{Index, PersistentLimits, PersistentLineageRegistry, Result, io};
use bonsai_bundle::{SegmentError, validate_segment, visit_segment_file};
use bonsai_contracts::bonsai::artifact::v1::ArtifactLifecycleEvent;
use prost::Message;
use rusqlite::OptionalExtension;
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::Path;

/// Recovery describes only the committed prefix; orphan attempts stay on disk.
#[derive(Debug, Clone, Serialize)]
pub struct RecoveryReport {
    pub committed_batches: u64,
    pub committed_events: u64,
    pub orphan_batches: u64,
    pub truncated_orphans: u64,
    pub continuation: bool,
}

pub(super) fn open(path: &Path) -> Result<(PersistentLineageRegistry, RecoveryReport)> {
    reject_link(path)?;
    let root = fs::canonicalize(path).map_err(|e| io(&e))?;
    for entry in fs::read_dir(&root).map_err(|e| io(&e))? {
        reject_link(&entry.map_err(|e| io(&e))?.path())?;
    }
    let limits_path = root.join("limits.json");
    if fs::metadata(&limits_path).map_err(|e| io(&e))?.len() > 4096 {
        return Err("LINEAGE_LIMITS_INVALID".into());
    }
    let limits: PersistentLimits =
        serde_json::from_slice(&fs::read(limits_path).map_err(|e| io(&e))?)
            .map_err(|e| e.to_string())?;
    limits.validate()?;
    let segment_bytes = owned_size(&root.join("segments"))?;
    let total = owned_size(&root)?;
    if total > limits.output_bytes {
        return Err("LINEAGE_OUTPUT_QUOTA".into());
    }
    let mut retained_extra_bytes = 0_u64;
    for entry in fs::read_dir(&root).map_err(|e| io(&e))? {
        let entry = entry.map_err(|e| io(&e))?;
        let name = entry.file_name();
        if ![
            "segments",
            "limits.json",
            "execution-status.txt",
            "lineage.sqlite3",
            "lineage.sqlite3-journal",
        ]
        .iter()
        .any(|known| name == *known)
        {
            retained_extra_bytes = retained_extra_bytes
                .checked_add(owned_size(&entry.path())?)
                .ok_or("LINEAGE_SIZE_OVERFLOW")?;
        }
    }
    if segment_bytes
        .checked_add(retained_extra_bytes)
        .and_then(|v| v.checked_add(limits.database_bytes * 5 + 65_536))
        .is_none_or(|v| v > limits.output_bytes)
    {
        return Err("LINEAGE_OUTPUT_QUOTA".into());
    }
    let index = Index::open(&root.join("lineage.sqlite3"), limits, false)?;
    let (commits, events) = index.checkpoint()?;
    let mut report = RecoveryReport {
        committed_batches: commits,
        committed_events: events,
        orphan_batches: 0,
        truncated_orphans: 0,
        continuation: true,
    };
    verify_commits(&root, &index, limits, &report)?;
    verify_orphans(&root, &index, limits, &mut report)?;
    super::audit::verify(&root, &index, limits)?;
    index
        .connection
        .execute(
            "UPDATE metadata SET sessions=sessions+1 WHERE singleton=1",
            [],
        )
        .map_err(|e| sql(&e))?;
    fs::write(
        root.join("execution-status.txt"),
        "INCOMPLETE CONTINUATION\n",
    )
    .map_err(|e| io(&e))?;
    Ok((
        PersistentLineageRegistry {
            root,
            index,
            limits,
            pending: None,
            segment_bytes,
            retained_extra_bytes,
            continuation: true,
            poisoned: false,
        },
        report,
    ))
}

fn verify_commits(
    root: &Path,
    index: &Index,
    limits: PersistentLimits,
    report: &RecoveryReport,
) -> Result<()> {
    let mut statement = index
        .connection
        .prepare("SELECT sequence,directory,events,checksum FROM commits ORDER BY sequence")
        .map_err(|e| sql(&e))?;
    let mut rows = statement.query([]).map_err(|e| sql(&e))?;
    let mut count = 0_u64;
    let mut events = 0_u64;
    while let Some(row) = rows.next().map_err(|e| sql(&e))? {
        let sequence = unsigned(row, 0).map_err(|e| sql(&e))?;
        let directory: String = row.get(1).map_err(|e| sql(&e))?;
        let frames = unsigned(row, 2).map_err(|e| sql(&e))?;
        let checksum: Vec<u8> = row.get(3).map_err(|e| sql(&e))?;
        if sequence != count {
            return Err("LINEAGE_CHECKPOINT_SEQUENCE".into());
        }
        let directory = batch_path(root, &directory)?;
        let path = directory.join("segment-00000000000000000000.bseg");
        check_frame_bound(&path, limits.maximum_frame_bytes)?;
        let mut invalid = false;
        let summary = visit_segment_file(&path, |bytes| {
            if ArtifactLifecycleEvent::decode(bytes).is_err() {
                invalid = true;
            }
        })
        .map_err(|e| e.to_string())?;
        if invalid
            || summary.sequence != 0
            || summary.frame_count != frames
            || summary.checksum.as_slice() != checksum
        {
            return Err("LINEAGE_CHECKPOINT_SEGMENT_MISMATCH".into());
        }
        count += 1;
        events = events.checked_add(frames).ok_or("LINEAGE_SIZE_OVERFLOW")?;
    }
    if count != report.committed_batches || events != report.committed_events {
        return Err("LINEAGE_CHECKPOINT_PREFIX_MISMATCH".into());
    }
    // Row payload hashes detect damage even when SQLite's structural check passes.
    let mut statement = index
        .connection
        .prepare("SELECT id FROM artifacts ORDER BY id")
        .map_err(|e| sql(&e))?;
    let mut rows = statement.query([]).map_err(|e| sql(&e))?;
    while let Some(row) = rows.next().map_err(|e| sql(&e))? {
        let id: Vec<u8> = row.get(0).map_err(|e| sql(&e))?;
        bonsai_contracts::lineage::persisted::LineageStateReader::artifact(index, &id)?;
    }
    Ok(())
}

fn verify_orphans(
    root: &Path,
    index: &Index,
    limits: PersistentLimits,
    report: &mut RecoveryReport,
) -> Result<()> {
    for entry in fs::read_dir(root.join("segments")).map_err(|e| io(&e))? {
        let entry = entry.map_err(|e| io(&e))?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "LINEAGE_PATH_INVALID")?;
        let directory = batch_path(root, &name)?;
        let committed: Option<i64> = index
            .connection
            .query_row(
                "SELECT sequence FROM commits WHERE directory=?1",
                [&name],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| sql(&e))?;
        if committed.is_some() {
            continue;
        }
        report.orphan_batches += 1;
        for file in fs::read_dir(directory).map_err(|e| io(&e))? {
            let file = file.map_err(|e| io(&e))?;
            reject_link(&file.path())?;
            let name = file.file_name();
            let name = name.to_str().ok_or("LINEAGE_PATH_INVALID")?;
            if name != "segment-00000000000000000000.bseg"
                && name != "segment-00000000000000000000.open"
            {
                return Err("LINEAGE_PATH_INVALID".into());
            }
            if fs::metadata(file.path()).map_err(|e| io(&e))?.len() >= 24 {
                check_frame_bound(&file.path(), limits.maximum_frame_bytes)?;
            }
            match validate_segment(file.path()) {
                Ok(_) => {}
                Err(
                    SegmentError::HeaderTruncated
                    | SegmentError::FrameTruncated { .. }
                    | SegmentError::FooterTruncated,
                ) if name == "segment-00000000000000000000.open" => report.truncated_orphans += 1,
                Err(error) => return Err(error.to_string()),
            }
        }
    }
    Ok(())
}

fn check_frame_bound(path: &Path, maximum: u32) -> Result<()> {
    reject_link(path)?;
    let mut header = [0_u8; 24];
    fs::File::open(path)
        .map_err(|e| io(&e))?
        .read_exact(&mut header)
        .map_err(|e| io(&e))?;
    let declared = u32::from_le_bytes(
        header[20..24]
            .try_into()
            .map_err(|_| "LINEAGE_HEADER_INVALID")?,
    );
    if declared > maximum {
        return Err("LINEAGE_FRAME_LIMIT".into());
    }
    Ok(())
}

fn batch_path(root: &Path, name: &str) -> Result<std::path::PathBuf> {
    if !name.starts_with("batch-")
        || !name
            .bytes()
            .all(|b| b.is_ascii_digit() || b == b'-' || b"batch".contains(&b))
    {
        return Err("LINEAGE_PATH_INVALID".into());
    }
    let path = root.join("segments").join(name);
    reject_link(&path)?;
    if !path.is_dir() {
        return Err("LINEAGE_PATH_INVALID".into());
    }
    Ok(path)
}

fn reject_link(path: &Path) -> Result<()> {
    if fs::symlink_metadata(path)
        .map_err(|e| io(&e))?
        .file_type()
        .is_symlink()
    {
        return Err("LINEAGE_SYMLINK_REJECTED".into());
    }
    Ok(())
}

fn owned_size(path: &Path) -> Result<u64> {
    reject_link(path)?;
    let metadata = fs::metadata(path).map_err(|e| io(&e))?;
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    let mut size = 0_u64;
    for entry in fs::read_dir(path).map_err(|e| io(&e))? {
        size = size
            .checked_add(owned_size(&entry.map_err(|e| io(&e))?.path())?)
            .ok_or("LINEAGE_SIZE_OVERFLOW")?;
    }
    Ok(size)
}
