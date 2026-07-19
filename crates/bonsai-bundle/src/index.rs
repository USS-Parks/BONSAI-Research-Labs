use crate::{SegmentError, validate_bundle};
use rusqlite::{Connection, OpenFlags, params};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const INDEX_FILE_NAME: &str = "bundle-index.sqlite3";
const INDEX_APPLICATION_ID: i64 = 1_112_429_385;
const INDEX_SCHEMA_VERSION: i64 = 1;
const INDEX_FORMAT: &str = "bonsai.bundle-index/v1";
const MIGRATION_0001: &str = include_str!("../migrations/0001_bundle_index.sql");
const BLOB_SUFFIX: &str = ".blob";
static TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Stable SHA-256 content identity used by the bundle blob store.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct BlobId([u8; 32]);

impl BlobId {
    pub(crate) const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Parse the canonical lowercase 64-character hexadecimal representation.
    ///
    /// # Errors
    ///
    /// Returns `BLOB_ID_INVALID` for uppercase, non-hexadecimal, short, long,
    /// separator-bearing, or traversal-bearing input.
    pub fn from_hex(value: &str) -> Result<Self, BundleIndexError> {
        if value.len() != 64
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(BundleIndexError::InvalidBlobId);
        }
        let mut bytes = [0_u8; 32];
        for (destination, pair) in bytes.iter_mut().zip(value.as_bytes().chunks_exact(2)) {
            *destination = (hex_nibble(pair[0]) << 4) | hex_nibble(pair[1]);
        }
        Ok(Self(bytes))
    }

    /// Compute the identity of bytes without storing them.
    #[must_use]
    pub fn digest(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }

    /// Return the canonical lowercase hexadecimal representation.
    #[must_use]
    pub fn to_hex(self) -> String {
        encode_hex(&self.0)
    }
}

impl fmt::Display for BlobId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&encode_hex(&self.0))
    }
}

/// Validated immutable blob metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlobSummary {
    pub id: BlobId,
    pub byte_length: u64,
    pub relative_path: String,
}

/// One immutable event segment represented in the disposable index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedSegment {
    pub sequence: u64,
    pub relative_path: String,
    pub frame_count: u64,
    pub maximum_frame_size: u32,
    pub sha256: BlobId,
    pub byte_length: u64,
}

/// One content-addressed derived artifact represented in the disposable index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedArtifact {
    pub id: BlobId,
    pub relative_path: String,
    pub byte_length: u64,
}

/// Counts produced by a successful full index rebuild.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IndexSummary {
    pub segment_count: usize,
    pub artifact_count: usize,
}

/// Stable bundle-index and blob-store failure vocabulary.
#[derive(Debug)]
#[non_exhaustive]
pub enum BundleIndexError {
    Io(io::Error),
    Sqlite(rusqlite::Error),
    Segment(SegmentError),
    InvalidBlobId,
    BlobHashMismatch { expected: BlobId, actual: BlobId },
    BlobHashCollision(BlobId),
    PathTraversal,
    SymlinkRejected,
    UnexpectedBlobPath,
    IndexMissing,
    IndexApplicationId(i64),
    IndexSchemaVersion(i64),
    IndexFormat,
    IndexPathInvalid,
    IndexValueInvalid,
    IndexNotReadOnly,
}

impl BundleIndexError {
    /// Stable machine-oriented outcome code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Io(_) => "BUNDLE_INDEX_IO_ERROR",
            Self::Sqlite(_) => "BUNDLE_INDEX_SQLITE_ERROR",
            Self::Segment(error) => error.code(),
            Self::InvalidBlobId => "BLOB_ID_INVALID",
            Self::BlobHashMismatch { .. } => "BLOB_HASH_MISMATCH",
            Self::BlobHashCollision(_) => "BLOB_HASH_COLLISION",
            Self::PathTraversal => "BUNDLE_PATH_TRAVERSAL_REJECTED",
            Self::SymlinkRejected => "BUNDLE_SYMLINK_REJECTED",
            Self::UnexpectedBlobPath => "BLOB_PATH_NONCANONICAL",
            Self::IndexMissing => "BUNDLE_INDEX_MISSING",
            Self::IndexApplicationId(_) => "BUNDLE_INDEX_APPLICATION_ID_INVALID",
            Self::IndexSchemaVersion(_) => "BUNDLE_INDEX_SCHEMA_UNSUPPORTED",
            Self::IndexFormat => "BUNDLE_INDEX_FORMAT_INVALID",
            Self::IndexPathInvalid => "BUNDLE_INDEX_PATH_INVALID",
            Self::IndexValueInvalid => "BUNDLE_INDEX_VALUE_INVALID",
            Self::IndexNotReadOnly => "BUNDLE_INDEX_NOT_READ_ONLY",
        }
    }
}

