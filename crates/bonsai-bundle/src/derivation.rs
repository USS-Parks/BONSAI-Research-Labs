use crate::BlobId;
use arrow_array::{Array, ArrayRef, Float64Array, RecordBatch, StringArray, UInt64Array};
use arrow_schema::{ArrowError, DataType, Field, Schema, SchemaRef};
use parquet::arrow::ArrowWriter;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::errors::ParquetError;
use parquet::file::metadata::KeyValue;
use parquet::file::properties::WriterProperties;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;
use std::sync::Arc;

const DERIVATION_FORMAT: &str = "bonsai.derivation/v1";
const METADATA_PREFIX: &str = "bonsai.";

/// The four analytical tables defined by the epoch-1 derivation contract.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TableKind {
    Event,
    Metric,
    Lineage,
    Decision,
}

impl TableKind {
    /// Stable identifier stored in Parquet metadata.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Event => "event",
            Self::Metric => "metric",
            Self::Lineage => "lineage",
            Self::Decision => "decision",
        }
    }

    fn parse(value: &str) -> Result<Self, DerivationError> {
        match value {
            "event" => Ok(Self::Event),
            "metric" => Ok(Self::Metric),
            "lineage" => Ok(Self::Lineage),
            "decision" => Ok(Self::Decision),
            _ => Err(DerivationError::MetadataInvalid),
        }
    }

    fn schema_contract(self) -> &'static str {
        match self {
            Self::Event => concat!(
                "bonsai.event-table/v1|",
                "run_id:utf8|required|source_id:utf8|required|",
                "source_sequence:uint64|required|event_type:utf8|required|",
                "monotonic_time_ns:uint64|required|wall_time_unix_ns:uint64|optional|",
                "payload_sha256:utf8|required"
            ),
            Self::Metric => concat!(
                "bonsai.metric-table/v1|",
                "run_id:utf8|required|metric_id:utf8|required|",
                "metric_version:utf8|required|value:float64|optional|",
                "unit:utf8|required|availability:utf8|required|input_sha256:utf8|required"
            ),
            Self::Lineage => concat!(
                "bonsai.lineage-table/v1|",
                "artifact_id:utf8|required|revision:uint64|required|",
                "artifact_type:utf8|required|parent_artifact_id:utf8|optional|",
                "consumer_artifact_id:utf8|optional|disposition:utf8|required"
            ),
            Self::Decision => concat!(
                "bonsai.decision-table/v1|",
                "run_id:utf8|required|decision_id:utf8|required|",
                "policy_version:utf8|required|outcome:utf8|required|",
                "reason_code:utf8|required|observed_state_sha256:utf8|required|",
                "requested_work_sha256:utf8|required"
            ),
        }
    }

    fn schema(self) -> SchemaRef {
        let required_utf8 = |name| Field::new(name, DataType::Utf8, false);
        let optional_utf8 = |name| Field::new(name, DataType::Utf8, true);
        let required_u64 = |name| Field::new(name, DataType::UInt64, false);
        let optional_u64 = |name| Field::new(name, DataType::UInt64, true);
        let fields = match self {
            Self::Event => vec![
                required_utf8("run_id"),
                required_utf8("source_id"),
                required_u64("source_sequence"),
                required_utf8("event_type"),
                required_u64("monotonic_time_ns"),
                optional_u64("wall_time_unix_ns"),
                required_utf8("payload_sha256"),
            ],
            Self::Metric => vec![
                required_utf8("run_id"),
                required_utf8("metric_id"),
                required_utf8("metric_version"),
                Field::new("value", DataType::Float64, true),
                required_utf8("unit"),
                required_utf8("availability"),
                required_utf8("input_sha256"),
            ],
            Self::Lineage => vec![
                required_utf8("artifact_id"),
                required_u64("revision"),
                required_utf8("artifact_type"),
                optional_utf8("parent_artifact_id"),
                optional_utf8("consumer_artifact_id"),
                required_utf8("disposition"),
            ],
            Self::Decision => vec![
                required_utf8("run_id"),
                required_utf8("decision_id"),
                required_utf8("policy_version"),
                required_utf8("outcome"),
                required_utf8("reason_code"),
                required_utf8("observed_state_sha256"),
                required_utf8("requested_work_sha256"),
            ],
        };
        Arc::new(Schema::new(fields))
    }

    fn schema_sha256(self) -> BlobId {
        BlobId::digest(self.schema_contract().as_bytes())
    }
}

