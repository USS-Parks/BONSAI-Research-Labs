use crate::{AgentCapabilityAudit, LifecycleRecord, LifecycleState};
use bonsai_contracts::adapter::{AdapterProtocolMachine, Peer, ProtocolState, ProtocolViolation};
use bonsai_contracts::bonsai::adapter::v1::AdapterFrame;
use bonsai_contracts::track::{Track, TrackDeclaration, derive_track};
use bonsai_ingest::{ObservedEvent, OrderingLimits, classify_partial_order};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

const CERTIFICATION_VERSION: &str = "1.0";
const EXPECTED_ENVIRONMENT_KEYS: [&str; 3] =
    ["BONSAI_AGENT_ROOT", "BONSAI_INPUT_ROOT", "BONSAI_WORK_ROOT"];
const EXPECTED_HANDLES: [&str; 3] = [
    "stdin:protocol",
    "stdout:protocol",
    "stderr:bounded-diagnostic",
];

/// One peer-attributed frame in an adapter certification transcript.
#[derive(Clone, Debug)]
pub struct ProtocolExchange {
    pub peer: Peer,
    pub frame: AdapterFrame,
}

/// Result of an intentionally stalled adapter deadline probe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeoutProbe {
    pub exercised: bool,
    pub failure_code: Option<String>,
    pub process_contained: bool,
    pub deadline_ns: u64,
    pub observed_ns: u64,
}

impl TimeoutProbe {
    #[must_use]
    pub const fn not_run() -> Self {
        Self {
            exercised: false,
            failure_code: None,
            process_contained: false,
            deadline_ns: 0,
            observed_ns: 0,
        }
    }
}

/// Evidence submitted for one adapter conformance decision.
#[derive(Clone, Debug)]
pub struct AdapterCertificationInput {
    pub adapter_id: String,
    pub protocol_transcript: Vec<ProtocolExchange>,
    pub capability_audit: Option<AgentCapabilityAudit>,
    pub observed_events: Vec<ObservedEvent>,
    pub lifecycle_records: Vec<LifecycleRecord>,
    pub determinism_probe_sha256: Vec<String>,
    pub timeout_probe: TimeoutProbe,
    pub track_declaration: TrackDeclaration,
    pub expected_track: Track,
}

/// Required certification dimensions, serialized in stable evaluation order.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificationCheck {
    Protocol,
    Isolation,
    Ordering,
    Lifecycle,
    Determinism,
    Timeout,
    Classification,
}

/// Ternary outcome for one certification dimension.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckVerdict {
    Pass,
    Fail,
    Indeterminate,
}

/// Machine result for one required check.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckResult {
    pub check: CertificationCheck,
    pub verdict: CheckVerdict,
    pub code: String,
}

/// Overall adapter-certification outcome.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificationVerdict {
    Certified,
    Rejected,
    Indeterminate,
}

/// Complete machine-readable adapter certification report.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterCertificationReport {
    pub schema: String,
    pub certification_version: String,
    pub adapter_id: String,
    pub verdict: CertificationVerdict,
    pub checks: Vec<CheckResult>,
    pub derived_track: Track,
    pub scientific_quality_certified: bool,
}

/// Input-level certification failures that prevent a report from being identified safely.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertificationError {
    AdapterIdentity,
}

impl CertificationError {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::AdapterIdentity => "ADAPTER_CERTIFICATION_ID_INVALID",
        }
    }
}

impl fmt::Display for CertificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl Error for CertificationError {}

/// Deterministic, adapter-neutral certification rule set.
#[derive(Clone, Copy, Debug, Default)]
pub struct AdapterConformanceSuite;

