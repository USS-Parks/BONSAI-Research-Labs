//! Durable lineage admission with bounded memory and explicit commit boundaries.
//!
//! Append is provisional until commit succeeds. A reopened store is always a
//! continuation, never evidence of an uninterrupted Track A execution.
mod audit;
mod index;
mod recovery;

use bonsai_bundle::SegmentWriter;
use bonsai_contracts::bonsai::artifact::v1::ArtifactLifecycleEvent;
use bonsai_contracts::lineage::persisted::{
    LineageStateReader, PersistedArtifactState, validate_persisted_transition,
};
use index::{Index, integer, sql};
use prost::Message;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub use recovery::RecoveryReport;
type Result<T> = std::result::Result<T, String>;
static ATTEMPT: AtomicU64 = AtomicU64::new(0);

/// Explicit live-state, frame, database and complete owned-output allowances.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PersistentLimits {
    pub maximum_live_artifacts: u64,
    pub maximum_consumers: usize,
    pub maximum_frame_bytes: u32,
    pub database_bytes: u64,
    pub output_bytes: u64,
}
impl PersistentLimits {
    fn validate(self) -> Result<()> {
        if self.maximum_live_artifacts == 0
            || self.maximum_consumers == 0
            || self.maximum_consumers > 4096
            || self.maximum_frame_bytes == 0
            || self.maximum_frame_bytes > bonsai_bundle::HARD_MAX_FRAME_SIZE
            || self.database_bytes < 65_536
            || self
                .database_bytes
                .checked_mul(5)
                .and_then(|v| v.checked_add(65_536))
                .is_none_or(|v| v >= self.output_bytes)
        {
            return Err("LINEAGE_LIMITS_INVALID".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageCheckpoint {
    pub format: String,
    pub commits: u64,
    pub events: u64,
    pub continuation: bool,
}

struct Pending {
    writer: SegmentWriter,
    directory: String,
    events: u64,
    bytes: u64,
}

/// A single-writer durable store. `SQLite` caches are limited to 2 MiB and mmap is
/// disabled; historical identities, ancestry and traversal queues stay on disk.
pub struct PersistentLineageRegistry {
    root: PathBuf,
    index: Index,
    limits: PersistentLimits,
    pending: Option<Pending>,
    segment_bytes: u64,
    retained_extra_bytes: u64,
    continuation: bool,
    poisoned: bool,
}

impl PersistentLineageRegistry {
    /// Create an exclusively owned, previously absent output directory.
    ///
    /// # Errors
    /// Rejects invalid limits, existing paths and filesystem/database failures.
    pub fn create(root: impl AsRef<Path>, limits: PersistentLimits) -> Result<Self> {
        limits.validate()?;
        fs::create_dir(root.as_ref()).map_err(|e| io(&e))?;
        let root = fs::canonicalize(root.as_ref()).map_err(|e| io(&e))?;
        fs::create_dir(root.join("segments")).map_err(|e| io(&e))?;
        fs::write(
            root.join("limits.json"),
            serde_json::to_vec(&limits).map_err(|e| e.to_string())?,
        )
        .map_err(|e| io(&e))?;
        fs::write(root.join("execution-status.txt"), "INCOMPLETE\n").map_err(|e| io(&e))?;
        let index = Index::open(&root.join("lineage.sqlite3"), limits, true)?;
        Ok(Self {
            root,
            index,
            limits,
            pending: None,
            segment_bytes: 0,
            retained_extra_bytes: 0,
            continuation: false,
            poisoned: false,
        })
    }

    /// Recover the exact SQLite-committed prefix. Unindexed segments are retained
    /// as interrupted evidence and never silently admitted.
    ///
    /// # Errors
    /// Rejects checkpoint versions, corruption, links, quotas and storage failures.
    pub fn open(root: impl AsRef<Path>) -> Result<(Self, RecoveryReport)> {
        recovery::open(root.as_ref())
    }

    /// Validate and provisionally append one event in the current transaction.
    ///
    /// # Errors
    /// Returns unchanged contract codes, declared-limit failures, or storage errors.
    /// A storage failure poisons the session; reopen to recover committed work.
    pub fn append(&mut self, event: &ArtifactLifecycleEvent) -> Result<()> {
        if self.poisoned {
            return Err("LINEAGE_SESSION_POISONED".into());
        }
        if event.encoded_len() > self.limits.maximum_frame_bytes as usize {
            return Err("LINEAGE_FRAME_LIMIT".into());
        }
        if let Err(error) = self.begin() {
            return self.fail(error);
        }
        if let Err(error) = self.index.connection.execute_batch("SAVEPOINT candidate") {
            return self.fail(sql(&error));
        }
        let change = match validate_persisted_transition(&self.index, event) {
            Ok(Ok(change)) => change,
            Ok(Err(error)) => {
                self.index
                    .connection
                    .execute_batch("ROLLBACK TO candidate; RELEASE candidate")
                    .map_err(|e| sql(&e))?;
                return Err(error.to_string());
            }
            Err(error) => return self.fail(error),
        };
        if let Err(error) = self.index.apply(&change) {
            if error.starts_with("LINEAGE_LIVE_") {
                if let Err(rollback) = self
                    .index
                    .connection
                    .execute_batch("ROLLBACK TO candidate; RELEASE candidate")
                {
                    return self.fail(sql(&rollback));
                }
                return Err(error);
            }
            // SQLite may already have rolled back the whole transaction on
            // SQLITE_FULL. Preserve that storage error instead of masking it
            // with a follow-up missing-savepoint error.
            return self.fail(error);
        }
        let encoded = event.encode_to_vec();
        let increment = u64::try_from(encoded.len()).map_err(|e| e.to_string())? + 44;
        let pending = self.pending.as_mut().ok_or("LINEAGE_PENDING_MISSING")?;
        let next = pending
            .bytes
            .checked_add(increment)
            .ok_or("LINEAGE_SIZE_OVERFLOW")?;
        let reserved = self.limits.database_bytes * 5 + 65_536 + self.retained_extra_bytes;
        if self
            .segment_bytes
            .checked_add(next)
            .and_then(|v| v.checked_add(reserved))
            .is_none_or(|v| v > self.limits.output_bytes)
        {
            return self.fail("LINEAGE_OUTPUT_QUOTA".into());
        }
        if let Err(error) = pending.writer.append(&encoded) {
            return self.fail(error.to_string());
        }
        pending.bytes = next;
        pending.events += 1;
        if let Err(error) = self.index.connection.execute_batch("RELEASE candidate") {
            return self.fail(sql(&error));
        }
        Ok(())
    }

    /// Publish the segment before committing its index and validated state.
    ///
    /// # Errors
    /// Returns storage/quota failures. Earlier committed batches remain recoverable.
    pub fn commit(&mut self) -> Result<LineageCheckpoint> {
        self.commit_with(|_| Ok(()))
    }

    /// Commit with observable durability boundaries for fault-injection runners.
    /// Returning an error aborts this session; the hook cannot change evidence.
    ///
    /// # Errors
    /// Returns hook or storage errors. The index commit is the acceptance boundary.
    pub fn commit_with(
        &mut self,
        mut boundary: impl FnMut(&str) -> Result<()>,
    ) -> Result<LineageCheckpoint> {
        if self.poisoned {
            return Err("LINEAGE_SESSION_POISONED".into());
        }
        let Some(mut pending) = self.pending.take() else {
            return self.checkpoint();
        };
        let outcome = (|| {
            pending.writer.sync_pending().map_err(|e| e.to_string())?;
            boundary("before_segment_commit")?;
            let summary = pending.writer.finalize().map_err(|e| e.to_string())?;
            boundary("after_segment_commit")?;
            let (commits, events) = self.index.checkpoint()?;
            let next_events = events
                .checked_add(pending.events)
                .ok_or("LINEAGE_SIZE_OVERFLOW")?;
            self.index
                .connection
                .execute(
                    "INSERT INTO commits(sequence,directory,events,checksum) VALUES(?1,?2,?3,?4)",
                    params![
                        integer(commits)?,
                        pending.directory,
                        integer(pending.events)?,
                        summary.checksum.to_vec()
                    ],
                )
                .map_err(|e| sql(&e))?;
            self.index
                .connection
                .execute(
                    "UPDATE metadata SET commits=?1,events=?2 WHERE singleton=1",
                    params![integer(commits + 1)?, integer(next_events)?],
                )
                .map_err(|e| sql(&e))?;
            boundary("before_index_commit")?;
            self.index
                .connection
                .execute_batch("COMMIT")
                .map_err(|e| sql(&e))?;
            boundary("after_index_commit")?;
            self.segment_bytes += pending.bytes;
            self.checkpoint()
        })();
        match outcome {
            Ok(value) => Ok(value),
            Err(error) => self.fail(error),
        }
    }

    /// Read one bounded current artifact; historical event payloads remain on disk.
    ///
    /// # Errors
    /// Returns state corruption or indexed read errors.
    pub fn artifact(&self, id: &[u8]) -> Result<Option<PersistedArtifactState>> {
        self.index.artifact(id)
    }

    /// Query historical revision ownership without loading completed history.
    ///
    /// # Errors
    /// Returns indexed read errors.
    pub fn revision_owner(&self, id: &[u8]) -> Result<Option<Vec<u8>>> {
        self.index.revision_owner(id)
    }

    /// Query historical ancestry using a disk-backed traversal queue.
    ///
    /// # Errors
    /// Returns database/quota errors; scratch changes are rolled back.
    pub fn is_ancestor(&mut self, ancestor: &[u8], target: &[u8]) -> Result<bool> {
        if self.poisoned {
            return Err("LINEAGE_SESSION_POISONED".into());
        }
        if let Err(error) = self
            .index
            .connection
            .execute_batch("SAVEPOINT ancestry_read")
        {
            return self.fail(sql(&error));
        }
        let result = match self.index.reaches(target, ancestor) {
            Ok(value) => value,
            Err(error) => return self.fail(error),
        };
        if let Err(error) = self
            .index
            .connection
            .execute_batch("ROLLBACK TO ancestry_read; RELEASE ancestry_read")
        {
            return self.fail(sql(&error));
        }
        Ok(result)
    }

    /// Return only the last accepted batch boundary, excluding pending events.
    ///
    /// # Errors
    /// Returns database errors.
    pub fn checkpoint(&self) -> Result<LineageCheckpoint> {
        let (commits, events) = self.index.checkpoint()?;
        Ok(LineageCheckpoint {
            format: "bonsai.lineage-checkpoint/v1".into(),
            commits,
            events,
            continuation: self.continuation,
        })
    }

    fn begin(&mut self) -> Result<()> {
        if self.pending.is_some() {
            return Ok(());
        }
        self.index
            .connection
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| sql(&e))?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let directory = format!(
            "batch-{nonce}-{}-{}",
            std::process::id(),
            ATTEMPT.fetch_add(1, Ordering::Relaxed)
        );
        let path = self.root.join("segments").join(&directory);
        fs::create_dir(&path).map_err(|e| io(&e))?;
        let writer = SegmentWriter::create(&path, 0, self.limits.maximum_frame_bytes)
            .map_err(|e| e.to_string())?;
        self.pending = Some(Pending {
            writer,
            directory,
            events: 0,
            bytes: 148,
        });
        Ok(())
    }

    fn fail<T>(&mut self, error: String) -> Result<T> {
        self.poisoned = true;
        let _ = self.index.connection.execute_batch("ROLLBACK");
        self.pending = None;
        Err(error)
    }
}

impl Drop for PersistentLineageRegistry {
    fn drop(&mut self) {
        let _ = self.index.connection.execute_batch("ROLLBACK");
    }
}

fn io(error: &std::io::Error) -> String {
    format!("LINEAGE_STORAGE_IO: {error}")
}
