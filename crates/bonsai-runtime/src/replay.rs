use crate::{AgentLaunchPolicy, IsolatedRunLayout, ObserverArtifactClass};
use bonsai_bundle::{
    AnalyticalTable, BlobId, DerivationError, DerivationSpec, DerivedTableSummary, EventRow,
    MetricRow, materialize_derivation,
};
use bonsai_contracts::bonsai::event::v1::EventEnvelope;
use bonsai_contracts::track::{Track, TrackDeclaration, derive_track};
use bonsai_contracts::validate_event;
use bonsai_ingest::{ObservedEvent, OrderingError, OrderingLimits, classify_partial_order};
use bonsai_metrics::{MetricAvailability, MetricError, MetricKey, MetricRegistry, RationalValue};
use bonsai_report::{ReportData, ReportError, generate_static_report};
use prost::Message;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

const REPLAY_PRODUCER_ID: &str = "bonsai-runtime.observer-replay";
const REPLAY_PRODUCER_VERSION: &str = "1.0";
const REPLAY_SEAL_DOMAIN: &[u8] = b"bonsai.observer-replay-seal/v1\0";

/// Kind of immutable observer-side output produced by replay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObserverReplayArtifactKind {
    EventTable,
    MetricTable,
    ReportJson,
    ReportHtml,
}

impl ObserverReplayArtifactKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::EventTable => "event_table",
            Self::MetricTable => "metric_table",
            Self::ReportJson => "report_json",
            Self::ReportHtml => "report_html",
        }
    }

    const fn observer_class(self) -> ObserverArtifactClass {
        match self {
            Self::EventTable | Self::MetricTable => ObserverArtifactClass::Index,
            Self::ReportJson | Self::ReportHtml => ObserverArtifactClass::Report,
        }
    }
}

/// Public metadata for one replay output. Bytes and filesystem paths remain private to the observer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObserverReplayArtifact {
    pub kind: ObserverReplayArtifactKind,
    pub content_sha256: String,
    pub source_sha256: String,
    pub seal_sha256: String,
    pub bytes: u64,
}

/// Deterministic replay result containing only summaries and sealed metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObserverReplayOutput {
    pub run_id_hex: String,
    pub source_sha256: String,
    pub event_summary: DerivedTableSummary,
    pub metric_summary: DerivedTableSummary,
    pub artifacts: Vec<ObserverReplayArtifact>,
    pub track_eligibility: Track,
}

/// Requested destination for replay-derived information.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayDestination {
    ObserverIndex,
    ObserverReport,
    AgentInput,
    AgentProtocolFeedback,
}

/// Machine decision for a proposed replay-output route.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayRouteDecision {
    pub allowed: bool,
    pub code: &'static str,
    pub derived_track: Track,
}

/// Stable replay-analysis failures.
#[derive(Debug)]
#[non_exhaustive]
pub enum ReplayError {
    Empty,
    Event(&'static str),
    RunMismatch,
    Ordering(OrderingError),
    IncompleteOrder,
    Metric(MetricError),
    Derivation(DerivationError),
    Report(ReportError),
    Json,
    Io(io::Error),
}

impl ReplayError {
    /// Stable machine-oriented outcome code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Empty => "REPLAY_INPUT_EMPTY",
            Self::Event(code) => code,
            Self::RunMismatch => "REPLAY_RUN_MISMATCH",
            Self::Ordering(error) => error.code(),
            Self::IncompleteOrder => "REPLAY_ORDER_INCOMPLETE",
            Self::Metric(_) => "REPLAY_METRIC_FAILED",
            Self::Derivation(_) => "REPLAY_DERIVATION_FAILED",
            Self::Report(_) => "REPLAY_REPORT_FAILED",
            Self::Json => "REPLAY_JSON_FAILED",
            Self::Io(_) => "REPLAY_IO_FAILED",
        }
    }
}

