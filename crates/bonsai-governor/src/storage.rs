//! Agent storage broker and replay-retention guard (BQ-06).
//!
//! Meters authorized agent persistence under hard byte/file caps, denies observer
//! paths and traversal/symlink escapes, and classifies transition-like retention so
//! Track A runs cannot silently keep a replay buffer. Model parameters and bounded
//! algorithm state remain admissible.

use bonsai_contracts::track::{
    Track, TrackDeclaration, TransitionAccess, UpdateSchedule, derive_track,
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Declared or detected persistence category.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionKind {
    ModelParameters,
    BoundedAlgorithmState,
    TransitionReplay,
    Unclassified,
}

/// Filesystem shape reported for a persistence target.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PathShape {
    RegularFile,
    Directory,
    Symlink,
    Missing,
}

/// Content signals used to detect transition-like retention.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
// Independent field presence flags; packing them would hide contradictory evidence.
#[allow(clippy::struct_excessive_bools)]
pub struct RetentionSignals {
    pub transition_record_count: u64,
    pub observation_fields: bool,
    pub action_fields: bool,
    pub reward_fields: bool,
    pub next_observation_fields: bool,
    pub parameter_tensor_count: u64,
    pub bounded_state_bytes: u64,
}

impl RetentionSignals {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            transition_record_count: 0,
            observation_fields: false,
            action_fields: false,
            reward_fields: false,
            next_observation_fields: false,
            parameter_tensor_count: 0,
            bounded_state_bytes: 0,
        }
    }

    #[must_use]
    pub const fn looks_like_transition_replay(&self) -> bool {
        self.transition_record_count > 0
            && self.observation_fields
            && self.action_fields
            && (self.reward_fields || self.next_observation_fields)
    }
}

/// Fixed storage policy for one supervised agent work tree.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StoragePolicy {
    pub policy_id: String,
    pub max_bytes: u64,
    pub max_files: u64,
    pub max_bounded_state_bytes: u64,
    pub allow_transition_replay: bool,
    pub allowed_replay_capacity_transitions: u64,
}

/// One immutable request to create or grow agent-owned persistence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersistenceRequest {
    pub request_id: String,
    pub logical_path: String,
    pub byte_delta: u64,
    pub file_delta: u64,
    pub declared_kind: RetentionKind,
    pub path_shape: PathShape,
    pub signals: RetentionSignals,
}

/// Machine outcome for a persistence request.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistenceOutcome {
    Admit,
    Reject,
}

/// Deterministic broker decision retaining classification and meter transition.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersistenceDecision {
    pub sequence: u64,
    pub request_id: String,
    pub logical_path: String,
    pub declared_kind: RetentionKind,
    pub classified_kind: RetentionKind,
    pub byte_delta: u64,
    pub file_delta: u64,
    pub bytes_before: u64,
    pub bytes_after: u64,
    pub files_before: u64,
    pub files_after: u64,
    pub outcome: PersistenceOutcome,
    pub reason_code: String,
    pub track_fact_replay_capacity: u64,
}

/// Current metered usage under the storage policy.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StorageUsage {
    pub bytes_consumed: u64,
    pub files_consumed: u64,
    pub max_bytes: u64,
    pub max_files: u64,
    pub classified_transition_capacity: u64,
}

/// Validated policy/request/path failures that never become broker decisions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageError {
    PolicyIdentity,
    PolicyBounds,
    RequestIdentity,
    Arithmetic,
}

impl StorageError {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::PolicyIdentity => "STORAGE_POLICY_ID_INVALID",
            Self::PolicyBounds => "STORAGE_POLICY_BOUNDS_INVALID",
            Self::RequestIdentity => "STORAGE_REQUEST_INVALID",
            Self::Arithmetic => "STORAGE_ARITHMETIC_FAILED",
        }
    }
}

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl Error for StorageError {}

