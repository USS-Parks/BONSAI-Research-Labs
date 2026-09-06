use super::index::sql;
use super::{Index, PersistentLimits, Result};
use bonsai_bundle::visit_segment_file;
use bonsai_contracts::bonsai::artifact::v1::ArtifactLifecycleEvent;
use bonsai_contracts::lineage::persisted::{LineageStateReader, validate_persisted_transition};
use prost::Message;
use std::path::Path;

/// Rebuild only a disposable bounded-cache index, never a full in-memory trace.
pub(super) fn verify(root: &Path, original: &Index, limits: PersistentLimits) -> Result<()> {
    let temporary = tempfile::NamedTempFile::new_in(root).map_err(|e| super::io(&e))?;
    let audit = Index::open(temporary.path(), limits, true)?;
    audit
        .connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| sql(&e))?;
    let mut statement = original
        .connection
        .prepare("SELECT directory FROM commits ORDER BY sequence")
        .map_err(|e| sql(&e))?;
    let mut rows = statement.query([]).map_err(|e| sql(&e))?;
    while let Some(row) = rows.next().map_err(|e| sql(&e))? {
        let directory: String = row.get(0).map_err(|e| sql(&e))?;
        let path = root
            .join("segments")
            .join(directory)
            .join("segment-00000000000000000000.bseg");
        let mut failure = None;
        visit_segment_file(path, |bytes| {
            if failure.is_some() {
                return;
            }
            let result = (|| {
                let event = ArtifactLifecycleEvent::decode(bytes).map_err(|e| e.to_string())?;
                let change = validate_persisted_transition(&audit, &event)?
                    .map_err(|e| format!("LINEAGE_RECOVERY_CONTRACT: {e}"))?;
                audit.apply(&change)
            })();
            if let Err(error) = result {
                failure = Some(error);
            }
        })
        .map_err(|e| e.to_string())?;
        if let Some(error) = failure {
            return Err(error);
        }
    }
    compare(&audit, original)?;
    audit
        .connection
        .execute_batch("ROLLBACK")
        .map_err(|e| sql(&e))?;
    drop(audit);
    temporary.close().map_err(|e| super::io(&e))?;
    Ok(())
}

fn compare(audit: &Index, original: &Index) -> Result<()> {
    for table in ["artifacts", "revisions", "history", "parents"] {
        let query = format!("SELECT count(*) FROM {table}");
        let expected: i64 = audit
            .connection
            .query_row(&query, [], |r| r.get(0))
            .map_err(|e| sql(&e))?;
        let actual: i64 = original
            .connection
            .query_row(&query, [], |r| r.get(0))
            .map_err(|e| sql(&e))?;
        if expected != actual {
            return Err("LINEAGE_CHECKPOINT_STATE_MISMATCH".into());
        }
    }
    let mut statement = audit
        .connection
        .prepare("SELECT id FROM artifacts ORDER BY id")
        .map_err(|e| sql(&e))?;
    let mut rows = statement.query([]).map_err(|e| sql(&e))?;
    while let Some(row) = rows.next().map_err(|e| sql(&e))? {
        let id: Vec<u8> = row.get(0).map_err(|e| sql(&e))?;
        if audit.artifact(&id)? != original.artifact(&id)? {
            return Err("LINEAGE_CHECKPOINT_STATE_MISMATCH".into());
        }
    }
    for query in [
        "SELECT id,owner FROM revisions ORDER BY id",
        "SELECT id,id FROM history ORDER BY id",
        "SELECT child,parent FROM parents ORDER BY child,parent",
    ] {
        let mut left = audit.connection.prepare(query).map_err(|e| sql(&e))?;
        let mut right = original.connection.prepare(query).map_err(|e| sql(&e))?;
        let mut a = left.query([]).map_err(|e| sql(&e))?;
        let mut b = right.query([]).map_err(|e| sql(&e))?;
        while let Some(row) = a.next().map_err(|e| sql(&e))? {
            let Some(other) = b.next().map_err(|e| sql(&e))? else {
                return Err("LINEAGE_CHECKPOINT_STATE_MISMATCH".into());
            };
            for column in 0..2 {
                let expected: Vec<u8> = row.get(column).map_err(|e| sql(&e))?;
                let actual: Vec<u8> = other.get(column).map_err(|e| sql(&e))?;
                if expected != actual {
                    return Err("LINEAGE_CHECKPOINT_STATE_MISMATCH".into());
                }
            }
        }
    }
    let query = "SELECT live_artifacts FROM metadata WHERE singleton=1";
    let expected: i64 = audit
        .connection
        .query_row(query, [], |r| r.get(0))
        .map_err(|e| sql(&e))?;
    let actual: i64 = original
        .connection
        .query_row(query, [], |r| r.get(0))
        .map_err(|e| sql(&e))?;
    if expected != actual {
        return Err("LINEAGE_CHECKPOINT_STATE_MISMATCH".into());
    }
    Ok(())
}