impl fmt::Display for ReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.code())?;
        match self {
            Self::Metric(error) => write!(formatter, ": {error}"),
            Self::Derivation(error) => write!(formatter, ": {error}"),
            Self::Report(error) => write!(formatter, ": {error}"),
            Self::Io(error) => write!(formatter, ": {error}"),
            _ => Ok(()),
        }
    }
}

impl Error for ReplayError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Metric(error) => Some(error),
            Self::Derivation(error) => Some(error),
            Self::Report(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<MetricError> for ReplayError {
    fn from(error: MetricError) -> Self {
        Self::Metric(error)
    }
}

impl From<DerivationError> for ReplayError {
    fn from(error: DerivationError) -> Self {
        Self::Derivation(error)
    }
}

impl From<ReportError> for ReplayError {
    fn from(error: ReportError) -> Self {
        Self::Report(error)
    }
}

impl From<io::Error> for ReplayError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Observer-owned deterministic telemetry replay service.
pub struct ObserverReplayAnalyzer<'context> {
    layout: &'context IsolatedRunLayout,
    registry: &'context MetricRegistry,
    declaration: &'context TrackDeclaration,
}

impl<'context> ObserverReplayAnalyzer<'context> {
    #[must_use]
    pub const fn new(
        layout: &'context IsolatedRunLayout,
        registry: &'context MetricRegistry,
        declaration: &'context TrackDeclaration,
    ) -> Self {
        Self {
            layout,
            registry,
            declaration,
        }
    }

    /// Reconstruct canonical event order, derive metrics, and emit sealed observer artifacts.
    ///
    /// # Errors
    ///
    /// Rejects invalid or incomplete telemetry, metric/report failures, and any existing output.
    pub fn analyze(
        &self,
        events: &[EventEnvelope],
        metric_inputs: &BTreeMap<MetricKey, Option<RationalValue>>,
    ) -> Result<ObserverReplayOutput, ReplayError> {
        let canonical = canonical_events(events)?;
        let run_id_hex = hex(&canonical[0].run_id);
        let source_sha256 = source_digest(&canonical);
        let source_blob = BlobId::from_hex(&source_sha256)
            .map_err(|_| ReplayError::Event("REPLAY_SOURCE_HASH_INVALID"))?;
        let metric_table = self.registry.compute(metric_inputs)?;
        let event_rows = canonical
            .iter()
            .map(event_row)
            .collect::<Result<Vec<_>, _>>()?;
        let metric_rows = metric_table
            .rows
            .iter()
            .map(|row| metric_row(&run_id_hex, source_blob, row))
            .collect::<Result<Vec<_>, _>>()?;

        let index_directory = create_output_directory(self.layout.index_root(), &source_sha256)?;
        let spec = DerivationSpec {
            source_hashes: vec![source_blob],
            producer_id: REPLAY_PRODUCER_ID.to_owned(),
            producer_version: REPLAY_PRODUCER_VERSION.to_owned(),
        };
        let event_path = index_directory.join("events.parquet");
        let metric_path = index_directory.join("metrics.parquet");
        let event_summary =
            materialize_derivation(&event_path, &AnalyticalTable::Events(event_rows), &spec)?;
        let metric_summary =
            materialize_derivation(&metric_path, &AnalyticalTable::Metrics(metric_rows), &spec)?;

        let event_artifact = artifact(
            ObserverReplayArtifactKind::EventTable,
            &event_path,
            &run_id_hex,
            &source_sha256,
        )?;
        let metric_artifact = artifact(
            ObserverReplayArtifactKind::MetricTable,
            &metric_path,
            &run_id_hex,
            &source_sha256,
        )?;
        let report = generate_static_report(&ReportData {
            schema: "bonsai.static-report/v1".to_owned(),
            title: format!("Observer replay {run_id_hex}"),
            manifest: json!({"run_id": run_id_hex, "producer": REPLAY_PRODUCER_ID}),
            platform: json!({"analysis_location": "observer"}),
            track: serde_json::to_value(self.declaration).map_err(|_| ReplayError::Json)?,
            resources: json!({"observer_accounting": "required"}),
            overhead: json!({"classification": "observer"}),
            behavior: serde_json::to_value(&metric_table).map_err(|_| ReplayError::Json)?,
            failures: json!([]),
            claims: json!({"status": "not_adjudicated"}),
            limitations: vec![
                "Observer replay is not available to the agent.".to_owned(),
                "Replay does not establish scientific utility.".to_owned(),
            ],
            hashes: BTreeMap::from([
                (
                    "event_table".to_owned(),
                    event_artifact.content_sha256.clone(),
                ),
                (
                    "metric_table".to_owned(),
                    metric_artifact.content_sha256.clone(),
                ),
                ("source".to_owned(), source_sha256.clone()),
            ]),
        })?;
        let report_directory = create_output_directory(self.layout.report_root(), &source_sha256)?;
        let json_path = report_directory.join("report.json");
        let html_path = report_directory.join("report.html");
        write_new(&json_path, report.machine_json.as_bytes())?;
        write_new(&html_path, report.html.as_bytes())?;
        let json_artifact = artifact(
            ObserverReplayArtifactKind::ReportJson,
            &json_path,
            &run_id_hex,
            &source_sha256,
        )?;
        let html_artifact = artifact(
            ObserverReplayArtifactKind::ReportHtml,
            &html_path,
            &run_id_hex,
            &source_sha256,
        )?;

        Ok(ObserverReplayOutput {
            run_id_hex,
            source_sha256,
            event_summary,
            metric_summary,
            artifacts: vec![
                event_artifact,
                metric_artifact,
                json_artifact,
                html_artifact,
            ],
            track_eligibility: derive_track(self.declaration).derived,
        })
    }