impl StoragePolicy {
    /// Validate positive meters and replay-capacity consistency.
    ///
    /// # Errors
    ///
    /// Rejects empty/invalid identity, zero meters, or replay capacity when replay is denied.
    pub fn validate(&self) -> Result<(), StorageError> {
        if !valid_identity(&self.policy_id) {
            return Err(StorageError::PolicyIdentity);
        }
        if self.max_bytes == 0 || self.max_files == 0 || self.max_bounded_state_bytes == 0 {
            return Err(StorageError::PolicyBounds);
        }
        if !self.allow_transition_replay && self.allowed_replay_capacity_transitions > 0 {
            return Err(StorageError::PolicyBounds);
        }
        if self.allow_transition_replay && self.allowed_replay_capacity_transitions == 0 {
            return Err(StorageError::PolicyBounds);
        }
        Ok(())
    }
}

/// Stateful agent storage broker.
#[derive(Clone, Debug)]
pub struct StorageBroker {
    policy: StoragePolicy,
    bytes_consumed: u64,
    files_consumed: u64,
    classified_transition_capacity: u64,
    next_sequence: u64,
}

#[derive(Clone, Copy)]
struct MeterProjection {
    sequence: u64,
    bytes_before: u64,
    files_before: u64,
    projected_bytes: u64,
    projected_files: u64,
}

impl StorageBroker {
    /// Construct an empty broker after validating the storage policy.
    ///
    /// # Errors
    ///
    /// Returns the policy validation failure without retaining partial state.
    pub fn new(policy: StoragePolicy) -> Result<Self, StorageError> {
        policy.validate()?;
        Ok(Self {
            policy,
            bytes_consumed: 0,
            files_consumed: 0,
            classified_transition_capacity: 0,
            next_sequence: 0,
        })
    }

    #[must_use]
    pub fn policy(&self) -> &StoragePolicy {
        &self.policy
    }

    #[must_use]
    pub fn usage(&self) -> StorageUsage {
        StorageUsage {
            bytes_consumed: self.bytes_consumed,
            files_consumed: self.files_consumed,
            max_bytes: self.policy.max_bytes,
            max_files: self.policy.max_files,
            classified_transition_capacity: self.classified_transition_capacity,
        }
    }

    /// Classify, authorize, and only on admission meter one persistence request.
    ///
    /// # Errors
    ///
    /// Rejects malformed request identity or checked-arithmetic overflow. Path, observer,
    /// symlink, growth, and retention denials are retained as ordinary reject decisions.
    pub fn persist(
        &mut self,
        request: &PersistenceRequest,
    ) -> Result<PersistenceDecision, StorageError> {
        validate_request(request)?;
        let sequence = self.next_sequence;
        let next_sequence = sequence.checked_add(1).ok_or(StorageError::Arithmetic)?;
        let bytes_before = self.bytes_consumed;
        let files_before = self.files_consumed;
        let classified = classify_retention(request, &self.policy);

        if let Some(reason) = path_rejection(request) {
            self.next_sequence = next_sequence;
            return Ok(reject(
                sequence,
                request,
                classified,
                bytes_before,
                files_before,
                reason,
                0,
            ));
        }

        let projected_bytes = bytes_before
            .checked_add(request.byte_delta)
            .ok_or(StorageError::Arithmetic)?;
        let projected_files = files_before
            .checked_add(request.file_delta)
            .ok_or(StorageError::Arithmetic)?;
        if projected_bytes > self.policy.max_bytes {
            self.next_sequence = next_sequence;
            return Ok(reject(
                sequence,
                request,
                classified,
                bytes_before,
                files_before,
                "STORAGE_BYTE_BUDGET_EXHAUSTED",
                0,
            ));
        }
        if projected_files > self.policy.max_files {
            self.next_sequence = next_sequence;
            return Ok(reject(
                sequence,
                request,
                classified,
                bytes_before,
                files_before,
                "STORAGE_FILE_BUDGET_EXHAUSTED",
                0,
            ));
        }

        let projection = MeterProjection {
            sequence,
            bytes_before,
            files_before,
            projected_bytes,
            projected_files,
        };
        let decision = self.authorize_classified(request, classified, projection);
        self.next_sequence = next_sequence;
        Ok(decision)
    }

