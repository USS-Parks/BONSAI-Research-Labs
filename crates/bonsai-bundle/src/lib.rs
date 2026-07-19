//! Append-only, checksummed BONSAI event segments.

#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

mod derivation;
mod index;
mod validation;

pub use derivation::{
    AnalyticalTable, DecisionRow, DerivationError, DerivationExpectation, DerivationSpec,
    DerivedTableSummary, EventRow, LineageRow, MetricRow, TableKind, materialize_derivation,
    validate_derivation,
};
pub use index::{
    BlobId, BlobSummary, BundleIndex, BundleIndexError, IndexSummary, IndexedArtifact,
    IndexedSegment, put_blob, put_blob_verified, rebuild_index, validate_blob,
};
pub use validation::{
    AccessMode, BundleSchemas, BundleValidationError, BundleValidationReport, CheckResult,
    CheckStatus, OverallVerdict, migrate_v0_manifest, validate_result_bundle,
};

const HEADER_MAGIC: [u8; 8] = *b"BNSSEG01";
const FRAME_MAGIC: [u8; 8] = *b"BNSFRM01";
const FOOTER_MAGIC: [u8; 8] = *b"BNSEND01";
const FORMAT_EPOCH: u16 = 1;
const HEADER_PREFIX_LEN: usize = 28;
const HEADER_LEN: usize = HEADER_PREFIX_LEN + 32;
const FRAME_PREFIX_LEN: usize = 12;
const FOOTER_PREFIX_LEN: usize = 56;
const FOOTER_LEN: usize = FOOTER_PREFIX_LEN + 32;

/// Hard implementation ceiling for an encoded event frame (16 MiB).
pub const HARD_MAX_FRAME_SIZE: u32 = 16 * 1024 * 1024;

#[derive(Debug)]
#[non_exhaustive]
pub enum SegmentError {
    Io(io::Error),
    HeaderTruncated,
    HeaderMagic,
    HeaderVersion,
    HeaderReserved,
    HeaderChecksum,
    FrameTruncated { index: u64 },
    FrameMarker { index: u64 },
    FrameTooLarge { length: u32, maximum: u32 },
    FrameChecksum { index: u64 },
    FooterTruncated,
    FooterSequence,
    FooterFrameCount,
    FooterChecksum,
    SegmentChecksum,
    TrailingBytes,
    InvalidMaximumFrameSize(u32),
    SequenceDuplicate(u64),
    SequenceExpected { expected: u64, actual: u64 },
    FileNameMismatch { sequence: u64 },
    SegmentAlreadyExists(u64),
    FinalizationConflict(u64),
    WriterFinalized,
    SequenceOverflow,
}

impl SegmentError {
    /// Stable machine-oriented outcome code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Io(_) => "SEGMENT_IO_ERROR",
            Self::HeaderTruncated => "SEGMENT_HEADER_TRUNCATED",
            Self::HeaderMagic => "SEGMENT_HEADER_MAGIC_INVALID",
            Self::HeaderVersion => "SEGMENT_FORMAT_VERSION_UNSUPPORTED",
            Self::HeaderReserved => "SEGMENT_HEADER_RESERVED_NONZERO",
            Self::HeaderChecksum => "SEGMENT_HEADER_CHECKSUM_MISMATCH",
            Self::FrameTruncated { .. } => "SEGMENT_FRAME_TRUNCATED",
            Self::FrameMarker { .. } => "SEGMENT_FRAME_MARKER_INVALID",
            Self::FrameTooLarge { .. } => "SEGMENT_FRAME_TOO_LARGE",
            Self::FrameChecksum { .. } => "SEGMENT_FRAME_CHECKSUM_MISMATCH",
            Self::FooterTruncated => "SEGMENT_FOOTER_TRUNCATED",
            Self::FooterSequence => "SEGMENT_FOOTER_SEQUENCE_MISMATCH",
            Self::FooterFrameCount => "SEGMENT_FOOTER_FRAME_COUNT_MISMATCH",
            Self::FooterChecksum => "SEGMENT_FOOTER_CHECKSUM_MISMATCH",
            Self::SegmentChecksum => "SEGMENT_CHECKSUM_MISMATCH",
            Self::TrailingBytes => "SEGMENT_TRAILING_BYTES",
            Self::InvalidMaximumFrameSize(_) => "SEGMENT_MAX_FRAME_SIZE_INVALID",
            Self::SequenceDuplicate(_) => "BUNDLE_SEGMENT_SEQUENCE_DUPLICATE",
            Self::SequenceExpected { .. } => "BUNDLE_SEGMENT_SEQUENCE_NON_MONOTONIC",
            Self::FileNameMismatch { .. } => "BUNDLE_SEGMENT_FILE_NAME_MISMATCH",
            Self::SegmentAlreadyExists(_) => "BUNDLE_SEGMENT_ALREADY_EXISTS",
            Self::FinalizationConflict(_) => "BUNDLE_SEGMENT_FINALIZATION_CONFLICT",
            Self::WriterFinalized => "SEGMENT_WRITER_FINALIZED",
            Self::SequenceOverflow => "BUNDLE_SEGMENT_SEQUENCE_OVERFLOW",
        }
    }
}