impl fmt::Display for BundleIndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.code())?;
        match self {
            Self::Io(error) => write!(formatter, ": {error}"),
            Self::Sqlite(error) => write!(formatter, ": {error}"),
            Self::Segment(error) => write!(formatter, ": {error}"),
            Self::BlobHashMismatch { expected, actual } => {
                write!(formatter, ": expected={expected}, actual={actual}")
            }
            Self::BlobHashCollision(id) => write!(formatter, ": sha256={id}"),
            Self::IndexApplicationId(value) | Self::IndexSchemaVersion(value) => {
                write!(formatter, ": value={value}")
            }
            _ => Ok(()),
        }
    }
}

impl Error for BundleIndexError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Segment(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for BundleIndexError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<rusqlite::Error> for BundleIndexError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

impl From<SegmentError> for BundleIndexError {
    fn from(error: SegmentError) -> Self {
        Self::Segment(error)
    }
}

/// Read-only handle to a validated portable bundle index.
pub struct BundleIndex {
    connection: Connection,
}

impl BundleIndex {
    /// Open and structurally validate the derived index without write access.
    ///
    /// # Errors
    ///
    /// Returns an explicit error for a missing/non-regular/symlinked database,
    /// unsupported schema identity, noncanonical stored path, or `SQLite` error.
    pub fn open_read_only(bundle_directory: impl AsRef<Path>) -> Result<Self, BundleIndexError> {
        let root = checked_root(bundle_directory.as_ref())?;
        let path = root.join(INDEX_FILE_NAME);
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                BundleIndexError::IndexMissing
            } else {
                BundleIndexError::Io(error)
            }
        })?;
        if metadata.file_type().is_symlink() {
            return Err(BundleIndexError::SymlinkRejected);
        }
        if !metadata.is_file() {
            return Err(BundleIndexError::IndexMissing);
        }
        let connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        connection.pragma_update(None, "query_only", true)?;
        validate_index_identity(&connection)?;
        validate_index_paths(&connection)?;
        let index = Self { connection };
        if !index.is_read_only()? {
            return Err(BundleIndexError::IndexNotReadOnly);
        }
        Ok(index)
    }

    /// Report whether `SQLite` has query-only mode enabled for this handle.
    ///
    /// # Errors
    ///
    /// Returns an error if `SQLite` cannot read the connection pragma.
    pub fn is_read_only(&self) -> Result<bool, BundleIndexError> {
        let value: i64 = self
            .connection
            .pragma_query_value(None, "query_only", |row| row.get(0))?;
        Ok(value == 1)
    }

    /// Load event segment metadata in canonical sequence order.
    ///
    /// # Errors
    ///
    /// Returns an explicit error for malformed stored values or `SQLite` errors.
    pub fn segments(&self) -> Result<Vec<IndexedSegment>, BundleIndexError> {
        let mut statement = self.connection.prepare(
            "SELECT sequence_decimal, relative_path, frame_count_decimal, \
             maximum_frame_size, sha256, byte_length_decimal \
             FROM event_segments ORDER BY length(sequence_decimal), sequence_decimal",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;
        rows.map(|row| {
            let (sequence, path, frame_count, maximum, sha256, byte_length) = row?;
            Ok(IndexedSegment {
                sequence: parse_u64(&sequence)?,
                relative_path: path,
                frame_count: parse_u64(&frame_count)?,
                maximum_frame_size: u32::try_from(maximum)
                    .map_err(|_| BundleIndexError::IndexValueInvalid)?,
                sha256: BlobId::from_hex(&sha256)
                    .map_err(|_| BundleIndexError::IndexValueInvalid)?,
                byte_length: parse_u64(&byte_length)?,
            })
        })
        .collect()
    }

    /// Load content-addressed derived artifact metadata in digest order.
    ///
    /// # Errors
    ///
    /// Returns an explicit error for malformed stored values or `SQLite` errors.
    pub fn artifacts(&self) -> Result<Vec<IndexedArtifact>, BundleIndexError> {
        let mut statement = self.connection.prepare(
            "SELECT sha256, relative_path, byte_length_decimal \
             FROM derived_artifacts ORDER BY sha256",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        rows.map(|row| {
            let (sha256, path, byte_length) = row?;
            Ok(IndexedArtifact {
                id: BlobId::from_hex(&sha256).map_err(|_| BundleIndexError::IndexValueInvalid)?,
                relative_path: path,
                byte_length: parse_u64(&byte_length)?,
            })
        })
        .collect()
    }
}

/// Store bytes under their SHA-256 identity, without replacing existing data.
///
/// # Errors
///
/// Returns an explicit error for unsafe bundle paths, a pre-existing corrupt or
/// colliding target, or an I/O failure.
pub fn put_blob(
    bundle_directory: impl AsRef<Path>,
    bytes: &[u8],
) -> Result<BlobSummary, BundleIndexError> {
    put_blob_verified(bundle_directory, bytes, BlobId::digest(bytes))
}

/// Store bytes only when their digest matches the caller's expected identity.
///
/// # Errors
///
/// Returns `BLOB_HASH_MISMATCH` before writing when the supplied expectation is
/// wrong, and otherwise returns the same failures as [`put_blob`].
pub fn put_blob_verified(
    bundle_directory: impl AsRef<Path>,
    bytes: &[u8],
    expected: BlobId,
) -> Result<BlobSummary, BundleIndexError> {
    let actual = BlobId::digest(bytes);
    if actual != expected {
        return Err(BundleIndexError::BlobHashMismatch { expected, actual });
    }
    let root = checked_root_or_create(bundle_directory.as_ref())?;
    let relative_path = blob_relative_path(expected);
    let destination = root.join(blob_path_components(expected));
    ensure_contained(&root, &destination)?;
    if destination.exists() {
        return existing_blob_summary(&root, &destination, expected);
    }
    let parent = destination
        .parent()
        .ok_or(BundleIndexError::PathTraversal)?;
    fs::create_dir_all(parent)?;
    ensure_contained(&root, parent)?;
    let staging_directory = root.join("blobs").join("staging");
    fs::create_dir_all(&staging_directory)?;
    let staging_path = create_staging_blob(&staging_directory, expected, bytes)?;
    match publish_blob_no_clobber(&staging_path, &destination) {
        Ok(()) => {
            sync_file(&destination)?;
            remove_if_exists(&staging_path)?;
            sync_directory_best_effort(parent)?;
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists || destination.exists() => {
            remove_if_exists(&staging_path)?;
            return existing_blob_summary(&root, &destination, expected);
        }
        Err(error) => {
            remove_if_exists(&staging_path)?;
            return Err(error.into());
        }
    }
    Ok(BlobSummary {
        id: expected,
        byte_length: u64::try_from(bytes.len()).map_err(|_| BundleIndexError::IndexValueInvalid)?,
        relative_path,
    })
}

/// Validate one canonical content-addressed blob against its path identity.
///
/// # Errors
///
/// Returns an explicit error for path escape, symlink, missing file, or digest
/// mismatch.
pub fn validate_blob(
    bundle_directory: impl AsRef<Path>,
    id: BlobId,
) -> Result<BlobSummary, BundleIndexError> {
    let root = checked_root(bundle_directory.as_ref())?;
    let path = root.join(blob_path_components(id));
    ensure_contained(&root, &path)?;
    validate_blob_path(&root, &path, id)
}

/// Rebuild the complete disposable `SQLite` index from authoritative bundle files.
///
/// # Errors
///
/// Returns an explicit error when any segment or blob is invalid, a path is
/// noncanonical/unsafe, a migration fails, or the rebuilt database cannot be
/// published.
pub fn rebuild_index(bundle_directory: impl AsRef<Path>) -> Result<IndexSummary, BundleIndexError> {
    let root = checked_root(bundle_directory.as_ref())?;
    let segment_summaries = validate_bundle(&root)?;
    let mut segments = Vec::with_capacity(segment_summaries.len());
    for summary in segment_summaries {
        let file_name = format!("segment-{:020}.bseg", summary.sequence);
        let path = root.join(&file_name);
        let (sha256, byte_length) = hash_file(&path)?;
        segments.push(IndexedSegment {
            sequence: summary.sequence,
            relative_path: file_name,
            frame_count: summary.frame_count,
            maximum_frame_size: summary.maximum_frame_size,
            sha256,
            byte_length,
        });
    }
    let artifacts = scan_blobs(&root)?;
    let temporary = unique_sibling(&root, "bundle-index", "sqlite3.next");
    build_index_database(&temporary, &segments, &artifacts)?;
    sync_file(&temporary)?;
    publish_rebuilt_index(&root, &temporary)?;
    sync_directory_best_effort(&root)?;
    Ok(IndexSummary {
        segment_count: segments.len(),
        artifact_count: artifacts.len(),
    })
}

fn build_index_database(
    path: &Path,
    segments: &[IndexedSegment],
    artifacts: &[IndexedArtifact],
) -> Result<(), BundleIndexError> {
    let mut connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    apply_migrations(&connection)?;
    let transaction = connection.transaction()?;
    for segment in segments {
        transaction.execute(
            "INSERT INTO event_segments( \
             sequence_decimal, relative_path, frame_count_decimal, \
             maximum_frame_size, sha256, byte_length_decimal \
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                segment.sequence.to_string(),
                segment.relative_path,
                segment.frame_count.to_string(),
                i64::from(segment.maximum_frame_size),
                segment.sha256.to_hex(),
                segment.byte_length.to_string(),
            ],
        )?;
    }
    for artifact in artifacts {
        transaction.execute(
            "INSERT INTO derived_artifacts(sha256, relative_path, byte_length_decimal) \
             VALUES (?1, ?2, ?3)",
            params![
                artifact.id.to_hex(),
                artifact.relative_path,
                artifact.byte_length.to_string(),
            ],
        )?;
    }
    transaction.commit()?;
    connection.execute_batch("PRAGMA optimize;")?;
    connection
        .close()
        .map_err(|(_, error)| BundleIndexError::Sqlite(error))
}

fn apply_migrations(connection: &Connection) -> Result<(), BundleIndexError> {
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    match version {
        0 => connection.execute_batch(MIGRATION_0001)?,
        INDEX_SCHEMA_VERSION => {}
        other => return Err(BundleIndexError::IndexSchemaVersion(other)),
    }
    validate_index_identity(connection)
}

fn validate_index_identity(connection: &Connection) -> Result<(), BundleIndexError> {
    let application_id: i64 =
        connection.pragma_query_value(None, "application_id", |row| row.get(0))?;
    if application_id != INDEX_APPLICATION_ID {
        return Err(BundleIndexError::IndexApplicationId(application_id));
    }
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version != INDEX_SCHEMA_VERSION {
        return Err(BundleIndexError::IndexSchemaVersion(version));
    }
    let format: String = connection.query_row(
        "SELECT format FROM index_metadata WHERE singleton = 1",
        [],
        |row| row.get(0),
    )?;
    if format != INDEX_FORMAT {
        return Err(BundleIndexError::IndexFormat);
    }
    Ok(())
}

fn validate_index_paths(connection: &Connection) -> Result<(), BundleIndexError> {
    let mut segments =
        connection.prepare("SELECT sequence_decimal, relative_path FROM event_segments")?;
    let segment_rows = segments.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    for row in segment_rows {
        let (sequence, path) = row?;
        let sequence = parse_u64(&sequence)?;
        if path != format!("segment-{sequence:020}.bseg") {
            return Err(BundleIndexError::IndexPathInvalid);
        }
    }
    let mut artifacts =
        connection.prepare("SELECT sha256, relative_path FROM derived_artifacts")?;
    let artifact_rows = artifacts.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    for row in artifact_rows {
        let (sha256, path) = row?;
        let id = BlobId::from_hex(&sha256).map_err(|_| BundleIndexError::IndexValueInvalid)?;
        if path != blob_relative_path(id) {
            return Err(BundleIndexError::IndexPathInvalid);
        }
    }
    Ok(())
}

fn scan_blobs(root: &Path) -> Result<Vec<IndexedArtifact>, BundleIndexError> {
    let digest_root = root.join("blobs").join("sha256");
    if !digest_root.exists() {
        return Ok(Vec::new());
    }
    reject_symlink(&digest_root)?;
    if !digest_root.is_dir() {
        return Err(BundleIndexError::UnexpectedBlobPath);
    }
    let mut artifacts = Vec::new();
    for prefix_entry in fs::read_dir(&digest_root)? {
        let prefix_entry = prefix_entry?;
        let prefix_path = prefix_entry.path();
        reject_symlink(&prefix_path)?;
        let prefix = prefix_entry
            .file_name()
            .into_string()
            .map_err(|_| BundleIndexError::UnexpectedBlobPath)?;
        if prefix.len() != 2
            || !prefix
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || !prefix_path.is_dir()
        {
            return Err(BundleIndexError::UnexpectedBlobPath);
        }
        for blob_entry in fs::read_dir(prefix_path)? {
            let blob_entry = blob_entry?;
            let path = blob_entry.path();
            reject_symlink(&path)?;
            if !path.is_file() {
                return Err(BundleIndexError::UnexpectedBlobPath);
            }
            let name = blob_entry
                .file_name()
                .into_string()
                .map_err(|_| BundleIndexError::UnexpectedBlobPath)?;
            let remainder = name
                .strip_suffix(BLOB_SUFFIX)
                .ok_or(BundleIndexError::UnexpectedBlobPath)?;
            if remainder.len() != 62 {
                return Err(BundleIndexError::UnexpectedBlobPath);
            }
            let id = BlobId::from_hex(&format!("{prefix}{remainder}"))
                .map_err(|_| BundleIndexError::UnexpectedBlobPath)?;
            let summary = validate_blob_path(root, &path, id)?;
            artifacts.push(IndexedArtifact {
                id,
                relative_path: summary.relative_path,
                byte_length: summary.byte_length,
            });
        }
    }
    artifacts.sort_by_key(|artifact| artifact.id);
    Ok(artifacts)
}

fn validate_blob_path(
    root: &Path,
    path: &Path,
    expected: BlobId,
) -> Result<BlobSummary, BundleIndexError> {
    reject_symlink(path)?;
    if !path.is_file() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "blob file missing").into());
    }
    ensure_contained(root, path)?;
    let (actual, byte_length) = hash_file(path)?;
    if actual != expected {
        return Err(BundleIndexError::BlobHashMismatch { expected, actual });
    }
    Ok(BlobSummary {
        id: expected,
        byte_length,
        relative_path: blob_relative_path(expected),
    })
}

