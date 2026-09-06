use super::{
    AnalyticalTable, DerivationError, DerivationSpec, DerivedTableSummary, TableKind,
    canonical_source_hashes, hash_batch, metadata_for, table_to_batch, validate_spec,
    write_hash_bytes,
};
use crate::BlobId;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

/// Bounds for one derived file. Rotate files before the row cap; Parquet footer
/// metadata is bounded by the declared file and batch limits.
#[derive(Clone, Copy, Debug)]
pub struct DerivationStreamLimits {
    pub maximum_batch_rows: usize,
    pub maximum_batch_bytes: usize,
    pub maximum_file_rows: u64,
    pub maximum_output_bytes: u64,
}

/// Materialize typed row batches without collecting a completed history.
///
/// The expected row count is normally supplied by a committed input checkpoint.
/// Strings and fixed row fields are checked before Arrow allocation. Each batch
/// is flushed; complete metadata is added only after the exact row count matches.
///
/// # Errors
/// Rejects wrong kinds/counts, oversized batches/files, invalid provenance and
/// output exhaustion. An error leaves an incomplete new file, never replaces data.
pub fn materialize_derivation_stream(
    path: impl AsRef<Path>,
    kind: TableKind,
    expected_rows: u64,
    batches: impl IntoIterator<Item = AnalyticalTable>,
    spec: &DerivationSpec,
    limits: DerivationStreamLimits,
) -> Result<DerivedTableSummary, DerivationError> {
    validate_spec(
        &spec.source_hashes,
        &spec.producer_id,
        &spec.producer_version,
    )?;
    if limits.maximum_batch_rows == 0
        || limits.maximum_batch_bytes == 0
        || limits.maximum_output_bytes == 0
        || expected_rows > limits.maximum_file_rows
        || spec.source_hashes.len() > 1024
    {
        return Err(DerivationError::InputInvalid);
    }
    let source_hashes = canonical_source_hashes(&spec.source_hashes)?;
    let mut hasher = Sha256::new();
    write_hash_bytes(&mut hasher, kind.schema_contract().as_bytes())?;
    hasher.update(expected_rows.to_le_bytes());
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path.as_ref())?;
    let output = BoundedOutput {
        file,
        remaining: limits.maximum_output_bytes,
    };
    let properties = WriterProperties::builder()
        .set_created_by("BONSAI parquet derivation v1".into())
        .build();
    let mut writer = ArrowWriter::try_new(output, kind.schema(), Some(properties))?;
    let mut rows = 0_u64;
    for table in batches {
        let (count, bytes) = table_size(&table);
        if table.kind() != kind
            || count == 0
            || count > limits.maximum_batch_rows
            || bytes > limits.maximum_batch_bytes
        {
            return Err(DerivationError::InputInvalid);
        }
        rows = rows
            .checked_add(u64::try_from(count).map_err(|_| DerivationError::InputInvalid)?)
            .ok_or(DerivationError::InputInvalid)?;
        if rows > expected_rows {
            return Err(DerivationError::InputInvalid);
        }
        let batch = table_to_batch(&table)?;
        hash_batch(&mut hasher, kind, &batch)?;
        writer.write(&batch)?;
        writer.flush()?;
    }
    if rows != expected_rows {
        return Err(DerivationError::InputInvalid);
    }
    let summary = DerivedTableSummary {
        kind,
        row_count: rows,
        schema_sha256: kind.schema_sha256(),
        semantic_sha256: BlobId::from_bytes(hasher.finalize().into()),
        source_hashes,
        producer_id: spec.producer_id.clone(),
        producer_version: spec.producer_version.clone(),
    };
    for metadata in metadata_for(&summary) {
        writer.append_key_value_metadata(metadata);
    }
    writer.close()?;
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?
        .sync_all()?;
    Ok(summary)
}

fn table_size(table: &AnalyticalTable) -> (usize, usize) {
    let (rows, strings): (usize, usize) = match table {
        AnalyticalTable::Events(rows) => (
            rows.len(),
            rows.iter()
                .map(|r| r.run_id.len() + r.source_id.len() + r.event_type.len())
                .sum(),
        ),
        AnalyticalTable::Metrics(rows) => (
            rows.len(),
            rows.iter()
                .map(|r| {
                    r.run_id.len()
                        + r.metric_id.len()
                        + r.metric_version.len()
                        + r.unit.len()
                        + r.availability.len()
                })
                .sum(),
        ),
        AnalyticalTable::Lineage(rows) => (
            rows.len(),
            rows.iter()
                .map(|r| {
                    r.artifact_id.len()
                        + r.artifact_type.len()
                        + r.disposition.len()
                        + r.parent_artifact_id.as_ref().map_or(0, String::len)
                        + r.consumer_artifact_id.as_ref().map_or(0, String::len)
                })
                .sum(),
        ),
        AnalyticalTable::Decisions(rows) => (
            rows.len(),
            rows.iter()
                .map(|r| {
                    r.run_id.len()
                        + r.decision_id.len()
                        + r.policy_version.len()
                        + r.outcome.len()
                        + r.reason_code.len()
                })
                .sum(),
        ),
    };
    (rows, strings.saturating_add(rows.saturating_mul(512)))
}

struct BoundedOutput {
    file: File,
    remaining: u64,
}
impl Write for BoundedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if u64::try_from(bytes.len()).map_or(true, |n| n > self.remaining) {
            return Err(io::Error::new(
                io::ErrorKind::StorageFull,
                "DERIVATION_OUTPUT_QUOTA",
            ));
        }
        let written = self.file.write(bytes)?;
        self.remaining -= u64::try_from(written).map_err(io::Error::other)?;
        Ok(written)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}