impl fmt::Display for SegmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.code())?;
        match self {
            Self::Io(error) => write!(formatter, ": {error}"),
            Self::FrameTruncated { index }
            | Self::FrameMarker { index }
            | Self::FrameChecksum { index } => write!(formatter, ": frame={index}"),
            Self::FrameTooLarge { length, maximum } => {
                write!(formatter, ": length={length}, maximum={maximum}")
            }
            Self::InvalidMaximumFrameSize(value) => write!(formatter, ": value={value}"),
            Self::SequenceDuplicate(sequence)
            | Self::SegmentAlreadyExists(sequence)
            | Self::FinalizationConflict(sequence)
            | Self::FileNameMismatch { sequence } => write!(formatter, ": sequence={sequence}"),
            Self::SequenceExpected { expected, actual } => {
                write!(formatter, ": expected={expected}, actual={actual}")
            }
            _ => Ok(()),
        }
    }
}

impl Error for SegmentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for SegmentError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SegmentSummary {
    pub sequence: u64,
    pub frame_count: u64,
    pub maximum_frame_size: u32,
    pub checksum: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryOutcome {
    Recovered(SegmentSummary),
    AlreadyFinalized(SegmentSummary),
}

#[derive(Debug)]
struct Header {
    sequence: u64,
    maximum_frame_size: u32,
    bytes: [u8; HEADER_LEN],
}

#[derive(Debug)]
struct OpenInspection {
    header: Header,
    frame_count: u64,
    content_hasher: Sha256,
    complete_summary: Option<SegmentSummary>,
}

/// Writer for one immutable event segment.
pub struct SegmentWriter {
    directory: PathBuf,
    open_path: PathBuf,
    final_path: PathBuf,
    writer: Option<BufWriter<File>>,
    sequence: u64,
    maximum_frame_size: u32,
    frame_count: u64,
    content_hasher: Sha256,
}

impl SegmentWriter {
    /// Create the next segment in a bundle directory.
    ///
    /// Sequence zero starts an empty bundle; every subsequent segment must be
    /// exactly the prior finalized sequence plus one.
    ///
    /// # Errors
    ///
    /// Returns an explicit error for an invalid bound, a non-monotonic
    /// sequence, an existing staging/final path, or an I/O failure.
    pub fn create(
        directory: impl AsRef<Path>,
        sequence: u64,
        maximum_frame_size: u32,
    ) -> Result<Self, SegmentError> {
        validate_maximum(maximum_frame_size)?;
        let directory = directory.as_ref().to_path_buf();
        fs::create_dir_all(&directory)?;
        let summaries = validate_bundle(&directory)?;
        let expected = match summaries.last() {
            Some(summary) => summary
                .sequence
                .checked_add(1)
                .ok_or(SegmentError::SequenceOverflow)?,
            None => 0,
        };
        if sequence != expected {
            return Err(SegmentError::SequenceExpected {
                expected,
                actual: sequence,
            });
        }

        let open_path = directory.join(open_file_name(sequence));
        let final_path = directory.join(final_file_name(sequence));
        if open_path.exists() || final_path.exists() {
            return Err(SegmentError::SegmentAlreadyExists(sequence));
        }
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&open_path)?;
        let header = encode_header(sequence, maximum_frame_size);
        let mut writer = BufWriter::new(file);
        writer.write_all(&header)?;
        let mut content_hasher = Sha256::new();
        content_hasher.update(header);

        Ok(Self {
            directory,
            open_path,
            final_path,
            writer: Some(writer),
            sequence,
            maximum_frame_size,
            frame_count: 0,
            content_hasher,
        })
    }