fn existing_blob_summary(
    root: &Path,
    path: &Path,
    expected: BlobId,
) -> Result<BlobSummary, BundleIndexError> {
    match validate_blob_path(root, path, expected) {
        Ok(summary) => Ok(summary),
        Err(BundleIndexError::BlobHashMismatch { .. }) => {
            Err(BundleIndexError::BlobHashCollision(expected))
        }
        Err(error) => Err(error),
    }
}

fn hash_file(path: &Path) -> Result<(BlobId, u64), BundleIndexError> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut hasher = Sha256::new();
    let mut byte_length = 0_u64;
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        byte_length = byte_length
            .checked_add(u64::try_from(read).map_err(|_| BundleIndexError::IndexValueInvalid)?)
            .ok_or(BundleIndexError::IndexValueInvalid)?;
    }
    Ok((BlobId(hasher.finalize().into()), byte_length))
}

fn create_staging_blob(
    staging_directory: &Path,
    id: BlobId,
    bytes: &[u8],
) -> Result<PathBuf, BundleIndexError> {
    for _ in 0..100 {
        let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path =
            staging_directory.join(format!("{}.{}.{sequence}.pending", id, std::process::id()));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => {
                let mut writer = BufWriter::new(file);
                writer.write_all(bytes)?;
                writer.flush()?;
                writer.get_ref().sync_all()?;
                return Ok(path);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "unable to allocate a unique blob staging path",
    )
    .into())
}