/// Canonical analytical event row.
#[derive(Clone, Debug, PartialEq)]
pub struct EventRow {
    pub run_id: String,
    pub source_id: String,
    pub source_sequence: u64,
    pub event_type: String,
    pub monotonic_time_ns: u64,
    pub wall_time_unix_ns: Option<u64>,
    pub payload_sha256: BlobId,
}

/// Canonical analytical metric row.
#[derive(Clone, Debug, PartialEq)]
pub struct MetricRow {
    pub run_id: String,
    pub metric_id: String,
    pub metric_version: String,
    pub value: Option<f64>,
    pub unit: String,
    pub availability: String,
    pub input_sha256: BlobId,
}

/// Canonical analytical lineage row.
#[derive(Clone, Debug, PartialEq)]
pub struct LineageRow {
    pub artifact_id: String,
    pub revision: u64,
    pub artifact_type: String,
    pub parent_artifact_id: Option<String>,
    pub consumer_artifact_id: Option<String>,
    pub disposition: String,
}

/// Canonical analytical resource-governor decision row.
#[derive(Clone, Debug, PartialEq)]
pub struct DecisionRow {
    pub run_id: String,
    pub decision_id: String,
    pub policy_version: String,
    pub outcome: String,
    pub reason_code: String,
    pub observed_state_sha256: BlobId,
    pub requested_work_sha256: BlobId,
}

/// Typed data accepted by the four epoch-1 analytical schemas.
#[derive(Clone, Debug, PartialEq)]
pub enum AnalyticalTable {
    Events(Vec<EventRow>),
    Metrics(Vec<MetricRow>),
    Lineage(Vec<LineageRow>),
    Decisions(Vec<DecisionRow>),
}

impl AnalyticalTable {
    /// Table identity implied by this typed row collection.
    #[must_use]
    pub const fn kind(&self) -> TableKind {
        match self {
            Self::Events(_) => TableKind::Event,
            Self::Metrics(_) => TableKind::Metric,
            Self::Lineage(_) => TableKind::Lineage,
            Self::Decisions(_) => TableKind::Decision,
        }
    }
}

/// Provenance bound into every derived Parquet file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivationSpec {
    pub source_hashes: Vec<BlobId>,
    pub producer_id: String,
    pub producer_version: String,
}

/// Current inputs and producer identity expected by validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivationExpectation {
    pub kind: TableKind,
    pub source_hashes: Vec<BlobId>,
    pub producer_id: String,
    pub producer_version: String,
}

/// Semantic identity and provenance of a validated derived table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivedTableSummary {
    pub kind: TableKind,
    pub row_count: u64,
    pub schema_sha256: BlobId,
    pub semantic_sha256: BlobId,
    pub source_hashes: Vec<BlobId>,
    pub producer_id: String,
    pub producer_version: String,
}

/// Stable derivation-contract failures.
#[derive(Debug)]
#[non_exhaustive]
pub enum DerivationError {
    Io(io::Error),
    Arrow(ArrowError),
    Parquet(ParquetError),
    InputInvalid,
    SourceHashesInvalid,
    MetadataInvalid,
    SchemaMismatch,
    TableKindMismatch,
    InputMismatch,
    Stale,
}

impl DerivationError {
    /// Stable machine-oriented outcome code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Io(_) => "DERIVATION_IO_ERROR",
            Self::Arrow(_) => "DERIVATION_ARROW_ERROR",
            Self::Parquet(_) => "DERIVATION_PARQUET_ERROR",
            Self::InputInvalid => "DERIVATION_INPUT_INVALID",
            Self::SourceHashesInvalid => "DERIVATION_SOURCE_HASHES_INVALID",
            Self::MetadataInvalid => "DERIVATION_METADATA_INVALID",
            Self::SchemaMismatch => "DERIVATION_SCHEMA_MISMATCH",
            Self::TableKindMismatch => "DERIVATION_TABLE_KIND_MISMATCH",
            Self::InputMismatch => "DERIVATION_INPUT_MISMATCH",
            Self::Stale => "DERIVATION_STALE",
        }
    }
}