    fn authorize_classified(
        &mut self,
        request: &PersistenceRequest,
        classified: RetentionKind,
        projection: MeterProjection,
    ) -> PersistenceDecision {
        match classified {
            RetentionKind::TransitionReplay => self.authorize_transition(request, projection),
            RetentionKind::ModelParameters => self.commit(
                projection.projected_bytes,
                projection.projected_files,
                admit(
                    projection.sequence,
                    request,
                    classified,
                    projection.bytes_before,
                    projection.files_before,
                    "MODEL_PARAMETERS_ALLOWED",
                    0,
                ),
            ),
            RetentionKind::BoundedAlgorithmState => {
                if request.signals.bounded_state_bytes > self.policy.max_bounded_state_bytes
                    || request.byte_delta > self.policy.max_bounded_state_bytes
                {
                    return reject(
                        projection.sequence,
                        request,
                        classified,
                        projection.bytes_before,
                        projection.files_before,
                        "BOUNDED_ALGORITHM_STATE_LIMIT_EXCEEDED",
                        0,
                    );
                }
                self.commit(
                    projection.projected_bytes,
                    projection.projected_files,
                    admit(
                        projection.sequence,
                        request,
                        classified,
                        projection.bytes_before,
                        projection.files_before,
                        "BOUNDED_ALGORITHM_STATE_ALLOWED",
                        0,
                    ),
                )
            }
            RetentionKind::Unclassified => reject(
                projection.sequence,
                request,
                classified,
                projection.bytes_before,
                projection.files_before,
                "STORAGE_RETENTION_UNCLASSIFIED",
                0,
            ),
        }
    }

    fn authorize_transition(
        &mut self,
        request: &PersistenceRequest,
        projection: MeterProjection,
    ) -> PersistenceDecision {
        let capacity = transition_capacity(request);
        if !self.policy.allow_transition_replay
            || capacity > self.policy.allowed_replay_capacity_transitions
        {
            return reject(
                projection.sequence,
                request,
                RetentionKind::TransitionReplay,
                projection.bytes_before,
                projection.files_before,
                "TRANSITION_REPLAY_RETENTION_DENIED",
                capacity,
            );
        }
        self.classified_transition_capacity =
            self.classified_transition_capacity.saturating_add(capacity);
        self.commit(
            projection.projected_bytes,
            projection.projected_files,
            admit(
                projection.sequence,
                request,
                RetentionKind::TransitionReplay,
                projection.bytes_before,
                projection.files_before,
                "TRANSITION_REPLAY_RETENTION_CLASSIFIED",
                capacity,
            ),
        )
    }

    fn commit(
        &mut self,
        projected_bytes: u64,
        projected_files: u64,
        decision: PersistenceDecision,
    ) -> PersistenceDecision {
        self.bytes_consumed = projected_bytes;
        self.files_consumed = projected_files;
        decision
    }

    /// Project BC-05 track facts from accumulated broker classification.
    #[must_use]
    pub fn track_declaration_overlay(&self, declared: Track) -> TrackDeclaration {
        let replay_capacity = self.classified_transition_capacity;
        TrackDeclaration {
            schema_version: "1.0".to_owned(),
            declared_track: declared,
            runtime_facts_complete: true,
            batch_size: 1,
            transition_access: if replay_capacity > 0 {
                TransitionAccess::Replay
            } else {
                TransitionAccess::SinglePass
            },
            replay_capacity_transitions: replay_capacity,
            offline_updates: false,
            observer_data_access: false,
            privileged_state: false,
            human_labels: false,
            domain_feature_targets: false,
            update_schedule: UpdateSchedule::EventDriven,
            fixed_external_budgets: true,
        }
    }