fn publish_blob_no_clobber(source: &Path, destination: &Path) -> io::Result<()> {
    match fs::hard_link(source, destination) {
        Ok(()) => Ok(()),
        #[cfg(windows)]
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::PermissionDenied | io::ErrorKind::Unsupported
            ) =>
        {
            fs::rename(source, destination)
        }
        Err(error) => Err(error),
    }
}

fn publish_rebuilt_index(root: &Path, temporary: &Path) -> Result<(), BundleIndexError> {
    let destination = root.join(INDEX_FILE_NAME);
    if destination.exists() {
        reject_symlink(&destination)?;
        if !destination.is_file() {
            return Err(BundleIndexError::PathTraversal);
        }
        let backup = unique_sibling(root, "bundle-index", "sqlite3.previous");
        fs::rename(&destination, &backup)?;
        if let Err(error) = fs::rename(temporary, &destination) {
            let _ = fs::rename(&backup, &destination);
            return Err(error.into());
        }
        fs::remove_file(backup)?;
    } else {
        fs::rename(temporary, destination)?;
    }
    Ok(())
}

fn checked_root_or_create(path: &Path) -> Result<PathBuf, BundleIndexError> {
    fs::create_dir_all(path)?;
    checked_root(path)
}

fn checked_root(path: &Path) -> Result<PathBuf, BundleIndexError> {
    reject_symlink(path)?;
    if !path.is_dir() {
        return Err(BundleIndexError::PathTraversal);
    }
    Ok(fs::canonicalize(path)?)
}