    /// Decide whether a sealed replay artifact may enter a named destination.
    #[must_use]
    pub fn authorize_route(
        &self,
        artifact: &ObserverReplayArtifact,
        destination: ReplayDestination,
    ) -> ReplayRouteDecision {
        let observer_match = matches!(
            (artifact.kind.observer_class(), destination),
            (
                ObserverArtifactClass::Index,
                ReplayDestination::ObserverIndex
            ) | (
                ObserverArtifactClass::Report,
                ReplayDestination::ObserverReport
            )
        );
        if observer_match {
            return ReplayRouteDecision {
                allowed: true,
                code: "OBSERVER_REPLAY_ROUTE_ALLOWED",
                derived_track: derive_track(self.declaration).derived,
            };
        }
        let denial = AgentLaunchPolicy::new(self.layout.clone())
            .deny_observer_access(self.declaration, artifact.kind.observer_class());
        ReplayRouteDecision {
            allowed: false,
            code: denial.code,
            derived_track: denial.derived_track,
        }
    }
}

fn canonical_events(events: &[EventEnvelope]) -> Result<Vec<EventEnvelope>, ReplayError> {
    let Some(first) = events.first() else {
        return Err(ReplayError::Empty);
    };
    for event in events {
        validate_event(event).map_err(|error| ReplayError::Event(error.code()))?;
        if event.run_id != first.run_id {
            return Err(ReplayError::RunMismatch);
        }
    }
    let observations = events
        .iter()
        .enumerate()
        .map(|(arrival_index, envelope)| {
            Ok(ObservedEvent {
                envelope: envelope.clone(),
                arrival_index: u64::try_from(arrival_index)
                    .map_err(|_| ReplayError::IncompleteOrder)?,
            })
        })
        .collect::<Result<Vec<_>, ReplayError>>()?;
    let report = classify_partial_order(&observations, OrderingLimits::default())
        .map_err(ReplayError::Ordering)?;
    if !report.duplicate_event_ids.is_empty()
        || !report.missing_parents.is_empty()
        || !report.sequence_conflicts.is_empty()
        || !report.sequence_gaps.is_empty()
        || !report.cycle_event_ids.is_empty()
    {
        return Err(ReplayError::IncompleteOrder);
    }

    let mut by_id = BTreeMap::new();
    for event in events {
        by_id.insert(event_id(event)?, event.clone());
    }
    let mut indegree = by_id
        .keys()
        .copied()
        .map(|event_id| (event_id, 0_usize))
        .collect::<BTreeMap<_, _>>();
    let mut children = BTreeMap::<[u8; 16], BTreeSet<[u8; 16]>>::new();
    for edge in report.edges {
        if children.entry(edge.before).or_default().insert(edge.after) {
            let degree = indegree
                .get_mut(&edge.after)
                .ok_or(ReplayError::IncompleteOrder)?;
            *degree = degree.checked_add(1).ok_or(ReplayError::IncompleteOrder)?;
        }
    }
    let mut ready = indegree
        .iter()
        .filter_map(|(event_id, degree)| (*degree == 0).then_some(*event_id))
        .collect::<BTreeSet<_>>();
    let mut canonical = Vec::with_capacity(events.len());
    while let Some(event_id) = ready.pop_first() {
        canonical.push(
            by_id
                .remove(&event_id)
                .ok_or(ReplayError::IncompleteOrder)?,
        );
        for child in children.get(&event_id).into_iter().flatten() {
            let degree = indegree
                .get_mut(child)
                .ok_or(ReplayError::IncompleteOrder)?;
            *degree = degree.checked_sub(1).ok_or(ReplayError::IncompleteOrder)?;
            if *degree == 0 {
                ready.insert(*child);
            }
        }
    }
    if canonical.len() != events.len() {
        return Err(ReplayError::IncompleteOrder);
    }
    Ok(canonical)
}