impl AdapterConformanceSuite {
    /// Evaluate all runtime-conformance dimensions exactly once.
    ///
    /// # Errors
    ///
    /// Rejects an unsafe or empty adapter identity before producing a report.
    pub fn certify(
        self,
        input: &AdapterCertificationInput,
    ) -> Result<AdapterCertificationReport, CertificationError> {
        validate_adapter_id(&input.adapter_id)?;
        let track_verdict = derive_track(&input.track_declaration);
        let checks = vec![
            check_protocol(&input.protocol_transcript),
            check_isolation(input.capability_audit.as_ref()),
            check_ordering(&input.observed_events),
            check_lifecycle(&input.lifecycle_records),
            check_determinism(&input.determinism_probe_sha256),
            check_timeout(&input.timeout_probe),
            check_classification(input.expected_track, track_verdict),
        ];
        let verdict = if checks
            .iter()
            .any(|result| result.verdict == CheckVerdict::Fail)
        {
            CertificationVerdict::Rejected
        } else if checks
            .iter()
            .any(|result| result.verdict == CheckVerdict::Indeterminate)
        {
            CertificationVerdict::Indeterminate
        } else {
            CertificationVerdict::Certified
        };
        Ok(AdapterCertificationReport {
            schema: "bonsai.adapter-certification-report/v1".to_owned(),
            certification_version: CERTIFICATION_VERSION.to_owned(),
            adapter_id: input.adapter_id.clone(),
            verdict,
            checks,
            derived_track: track_verdict.derived,
            scientific_quality_certified: false,
        })
    }
}

fn validate_adapter_id(adapter_id: &str) -> Result<(), CertificationError> {
    if adapter_id.is_empty()
        || adapter_id.len() > 96
        || !adapter_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        Err(CertificationError::AdapterIdentity)
    } else {
        Ok(())
    }
}

fn result(check: CertificationCheck, verdict: CheckVerdict, code: &str) -> CheckResult {
    CheckResult {
        check,
        verdict,
        code: code.to_owned(),
    }
}

fn check_protocol(transcript: &[ProtocolExchange]) -> CheckResult {
    if transcript.is_empty() {
        return result(
            CertificationCheck::Protocol,
            CheckVerdict::Indeterminate,
            "ADAPTER_PROTOCOL_EVIDENCE_MISSING",
        );
    }
    let mut machine = AdapterProtocolMachine::default();
    for exchange in transcript {
        if let Err(error) = machine.apply(exchange.peer, &exchange.frame) {
            return result(
                CertificationCheck::Protocol,
                CheckVerdict::Fail,
                protocol_code(&error),
            );
        }
    }
    if machine.state() == ProtocolState::Stopped {
        result(
            CertificationCheck::Protocol,
            CheckVerdict::Pass,
            "ADAPTER_PROTOCOL_CONFORMANT",
        )
    } else {
        result(
            CertificationCheck::Protocol,
            CheckVerdict::Indeterminate,
            "ADAPTER_PROTOCOL_TERMINAL_EVIDENCE_MISSING",
        )
    }
}

fn protocol_code(error: &ProtocolViolation) -> &'static str {
    match error {
        ProtocolViolation::MissingMessage => "ADAPTER_PROTOCOL_MESSAGE_MISSING",
        ProtocolViolation::OutOfOrder { .. } => "ADAPTER_PROTOCOL_OUT_OF_ORDER",
        ProtocolViolation::WrongPeer => "ADAPTER_PROTOCOL_WRONG_PEER",
        ProtocolViolation::Sequence => "ADAPTER_PROTOCOL_SEQUENCE_INVALID",
        ProtocolViolation::VersionMismatch => "ADAPTER_PROTOCOL_VERSION_MISMATCH",
        ProtocolViolation::CapabilityDeclaration => "ADAPTER_PROTOCOL_CAPABILITY_INVALID",
        ProtocolViolation::CapabilityChanged => "ADAPTER_PROTOCOL_CAPABILITY_CHANGED",
        ProtocolViolation::CapabilityNotDeclared(_) => "ADAPTER_PROTOCOL_CAPABILITY_UNDECLARED",
        ProtocolViolation::InvalidField(_) => "ADAPTER_PROTOCOL_FIELD_INVALID",
        ProtocolViolation::Deadline => "ADAPTER_PROTOCOL_DEADLINE_INVALID",
        ProtocolViolation::Digest(_) => "ADAPTER_PROTOCOL_DIGEST_INVALID",
        ProtocolViolation::PostStop => "ADAPTER_PROTOCOL_POST_STOP",
    }
}