fn ensure_contained(root: &Path, path: &Path) -> Result<(), BundleIndexError> {
    if !path.starts_with(root) {
        return Err(BundleIndexError::PathTraversal);
    }
    let relative = path
        .strip_prefix(root)
        .map_err(|_| BundleIndexError::PathTraversal)?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(BundleIndexError::SymlinkRejected);
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => break,
            Err(error) => return Err(error.into()),
        }
    }
    let existing = if path.exists() {
        Some(path)
    } else {
        path.parent().filter(|parent| parent.exists())
    };
    if let Some(existing) = existing {
        let canonical = fs::canonicalize(existing)?;
        if !canonical.starts_with(root) {
            return Err(BundleIndexError::PathTraversal);
        }
    }
    Ok(())
}

fn reject_symlink(path: &Path) -> Result<(), BundleIndexError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(BundleIndexError::SymlinkRejected),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn blob_relative_path(id: BlobId) -> String {
    let hex = id.to_hex();
    format!("blobs/sha256/{}/{}.blob", &hex[..2], &hex[2..])
}

fn blob_path_components(id: BlobId) -> PathBuf {
    let hex = id.to_hex();
    PathBuf::from("blobs")
        .join("sha256")
        .join(&hex[..2])
        .join(format!("{}.blob", &hex[2..]))
}

fn unique_sibling(root: &Path, stem: &str, suffix: &str) -> PathBuf {
    let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    root.join(format!(
        ".{stem}.{}.{sequence}.{suffix}",
        std::process::id()
    ))
}

fn parse_u64(value: &str) -> Result<u64, BundleIndexError> {
    if value.is_empty()
        || value.len() > 20
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(BundleIndexError::IndexValueInvalid);
    }
    value
        .parse()
        .map_err(|_| BundleIndexError::IndexValueInvalid)
}

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => unreachable!("BlobId::from_hex validated every byte"),
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn remove_if_exists(path: &Path) -> Result<(), BundleIndexError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn sync_file(path: &Path) -> Result<(), BundleIndexError> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?
        .sync_all()?;
    Ok(())
}

fn sync_directory_best_effort(directory: &Path) -> Result<(), BundleIndexError> {
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
        Err(error) => Err(error.into()),
    }
}