    /// Append one opaque encoded event frame.
    ///
    /// # Errors
    ///
    /// Rejects a frame larger than the segment's declared bound before any of
    /// its bytes are written, and reports I/O or finalized-writer failures.
    pub fn append(&mut self, frame: &[u8]) -> Result<(), SegmentError> {
        let length = u32::try_from(frame.len()).unwrap_or(u32::MAX);
        if length > self.maximum_frame_size {
            return Err(SegmentError::FrameTooLarge {
                length,
                maximum: self.maximum_frame_size,
            });
        }
        let writer = self.writer.as_mut().ok_or(SegmentError::WriterFinalized)?;
        let mut prefix = [0_u8; FRAME_PREFIX_LEN];
        prefix[..8].copy_from_slice(&FRAME_MAGIC);
        prefix[8..].copy_from_slice(&length.to_le_bytes());
        let checksum: [u8; 32] = Sha256::digest(frame).into();
        writer.write_all(&prefix)?;
        writer.write_all(frame)?;
        writer.write_all(&checksum)?;
        self.content_hasher.update(prefix);
        self.content_hasher.update(frame);
        self.content_hasher.update(checksum);
        self.frame_count = self
            .frame_count
            .checked_add(1)
            .ok_or(SegmentError::SequenceOverflow)?;
        Ok(())
    }

    /// Flush and synchronize all complete pending frames without finalizing the segment.
    ///
    /// This establishes a recoverable crash boundary: recovery may append the
    /// footer to a separate file, while the synchronized `.open` source remains
    /// untouched until no-clobber publication succeeds.
    ///
    /// # Errors
    ///
    /// Returns an I/O or already-finalized writer failure.
    pub fn sync_pending(&mut self) -> Result<(), SegmentError> {
        let writer = self.writer.as_mut().ok_or(SegmentError::WriterFinalized)?;
        writer.flush()?;
        writer.get_ref().sync_all()?;
        Ok(())
    }

    /// Durably close and atomically publish the immutable segment.
    ///
    /// The final path is created with a no-clobber hard link. The staging name
    /// is removed only after the published file has been synchronized.
    ///
    /// # Errors
    ///
    /// Returns an explicit conflict if the final path already exists, or an
    /// I/O/finalized-writer error.
    pub fn finalize(mut self) -> Result<SegmentSummary, SegmentError> {
        let checksum: [u8; 32] = self.content_hasher.clone().finalize().into();
        let footer = encode_footer(self.sequence, self.frame_count, checksum);
        let mut writer = self.writer.take().ok_or(SegmentError::WriterFinalized)?;
        writer.write_all(&footer)?;
        writer.flush()?;
        writer.get_ref().sync_all()?;
        drop(writer);
        publish_no_clobber(&self.open_path, &self.final_path, self.sequence)?;
        sync_file(&self.final_path)?;
        remove_if_exists(&self.open_path)?;
        sync_directory_best_effort(&self.directory)?;
        Ok(SegmentSummary {
            sequence: self.sequence,
            frame_count: self.frame_count,
            maximum_frame_size: self.maximum_frame_size,
            checksum,
        })
    }
}

/// Validate one finalized segment without decoding its event payloads.
///
/// # Errors
///
/// Returns stable, corruption-specific outcomes for malformed headers,
/// truncated or oversized frames, checksum mismatches, malformed footers, and
/// trailing bytes.
pub fn validate_segment(path: impl AsRef<Path>) -> Result<SegmentSummary, SegmentError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let inspection = inspect_stream(&mut reader, true)?;
    inspection
        .complete_summary
        .ok_or(SegmentError::FooterTruncated)
}

