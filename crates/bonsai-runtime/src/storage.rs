//! Agent storage broker: metered persistence, replay classification, and path denial.

use crate::IsolatedRunLayout;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

const MAX_RELATIVE_PATH_BYTES: usize = 128;
const MAX_PATH_COMPONENTS: usize = 8;
const MAX_COMPONENT_BYTES: usize = 64;

/// Declared or inspected persistence class for one authorized write.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistenceClass {
    ModelParameters,
    BoundedAlgorithmState,
    ReplayBuffer,
}

/// Machine outcome for one persist attempt.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistOutcome {
    Admit,
    Reject,
}

/// Hard bounds on authorized agent-tree persistence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StoragePolicy {
    pub policy_id: String,
    pub max_bytes: u64,
    pub max_files: u64,
    pub max_file_bytes: u64,
    pub allow_replay: bool,
}

/// One immutable persist request against the agent work tree.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersistRequest {
    pub relative_path: String,
    pub declared_class: PersistenceClass,
    pub bytes: Vec<u8>,
}

/// Deterministic persist decision retaining classification and exact meter transition.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersistDecision {
    pub relative_path: String,
    pub declared_class: PersistenceClass,
    pub classified_as: PersistenceClass,
    pub outcome: PersistOutcome,
    pub reason_code: String,
    pub bytes_before: u64,
    pub bytes_after: u64,
    pub files_before: u64,
    pub files_after: u64,
    pub persisted_bytes: u64,
}

/// One inspected object remaining in the authorized work tree.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InspectedObject {
    pub relative_path: String,
    pub classified_as: PersistenceClass,
    pub bytes: u64,
}

/// Validated storage-policy and persist-path failures.
#[derive(Debug)]
#[non_exhaustive]
pub enum StorageError {
    PolicyIdentity,
    PolicyBound,
    RequestIdentity,
    PathTraversal,
    SymlinkDenied,
    ObserverPathDenied,
    Io(io::Error),
}

impl StorageError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::PolicyIdentity => "STORAGE_POLICY_ID_INVALID",
            Self::PolicyBound => "STORAGE_POLICY_BOUND_INVALID",
            Self::RequestIdentity => "STORAGE_REQUEST_INVALID",
            Self::PathTraversal => "STORAGE_PATH_TRAVERSAL",
            Self::SymlinkDenied => "STORAGE_SYMLINK_DENIED",
            Self::ObserverPathDenied => "STORAGE_OBSERVER_PATH_DENIED",
            Self::Io(_) => "STORAGE_IO_FAILED",
        }
    }
}

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.code())?;
        if let Self::Io(error) = self {
            write!(formatter, ": {error}")?;
        }
        Ok(())
    }
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for StorageError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl StoragePolicy {
    /// Validate identity and positive nested byte/file bounds.
    ///
    /// # Errors
    ///
    /// Rejects an empty or unsafe policy identity and zero or inverted bounds.
    pub fn validate(&self) -> Result<(), StorageError> {
        if self.policy_id.is_empty()
            || self.policy_id.len() > 96
            || !self
                .policy_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        {
            return Err(StorageError::PolicyIdentity);
        }
        if self.max_bytes == 0
            || self.max_files == 0
            || self.max_file_bytes == 0
            || self.max_file_bytes > self.max_bytes
        {
            return Err(StorageError::PolicyBound);
        }
        Ok(())
    }
}

/// Stateful broker over one isolated agent work tree.
#[derive(Clone, Debug)]
pub struct AgentStorageBroker {
    layout: IsolatedRunLayout,
    policy: StoragePolicy,
    used_bytes: u64,
    used_files: u64,
}

impl AgentStorageBroker {
    /// Construct an empty broker after validating the persistence policy.
    ///
    /// # Errors
    ///
    /// Returns the policy validation failure without retaining broker state.
    pub fn new(layout: IsolatedRunLayout, policy: StoragePolicy) -> Result<Self, StorageError> {
        policy.validate()?;
        Ok(Self {
            layout,
            policy,
            used_bytes: 0,
            used_files: 0,
        })
    }

    #[must_use]
    pub fn used_bytes(&self) -> u64 {
        self.used_bytes
    }

    #[must_use]
    pub fn used_files(&self) -> u64 {
        self.used_files
    }