fn event_id(event: &EventEnvelope) -> Result<[u8; 16], ReplayError> {
    event
        .event_id
        .as_slice()
        .try_into()
        .map_err(|_| ReplayError::Event("EVENT_ID_INVALID"))
}

fn source_digest(events: &[EventEnvelope]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"bonsai.observer-replay-source/v1\0");
    for event in events {
        let bytes = event.encode_to_vec();
        hasher.update(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_le_bytes());
        hasher.update(bytes);
    }
    hex(&hasher.finalize())
}

fn event_row(event: &EventEnvelope) -> Result<EventRow, ReplayError> {
    let wall_time_unix_ns = event
        .wall_time_unix_ns
        .map(u64::try_from)
        .transpose()
        .map_err(|_| ReplayError::Event("EVENT_TIME_INVALID"))?;
    let payload_sha256 = BlobId::from_hex(&hex(&event.payload_sha256))
        .map_err(|_| ReplayError::Event("EVENT_PAYLOAD_HASH_INVALID"))?;
    Ok(EventRow {
        run_id: hex(&event.run_id),
        source_id: hex(&event.source_id),
        source_sequence: event.source_sequence,
        event_type: event.event_type.clone(),
        monotonic_time_ns: event.monotonic_time_ns,
        wall_time_unix_ns,
        payload_sha256,
    })
}

fn metric_row(
    run_id: &str,
    source: BlobId,
    row: &bonsai_metrics::MetricValue,
) -> Result<MetricRow, ReplayError> {
    Ok(MetricRow {
        run_id: run_id.to_owned(),
        metric_id: row.key.id.clone(),
        metric_version: row.key.version.clone(),
        value: row.value.as_ref().map(rational_to_f64).transpose()?,
        unit: row.unit.clone(),
        availability: match row.availability {
            MetricAvailability::Available => "available",
            MetricAvailability::Unavailable => "unavailable",
        }
        .to_owned(),
        input_sha256: source,
    })
}