/// Validate every finalized segment and enforce canonical contiguous sequence.
///
/// # Errors
///
/// Rejects duplicate, missing, out-of-order, or non-canonically named segment
/// sequences in addition to all per-segment validation errors.
pub fn validate_bundle(directory: impl AsRef<Path>) -> Result<Vec<SegmentSummary>, SegmentError> {
    let directory = directory.as_ref();
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut by_sequence: BTreeMap<u64, (PathBuf, SegmentSummary)> = BTreeMap::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "bseg")
        {
            let summary = validate_segment(&path)?;
            if by_sequence
                .insert(summary.sequence, (path, summary.clone()))
                .is_some()
            {
                return Err(SegmentError::SequenceDuplicate(summary.sequence));
            }
        }
    }
    let mut summaries = Vec::with_capacity(by_sequence.len());
    for (expected, (sequence, (path, summary))) in by_sequence.into_iter().enumerate() {
        let expected = u64::try_from(expected).map_err(|_| SegmentError::SequenceOverflow)?;
        if sequence != expected {
            return Err(SegmentError::SequenceExpected {
                expected,
                actual: sequence,
            });
        }
        if path.file_name().and_then(|name| name.to_str()) != Some(&final_file_name(sequence)) {
            return Err(SegmentError::FileNameMismatch { sequence });
        }
        summaries.push(summary);
    }
    Ok(summaries)
}

/// Recover a canonical `.open` segment without modifying its contents.
///
/// A complete staged segment is published directly. A crash before footer
/// write is recovered by copying complete frames to a new recovery file and
/// appending a footer there. Partial frames and corruption are left untouched
/// and returned as explicit errors.
///
/// # Errors
///
/// Returns the same deterministic validation errors as [`validate_segment`],
/// or a finalization conflict if a different final segment already exists.
pub fn recover_open_segment(path: impl AsRef<Path>) -> Result<RecoveryOutcome, SegmentError> {
    let open_path = path.as_ref();
    let directory = open_path.parent().unwrap_or_else(|| Path::new("."));
    let mut reader = BufReader::new(File::open(open_path)?);
    let inspection = inspect_stream(&mut reader, false)?;
    let sequence = inspection.header.sequence;
    let canonical_open = directory.join(open_file_name(sequence));
    if open_path != canonical_open {
        return Err(SegmentError::FileNameMismatch { sequence });
    }
    let final_path = directory.join(final_file_name(sequence));
    if final_path.exists() {
        let final_summary = validate_segment(&final_path)?;
        let staged_summary = inspection.complete_summary.as_ref();
        if staged_summary == Some(&final_summary) {
            fs::remove_file(open_path)?;
            sync_directory_best_effort(directory)?;
            return Ok(RecoveryOutcome::AlreadyFinalized(final_summary));
        }
        return Err(SegmentError::FinalizationConflict(sequence));
    }

    let summary = if let Some(summary) = inspection.complete_summary {
        publish_no_clobber(open_path, &final_path, sequence)?;
        sync_file(&final_path)?;
        remove_if_exists(open_path)?;
        summary
    } else {
        let recovery_path = directory.join(recovery_file_name(sequence));
        let mut source = File::open(open_path)?;
        let recovery_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&recovery_path)?;
        let mut recovery_writer = BufWriter::new(recovery_file);
        io::copy(&mut source, &mut recovery_writer)?;
        let checksum: [u8; 32] = inspection.content_hasher.finalize().into();
        let footer = encode_footer(sequence, inspection.frame_count, checksum);
        recovery_writer.write_all(&footer)?;
        recovery_writer.flush()?;
        recovery_writer.get_ref().sync_all()?;
        drop(recovery_writer);
        publish_no_clobber(&recovery_path, &final_path, sequence)?;
        sync_file(&final_path)?;
        remove_if_exists(&recovery_path)?;
        fs::remove_file(open_path)?;
        SegmentSummary {
            sequence,
            frame_count: inspection.frame_count,
            maximum_frame_size: inspection.header.maximum_frame_size,
            checksum,
        }
    };
    sync_directory_best_effort(directory)?;
    Ok(RecoveryOutcome::Recovered(summary))
}