fn check_isolation(audit: Option<&AgentCapabilityAudit>) -> CheckResult {
    let Some(audit) = audit else {
        return result(
            CertificationCheck::Isolation,
            CheckVerdict::Indeterminate,
            "ADAPTER_ISOLATION_EVIDENCE_MISSING",
        );
    };
    let environment = audit
        .environment_keys
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let handles = audit
        .inherited_handles
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected_environment = EXPECTED_ENVIRONMENT_KEYS
        .into_iter()
        .collect::<BTreeSet<_>>();
    let expected_handles = EXPECTED_HANDLES.into_iter().collect::<BTreeSet<_>>();
    if audit.observer_path_exposed {
        result(
            CertificationCheck::Isolation,
            CheckVerdict::Fail,
            "ISOLATION_OBSERVER_PATH_EXPOSURE",
        )
    } else if audit.current_directory.is_empty()
        || environment != expected_environment
        || handles != expected_handles
        || environment.len() != audit.environment_keys.len()
        || handles.len() != audit.inherited_handles.len()
    {
        result(
            CertificationCheck::Isolation,
            CheckVerdict::Fail,
            "ADAPTER_ISOLATION_AUDIT_INVALID",
        )
    } else {
        result(
            CertificationCheck::Isolation,
            CheckVerdict::Pass,
            "ADAPTER_ISOLATION_CONFORMANT",
        )
    }
}

fn check_ordering(events: &[ObservedEvent]) -> CheckResult {
    if events.is_empty() {
        return result(
            CertificationCheck::Ordering,
            CheckVerdict::Indeterminate,
            "ADAPTER_ORDERING_EVIDENCE_MISSING",
        );
    }
    let report = match classify_partial_order(events, OrderingLimits::default()) {
        Ok(report) => report,
        Err(error) => {
            return result(
                CertificationCheck::Ordering,
                CheckVerdict::Fail,
                error.code(),
            );
        }
    };
    let failure = [
        (
            !report.duplicate_event_ids.is_empty(),
            "ADAPTER_ORDERING_DUPLICATE",
        ),
        (
            !report.missing_parents.is_empty(),
            "ADAPTER_ORDERING_PARENT_MISSING",
        ),
        (
            !report.sequence_conflicts.is_empty(),
            "ADAPTER_ORDERING_SEQUENCE_CONFLICT",
        ),
        (
            !report.sequence_gaps.is_empty(),
            "ADAPTER_ORDERING_SEQUENCE_GAP",
        ),
        (!report.cycle_event_ids.is_empty(), "ADAPTER_ORDERING_CYCLE"),
        (
            !report.clock_regression_event_ids.is_empty(),
            "ADAPTER_ORDERING_CLOCK_REGRESSION",
        ),
    ]
    .into_iter()
    .find_map(|(failed, code)| failed.then_some(code));
    failure.map_or_else(
        || {
            result(
                CertificationCheck::Ordering,
                CheckVerdict::Pass,
                "ADAPTER_ORDERING_CONFORMANT",
            )
        },
        |code| result(CertificationCheck::Ordering, CheckVerdict::Fail, code),
    )
}

fn check_lifecycle(records: &[LifecycleRecord]) -> CheckResult {
    let Some(first) = records.first() else {
        return result(
            CertificationCheck::Lifecycle,
            CheckVerdict::Indeterminate,
            "ADAPTER_LIFECYCLE_EVIDENCE_MISSING",
        );
    };
    if first.ordinal != 0 || first.state != LifecycleState::Created {
        return result(
            CertificationCheck::Lifecycle,
            CheckVerdict::Fail,
            "ADAPTER_LIFECYCLE_INITIAL_INVALID",
        );
    }
    for (index, record) in records.iter().enumerate() {
        if record.ordinal != u64::try_from(index).unwrap_or(u64::MAX)
            || (index > 0 && !lifecycle_permits(records[index - 1].state, record.state))
        {
            return result(
                CertificationCheck::Lifecycle,
                CheckVerdict::Fail,
                "ADAPTER_LIFECYCLE_SEQUENCE_INVALID",
            );
        }
    }
    match records.last().map(|record| record.state) {
        Some(LifecycleState::Completed | LifecycleState::Recovered) => result(
            CertificationCheck::Lifecycle,
            CheckVerdict::Pass,
            "ADAPTER_LIFECYCLE_CONFORMANT",
        ),
        Some(LifecycleState::Failed) => result(
            CertificationCheck::Lifecycle,
            CheckVerdict::Fail,
            "ADAPTER_LIFECYCLE_FAILED",
        ),
        Some(_) | None => result(
            CertificationCheck::Lifecycle,
            CheckVerdict::Indeterminate,
            "ADAPTER_LIFECYCLE_TERMINAL_EVIDENCE_MISSING",
        ),
    }
}