    /// Classify payload bytes. Transition-shaped records override a non-replay declaration.
    #[must_use]
    pub fn classify(bytes: &[u8], declared: PersistenceClass) -> PersistenceClass {
        if looks_like_replay(bytes) || declared == PersistenceClass::ReplayBuffer {
            PersistenceClass::ReplayBuffer
        } else if let Some(kind) = payload_kind(bytes) {
            kind
        } else {
            declared
        }
    }

    /// Persist one authorized object or reject it without changing meters.
    ///
    /// Replay-buffer payloads are classified even when the write is denied. Path traversal,
    /// symlink ancestors, and observer-tree targets fail closed. Model parameters and bounded
    /// algorithm state remain admissible within the declared byte and file budgets.
    ///
    /// # Errors
    ///
    /// Returns a stable identity, traversal, symlink, observer, or I/O failure. Budget
    /// exhaustion and leftover-file meter mismatch are reject decisions, not errors.
    pub fn persist(&mut self, request: &PersistRequest) -> Result<PersistDecision, StorageError> {
        if request.bytes.is_empty() {
            return Err(StorageError::RequestIdentity);
        }
        let destination = self.resolve_destination(&request.relative_path)?;
        let classified = Self::classify(&request.bytes, request.declared_class);
        let requested =
            u64::try_from(request.bytes.len()).map_err(|_| StorageError::RequestIdentity)?;
        let bytes_before = self.used_bytes;
        let files_before = self.used_files;
        let previous = match fs::symlink_metadata(&destination) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(StorageError::SymlinkDenied);
                }
                Some(metadata.len())
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };

        if let Some(reason) = self.reject_reason(classified, requested, previous) {
            return Ok(PersistDecision {
                relative_path: request.relative_path.clone(),
                declared_class: request.declared_class,
                classified_as: classified,
                outcome: PersistOutcome::Reject,
                reason_code: reason.to_owned(),
                bytes_before,
                bytes_after: bytes_before,
                files_before,
                files_after: files_before,
                persisted_bytes: 0,
            });
        }

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
            self.reject_symlink_or_observer(parent)?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&destination)?;
        file.write_all(&request.bytes)?;
        file.flush()?;
        file.sync_all()?;
        self.reject_symlink_or_observer(&destination)?;

        let next_bytes = match previous {
            Some(existing) => bytes_before
                .saturating_sub(existing)
                .saturating_add(requested),
            None => bytes_before.saturating_add(requested),
        };
        let next_files = if previous.is_some() {
            files_before
        } else {
            files_before.saturating_add(1)
        };
        self.used_bytes = next_bytes;
        self.used_files = next_files;
        Ok(PersistDecision {
            relative_path: request.relative_path.clone(),
            declared_class: request.declared_class,
            classified_as: classified,
            outcome: PersistOutcome::Admit,
            reason_code: "STORAGE_PERSIST_ADMITTED".to_owned(),
            bytes_before,
            bytes_after: next_bytes,
            files_before,
            files_after: next_files,
            persisted_bytes: requested,
        })
    }

    /// Inspect every regular file remaining under the authorized work tree.
    ///
    /// # Errors
    ///
    /// Fails closed on I/O or if a symlink is discovered in the work tree.
    pub fn inspect(&self) -> Result<Vec<InspectedObject>, StorageError> {
        let mut objects = BTreeMap::new();
        self.walk(self.layout.writable_root(), &mut objects)?;
        Ok(objects.into_values().collect())
    }

    fn reject_reason(
        &self,
        classified: PersistenceClass,
        requested: u64,
        previous: Option<u64>,
    ) -> Option<&'static str> {
        if classified == PersistenceClass::ReplayBuffer && !self.policy.allow_replay {
            return Some("REPLAY_BUFFER_DETECTED");
        }
        if requested > self.policy.max_file_bytes {
            return Some("STORAGE_FILE_TOO_LARGE");
        }
        if previous.is_some_and(|existing| existing > self.used_bytes) {
            return Some("STORAGE_METER_INCONSISTENT");
        }
        let next_bytes = match previous {
            Some(existing) => self
                .used_bytes
                .saturating_sub(existing)
                .saturating_add(requested),
            None => self.used_bytes.saturating_add(requested),
        };
        if next_bytes > self.policy.max_bytes {
            return Some("STORAGE_BYTE_BUDGET_EXHAUSTED");
        }
        if previous.is_none() && self.used_files.saturating_add(1) > self.policy.max_files {
            return Some("STORAGE_FILE_BUDGET_EXHAUSTED");
        }
        None
    }

    fn resolve_destination(&self, relative: &str) -> Result<PathBuf, StorageError> {
        let components = parse_relative_components(relative)?;
        let mut current = self.layout.writable_root().to_path_buf();
        self.reject_symlink_or_observer(&current)?;
        for component in components {
            current.push(component);
            if current.exists() {
                self.reject_symlink_or_observer(&current)?;
            }
        }
        if !current.starts_with(self.layout.writable_root()) {
            return Err(StorageError::PathTraversal);
        }
        if current.starts_with(self.layout.observer_root())
            || current.starts_with(self.layout.telemetry_root())
            || current.starts_with(self.layout.index_root())
            || current.starts_with(self.layout.report_root())
        {
            return Err(StorageError::ObserverPathDenied);
        }
        Ok(current)
    }

    fn reject_symlink_or_observer(&self, path: &Path) -> Result<(), StorageError> {
        let metadata = fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink() {
            return Err(StorageError::SymlinkDenied);
        }
        let canonical = fs::canonicalize(path)?;
        if canonical.starts_with(self.layout.observer_root()) {
            return Err(StorageError::ObserverPathDenied);
        }
        if !canonical.starts_with(self.layout.agent_root()) {
            return Err(StorageError::PathTraversal);
        }
        Ok(())
    }

    fn walk(
        &self,
        directory: &Path,
        objects: &mut BTreeMap<String, InspectedObject>,
    ) -> Result<(), StorageError> {
        self.reject_symlink_or_observer(directory)?;
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)?;
            if metadata.file_type().is_symlink() {
                return Err(StorageError::SymlinkDenied);
            }
            if metadata.is_dir() {
                self.walk(&path, objects)?;
                continue;
            }
            if !metadata.is_file() {
                return Err(StorageError::RequestIdentity);
            }
            let relative = path
                .strip_prefix(self.layout.writable_root())
                .map_err(|_| StorageError::PathTraversal)?
                .to_string_lossy()
                .replace('\\', "/");
            let bytes = fs::read(&path)?;
            objects.insert(
                relative.clone(),
                InspectedObject {
                    relative_path: relative,
                    classified_as: Self::classify(&bytes, PersistenceClass::BoundedAlgorithmState),
                    bytes: u64::try_from(bytes.len()).map_err(|_| StorageError::RequestIdentity)?,
                },
            );
        }
        Ok(())
    }
}