fn inspect_stream<R: Read>(
    reader: &mut R,
    require_footer: bool,
) -> Result<OpenInspection, SegmentError> {
    let header = read_header(reader)?;
    let mut content_hasher = Sha256::new();
    content_hasher.update(header.bytes);
    let mut frame_count = 0_u64;
    loop {
        let mut marker = [0_u8; 8];
        let marker_read = read_until_eof(reader, &mut marker)?;
        if marker_read == 0 {
            if require_footer {
                return Err(SegmentError::FooterTruncated);
            }
            return Ok(OpenInspection {
                header,
                frame_count,
                content_hasher,
                complete_summary: None,
            });
        }
        if marker_read < marker.len() {
            return Err(SegmentError::FrameTruncated { index: frame_count });
        }
        if marker == FOOTER_MAGIC {
            let summary = read_footer(reader, &header, frame_count, &content_hasher)?;
            let mut trailing = [0_u8; 1];
            if reader.read(&mut trailing)? != 0 {
                return Err(SegmentError::TrailingBytes);
            }
            return Ok(OpenInspection {
                header,
                frame_count,
                content_hasher,
                complete_summary: Some(summary),
            });
        }
        if marker != FRAME_MAGIC {
            return Err(SegmentError::FrameMarker { index: frame_count });
        }
        let mut length_bytes = [0_u8; 4];
        read_exact_or(
            reader,
            &mut length_bytes,
            SegmentError::FrameTruncated { index: frame_count },
        )?;
        let length = u32::from_le_bytes(length_bytes);
        if length > header.maximum_frame_size || length > HARD_MAX_FRAME_SIZE {
            return Err(SegmentError::FrameTooLarge {
                length,
                maximum: header.maximum_frame_size,
            });
        }
        let mut frame = vec![0_u8; length as usize];
        read_exact_or(
            reader,
            &mut frame,
            SegmentError::FrameTruncated { index: frame_count },
        )?;
        let mut checksum = [0_u8; 32];
        read_exact_or(
            reader,
            &mut checksum,
            SegmentError::FrameTruncated { index: frame_count },
        )?;
        if checksum != <[u8; 32]>::from(Sha256::digest(&frame)) {
            return Err(SegmentError::FrameChecksum { index: frame_count });
        }
        content_hasher.update(marker);
        content_hasher.update(length_bytes);
        content_hasher.update(frame);
        content_hasher.update(checksum);
        frame_count = frame_count
            .checked_add(1)
            .ok_or(SegmentError::SequenceOverflow)?;
    }
}

fn read_header<R: Read>(reader: &mut R) -> Result<Header, SegmentError> {
    let mut bytes = [0_u8; HEADER_LEN];
    read_exact_or(reader, &mut bytes, SegmentError::HeaderTruncated)?;
    if bytes[..8] != HEADER_MAGIC {
        return Err(SegmentError::HeaderMagic);
    }
    if u16::from_le_bytes(bytes[8..10].try_into().expect("fixed header slice")) != FORMAT_EPOCH {
        return Err(SegmentError::HeaderVersion);
    }
    if bytes[10..12] != [0, 0] || bytes[24..28] != [0, 0, 0, 0] {
        return Err(SegmentError::HeaderReserved);
    }
    if &bytes[HEADER_PREFIX_LEN..] != Sha256::digest(&bytes[..HEADER_PREFIX_LEN]).as_slice() {
        return Err(SegmentError::HeaderChecksum);
    }
    let sequence = u64::from_le_bytes(bytes[12..20].try_into().expect("fixed header slice"));
    let maximum_frame_size =
        u32::from_le_bytes(bytes[20..24].try_into().expect("fixed header slice"));
    validate_maximum(maximum_frame_size)?;
    Ok(Header {
        sequence,
        maximum_frame_size,
        bytes,
    })
}

fn read_footer<R: Read>(
    reader: &mut R,
    header: &Header,
    frame_count: u64,
    content_hasher: &Sha256,
) -> Result<SegmentSummary, SegmentError> {
    let mut remainder = [0_u8; FOOTER_LEN - 8];
    read_exact_or(reader, &mut remainder, SegmentError::FooterTruncated)?;
    let sequence = u64::from_le_bytes(remainder[..8].try_into().expect("fixed footer slice"));
    if sequence != header.sequence {
        return Err(SegmentError::FooterSequence);
    }
    let declared_count =
        u64::from_le_bytes(remainder[8..16].try_into().expect("fixed footer slice"));
    if declared_count != frame_count {
        return Err(SegmentError::FooterFrameCount);
    }
    let checksum: [u8; 32] = remainder[16..48].try_into().expect("fixed footer slice");
    if checksum != <[u8; 32]>::from(content_hasher.clone().finalize()) {
        return Err(SegmentError::SegmentChecksum);
    }
    let mut footer_prefix = [0_u8; FOOTER_PREFIX_LEN];
    footer_prefix[..8].copy_from_slice(&FOOTER_MAGIC);
    footer_prefix[8..].copy_from_slice(&remainder[..48]);
    if &remainder[48..] != Sha256::digest(footer_prefix).as_slice() {
        return Err(SegmentError::FooterChecksum);
    }
    Ok(SegmentSummary {
        sequence,
        frame_count,
        maximum_frame_size: header.maximum_frame_size,
        checksum,
    })
}