const fn lifecycle_permits(current: LifecycleState, next: LifecycleState) -> bool {
    matches!(
        (current, next),
        (
            LifecycleState::Created,
            LifecycleState::Running | LifecycleState::Failed
        ) | (
            LifecycleState::Running,
            LifecycleState::Degraded | LifecycleState::Terminating | LifecycleState::Failed
        ) | (
            LifecycleState::Degraded,
            LifecycleState::Running | LifecycleState::Terminating | LifecycleState::Failed
        ) | (
            LifecycleState::Terminating,
            LifecycleState::Completed | LifecycleState::Failed
        ) | (LifecycleState::Failed, LifecycleState::Recovered)
    )
}

fn check_determinism(hashes: &[String]) -> CheckResult {
    if hashes.len() < 2 {
        return result(
            CertificationCheck::Determinism,
            CheckVerdict::Indeterminate,
            "ADAPTER_DETERMINISM_EVIDENCE_MISSING",
        );
    }
    if hashes.iter().any(|hash| !valid_sha256(hash)) {
        return result(
            CertificationCheck::Determinism,
            CheckVerdict::Fail,
            "ADAPTER_DETERMINISM_HASH_INVALID",
        );
    }
    if hashes.windows(2).all(|pair| pair[0] == pair[1]) {
        result(
            CertificationCheck::Determinism,
            CheckVerdict::Pass,
            "ADAPTER_DETERMINISM_CONFORMANT",
        )
    } else {
        result(
            CertificationCheck::Determinism,
            CheckVerdict::Fail,
            "ADAPTER_DETERMINISM_MISMATCH",
        )
    }
}

fn valid_sha256(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn check_timeout(probe: &TimeoutProbe) -> CheckResult {
    if !probe.exercised {
        return result(
            CertificationCheck::Timeout,
            CheckVerdict::Indeterminate,
            "ADAPTER_TIMEOUT_EVIDENCE_MISSING",
        );
    }
    let recognized = matches!(
        probe.failure_code.as_deref(),
        Some("TRANSPORT_READ_TIMEOUT" | "TRANSPORT_SHUTDOWN_TIMEOUT")
    );
    if recognized
        && probe.process_contained
        && probe.deadline_ns > 0
        && probe.observed_ns >= probe.deadline_ns
    {
        result(
            CertificationCheck::Timeout,
            CheckVerdict::Pass,
            "ADAPTER_TIMEOUT_CONFORMANT",
        )
    } else {
        result(
            CertificationCheck::Timeout,
            CheckVerdict::Fail,
            "ADAPTER_TIMEOUT_CONTAINMENT_FAILED",
        )
    }
}

fn check_classification(
    expected: Track,
    verdict: bonsai_contracts::track::TrackVerdict,
) -> CheckResult {
    if verdict.derived == Track::Indeterminate {
        result(
            CertificationCheck::Classification,
            CheckVerdict::Indeterminate,
            verdict.reason_code,
        )
    } else if !verdict.declaration_matches || verdict.derived != expected {
        result(
            CertificationCheck::Classification,
            CheckVerdict::Fail,
            "ADAPTER_TRACK_MISMATCH",
        )
    } else {
        result(
            CertificationCheck::Classification,
            CheckVerdict::Pass,
            verdict.reason_code,
        )
    }
}