fn rational_to_f64(value: &RationalValue) -> Result<f64, ReplayError> {
    let numerator = value
        .numerator
        .to_string()
        .parse::<f64>()
        .map_err(|_| ReplayError::Metric(MetricError::Value))?;
    let denominator = value
        .denominator
        .to_string()
        .parse::<f64>()
        .map_err(|_| ReplayError::Metric(MetricError::Value))?;
    let result = numerator / denominator;
    if result.is_finite() {
        Ok(result)
    } else {
        Err(ReplayError::Metric(MetricError::Value))
    }
}

fn create_output_directory(root: &Path, source_sha256: &str) -> Result<PathBuf, ReplayError> {
    let directory = root.join(format!("replay-{}", &source_sha256[..16]));
    fs::create_dir(&directory)?;
    Ok(directory)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), ReplayError> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.flush()?;
    file.sync_all()?;
    Ok(())
}

fn artifact(
    kind: ObserverReplayArtifactKind,
    path: &Path,
    run_id_hex: &str,
    source_sha256: &str,
) -> Result<ObserverReplayArtifact, ReplayError> {
    let (content_sha256, bytes) = sha256_file(path)?;
    let mut hasher = Sha256::new();
    hasher.update(REPLAY_SEAL_DOMAIN);
    for value in [
        kind.as_str().as_bytes(),
        run_id_hex.as_bytes(),
        source_sha256.as_bytes(),
        content_sha256.as_bytes(),
    ] {
        hasher.update(u64::try_from(value.len()).unwrap_or(u64::MAX).to_le_bytes());
        hasher.update(value);
    }
    Ok(ObserverReplayArtifact {
        kind,
        content_sha256,
        source_sha256: source_sha256.to_owned(),
        seal_sha256: hex(&hasher.finalize()),
        bytes,
    })
}