impl fmt::Display for DerivationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.code())?;
        match self {
            Self::Io(error) => write!(formatter, ": {error}"),
            Self::Arrow(error) => write!(formatter, ": {error}"),
            Self::Parquet(error) => write!(formatter, ": {error}"),
            _ => Ok(()),
        }
    }
}

impl Error for DerivationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Arrow(error) => Some(error),
            Self::Parquet(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for DerivationError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<ArrowError> for DerivationError {
    fn from(error: ArrowError) -> Self {
        Self::Arrow(error)
    }
}

impl From<ParquetError> for DerivationError {
    fn from(error: ParquetError) -> Self {
        Self::Parquet(error)
    }
}

/// Materialize one typed analytical table as a provenance-bound Parquet file.
///
/// # Errors
///
/// Returns an explicit error for invalid rows/provenance, an existing output,
/// or Arrow, Parquet, filesystem, and durability failures. Existing output is
/// never replaced.
pub fn materialize_derivation(
    path: impl AsRef<Path>,
    table: &AnalyticalTable,
    spec: &DerivationSpec,
) -> Result<DerivedTableSummary, DerivationError> {
    validate_spec(
        &spec.source_hashes,
        &spec.producer_id,
        &spec.producer_version,
    )?;
    let source_hashes = canonical_source_hashes(&spec.source_hashes)?;
    let batch = table_to_batch(table)?;
    let kind = table.kind();
    let semantic_sha256 = semantic_sha256(kind, std::slice::from_ref(&batch))?;
    let row_count = u64::try_from(batch.num_rows()).map_err(|_| DerivationError::InputInvalid)?;
    let summary = DerivedTableSummary {
        kind,
        row_count,
        schema_sha256: kind.schema_sha256(),
        semantic_sha256,
        source_hashes,
        producer_id: spec.producer_id.clone(),
        producer_version: spec.producer_version.clone(),
    };
    let properties = WriterProperties::builder()
        .set_created_by("BONSAI parquet derivation v1".to_owned())
        .set_key_value_metadata(Some(metadata_for(&summary)))
        .build();
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path.as_ref())?;
    let mut writer = ArrowWriter::try_new(file, batch.schema(), Some(properties))?;
    writer.write(&batch)?;
    writer.close()?;
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path.as_ref())?
        .sync_all()?;
    Ok(summary)
}

/// Validate a derived file against current source hashes and producer identity.
///
/// # Errors
///
/// Returns distinct stable outcomes for the wrong source set, stale producer or
/// semantic state, table/schema mismatch, malformed metadata, corruption, and
/// filesystem failures.
pub fn validate_derivation(
    path: impl AsRef<Path>,
    expected: &DerivationExpectation,
) -> Result<DerivedTableSummary, DerivationError> {
    validate_spec(
        &expected.source_hashes,
        &expected.producer_id,
        &expected.producer_version,
    )?;
    let file = File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let metadata = parse_metadata(
        builder
            .metadata()
            .file_metadata()
            .key_value_metadata()
            .map_or(&[], Vec::as_slice),
    )?;
    let stored = summary_from_metadata(&metadata)?;
    if stored.kind != expected.kind {
        return Err(DerivationError::TableKindMismatch);
    }
    if stored.source_hashes != canonical_source_hashes(&expected.source_hashes)? {
        return Err(DerivationError::InputMismatch);
    }
    if stored.producer_id != expected.producer_id
        || stored.producer_version != expected.producer_version
    {
        return Err(DerivationError::Stale);
    }
    let expected_schema = expected.kind.schema();
    if !schemas_match(builder.schema().as_ref(), expected_schema.as_ref())
        || stored.schema_sha256 != expected.kind.schema_sha256()
    {
        return Err(DerivationError::SchemaMismatch);
    }
    let batches = builder.build()?.collect::<Result<Vec<_>, _>>()?;
    let observed_rows = batches.iter().try_fold(0_u64, |total, batch| {
        total
            .checked_add(u64::try_from(batch.num_rows()).map_err(|_| DerivationError::Stale)?)
            .ok_or(DerivationError::Stale)
    })?;
    let observed_semantic = semantic_sha256(expected.kind, &batches)?;
    if observed_rows != stored.row_count || observed_semantic != stored.semantic_sha256 {
        return Err(DerivationError::Stale);
    }
    Ok(stored)
}