fn parse_relative_components(relative: &str) -> Result<Vec<&str>, StorageError> {
    if relative.is_empty()
        || relative.len() > MAX_RELATIVE_PATH_BYTES
        || relative.starts_with('/')
        || relative.starts_with('\\')
        || Path::new(relative).is_absolute()
    {
        return Err(StorageError::PathTraversal);
    }
    let mut components = Vec::new();
    for raw in relative.split(['/', '\\']) {
        if raw.is_empty() || raw == "." || raw == ".." {
            return Err(StorageError::PathTraversal);
        }
        if raw.len() > MAX_COMPONENT_BYTES
            || !raw
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        {
            return Err(StorageError::RequestIdentity);
        }
        if Path::new(raw).components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        }) {
            return Err(StorageError::PathTraversal);
        }
        components.push(raw);
    }
    if components.is_empty() || components.len() > MAX_PATH_COMPONENTS {
        return Err(StorageError::PathTraversal);
    }
    Ok(components)
}

fn payload_kind(bytes: &[u8]) -> Option<PersistenceClass> {
    let value = serde_json::from_slice::<serde_json::Value>(bytes).ok()?;
    match value.get("kind").and_then(serde_json::Value::as_str) {
        Some("replay_buffer") => Some(PersistenceClass::ReplayBuffer),
        Some("model_parameters") => Some(PersistenceClass::ModelParameters),
        Some("algorithm_state") => Some(PersistenceClass::BoundedAlgorithmState),
        _ if value.get("slots").is_some() => Some(PersistenceClass::BoundedAlgorithmState),
        _ if value.get("values").is_some() => Some(PersistenceClass::ModelParameters),
        _ => None,
    }
}

fn looks_like_replay(bytes: &[u8]) -> bool {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return false;
    };
    if value.get("kind").and_then(serde_json::Value::as_str) == Some("replay_buffer") {
        return true;
    }
    value
        .get("transitions")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|transitions| transitions.iter().any(is_transition_record))
}

fn is_transition_record(value: &serde_json::Value) -> bool {
    value.get("state").is_some()
        && value.get("action").is_some()
        && value.get("reward").is_some()
        && value.get("next_state").is_some()
}