    /// Derive the evaluation track after applying broker retention facts.
    #[must_use]
    pub fn derived_track(&self, declared: Track) -> Track {
        derive_track(&self.track_declaration_overlay(declared)).derived
    }
}

/// Inspect a path beneath an agent work root without following symlinks.
///
/// # Errors
///
/// Returns I/O failures other than not-found. Symlinks are reported, not followed.
pub fn inspect_path_shape(path: impl AsRef<Path>) -> Result<PathShape, std::io::Error> {
    match fs::symlink_metadata(path.as_ref()) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                Ok(PathShape::Symlink)
            } else if metadata.is_dir() {
                Ok(PathShape::Directory)
            } else {
                Ok(PathShape::RegularFile)
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(PathShape::Missing),
        Err(error) => Err(error),
    }
}

/// Resolve a logical agent path against a work root, rejecting traversal.
///
/// # Errors
///
/// Returns `None` when the logical path escapes the work root or is absolute.
#[must_use]
pub fn resolve_under_work_root(work_root: &Path, logical_path: &str) -> Option<PathBuf> {
    if logical_path.is_empty() || Path::new(logical_path).is_absolute() {
        return None;
    }
    let mut resolved = work_root.to_path_buf();
    for component in Path::new(logical_path).components() {
        match component {
            Component::Normal(part) => resolved.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    let work = normalize_lexical(work_root);
    let candidate = normalize_lexical(&resolved);
    if candidate.starts_with(&work) {
        Some(resolved)
    } else {
        None
    }
}

fn normalize_lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                let _ = out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn validate_request(request: &PersistenceRequest) -> Result<(), StorageError> {
    if !valid_identity(&request.request_id)
        || request.logical_path.is_empty()
        || request.logical_path.len() > 256
        || (request.byte_delta == 0 && request.file_delta == 0)
    {
        Err(StorageError::RequestIdentity)
    } else {
        Ok(())
    }
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

fn classify_retention(request: &PersistenceRequest, policy: &StoragePolicy) -> RetentionKind {
    if request.signals.looks_like_transition_replay()
        || request.declared_kind == RetentionKind::TransitionReplay
        || logical_name_implies_replay(&request.logical_path)
    {
        return RetentionKind::TransitionReplay;
    }
    if request.declared_kind == RetentionKind::ModelParameters
        && request.signals.parameter_tensor_count > 0
        && !request.signals.looks_like_transition_replay()
    {
        return RetentionKind::ModelParameters;
    }
    if request.declared_kind == RetentionKind::BoundedAlgorithmState
        && request.signals.bounded_state_bytes > 0
        && request.signals.bounded_state_bytes <= policy.max_bounded_state_bytes
        && request.signals.transition_record_count == 0
    {
        return RetentionKind::BoundedAlgorithmState;
    }
    if request.declared_kind == RetentionKind::ModelParameters {
        return RetentionKind::ModelParameters;
    }
    if request.declared_kind == RetentionKind::BoundedAlgorithmState {
        return RetentionKind::BoundedAlgorithmState;
    }
    RetentionKind::Unclassified
}

fn logical_name_implies_replay(logical_path: &str) -> bool {
    let lower = logical_path.to_ascii_lowercase();
    lower.contains("replay_buffer")
        || lower.contains("experience_replay")
        || lower.contains("transition_buffer")
        || lower.contains("/transitions.")
        || lower.ends_with("transitions.bin")
        || lower.ends_with("transitions.json")
}

fn transition_capacity(request: &PersistenceRequest) -> u64 {
    request
        .signals
        .transition_record_count
        .max(u64::from(logical_name_implies_replay(
            &request.logical_path,
        )))
        .max(u64::from(
            request.declared_kind == RetentionKind::TransitionReplay,
        ))
}

fn path_rejection(request: &PersistenceRequest) -> Option<&'static str> {
    if request.path_shape == PathShape::Symlink {
        return Some("STORAGE_SYMLINK_DENIED");
    }
    if !logical_path_is_safe(&request.logical_path) {
        return Some("STORAGE_PATH_TRAVERSAL_DENIED");
    }
    if exposes_observer_path(&request.logical_path) {
        return Some("STORAGE_OBSERVER_PATH_DENIED");
    }
    None
}

fn exposes_observer_path(logical_path: &str) -> bool {
    let lower = logical_path.replace('\\', "/").to_ascii_lowercase();
    lower.contains("observer/telemetry")
        || lower.contains("observer/index")
        || lower.contains("observer/reports")
        || lower.starts_with("observer/")
        || lower.contains("/observer/")
}

fn logical_path_is_safe(logical_path: &str) -> bool {
    if logical_path.is_empty() || Path::new(logical_path).is_absolute() {
        return false;
    }
    let normalized = logical_path.replace('\\', "/");
    if normalized.starts_with('/') || normalized.contains(':') {
        return false;
    }
    for component in normalized.split('/') {
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".." || component.contains('\0') {
            return false;
        }
    }
    true
}

fn admit(
    sequence: u64,
    request: &PersistenceRequest,
    classified: RetentionKind,
    bytes_before: u64,
    files_before: u64,
    reason_code: &str,
    track_fact_replay_capacity: u64,
) -> PersistenceDecision {
    finalize(
        sequence,
        request,
        classified,
        bytes_before,
        files_before,
        PersistenceOutcome::Admit,
        reason_code,
        track_fact_replay_capacity,
    )
}

fn reject(
    sequence: u64,
    request: &PersistenceRequest,
    classified: RetentionKind,
    bytes_before: u64,
    files_before: u64,
    reason_code: &str,
    track_fact_replay_capacity: u64,
) -> PersistenceDecision {
    finalize(
        sequence,
        request,
        classified,
        bytes_before,
        files_before,
        PersistenceOutcome::Reject,
        reason_code,
        track_fact_replay_capacity,
    )
}

#[allow(clippy::too_many_arguments)]
fn finalize(
    sequence: u64,
    request: &PersistenceRequest,
    classified: RetentionKind,
    bytes_before: u64,
    files_before: u64,
    outcome: PersistenceOutcome,
    reason_code: &str,
    track_fact_replay_capacity: u64,
) -> PersistenceDecision {
    let (bytes_after, files_after) = if outcome == PersistenceOutcome::Admit {
        (
            bytes_before.saturating_add(request.byte_delta),
            files_before.saturating_add(request.file_delta),
        )
    } else {
        (bytes_before, files_before)
    };
    PersistenceDecision {
        sequence,
        request_id: request.request_id.clone(),
        logical_path: request.logical_path.clone(),
        declared_kind: request.declared_kind,
        classified_kind: classified,
        byte_delta: request.byte_delta,
        file_delta: request.file_delta,
        bytes_before,
        bytes_after,
        files_before,
        files_after,
        outcome,
        reason_code: reason_code.to_owned(),
        track_fact_replay_capacity,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PathShape, PersistenceOutcome, PersistenceRequest, RetentionKind, RetentionSignals,
        StorageBroker, StoragePolicy, classify_retention, logical_path_is_safe,
    };
    use bonsai_contracts::track::Track;

    fn policy() -> StoragePolicy {
        StoragePolicy {
            policy_id: "m2.agent-storage.v1".to_owned(),
            max_bytes: 1_024,
            max_files: 8,
            max_bounded_state_bytes: 128,
            allow_transition_replay: false,
            allowed_replay_capacity_transitions: 0,
        }
    }

    fn request(
        request_id: &str,
        logical_path: &str,
        byte_delta: u64,
        file_delta: u64,
        declared_kind: RetentionKind,
        signals: RetentionSignals,
    ) -> PersistenceRequest {
        PersistenceRequest {
            request_id: request_id.to_owned(),
            logical_path: logical_path.to_owned(),
            byte_delta,
            file_delta,
            declared_kind,
            path_shape: PathShape::RegularFile,
            signals,
        }
    }

    #[test]
    fn model_parameters_are_admitted_under_byte_meter() {
        let mut broker = StorageBroker::new(policy()).expect("broker");
        let decision = broker
            .persist(&request(
                "params",
                "models/q_table.bin",
                64,
                1,
                RetentionKind::ModelParameters,
                RetentionSignals {
                    parameter_tensor_count: 1,
                    ..RetentionSignals::empty()
                },
            ))
            .expect("decision");
        assert_eq!(decision.outcome, PersistenceOutcome::Admit);
        assert_eq!(decision.reason_code, "MODEL_PARAMETERS_ALLOWED");
        assert_eq!(broker.derived_track(Track::A), Track::A);
    }

    #[test]
    fn replay_buffer_fixture_is_classified_and_denied_for_track_a() {
        let mut broker = StorageBroker::new(policy()).expect("broker");
        let decision = broker
            .persist(&request(
                "replay",
                "buffers/replay_buffer.json",
                128,
                1,
                RetentionKind::Unclassified,
                RetentionSignals {
                    transition_record_count: 32,
                    observation_fields: true,
                    action_fields: true,
                    reward_fields: true,
                    next_observation_fields: true,
                    ..RetentionSignals::empty()
                },
            ))
            .expect("decision");
        assert_eq!(decision.classified_kind, RetentionKind::TransitionReplay);
        assert_eq!(decision.outcome, PersistenceOutcome::Reject);
        assert_eq!(decision.reason_code, "TRANSITION_REPLAY_RETENTION_DENIED");
        assert_eq!(decision.track_fact_replay_capacity, 32);
        assert_eq!(broker.usage().bytes_consumed, 0);
    }

    #[test]
    fn traversal_and_observer_paths_fail_closed() {
        assert!(!logical_path_is_safe("../secrets"));
        assert!(!logical_path_is_safe("work/../../observer/telemetry/x"));
        let mut broker = StorageBroker::new(policy()).expect("broker");
        let traversal = broker
            .persist(&request(
                "escape",
                "../observer/telemetry/events.bin",
                1,
                1,
                RetentionKind::ModelParameters,
                RetentionSignals {
                    parameter_tensor_count: 1,
                    ..RetentionSignals::empty()
                },
            ))
            .expect("decision");
        assert_eq!(traversal.outcome, PersistenceOutcome::Reject);
        assert_eq!(traversal.reason_code, "STORAGE_PATH_TRAVERSAL_DENIED");
        let mut observer_req = request(
            "observer",
            "observer/reports/leak.html",
            1,
            1,
            RetentionKind::BoundedAlgorithmState,
            RetentionSignals {
                bounded_state_bytes: 1,
                ..RetentionSignals::empty()
            },
        );
        let observer = broker.persist(&observer_req).expect("decision");
        assert_eq!(observer.reason_code, "STORAGE_OBSERVER_PATH_DENIED");
        observer_req.path_shape = PathShape::Symlink;
        observer_req.logical_path = "work/link.bin".to_owned();
        observer_req.request_id = "symlink".to_owned();
        let symlink = broker.persist(&observer_req).expect("decision");
        assert_eq!(symlink.reason_code, "STORAGE_SYMLINK_DENIED");
    }

    #[test]
    fn name_based_replay_detection_does_not_depend_on_declaration() {
        let policy = policy();
        let request = request(
            "hidden",
            "cache/experience_replay.dat",
            8,
            1,
            RetentionKind::ModelParameters,
            RetentionSignals::empty(),
        );
        assert_eq!(
            classify_retention(&request, &policy),
            RetentionKind::TransitionReplay
        );
    }
}