fn table_to_batch(table: &AnalyticalTable) -> Result<RecordBatch, DerivationError> {
    match table {
        AnalyticalTable::Events(rows) => event_batch(rows),
        AnalyticalTable::Metrics(rows) => metric_batch(rows),
        AnalyticalTable::Lineage(rows) => lineage_batch(rows),
        AnalyticalTable::Decisions(rows) => decision_batch(rows),
    }
}

fn event_batch(rows: &[EventRow]) -> Result<RecordBatch, DerivationError> {
    validate_common_strings(rows.iter().flat_map(|row| {
        [
            row.run_id.as_str(),
            row.source_id.as_str(),
            row.event_type.as_str(),
        ]
    }))?;
    validate_blob_hex(rows.iter().map(|row| row.payload_sha256))?;
    make_batch(
        TableKind::Event,
        vec![
            string_array(rows.iter().map(|row| row.run_id.as_str())),
            string_array(rows.iter().map(|row| row.source_id.as_str())),
            Arc::new(UInt64Array::from_iter_values(
                rows.iter().map(|row| row.source_sequence),
            )),
            string_array(rows.iter().map(|row| row.event_type.as_str())),
            Arc::new(UInt64Array::from_iter_values(
                rows.iter().map(|row| row.monotonic_time_ns),
            )),
            Arc::new(UInt64Array::from(
                rows.iter()
                    .map(|row| row.wall_time_unix_ns)
                    .collect::<Vec<_>>(),
            )),
            string_array(rows.iter().map(|row| row.payload_sha256.to_hex())),
        ],
    )
}

fn metric_batch(rows: &[MetricRow]) -> Result<RecordBatch, DerivationError> {
    validate_common_strings(rows.iter().flat_map(|row| {
        [
            row.run_id.as_str(),
            row.metric_id.as_str(),
            row.metric_version.as_str(),
            row.unit.as_str(),
            row.availability.as_str(),
        ]
    }))?;
    if rows
        .iter()
        .filter_map(|row| row.value)
        .any(|value| !value.is_finite())
    {
        return Err(DerivationError::InputInvalid);
    }
    validate_blob_hex(rows.iter().map(|row| row.input_sha256))?;
    make_batch(
        TableKind::Metric,
        vec![
            string_array(rows.iter().map(|row| row.run_id.as_str())),
            string_array(rows.iter().map(|row| row.metric_id.as_str())),
            string_array(rows.iter().map(|row| row.metric_version.as_str())),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.value).collect::<Vec<_>>(),
            )),
            string_array(rows.iter().map(|row| row.unit.as_str())),
            string_array(rows.iter().map(|row| row.availability.as_str())),
            string_array(rows.iter().map(|row| row.input_sha256.to_hex())),
        ],
    )
}

fn lineage_batch(rows: &[LineageRow]) -> Result<RecordBatch, DerivationError> {
    validate_common_strings(rows.iter().flat_map(|row| {
        [
            row.artifact_id.as_str(),
            row.artifact_type.as_str(),
            row.disposition.as_str(),
        ]
    }))?;
    make_batch(
        TableKind::Lineage,
        vec![
            string_array(rows.iter().map(|row| row.artifact_id.as_str())),
            Arc::new(UInt64Array::from_iter_values(
                rows.iter().map(|row| row.revision),
            )),
            string_array(rows.iter().map(|row| row.artifact_type.as_str())),
            optional_string_array(rows.iter().map(|row| row.parent_artifact_id.as_deref())),
            optional_string_array(rows.iter().map(|row| row.consumer_artifact_id.as_deref())),
            string_array(rows.iter().map(|row| row.disposition.as_str())),
        ],
    )
}