fn encode_header(sequence: u64, maximum_frame_size: u32) -> [u8; HEADER_LEN] {
    let mut bytes = [0_u8; HEADER_LEN];
    bytes[..8].copy_from_slice(&HEADER_MAGIC);
    bytes[8..10].copy_from_slice(&FORMAT_EPOCH.to_le_bytes());
    bytes[12..20].copy_from_slice(&sequence.to_le_bytes());
    bytes[20..24].copy_from_slice(&maximum_frame_size.to_le_bytes());
    let checksum = Sha256::digest(&bytes[..HEADER_PREFIX_LEN]);
    bytes[HEADER_PREFIX_LEN..].copy_from_slice(&checksum);
    bytes
}

fn encode_footer(sequence: u64, frame_count: u64, checksum: [u8; 32]) -> [u8; FOOTER_LEN] {
    let mut bytes = [0_u8; FOOTER_LEN];
    bytes[..8].copy_from_slice(&FOOTER_MAGIC);
    bytes[8..16].copy_from_slice(&sequence.to_le_bytes());
    bytes[16..24].copy_from_slice(&frame_count.to_le_bytes());
    bytes[24..56].copy_from_slice(&checksum);
    let footer_checksum = Sha256::digest(&bytes[..FOOTER_PREFIX_LEN]);
    bytes[FOOTER_PREFIX_LEN..].copy_from_slice(&footer_checksum);
    bytes
}

fn validate_maximum(maximum: u32) -> Result<(), SegmentError> {
    if maximum == 0 || maximum > HARD_MAX_FRAME_SIZE {
        return Err(SegmentError::InvalidMaximumFrameSize(maximum));
    }
    Ok(())
}

fn read_exact_or<R: Read>(
    reader: &mut R,
    buffer: &mut [u8],
    error: SegmentError,
) -> Result<(), SegmentError> {
    reader.read_exact(buffer).map_err(|io_error| {
        if io_error.kind() == io::ErrorKind::UnexpectedEof {
            error
        } else {
            SegmentError::Io(io_error)
        }
    })
}

fn read_until_eof<R: Read>(reader: &mut R, buffer: &mut [u8]) -> Result<usize, SegmentError> {
    let mut offset = 0;
    while offset < buffer.len() {
        match reader.read(&mut buffer[offset..]) {
            Ok(0) => break,
            Ok(read) => offset += read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(SegmentError::Io(error)),
        }
    }
    Ok(offset)
}

fn publish_no_clobber(
    source: &Path,
    destination: &Path,
    sequence: u64,
) -> Result<(), SegmentError> {
    match fs::hard_link(source, destination) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            Err(SegmentError::FinalizationConflict(sequence))
        }
        #[cfg(windows)]
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::PermissionDenied | io::ErrorKind::Unsupported
            ) =>
        {
            // Windows rename is same-volume atomic and refuses to replace an
            // existing destination. This is the sandbox-compatible fallback
            // when hard-link creation is denied by local policy.
            fs::rename(source, destination).map_err(|rename_error| {
                if rename_error.kind() == io::ErrorKind::AlreadyExists || destination.exists() {
                    SegmentError::FinalizationConflict(sequence)
                } else {
                    SegmentError::Io(rename_error)
                }
            })
        }
        Err(error) => Err(SegmentError::Io(error)),
    }
}

fn remove_if_exists(path: &Path) -> Result<(), SegmentError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(SegmentError::Io(error)),
    }
}

fn sync_file(path: &Path) -> Result<(), SegmentError> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?
        .sync_all()?;
    Ok(())
}

fn sync_directory_best_effort(directory: &Path) -> Result<(), SegmentError> {
    match File::open(directory).and_then(|file| file.sync_all()) {
        Ok(()) => Ok(()),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::InvalidInput
                    | io::ErrorKind::PermissionDenied
                    | io::ErrorKind::Unsupported
            ) =>
        {
            Ok(())
        }
        Err(error) => Err(SegmentError::Io(error)),
    }
}

fn final_file_name(sequence: u64) -> String {
    format!("segment-{sequence:020}.bseg")
}

fn open_file_name(sequence: u64) -> String {
    format!("segment-{sequence:020}.open")
}

fn recovery_file_name(sequence: u64) -> String {
    format!("segment-{sequence:020}.recovering")
}
