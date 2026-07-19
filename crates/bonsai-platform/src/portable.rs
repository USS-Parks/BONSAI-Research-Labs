//! Portable process-tree, storage, and operation accounting.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;
use sysinfo::{Pid, System};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessTreeSnapshot {
    pub root_process_id: u32,
    pub observed_process_ids: Vec<u32>,
    pub process_count: u64,
    pub accumulated_cpu_time_ns: u64,
    pub resident_memory_bytes: u64,
    pub virtual_memory_bytes: u64,
    pub total_io_read_bytes: u64,
    pub total_io_written_bytes: u64,
    pub memory_semantics: String,
    pub io_semantics: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StorageSnapshot {
    pub root_class: StorageRootClass,
    pub file_count: u64,
    pub directory_count: u64,
    pub bytes: u64,
    pub skipped_symlinks: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageRootClass {
    AgentOwned,
    ObserverOwned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageLimits {
    pub maximum_entries: u64,
    pub maximum_depth: usize,
}

impl Default for StorageLimits {
    fn default() -> Self {
        Self {
            maximum_entries: 100_000,
            maximum_depth: 64,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    EnvironmentStep,
    Update,
    Touch,
    WorkItem,
    ModelCall,
    PlanningBackup,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct OperationSnapshot(pub BTreeMap<OperationKind, u64>);

#[derive(Clone, Debug, Default)]
pub struct OperationLedger {
    counters: BTreeMap<OperationKind, u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortableAccountingError {
    UnsupportedSystem,
    ProcessMissing,
    Arithmetic,
    StorageLimit,
    StorageIo,
    InvalidPath,
}

impl fmt::Display for PortableAccountingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedSystem => "PORTABLE_ACCOUNTING_SYSTEM_UNSUPPORTED",
            Self::ProcessMissing => "PORTABLE_ACCOUNTING_PROCESS_MISSING",
            Self::Arithmetic => "PORTABLE_ACCOUNTING_ARITHMETIC_FAILED",
            Self::StorageLimit => "PORTABLE_ACCOUNTING_STORAGE_LIMIT_EXCEEDED",
            Self::StorageIo => "PORTABLE_ACCOUNTING_STORAGE_IO_FAILED",
            Self::InvalidPath => "PORTABLE_ACCOUNTING_PATH_INVALID",
        })
    }
}

impl Error for PortableAccountingError {}

impl OperationLedger {
    /// Charge an exact number of externally visible operations.
    ///
    /// # Errors
    ///
    /// Fails closed rather than saturating when the counter would overflow.
    pub fn charge(
        &mut self,
        kind: OperationKind,
        amount: u64,
    ) -> Result<(), PortableAccountingError> {
        if amount == 0 {
            return Ok(());
        }
        let current = self.counters.entry(kind).or_default();
        *current = current
            .checked_add(amount)
            .ok_or(PortableAccountingError::Arithmetic)?;
        Ok(())
    }

    #[must_use]
    pub fn snapshot(&self) -> OperationSnapshot {
        OperationSnapshot(self.counters.clone())
    }
}

/// Collect one live process-tree snapshot using public cross-platform process APIs.
///
/// # Errors
///
/// Returns a stable error when the platform is unsupported, the root process
/// is absent from the refreshed snapshot, or aggregation overflows.
pub fn collect_process_tree(
    root_process_id: u32,
) -> Result<ProcessTreeSnapshot, PortableAccountingError> {
    if !sysinfo::IS_SUPPORTED_SYSTEM {
        return Err(PortableAccountingError::UnsupportedSystem);
    }
    let system = System::new_all();
    let root = Pid::from_u32(root_process_id);
    if system.process(root).is_none() {
        return Err(PortableAccountingError::ProcessMissing);
    }
    let mut included = BTreeSet::from([root]);
    loop {
        let before = included.len();
        for (pid, process) in system.processes() {
            if process
                .parent()
                .is_some_and(|parent| included.contains(&parent))
            {
                included.insert(*pid);
            }
        }
        if included.len() == before {
            break;
        }
    }

    let mut cpu_ms = 0_u64;
    let mut resident = 0_u64;
    let mut virtual_memory = 0_u64;
    let mut io_read = 0_u64;
    let mut io_written = 0_u64;
    for pid in &included {
        let process = system
            .process(*pid)
            .ok_or(PortableAccountingError::ProcessMissing)?;
        let io = process.disk_usage();
        cpu_ms = checked_add(cpu_ms, process.accumulated_cpu_time())?;
        resident = checked_add(resident, process.memory())?;
        virtual_memory = checked_add(virtual_memory, process.virtual_memory())?;
        io_read = checked_add(io_read, io.total_read_bytes)?;
        io_written = checked_add(io_written, io.total_written_bytes)?;
    }
    let accumulated_cpu_time_ns = cpu_ms
        .checked_mul(1_000_000)
        .ok_or(PortableAccountingError::Arithmetic)?;
    Ok(ProcessTreeSnapshot {
        root_process_id,
        observed_process_ids: included.iter().map(|pid| pid.as_u32()).collect(),
        process_count: u64::try_from(included.len())
            .map_err(|_| PortableAccountingError::Arithmetic)?,
        accumulated_cpu_time_ns,
        resident_memory_bytes: resident,
        virtual_memory_bytes: virtual_memory,
        total_io_read_bytes: io_read,
        total_io_written_bytes: io_written,
        memory_semantics: "rss_bytes_and_platform_qualified_virtual_bytes".to_owned(),
        io_semantics: if cfg!(windows) {
            "windows_all_process_io_bytes"
        } else {
            "process_disk_io_bytes_subject_to_cache"
        }
        .to_owned(),
    })
}

/// Measure one owned directory tree without following symbolic links.
///
/// # Errors
///
/// Returns a stable error for a non-directory root, I/O failure, arithmetic
/// overflow, or configured entry/depth bound.
pub fn collect_storage(
    root: &Path,
    root_class: StorageRootClass,
    limits: StorageLimits,
) -> Result<StorageSnapshot, PortableAccountingError> {
    if limits.maximum_entries == 0 || limits.maximum_depth == 0 || !root.is_dir() {
        return Err(PortableAccountingError::InvalidPath);
    }
    let mut snapshot = StorageSnapshot {
        root_class,
        file_count: 0,
        directory_count: 1,
        bytes: 0,
        skipped_symlinks: 0,
    };
    let mut pending = vec![(root.to_path_buf(), 0_usize)];
    let mut entries = 0_u64;
    while let Some((directory, depth)) = pending.pop() {
        if depth >= limits.maximum_depth {
            return Err(PortableAccountingError::StorageLimit);
        }
        let read_dir = fs::read_dir(directory).map_err(|_| PortableAccountingError::StorageIo)?;
        for entry in read_dir {
            let entry = entry.map_err(|_| PortableAccountingError::StorageIo)?;
            entries = checked_add(entries, 1)?;
            if entries > limits.maximum_entries {
                return Err(PortableAccountingError::StorageLimit);
            }
            let file_type = entry
                .file_type()
                .map_err(|_| PortableAccountingError::StorageIo)?;
            if file_type.is_symlink() {
                snapshot.skipped_symlinks = checked_add(snapshot.skipped_symlinks, 1)?;
            } else if file_type.is_dir() {
                snapshot.directory_count = checked_add(snapshot.directory_count, 1)?;
                pending.push((entry.path(), depth + 1));
            } else if file_type.is_file() {
                snapshot.file_count = checked_add(snapshot.file_count, 1)?;
                let bytes = entry
                    .metadata()
                    .map_err(|_| PortableAccountingError::StorageIo)?
                    .len();
                snapshot.bytes = checked_add(snapshot.bytes, bytes)?;
            }
        }
    }
    Ok(snapshot)
}

fn checked_add(left: u64, right: u64) -> Result<u64, PortableAccountingError> {
    left.checked_add(right)
        .ok_or(PortableAccountingError::Arithmetic)
}

#[cfg(test)]
mod tests {
    use super::{
        OperationKind, OperationLedger, PortableAccountingError, StorageLimits, StorageRootClass,
        collect_storage,
    };
    use std::fs;

    #[test]
    fn operation_ledger_is_exact_and_fails_closed_on_overflow() {
        let mut ledger = OperationLedger::default();
        ledger
            .charge(OperationKind::EnvironmentStep, 3)
            .expect("charge");
        ledger
            .charge(OperationKind::PlanningBackup, 7)
            .expect("charge");
        let snapshot = ledger.snapshot();
        assert_eq!(snapshot.0[&OperationKind::EnvironmentStep], 3);
        assert_eq!(snapshot.0[&OperationKind::PlanningBackup], 7);

        ledger
            .charge(OperationKind::ModelCall, u64::MAX)
            .expect("initial maximum");
        assert_eq!(
            ledger.charge(OperationKind::ModelCall, 1),
            Err(PortableAccountingError::Arithmetic)
        );
    }

    #[test]
    fn live_storage_accounting_is_byte_exact_and_scope_separated() {
        let root = tempfile::tempdir().expect("tempdir");
        let agent = root.path().join("agent");
        let observer = root.path().join("observer");
        fs::create_dir_all(agent.join("nested")).expect("agent dirs");
        fs::create_dir(&observer).expect("observer dir");
        fs::write(agent.join("state.bin"), [1_u8; 17]).expect("agent file");
        fs::write(agent.join("nested/weights.bin"), [2_u8; 31]).expect("weights");
        fs::write(observer.join("events.bin"), [3_u8; 43]).expect("observer file");

        let agent_snapshot = collect_storage(
            &agent,
            StorageRootClass::AgentOwned,
            StorageLimits::default(),
        )
        .expect("agent storage");
        let observer_snapshot = collect_storage(
            &observer,
            StorageRootClass::ObserverOwned,
            StorageLimits::default(),
        )
        .expect("observer storage");
        assert_eq!((agent_snapshot.file_count, agent_snapshot.bytes), (2, 48));
        assert_eq!(
            (observer_snapshot.file_count, observer_snapshot.bytes),
            (1, 43)
        );
        assert_ne!(agent_snapshot.root_class, observer_snapshot.root_class);
    }
}