fn decision_batch(rows: &[DecisionRow]) -> Result<RecordBatch, DerivationError> {
    validate_common_strings(rows.iter().flat_map(|row| {
        [
            row.run_id.as_str(),
            row.decision_id.as_str(),
            row.policy_version.as_str(),
            row.outcome.as_str(),
            row.reason_code.as_str(),
        ]
    }))?;
    validate_blob_hex(
        rows.iter()
            .flat_map(|row| [row.observed_state_sha256, row.requested_work_sha256]),
    )?;
    make_batch(
        TableKind::Decision,
        vec![
            string_array(rows.iter().map(|row| row.run_id.as_str())),
            string_array(rows.iter().map(|row| row.decision_id.as_str())),
            string_array(rows.iter().map(|row| row.policy_version.as_str())),
            string_array(rows.iter().map(|row| row.outcome.as_str())),
            string_array(rows.iter().map(|row| row.reason_code.as_str())),
            string_array(rows.iter().map(|row| row.observed_state_sha256.to_hex())),
            string_array(rows.iter().map(|row| row.requested_work_sha256.to_hex())),
        ],
    )
}

fn make_batch(kind: TableKind, columns: Vec<ArrayRef>) -> Result<RecordBatch, DerivationError> {
    Ok(RecordBatch::try_new(kind.schema(), columns)?)
}

fn string_array<I, S>(values: I) -> ArrayRef
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    Arc::new(StringArray::from_iter_values(values))
}

fn optional_string_array<'a>(values: impl IntoIterator<Item = Option<&'a str>>) -> ArrayRef {
    Arc::new(StringArray::from(values.into_iter().collect::<Vec<_>>()))
}

fn validate_common_strings<'a>(
    values: impl IntoIterator<Item = &'a str>,
) -> Result<(), DerivationError> {
    if values.into_iter().any(str::is_empty) {
        return Err(DerivationError::InputInvalid);
    }
    Ok(())
}

fn validate_blob_hex(values: impl IntoIterator<Item = BlobId>) -> Result<(), DerivationError> {
    if values.into_iter().any(|value| value.to_hex().len() != 64) {
        return Err(DerivationError::InputInvalid);
    }
    Ok(())
}

fn validate_spec(
    source_hashes: &[BlobId],
    producer_id: &str,
    producer_version: &str,
) -> Result<(), DerivationError> {
    if source_hashes.is_empty() || producer_id.is_empty() || producer_version.is_empty() {
        return Err(DerivationError::SourceHashesInvalid);
    }
    let _ = canonical_source_hashes(source_hashes)?;
    Ok(())
}

fn canonical_source_hashes(source_hashes: &[BlobId]) -> Result<Vec<BlobId>, DerivationError> {
    if source_hashes.is_empty() {
        return Err(DerivationError::SourceHashesInvalid);
    }
    let canonical = source_hashes.iter().copied().collect::<BTreeSet<_>>();
    if canonical.len() != source_hashes.len() {
        return Err(DerivationError::SourceHashesInvalid);
    }
    Ok(canonical.into_iter().collect())
}

fn metadata_for(summary: &DerivedTableSummary) -> Vec<KeyValue> {
    let source_hashes = summary
        .source_hashes
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");
    [
        ("format", DERIVATION_FORMAT.to_owned()),
        ("table_kind", summary.kind.as_str().to_owned()),
        ("schema_sha256", summary.schema_sha256.to_hex()),
        ("semantic_sha256", summary.semantic_sha256.to_hex()),
        ("source_hashes", source_hashes),
        ("producer_id", summary.producer_id.clone()),
        ("producer_version", summary.producer_version.clone()),
        ("row_count", summary.row_count.to_string()),
    ]
    .into_iter()
    .map(|(key, value)| KeyValue::new(format!("{METADATA_PREFIX}{key}"), Some(value)))
    .collect()
}

fn parse_metadata(values: &[KeyValue]) -> Result<BTreeMap<String, String>, DerivationError> {
    let mut metadata = BTreeMap::new();
    for value in values {
        let Some(key) = value.key.strip_prefix(METADATA_PREFIX) else {
            continue;
        };
        let content = value
            .value
            .clone()
            .ok_or(DerivationError::MetadataInvalid)?;
        if metadata.insert(key.to_owned(), content).is_some() {
            return Err(DerivationError::MetadataInvalid);
        }
    }
    Ok(metadata)
}