fn sha256_file(path: &Path) -> Result<(String, u64), ReplayError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut bytes = 0_u64;
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        bytes = bytes
            .checked_add(u64::try_from(read).map_err(|_| ReplayError::IncompleteOrder)?)
            .ok_or(ReplayError::IncompleteOrder)?;
        hasher.update(&buffer[..read]);
    }
    Ok((hex(&hasher.finalize()), bytes))
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{
        ObserverReplayAnalyzer, ReplayDestination, ReplayError, canonical_events, source_digest,
    };
    use crate::IsolatedRunLayout;
    use bonsai_bundle::{DerivationExpectation, TableKind, validate_derivation};
    use bonsai_contracts::bonsai::event::v1::{Availability, EventEnvelope, Precision};
    use bonsai_contracts::track::{Track, TrackDeclaration, TransitionAccess, UpdateSchedule};
    use bonsai_metrics::{
        MetricDirection, MetricFormula, MetricKey, MetricRegistry, MetricSpec, MetricWindow,
        RationalValue,
    };
    use sha2::{Digest, Sha256};
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn key(id: &str) -> MetricKey {
        MetricKey {
            id: id.to_owned(),
            version: "1.0".to_owned(),
        }
    }

    fn registry() -> MetricRegistry {
        let input = key("fixture.reward");
        MetricRegistry::new(vec![
            MetricSpec {
                key: input.clone(),
                formula: MetricFormula::Input,
                unit: "reward".to_owned(),
                window: MetricWindow::Lifetime,
                direction: MetricDirection::HigherIsBetter,
                inputs: Vec::new(),
                availability_rule: "all_inputs_required".to_owned(),
                claim_uses: vec!["diagnostic".to_owned()],
            },
            MetricSpec {
                key: key("fixture.reward_copy"),
                formula: MetricFormula::Sum,
                unit: "reward".to_owned(),
                window: MetricWindow::Lifetime,
                direction: MetricDirection::HigherIsBetter,
                inputs: vec![input],
                availability_rule: "all_inputs_required".to_owned(),
                claim_uses: vec!["diagnostic".to_owned()],
            },
        ])
        .expect("valid registry")
    }

    fn metric_inputs() -> BTreeMap<MetricKey, Option<RationalValue>> {
        BTreeMap::from([(
            key("fixture.reward"),
            Some(RationalValue {
                numerator: 7,
                denominator: 2,
            }),
        )])
    }

    fn strict_track() -> TrackDeclaration {
        TrackDeclaration {
            schema_version: "1.0".to_owned(),
            declared_track: Track::A,
            runtime_facts_complete: true,
            batch_size: 1,
            transition_access: TransitionAccess::SinglePass,
            replay_capacity_transitions: 0,
            offline_updates: false,
            observer_data_access: false,
            privileged_state: false,
            human_labels: false,
            domain_feature_targets: false,
            update_schedule: UpdateSchedule::EventDriven,
            fixed_external_budgets: true,
        }
    }

    fn event(
        id: u8,
        source: u8,
        sequence: u64,
        parents: &[u8],
        monotonic_time_ns: u64,
    ) -> EventEnvelope {
        let payload = format!("payload-{id}").into_bytes();
        EventEnvelope {
            run_id: vec![1; 16],
            source_id: vec![source; 16],
            event_id: vec![id; 16],
            source_sequence: sequence,
            causal_parent_event_ids: parents.iter().map(|parent| vec![*parent; 16]).collect(),
            monotonic_time_ns,
            wall_time_unix_ns: None,
            event_type: "fixture.event/v1".to_owned(),
            payload_schema_epoch: 1,
            payload_schema_minor: 0,
            payload_sha256: Sha256::digest(&payload).to_vec(),
            availability: Availability::Measured as i32,
            precision: Some(Precision {
                representation: "bytes".to_owned(),
                significant_bits: None,
            }),
            payload,
        }
    }

    fn events() -> Vec<EventEnvelope> {
        vec![
            event(1, 10, 0, &[], 10),
            event(2, 10, 1, &[], 20),
            event(3, 20, 0, &[1], 12),
            event(4, 20, 1, &[2, 3], 30),
        ]
    }

    fn setup() -> (TempDir, IsolatedRunLayout, MetricRegistry, TrackDeclaration) {
        let temporary = TempDir::new().expect("temporary directory");
        let layout = IsolatedRunLayout::create(temporary.path().join("run")).expect("layout");
        (temporary, layout, registry(), strict_track())
    }

    #[test]
    fn arrival_permutations_reproduce_identical_tables_and_reports() {
        let (_first_temp, first_layout, first_registry, first_track) = setup();
        let (_second_temp, second_layout, second_registry, second_track) = setup();
        let first = ObserverReplayAnalyzer::new(&first_layout, &first_registry, &first_track)
            .analyze(&events(), &metric_inputs())
            .expect("first replay");
        let mut permuted = events();
        permuted.swap(0, 3);
        permuted.swap(1, 2);
        let second = ObserverReplayAnalyzer::new(&second_layout, &second_registry, &second_track)
            .analyze(&permuted, &metric_inputs())
            .expect("permuted replay");
        assert_eq!(first, second);
        assert_eq!(first.event_summary.row_count, 4);
        assert_eq!(first.metric_summary.row_count, 2);
    }

    #[test]
    fn replay_derivations_validate_against_exact_sources() {
        let (_temporary, layout, registry, track) = setup();
        let fixture = events();
        let output = ObserverReplayAnalyzer::new(&layout, &registry, &track)
            .analyze(&fixture, &metric_inputs())
            .expect("replay");
        let directory = layout
            .index_root()
            .join(format!("replay-{}", &output.source_sha256[..16]));
        for (filename, kind, summary) in [
            ("events.parquet", TableKind::Event, output.event_summary),
            ("metrics.parquet", TableKind::Metric, output.metric_summary),
        ] {
            let validated = validate_derivation(
                directory.join(filename),
                &DerivationExpectation {
                    kind,
                    source_hashes: summary.source_hashes.clone(),
                    producer_id: summary.producer_id.clone(),
                    producer_version: summary.producer_version.clone(),
                },
            )
            .expect("valid derivation");
            assert_eq!(validated, summary);
        }
    }

    #[test]
    fn every_agent_route_is_denied_and_forces_indeterminate_track() {
        let (_temporary, layout, registry, track) = setup();
        let analyzer = ObserverReplayAnalyzer::new(&layout, &registry, &track);
        let output = analyzer
            .analyze(&events(), &metric_inputs())
            .expect("replay");
        assert_eq!(output.track_eligibility, Track::A);
        for artifact in &output.artifacts {
            for destination in [
                ReplayDestination::AgentInput,
                ReplayDestination::AgentProtocolFeedback,
            ] {
                let decision = analyzer.authorize_route(artifact, destination);
                assert!(!decision.allowed);
                assert_eq!(decision.code, "OBSERVER_ACCESS_DENIED");
                assert_eq!(decision.derived_track, Track::Indeterminate);
            }
        }
    }

    #[test]
    fn replay_writes_only_under_observer_roots() {
        let (_temporary, layout, registry, track) = setup();
        let before = files_below(layout.agent_root());
        let output = ObserverReplayAnalyzer::new(&layout, &registry, &track)
            .analyze(&events(), &metric_inputs())
            .expect("replay");
        assert_eq!(before, files_below(layout.agent_root()));
        assert_eq!(files_below(layout.index_root()).len(), 2);
        assert_eq!(files_below(layout.report_root()).len(), 2);
        let exposed = format!("{output:?}");
        assert!(!exposed.contains(&layout.observer_root().display().to_string()));
    }

    #[test]
    fn invalid_or_incomplete_telemetry_fails_closed() {
        let mut duplicate = events();
        duplicate.push(duplicate[0].clone());
        assert!(matches!(
            canonical_events(&duplicate),
            Err(ReplayError::IncompleteOrder)
        ));

        let mut missing_parent = events();
        missing_parent[2].causal_parent_event_ids = vec![vec![99; 16]];
        assert!(matches!(
            canonical_events(&missing_parent),
            Err(ReplayError::IncompleteOrder)
        ));

        let mut gap = events();
        gap[1].source_sequence = 2;
        assert!(matches!(
            canonical_events(&gap),
            Err(ReplayError::IncompleteOrder)
        ));

        let mut cycle = events();
        cycle[0].causal_parent_event_ids = vec![vec![4; 16]];
        assert!(matches!(
            canonical_events(&cycle),
            Err(ReplayError::IncompleteOrder)
        ));
    }

    #[test]
    fn replay_output_is_no_clobber_and_source_bound() {
        let (_temporary, layout, registry, track) = setup();
        let analyzer = ObserverReplayAnalyzer::new(&layout, &registry, &track);
        let fixture = events();
        let output = analyzer
            .analyze(&fixture, &metric_inputs())
            .expect("first replay");
        assert_eq!(
            output.source_sha256,
            source_digest(&canonical_events(&fixture).unwrap())
        );
        assert!(matches!(
            analyzer.analyze(&fixture, &metric_inputs()),
            Err(ReplayError::Io(error)) if error.kind() == std::io::ErrorKind::AlreadyExists
        ));
        assert!(output.artifacts.iter().all(|artifact| {
            artifact.source_sha256 == output.source_sha256
                && artifact.content_sha256.len() == 64
                && artifact.seal_sha256.len() == 64
                && artifact.bytes > 0
        }));
    }

    fn files_below(root: &Path) -> Vec<String> {
        let mut pending = vec![root.to_path_buf()];
        let mut files = Vec::new();
        while let Some(directory) = pending.pop() {
            for entry in fs::read_dir(directory).expect("read directory") {
                let entry = entry.expect("directory entry");
                if entry.file_type().expect("file type").is_dir() {
                    pending.push(entry.path());
                } else {
                    files.push(entry.path().display().to_string());
                }
            }
        }
        files.sort();
        files
    }
}
