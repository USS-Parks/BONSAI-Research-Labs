use super::{PersistentLimits, Result};
use bonsai_contracts::lineage::persisted::{
    LineageStateReader, PersistedArtifactState, ValidatedLineageChange,
};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::path::Path;

pub(super) struct Index {
    pub(super) connection: Connection,
    limits: PersistentLimits,
}

impl Index {
    pub(super) fn open(path: &Path, limits: PersistentLimits, create: bool) -> Result<Self> {
        let flags = rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
            | if create {
                rusqlite::OpenFlags::SQLITE_OPEN_CREATE
            } else {
                rusqlite::OpenFlags::empty()
            };
        let connection = Connection::open_with_flags(path, flags).map_err(|e| sql(&e))?;
        connection
            .execute_batch(
                "PRAGMA locking_mode=EXCLUSIVE; PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL;
             PRAGMA cache_size=-2048; PRAGMA mmap_size=0; PRAGMA temp_store=FILE;
             PRAGMA foreign_keys=ON;",
            )
            .map_err(|e| sql(&e))?;
        let page_size: u64 = connection
            .pragma_query_value(None, "page_size", |r| unsigned(r, 0))
            .map_err(|e| sql(&e))?;
        connection
            .pragma_update(
                None,
                "max_page_count",
                integer(limits.database_bytes / page_size)?,
            )
            .map_err(|e| sql(&e))?;
        if create {
            connection
                .execute_batch(include_str!("schema.sql"))
                .map_err(|e| sql(&e))?;
        }
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(|e| sql(&e))?;
        let app: i64 = connection
            .pragma_query_value(None, "application_id", |r| r.get(0))
            .map_err(|e| sql(&e))?;
        if version != 1 || app != 1_112_429_388 {
            return Err("LINEAGE_CHECKPOINT_UNSUPPORTED".into());
        }
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .map_err(|e| sql(&e))?;
        if integrity != "ok" {
            return Err("LINEAGE_INDEX_CORRUPT".into());
        }
        connection
            .execute_batch("BEGIN IMMEDIATE; COMMIT")
            .map_err(|e| sql(&e))?;
        Ok(Self { connection, limits })
    }

