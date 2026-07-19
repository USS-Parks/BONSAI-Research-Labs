use bonsai_bundle::{
    HARD_MAX_FRAME_SIZE, SegmentError, SegmentSummary, SegmentWriter, recover_open_segment,
    validate_bundle,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt::{self, Write as _};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

const JOURNAL_FILE: &str = "lifecycle.jsonl";
const SEGMENT_DIRECTORY: &str = "segments";
const TRANSITION_DIRECTORY: &str = "transitions";
const MAX_REASON_CODE_BYTES: usize = 96;
const MAX_TRANSITION_ID_BYTES: usize = 64;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleState {
    Created,
    Running,
    Degraded,
    Terminating,
    Completed,
    Failed,
    Recovered,
}

impl LifecycleState {
    const fn permits(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Created, Self::Running | Self::Failed)
                | (
                    Self::Running,
                    Self::Degraded | Self::Terminating | Self::Failed
                )
                | (
                    Self::Degraded,
                    Self::Running | Self::Terminating | Self::Failed
                )
                | (Self::Terminating, Self::Completed | Self::Failed)
                | (Self::Failed, Self::Recovered)
        )
    }

    const fn is_active(self) -> bool {
        matches!(
            self,
            Self::Created | Self::Running | Self::Degraded | Self::Terminating
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleRecord {
    pub ordinal: u64,
    pub state: LifecycleState,
    pub reason_code: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionOutcome {
    Preserved,
    AbandonedBeforeAppend,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RecoveredTransition {
    pub transition_id: String,
    pub segment_sequence: u64,
    pub outcome: TransitionOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryReport {
    pub final_state: LifecycleState,
    pub recovered_segment_sequences: Vec<u64>,
    pub validated_segments: Vec<SegmentSummary>,
    pub transitions: Vec<RecoveredTransition>,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum LifecycleError {
    Io(io::Error),
    Json(serde_json::Error),
    Segment(SegmentError),
    InvalidTransition {
        current: LifecycleState,
        requested: LifecycleState,
    },
    JournalEmpty,
    JournalOrdinal {
        expected: u64,
        actual: u64,
    },
    InvalidReasonCode,
    InvalidTransitionId,
    TransitionPending,
    TransitionAlreadyKnown,
    TransitionAlreadyConsumed,
    TransitionUnknown,
    TransitionPayloadMismatch,
    TransitionSequenceMismatch,
    TransitionNotAllowed(LifecycleState),
    SimulatedCrash,
}

impl LifecycleError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Io(_) => "LIFECYCLE_IO_FAILED",
            Self::Json(_) => "LIFECYCLE_JSON_INVALID",
            Self::Segment(error) => error.code(),
            Self::InvalidTransition { .. } => "LIFECYCLE_TRANSITION_INVALID",
            Self::JournalEmpty => "LIFECYCLE_JOURNAL_EMPTY",
            Self::JournalOrdinal { .. } => "LIFECYCLE_JOURNAL_ORDINAL_INVALID",
            Self::InvalidReasonCode => "LIFECYCLE_REASON_CODE_INVALID",
            Self::InvalidTransitionId => "LIFECYCLE_TRANSITION_ID_INVALID",
            Self::TransitionPending => "LIFECYCLE_TRANSITION_PENDING",
            Self::TransitionAlreadyKnown => "LIFECYCLE_TRANSITION_ALREADY_KNOWN",
            Self::TransitionAlreadyConsumed => "LIFECYCLE_TRANSITION_ALREADY_CONSUMED",
            Self::TransitionUnknown => "LIFECYCLE_TRANSITION_UNKNOWN",
            Self::TransitionPayloadMismatch => "LIFECYCLE_TRANSITION_PAYLOAD_MISMATCH",
            Self::TransitionSequenceMismatch => "LIFECYCLE_TRANSITION_SEQUENCE_MISMATCH",
            Self::TransitionNotAllowed(_) => "LIFECYCLE_EVENT_NOT_ALLOWED",
            Self::SimulatedCrash => "LIFECYCLE_SIMULATED_CRASH",
        }
    }
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.code())?;
        match self {
            Self::Io(error) => write!(formatter, ": {error}"),
            Self::Json(error) => write!(formatter, ": {error}"),
            Self::Segment(error) => write!(formatter, ": {error}"),
            Self::InvalidTransition { current, requested } => {
                write!(formatter, ": current={current:?}, requested={requested:?}")
            }
            Self::JournalOrdinal { expected, actual } => {
                write!(formatter, ": expected={expected}, actual={actual}")
            }
            Self::TransitionNotAllowed(state) => write!(formatter, ": state={state:?}"),
            _ => Ok(()),
        }
    }
}

impl Error for LifecycleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Segment(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for LifecycleError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for LifecycleError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<SegmentError> for LifecycleError {
    fn from(error: SegmentError) -> Self {
        Self::Segment(error)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct TransitionIntent {
    transition_id: String,
    segment_sequence: u64,
    payload_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct TransitionReceipt {
    transition_id: String,
    segment_sequence: u64,
    segment_sha256: String,
}

pub struct RunSupervisor {
    root: PathBuf,
    state: LifecycleState,
    next_ordinal: u64,
}

impl RunSupervisor {
    /// Create a new durable lifecycle evidence directory in the `created` state.
    ///
    /// # Errors
    ///
    /// Fails if a lifecycle journal already exists or storage cannot be synchronized.
    pub fn create(root: impl AsRef<Path>) -> Result<Self, LifecycleError> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(root.join(SEGMENT_DIRECTORY))?;
        fs::create_dir_all(root.join(TRANSITION_DIRECTORY))?;
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join(JOURNAL_FILE))?
            .sync_all()?;
        let mut supervisor = Self {
            root,
            state: LifecycleState::Created,
            next_ordinal: 0,
        };
        supervisor.append_state(LifecycleState::Created, None)?;
        Ok(supervisor)
    }

    /// Open crash evidence, close recoverable segments, settle every prepared
    /// transition exactly once, and terminate an interrupted run as recovered.
    /// Agent execution and learning are never resumed by this operation.
    ///
    /// # Errors
    ///
    /// Fails closed on malformed journals, corrupt segments, or inconsistent receipts.
    pub fn open_and_recover(
        root: impl AsRef<Path>,
    ) -> Result<(Self, RecoveryReport), LifecycleError> {
        let root = root.as_ref().to_path_buf();
        let records = read_journal(&root.join(JOURNAL_FILE))?;
        let last = records.last().ok_or(LifecycleError::JournalEmpty)?;
        let next_ordinal = last
            .ordinal
            .checked_add(1)
            .ok_or(LifecycleError::JournalOrdinal {
                expected: u64::MAX,
                actual: last.ordinal,
            })?;
        let mut supervisor = Self {
            root,
            state: last.state,
            next_ordinal,
        };
        let recovered_segment_sequences = supervisor.recover_open_segments()?;
        let validated_segments = validate_bundle(supervisor.segment_directory())?;
        let transitions = supervisor.settle_intents(&validated_segments)?;
        if supervisor.state.is_active() {
            supervisor.append_state(
                LifecycleState::Failed,
                Some("SUPERVISOR_EXITED_BEFORE_TERMINAL_STATE"),
            )?;
            supervisor.append_state(
                LifecycleState::Recovered,
                Some("CRASH_EVIDENCE_PRESERVED_NO_AGENT_RESUME"),
            )?;
        } else if supervisor.state == LifecycleState::Failed {
            supervisor.append_state(
                LifecycleState::Recovered,
                Some("CRASH_EVIDENCE_PRESERVED_NO_AGENT_RESUME"),
            )?;
        }
        let report = RecoveryReport {
            final_state: supervisor.state,
            recovered_segment_sequences,
            validated_segments,
            transitions,
        };
        Ok((supervisor, report))
    }

    #[must_use]
    pub const fn state(&self) -> LifecycleState {
        self.state
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Persist one legal state change before exposing it in memory.
    ///
    /// # Errors
    ///
    /// Rejects illegal edges and malformed reason codes before journal mutation.
    pub fn transition_to(
        &mut self,
        requested: LifecycleState,
        reason_code: Option<&str>,
    ) -> Result<(), LifecycleError> {
        if !self.state.permits(requested) {
            return Err(LifecycleError::InvalidTransition {
                current: self.state,
                requested,
            });
        }
        if matches!(
            requested,
            LifecycleState::Terminating | LifecycleState::Completed
        ) && self.any_pending_intent()?
        {
            return Err(LifecycleError::TransitionPending);
        }
        self.append_state(requested, reason_code)
    }

    /// Durably reserve the sole pending transition without executing it.
    ///
    /// # Errors
    ///
    /// Rejects terminal runs, malformed/duplicate IDs, oversized payloads, or
    /// an already pending transition.
    pub fn prepare_transition(
        &self,
        transition_id: &str,
        payload: &[u8],
    ) -> Result<(), LifecycleError> {
        self.ensure_event_state()?;
        validate_transition_id(transition_id)?;
        if payload.len() > HARD_MAX_FRAME_SIZE as usize {
            return Err(LifecycleError::Segment(SegmentError::FrameTooLarge {
                length: u32::try_from(payload.len()).unwrap_or(u32::MAX),
                maximum: HARD_MAX_FRAME_SIZE,
            }));
        }
        if self.any_pending_intent()? {
            return Err(LifecycleError::TransitionPending);
        }
        let path = self.intent_path(transition_id);
        if path.exists()
            || self.receipt_path(transition_id).exists()
            || self.consumed_path(transition_id).exists()
            || self.abandoned_path(transition_id).exists()
        {
            return Err(LifecycleError::TransitionAlreadyKnown);
        }
        let sequence = u64::try_from(validate_bundle(self.segment_directory())?.len())
            .map_err(|_| LifecycleError::TransitionSequenceMismatch)?;
        write_new_json(
            &path,
            &TransitionIntent {
                transition_id: transition_id.to_owned(),
                segment_sequence: sequence,
                payload_sha256: sha256_hex(payload),
            },
        )
    }

    /// Append and durably consume one prepared transition.
    ///
    /// # Errors
    ///
    /// Rejects unknown/already-consumed transitions and any payload or sequence mismatch.
    pub fn append_prepared(
        &self,
        transition_id: &str,
        payload: &[u8],
    ) -> Result<SegmentSummary, LifecycleError> {
        self.append_prepared_until(transition_id, payload, None)
    }

    fn append_state(
        &mut self,
        state: LifecycleState,
        reason_code: Option<&str>,
    ) -> Result<(), LifecycleError> {
        let reason_code = reason_code.map(validate_reason_code).transpose()?;
        let record = LifecycleRecord {
            ordinal: self.next_ordinal,
            state,
            reason_code,
        };
        let mut bytes = serde_json::to_vec(&record)?;
        bytes.push(b'\n');
        let mut file = OpenOptions::new()
            .append(true)
            .open(self.root.join(JOURNAL_FILE))?;
        file.write_all(&bytes)?;
        file.flush()?;
        file.sync_all()?;
        self.state = state;
        self.next_ordinal =
            self.next_ordinal
                .checked_add(1)
                .ok_or(LifecycleError::JournalOrdinal {
                    expected: u64::MAX,
                    actual: self.next_ordinal,
                })?;
        Ok(())
    }

    fn append_prepared_until(
        &self,
        transition_id: &str,
        payload: &[u8],
        crash_after: Option<CrashBoundary>,
    ) -> Result<SegmentSummary, LifecycleError> {
        self.ensure_event_state()?;
        validate_transition_id(transition_id)?;
        if self.consumed_path(transition_id).exists() {
            return Err(LifecycleError::TransitionAlreadyConsumed);
        }
        let intent: TransitionIntent =
            read_json(&self.intent_path(transition_id)).map_err(|error| match error {
                LifecycleError::Io(io_error) if io_error.kind() == io::ErrorKind::NotFound => {
                    LifecycleError::TransitionUnknown
                }
                other => other,
            })?;
        if intent.transition_id != transition_id || intent.payload_sha256 != sha256_hex(payload) {
            return Err(LifecycleError::TransitionPayloadMismatch);
        }
        if crash_after == Some(CrashBoundary::Intent) {
            return Err(LifecycleError::SimulatedCrash);
        }
        let expected_sequence = u64::try_from(validate_bundle(self.segment_directory())?.len())
            .map_err(|_| LifecycleError::TransitionSequenceMismatch)?;
        if intent.segment_sequence != expected_sequence {
            return Err(LifecycleError::TransitionSequenceMismatch);
        }
        let mut writer = SegmentWriter::create(
            self.segment_directory(),
            intent.segment_sequence,
            HARD_MAX_FRAME_SIZE,
        )?;
        writer.append(payload)?;
        writer.sync_pending()?;
        if crash_after == Some(CrashBoundary::FrameAppended) {
            drop(writer);
            return Err(LifecycleError::SimulatedCrash);
        }
        let summary = writer.finalize()?;
        if crash_after == Some(CrashBoundary::SegmentFinalized) {
            return Err(LifecycleError::SimulatedCrash);
        }
        self.write_receipt(&intent, &summary)?;
        if crash_after == Some(CrashBoundary::ReceiptWritten) {
            return Err(LifecycleError::SimulatedCrash);
        }
        write_new_bytes(&self.consumed_path(transition_id), b"consumed\n")?;
        if crash_after == Some(CrashBoundary::ConsumedWritten) {
            return Err(LifecycleError::SimulatedCrash);
        }
        Ok(summary)
    }

    fn ensure_event_state(&self) -> Result<(), LifecycleError> {
        if matches!(
            self.state,
            LifecycleState::Running | LifecycleState::Degraded
        ) {
            Ok(())
        } else {
            Err(LifecycleError::TransitionNotAllowed(self.state))
        }
    }

    fn recover_open_segments(&self) -> Result<Vec<u64>, LifecycleError> {
        let mut paths = fs::read_dir(self.segment_directory())?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;
        paths.retain(|path| {
            path.extension()
                .is_some_and(|extension| extension == "open")
        });
        paths.sort();
        let mut recovered = Vec::with_capacity(paths.len());
        for path in paths {
            let outcome = recover_open_segment(path)?;
            let sequence = match outcome {
                bonsai_bundle::RecoveryOutcome::Recovered(summary)
                | bonsai_bundle::RecoveryOutcome::AlreadyFinalized(summary) => summary.sequence,
            };
            recovered.push(sequence);
        }
        Ok(recovered)
    }

    fn settle_intents(
        &self,
        segments: &[SegmentSummary],
    ) -> Result<Vec<RecoveredTransition>, LifecycleError> {
        let mut paths = fs::read_dir(self.transition_directory())?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;
        paths.retain(|path| {
            path.extension()
                .is_some_and(|extension| extension == "intent")
        });
        paths.sort();
        let mut outcomes = Vec::with_capacity(paths.len());
        for path in paths {
            let intent: TransitionIntent = read_json(&path)?;
            validate_transition_id(&intent.transition_id)?;
            if self.consumed_path(&intent.transition_id).exists() {
                continue;
            }
            let outcome = if let Some(summary) = segments
                .iter()
                .find(|summary| summary.sequence == intent.segment_sequence)
            {
                if self.receipt_path(&intent.transition_id).exists() {
                    let receipt: TransitionReceipt =
                        read_json(&self.receipt_path(&intent.transition_id))?;
                    if receipt.transition_id != intent.transition_id
                        || receipt.segment_sequence != summary.sequence
                        || receipt.segment_sha256 != sha256_hex(&summary.checksum)
                    {
                        return Err(LifecycleError::TransitionSequenceMismatch);
                    }
                } else {
                    self.write_receipt(&intent, summary)?;
                }
                write_new_bytes(&self.consumed_path(&intent.transition_id), b"consumed\n")?;
                TransitionOutcome::Preserved
            } else {
                write_new_bytes(
                    &self.abandoned_path(&intent.transition_id),
                    b"abandoned-before-append\n",
                )?;
                TransitionOutcome::AbandonedBeforeAppend
            };
            outcomes.push(RecoveredTransition {
                transition_id: intent.transition_id,
                segment_sequence: intent.segment_sequence,
                outcome,
            });
        }
        Ok(outcomes)
    }

    fn write_receipt(
        &self,
        intent: &TransitionIntent,
        summary: &SegmentSummary,
    ) -> Result<(), LifecycleError> {
        if intent.segment_sequence != summary.sequence {
            return Err(LifecycleError::TransitionSequenceMismatch);
        }
        write_new_json(
            &self.receipt_path(&intent.transition_id),
            &TransitionReceipt {
                transition_id: intent.transition_id.clone(),
                segment_sequence: summary.sequence,
                segment_sha256: sha256_hex(&summary.checksum),
            },
        )
    }

    fn any_pending_intent(&self) -> Result<bool, LifecycleError> {
        for entry in fs::read_dir(self.transition_directory())? {
            let path = entry?.path();
            if path
                .extension()
                .is_some_and(|extension| extension == "intent")
            {
                let id = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .ok_or(LifecycleError::InvalidTransitionId)?;
                if !self.consumed_path(id).exists() && !self.abandoned_path(id).exists() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn segment_directory(&self) -> PathBuf {
        self.root.join(SEGMENT_DIRECTORY)
    }

    fn transition_directory(&self) -> PathBuf {
        self.root.join(TRANSITION_DIRECTORY)
    }

    fn intent_path(&self, id: &str) -> PathBuf {
        self.transition_directory().join(format!("{id}.intent"))
    }

    fn receipt_path(&self, id: &str) -> PathBuf {
        self.transition_directory().join(format!("{id}.receipt"))
    }

    fn consumed_path(&self, id: &str) -> PathBuf {
        self.transition_directory().join(format!("{id}.consumed"))
    }

    fn abandoned_path(&self, id: &str) -> PathBuf {
        self.transition_directory().join(format!("{id}.abandoned"))
    }
}

fn read_journal(path: &Path) -> Result<Vec<LifecycleRecord>, LifecycleError> {
    let file = BufReader::new(File::open(path)?);
    let mut records: Vec<LifecycleRecord> = Vec::new();
    for (expected, line) in file.lines().enumerate() {
        let record: LifecycleRecord = serde_json::from_str(&line?)?;
        let expected = u64::try_from(expected).map_err(|_| LifecycleError::JournalOrdinal {
            expected: u64::MAX,
            actual: record.ordinal,
        })?;
        if record.ordinal != expected {
            return Err(LifecycleError::JournalOrdinal {
                expected,
                actual: record.ordinal,
            });
        }
        if let Some(reason_code) = &record.reason_code {
            validate_reason_code(reason_code)?;
        }
        if expected == 0 && record.state != LifecycleState::Created {
            return Err(LifecycleError::InvalidTransition {
                current: record.state,
                requested: LifecycleState::Created,
            });
        }
        if let Some(previous) = records.last()
            && !previous.state.permits(record.state)
        {
            return Err(LifecycleError::InvalidTransition {
                current: previous.state,
                requested: record.state,
            });
        }
        records.push(record);
    }
    Ok(records)
}

fn validate_reason_code(value: &str) -> Result<String, LifecycleError> {
    if value.is_empty()
        || value.len() > MAX_REASON_CODE_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
    {
        Err(LifecycleError::InvalidReasonCode)
    } else {
        Ok(value.to_owned())
    }
}

fn validate_transition_id(value: &str) -> Result<(), LifecycleError> {
    if value.is_empty()
        || value.len() > MAX_TRANSITION_ID_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        Err(LifecycleError::InvalidTransitionId)
    } else {
        Ok(())
    }
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, LifecycleError> {
    Ok(serde_json::from_reader(BufReader::new(File::open(path)?))?)
}

fn write_new_json(path: &Path, value: &impl Serialize) -> Result<(), LifecycleError> {
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');
    write_new_bytes(path, &bytes)
}

fn write_new_bytes(path: &Path, bytes: &[u8]) -> Result<(), LifecycleError> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.flush()?;
    file.sync_all()?;
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to String is infallible");
    }
    encoded
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CrashBoundary {
    Intent,
    FrameAppended,
    SegmentFinalized,
    ReceiptWritten,
    ConsumedWritten,
}

#[cfg(test)]
mod tests {
    use super::{
        CrashBoundary, LifecycleError, LifecycleState, RunSupervisor, TransitionOutcome,
        read_journal,
    };
    use bonsai_bundle::validate_bundle;
    use tempfile::tempdir;

    #[test]
    fn lifecycle_state_machine_covers_every_declared_state_and_rejects_reentry() {
        let directory = tempdir().expect("temporary directory");
        let root = directory.path().join("run");
        let mut supervisor = RunSupervisor::create(&root).expect("create run");
        supervisor
            .transition_to(LifecycleState::Running, None)
            .expect("start");
        supervisor
            .transition_to(LifecycleState::Degraded, Some("SOFT_LIMIT"))
            .expect("degrade");
        supervisor
            .transition_to(LifecycleState::Running, Some("SOFT_LIMIT_CLEARED"))
            .expect("recover while live");
        supervisor
            .transition_to(LifecycleState::Terminating, None)
            .expect("terminate");
        supervisor
            .transition_to(LifecycleState::Completed, None)
            .expect("complete");
        assert!(matches!(
            supervisor.transition_to(LifecycleState::Running, None),
            Err(LifecycleError::InvalidTransition { .. })
        ));

        let (_, report) = RunSupervisor::open_and_recover(&root).expect("inspect completed run");
        assert_eq!(report.final_state, LifecycleState::Completed);
        assert!(report.validated_segments.is_empty());
    }

    #[test]
    fn kill_at_every_lifecycle_boundary_produces_terminal_recovery_evidence() {
        for boundary in [
            LifecycleState::Created,
            LifecycleState::Running,
            LifecycleState::Degraded,
            LifecycleState::Terminating,
            LifecycleState::Completed,
        ] {
            let directory = tempdir().expect("temporary directory");
            let root = directory.path().join("run");
            let mut supervisor = RunSupervisor::create(&root).expect("create run");
            if boundary != LifecycleState::Created {
                supervisor
                    .transition_to(LifecycleState::Running, None)
                    .expect("start");
            }
            if matches!(boundary, LifecycleState::Degraded) {
                supervisor
                    .transition_to(LifecycleState::Degraded, Some("FAULT_INJECTED"))
                    .expect("degrade");
            }
            if matches!(
                boundary,
                LifecycleState::Terminating | LifecycleState::Completed
            ) {
                supervisor
                    .transition_to(LifecycleState::Terminating, None)
                    .expect("terminate");
            }
            if boundary == LifecycleState::Completed {
                supervisor
                    .transition_to(LifecycleState::Completed, None)
                    .expect("complete");
            }
            drop(supervisor);
            let (_, report) = RunSupervisor::open_and_recover(&root).expect("recover boundary");
            let expected = if boundary == LifecycleState::Completed {
                LifecycleState::Completed
            } else {
                LifecycleState::Recovered
            };
            assert_eq!(report.final_state, expected, "boundary={boundary:?}");
            if boundary != LifecycleState::Completed {
                let journal = read_journal(&root.join("lifecycle.jsonl")).expect("valid journal");
                assert_eq!(journal[journal.len() - 2].state, LifecycleState::Failed);
                assert_eq!(
                    journal.last().expect("record").state,
                    LifecycleState::Recovered
                );
            }
        }
    }

    #[test]
    fn kill_at_every_transition_commit_boundary_never_double_consumes() {
        for boundary in [
            CrashBoundary::Intent,
            CrashBoundary::FrameAppended,
            CrashBoundary::SegmentFinalized,
            CrashBoundary::ReceiptWritten,
            CrashBoundary::ConsumedWritten,
        ] {
            let directory = tempdir().expect("temporary directory");
            let root = directory.path().join("run");
            let mut supervisor = RunSupervisor::create(&root).expect("create run");
            supervisor
                .transition_to(LifecycleState::Running, None)
                .expect("start");
            supervisor
                .prepare_transition("transition-001", b"opaque-transition")
                .expect("prepare");
            assert!(
                matches!(
                    supervisor.append_prepared_until(
                        "transition-001",
                        b"opaque-transition",
                        Some(boundary),
                    ),
                    Err(LifecycleError::SimulatedCrash)
                ),
                "boundary={boundary:?}"
            );
            drop(supervisor);

            let (recovered, report) =
                RunSupervisor::open_and_recover(&root).expect("recover transition");
            assert_eq!(report.final_state, LifecycleState::Recovered);
            let segments = validate_bundle(root.join("segments")).expect("valid event bundle");
            let expected_segments = usize::from(boundary != CrashBoundary::Intent);
            assert_eq!(segments.len(), expected_segments, "boundary={boundary:?}");
            assert!(segments.iter().all(|summary| summary.frame_count == 1));
            if boundary != CrashBoundary::ConsumedWritten {
                assert_eq!(report.transitions.len(), 1, "boundary={boundary:?}");
                let expected_outcome = if boundary == CrashBoundary::Intent {
                    TransitionOutcome::AbandonedBeforeAppend
                } else {
                    TransitionOutcome::Preserved
                };
                assert_eq!(report.transitions[0].outcome, expected_outcome);
            }
            assert!(matches!(
                recovered.append_prepared("transition-001", b"opaque-transition"),
                Err(
                    LifecycleError::TransitionNotAllowed(LifecycleState::Recovered)
                        | LifecycleError::TransitionAlreadyConsumed
                )
            ));
            assert_eq!(
                validate_bundle(root.join("segments"))
                    .expect("still valid")
                    .len(),
                expected_segments,
                "transition must not be appended twice"
            );
        }
    }

    #[test]
    fn normal_transition_is_exactly_once_and_pending_inputs_are_bounded() {
        let directory = tempdir().expect("temporary directory");
        let root = directory.path().join("run");
        let mut supervisor = RunSupervisor::create(&root).expect("create run");
        supervisor
            .transition_to(LifecycleState::Running, None)
            .expect("start");
        supervisor
            .prepare_transition("transition-001", b"first")
            .expect("prepare");
        assert!(matches!(
            supervisor.prepare_transition("transition-002", b"second"),
            Err(LifecycleError::TransitionPending)
        ));
        assert!(matches!(
            supervisor.transition_to(LifecycleState::Terminating, None),
            Err(LifecycleError::TransitionPending)
        ));
        supervisor
            .append_prepared("transition-001", b"first")
            .expect("append");
        assert!(matches!(
            supervisor.append_prepared("transition-001", b"first"),
            Err(LifecycleError::TransitionAlreadyConsumed)
        ));
        supervisor
            .transition_to(LifecycleState::Terminating, None)
            .expect("terminate");
        supervisor
            .transition_to(LifecycleState::Completed, None)
            .expect("complete");
        let (_, report) = RunSupervisor::open_and_recover(&root).expect("inspect");
        assert_eq!(report.final_state, LifecycleState::Completed);
        assert_eq!(report.validated_segments.len(), 1);
        assert_eq!(report.validated_segments[0].frame_count, 1);
    }

    #[test]
    fn committed_outcome_matrix_covers_every_crash_boundary() {
        let matrix: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/run-lifecycle/v1/expected-outcomes.json"
        ))
        .expect("valid committed matrix");
        assert_eq!(matrix["schema"], "bonsai.run-lifecycle-outcomes/v1");
        let outcomes = matrix["transition_crash_boundaries"]
            .as_array()
            .expect("boundary array");
        assert_eq!(outcomes.len(), 5);
        assert!(outcomes.iter().all(|outcome| {
            outcome["final_state"] == "recovered"
                && outcome["agent_learning_resumed"] == false
                && outcome["maximum_transition_consumptions"] == 1
        }));
    }
}