fn summary_from_metadata(
    metadata: &BTreeMap<String, String>,
) -> Result<DerivedTableSummary, DerivationError> {
    if required_metadata(metadata, "format")? != DERIVATION_FORMAT {
        return Err(DerivationError::MetadataInvalid);
    }
    let source_text = required_metadata(metadata, "source_hashes")?;
    let source_hashes = source_text
        .split(',')
        .map(BlobId::from_hex)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| DerivationError::MetadataInvalid)?;
    let source_hashes =
        canonical_source_hashes(&source_hashes).map_err(|_| DerivationError::MetadataInvalid)?;
    Ok(DerivedTableSummary {
        kind: TableKind::parse(required_metadata(metadata, "table_kind")?)?,
        row_count: required_metadata(metadata, "row_count")?
            .parse()
            .map_err(|_| DerivationError::MetadataInvalid)?,
        schema_sha256: BlobId::from_hex(required_metadata(metadata, "schema_sha256")?)
            .map_err(|_| DerivationError::MetadataInvalid)?,
        semantic_sha256: BlobId::from_hex(required_metadata(metadata, "semantic_sha256")?)
            .map_err(|_| DerivationError::MetadataInvalid)?,
        source_hashes,
        producer_id: required_metadata(metadata, "producer_id")?.to_owned(),
        producer_version: required_metadata(metadata, "producer_version")?.to_owned(),
    })
}

fn required_metadata<'a>(
    metadata: &'a BTreeMap<String, String>,
    key: &str,
) -> Result<&'a str, DerivationError> {
    metadata
        .get(key)
        .filter(|value| !value.is_empty())
        .map(String::as_str)
        .ok_or(DerivationError::MetadataInvalid)
}

fn semantic_sha256(kind: TableKind, batches: &[RecordBatch]) -> Result<BlobId, DerivationError> {
    let mut hasher = Sha256::new();
    write_hash_bytes(&mut hasher, kind.schema_contract().as_bytes())?;
    let row_count = batches.iter().try_fold(0_u64, |total, batch| {
        total
            .checked_add(
                u64::try_from(batch.num_rows()).map_err(|_| DerivationError::InputInvalid)?,
            )
            .ok_or(DerivationError::InputInvalid)
    })?;
    hasher.update(row_count.to_le_bytes());
    for batch in batches {
        if !schemas_match(batch.schema().as_ref(), kind.schema().as_ref()) {
            return Err(DerivationError::SchemaMismatch);
        }
        for row in 0..batch.num_rows() {
            for column in batch.columns() {
                hash_value(&mut hasher, column, row)?;
            }
        }
    }
    Ok(BlobId::from_bytes(hasher.finalize().into()))
}

fn hash_value(hasher: &mut Sha256, array: &ArrayRef, row: usize) -> Result<(), DerivationError> {
    if array.is_null(row) {
        hasher.update([0]);
        return Ok(());
    }
    hasher.update([1]);
    match array.data_type() {
        DataType::Utf8 => {
            hasher.update([1]);
            let value = array
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or(DerivationError::SchemaMismatch)?
                .value(row);
            write_hash_bytes(hasher, value.as_bytes())?;
        }
        DataType::UInt64 => {
            hasher.update([2]);
            let value = array
                .as_any()
                .downcast_ref::<UInt64Array>()
                .ok_or(DerivationError::SchemaMismatch)?
                .value(row);
            hasher.update(value.to_le_bytes());
        }
        DataType::Float64 => {
            hasher.update([3]);
            let value = array
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or(DerivationError::SchemaMismatch)?
                .value(row);
            if !value.is_finite() {
                return Err(DerivationError::InputInvalid);
            }
            hasher.update(value.to_bits().to_le_bytes());
        }
        _ => return Err(DerivationError::SchemaMismatch),
    }
    Ok(())
}

fn write_hash_bytes(hasher: &mut Sha256, bytes: &[u8]) -> Result<(), DerivationError> {
    let length = u64::try_from(bytes.len()).map_err(|_| DerivationError::InputInvalid)?;
    hasher.update(length.to_le_bytes());
    hasher.update(bytes);
    Ok(())
}

fn schemas_match(observed: &Schema, expected: &Schema) -> bool {
    observed.fields().len() == expected.fields().len()
        && observed
            .fields()
            .iter()
            .zip(expected.fields())
            .all(|(observed, expected)| {
                observed.name() == expected.name()
                    && observed.data_type() == expected.data_type()
                    && observed.is_nullable() == expected.is_nullable()
            })
}