    pub(super) fn apply(&self, change: &ValidatedLineageChange) -> Result<()> {
        if change.state.consumers.len() > self.limits.maximum_consumers {
            return Err("LINEAGE_LIVE_CONSUMER_LIMIT".into());
        }
        let old_terminal: Option<bool> = self
            .connection
            .query_row(
                "SELECT terminal FROM artifacts WHERE id=?1",
                [&change.artifact_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| sql(&e))?;
        let live: u64 = self
            .connection
            .query_row(
                "SELECT live_artifacts FROM metadata WHERE singleton=1",
                [],
                |r| unsigned(r, 0),
            )
            .map_err(|e| sql(&e))?;
        let old_live = u64::from(old_terminal == Some(false));
        let new_live = u64::from(!change.state.terminal);
        let next_live = live
            .checked_sub(old_live)
            .and_then(|v| v.checked_add(new_live))
            .ok_or("LINEAGE_INDEX_CORRUPT")?;
        if next_live > self.limits.maximum_live_artifacts {
            return Err("LINEAGE_LIVE_ARTIFACT_LIMIT".into());
        }
        let encoded = serde_json::to_vec(&change.state).map_err(|e| e.to_string())?;
        let digest = Sha256::digest(&encoded).to_vec();
        self.connection.execute(
            "INSERT INTO artifacts(id,state,sha256,terminal) VALUES(?1,?2,?3,?4)
             ON CONFLICT(id) DO UPDATE SET state=excluded.state,sha256=excluded.sha256,terminal=excluded.terminal",
            params![change.artifact_id, encoded, digest, change.state.terminal],
        ).map_err(|e| sql(&e))?;
        self.connection
            .execute(
                "UPDATE metadata SET live_artifacts=?1 WHERE singleton=1",
                [integer(next_live)?],
            )
            .map_err(|e| sql(&e))?;
        if let Some(revision) = &change.new_revision {
            self.connection
                .execute(
                    "INSERT INTO revisions(id,owner) VALUES(?1,?2)",
                    params![revision, change.artifact_id],
                )
                .map_err(|e| sql(&e))?;
        }
        for parent in &change.added_parents {
            self.connection
                .execute(
                    "INSERT OR IGNORE INTO parents(child,parent) VALUES(?1,?2)",
                    params![change.artifact_id, parent],
                )
                .map_err(|e| sql(&e))?;
        }
        if let Some(id) = &change.history_id {
            self.connection
                .execute("INSERT INTO history(id) VALUES(?1)", [id])
                .map_err(|e| sql(&e))?;
        }
        Ok(())
    }

    pub(super) fn checkpoint(&self) -> Result<(u64, u64)> {
        self.connection
            .query_row(
                "SELECT commits,events FROM metadata WHERE singleton=1",
                [],
                |r| Ok((unsigned(r, 0)?, unsigned(r, 1)?)),
            )
            .map_err(|e| sql(&e))
    }
}

impl LineageStateReader for Index {
    type Error = String;
    fn artifact(&self, id: &[u8]) -> Result<Option<PersistedArtifactState>> {
        let maximum = self
            .limits
            .maximum_consumers
            .saturating_mul(200)
            .saturating_add(512);
        let size: Option<i64> = self
            .connection
            .query_row(
                "SELECT length(state) FROM artifacts WHERE id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| sql(&e))?;
        if size.is_some_and(|v| usize::try_from(v).map_or(true, |v| v > maximum)) {
            return Err("LINEAGE_STATE_LIMIT".into());
        }
        let row: Option<(Vec<u8>, Vec<u8>)> = self
            .connection
            .query_row(
                "SELECT state,sha256 FROM artifacts WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|e| sql(&e))?;
        row.map(|(bytes, digest)| {
            if Sha256::digest(&bytes).as_slice() != digest {
                return Err("LINEAGE_STATE_CORRUPT".into());
            }
            let state: PersistedArtifactState =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            if state.consumers.len() > self.limits.maximum_consumers
                || state.current_revision_id.len() != 16
                || state.next_sequence < 2
            {
                return Err("LINEAGE_STATE_CORRUPT".into());
            }
            Ok(state)
        })
        .transpose()
    }
    fn revision_owner(&self, id: &[u8]) -> Result<Option<Vec<u8>>> {
        self.connection
            .query_row("SELECT owner FROM revisions WHERE id=?1", [id], |r| {
                r.get(0)
            })
            .optional()
            .map_err(|e| sql(&e))
    }
    fn history_contains(&self, id: &[u8]) -> Result<bool> {
        self.connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM history WHERE id=?1)",
                [id],
                |r| r.get(0),
            )
            .map_err(|e| sql(&e))
    }
    fn reaches(&self, ancestor: &[u8], target: &[u8]) -> Result<bool> {
        self.connection
            .execute("DELETE FROM traversal", [])
            .map_err(|e| sql(&e))?;
        self.connection
            .execute("INSERT INTO traversal(id,done) VALUES(?1,0)", [ancestor])
            .map_err(|e| sql(&e))?;
        let found = loop {
            let next: Option<Vec<u8>> = self
                .connection
                .query_row(
                    "SELECT id FROM traversal WHERE done=0 ORDER BY id LIMIT 1",
                    [],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| sql(&e))?;
            let Some(id) = next else {
                break false;
            };
            if id == target {
                break true;
            }
            self.connection
                .execute("UPDATE traversal SET done=1 WHERE id=?1", [&id])
                .map_err(|e| sql(&e))?;
            self.connection.execute(
                "INSERT OR IGNORE INTO traversal(id,done) SELECT parent,0 FROM parents WHERE child=?1",[&id],
            ).map_err(|e| sql(&e))?;
        };
        self.connection
            .execute("DELETE FROM traversal", [])
            .map_err(|e| sql(&e))?;
        Ok(found)
    }
}

pub(super) fn sql(error: &rusqlite::Error) -> String {
    format!("LINEAGE_INDEX_IO: {error}")
}

pub(super) fn integer(value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|_| "LINEAGE_INTEGER_LIMIT".into())
}
pub(super) fn unsigned(row: &rusqlite::Row<'_>, column: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(column)?;
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(column, value))
}
