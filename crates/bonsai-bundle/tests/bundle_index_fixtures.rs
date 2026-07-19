use bonsai_bundle::{
    BlobId, BundleIndex, SegmentWriter, put_blob, put_blob_verified, rebuild_index,
};
use rusqlite::{Connection, OpenFlags};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[derive(Debug, Deserialize)]
struct FixtureMatrix {
    format: String,
    cases: Vec<FixtureCase>,
}

#[derive(Debug, Deserialize)]
struct FixtureCase {
    name: String,
    expected: String,
}

#[test]
fn committed_fixture_matrix_has_exact_bundle_index_outcomes() {
    let matrix: FixtureMatrix = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/bundle-index/v1/expected-outcomes.json"
    )))
    .expect("fixture matrix must decode");
    assert_eq!(matrix.format, "bonsai.bundle-index/v1");
    for case in matrix.cases {
        let actual = run_case(&case.name);
        assert_eq!(actual, case.expected, "fixture {}", case.name);
    }
}

#[test]
fn index_is_rebuilt_only_from_authoritative_segments_and_blobs() {
    let directory = populated_bundle();
    let first = rebuild_index(directory.path()).expect("first rebuild");
    assert_eq!(first.segment_count, 2);
    assert_eq!(first.artifact_count, 2);

    let index = BundleIndex::open_read_only(directory.path()).expect("read-only index");
    let first_segments = index.segments().expect("segments");
    let first_artifacts = index.artifacts().expect("artifacts");
    assert!(index.is_read_only().expect("query-only pragma"));
    drop(index);

    fs::remove_file(directory.path().join("bundle-index.sqlite3"))
        .expect("delete disposable index");
    let second = rebuild_index(directory.path()).expect("second rebuild");
    assert_eq!(second, first);
    let rebuilt = BundleIndex::open_read_only(directory.path()).expect("rebuilt index");
    assert_eq!(
        rebuilt.segments().expect("rebuilt segments"),
        first_segments
    );
    assert_eq!(
        rebuilt.artifacts().expect("rebuilt artifacts"),
        first_artifacts
    );
}

#[test]
fn read_only_sqlite_handle_cannot_mutate_the_index() {
    let directory = populated_bundle();
    rebuild_index(directory.path()).expect("rebuild");
    let path = directory.path().join("bundle-index.sqlite3");
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .expect("portable read-only SQLite open");
    let error = connection
        .execute("DELETE FROM event_segments", [])
        .expect_err("read-only connection must reject mutation");
    assert!(error.to_string().contains("readonly"));
}

fn run_case(name: &str) -> String {
    match name {
        "rebuild_from_bundle_files" => rebuild_case(),
        "blob_hash_mismatch" => hash_mismatch_case(),
        "blob_hash_collision" => collision_case(),
        "blob_path_traversal" => BlobId::from_hex("../outside")
            .expect_err("traversal must not parse")
            .code()
            .to_owned(),
        "index_path_traversal" => index_path_traversal_case(),
        "read_only_open" => read_only_case(),
        unknown => panic!("unknown fixture case: {unknown}"),
    }
}

fn rebuild_case() -> String {
    let directory = populated_bundle();
    let summary = rebuild_index(directory.path()).expect("rebuild");
    assert_eq!(summary.segment_count, 2);
    assert_eq!(summary.artifact_count, 2);
    "BUNDLE_INDEX_REBUILT".to_owned()
}

fn hash_mismatch_case() -> String {
    let directory = TempDir::new().expect("temporary bundle");
    let expected = BlobId::digest(b"expected bytes");
    put_blob_verified(directory.path(), b"different bytes", expected)
        .expect_err("digest mismatch must fail before storage")
        .code()
        .to_owned()
}

fn collision_case() -> String {
    let directory = TempDir::new().expect("temporary bundle");
    let summary = put_blob(directory.path(), b"authoritative bytes").expect("initial blob");
    let path = relative_path(directory.path(), &summary.relative_path);
    fs::write(path, b"different bytes at the same digest path").expect("corrupt target");
    put_blob(directory.path(), b"authoritative bytes")
        .expect_err("pre-existing mismatch must fail closed")
        .code()
        .to_owned()
}

fn index_path_traversal_case() -> String {
    let directory = populated_bundle();
    rebuild_index(directory.path()).expect("rebuild");
    let path = directory.path().join("bundle-index.sqlite3");
    let connection = Connection::open(&path).expect("test-only writable connection");
    connection
        .execute(
            "UPDATE derived_artifacts SET relative_path = '../outside' \
             WHERE sha256 = (SELECT min(sha256) FROM derived_artifacts)",
            [],
        )
        .expect("tamper derived index path");
    drop(connection);
    BundleIndex::open_read_only(directory.path())
        .err()
        .expect("tampered path must fail")
        .code()
        .to_owned()
}

fn read_only_case() -> String {
    let directory = populated_bundle();
    rebuild_index(directory.path()).expect("rebuild");
    let index = BundleIndex::open_read_only(directory.path()).expect("read-only open");
    assert!(index.is_read_only().expect("query-only pragma"));
    "BUNDLE_INDEX_READ_ONLY".to_owned()
}

fn populated_bundle() -> TempDir {
    let directory = TempDir::new().expect("temporary bundle");
    let mut first = SegmentWriter::create(directory.path(), 0, 64).expect("first segment");
    first.append(b"first event").expect("first frame");
    first.finalize().expect("first finalize");
    let mut second = SegmentWriter::create(directory.path(), 1, 64).expect("second segment");
    second.append(b"second event").expect("second frame");
    second.finalize().expect("second finalize");
    put_blob(directory.path(), b"derived artifact one").expect("first blob");
    put_blob(directory.path(), b"derived artifact two").expect("second blob");
    directory
}

fn relative_path(root: &Path, portable: &str) -> PathBuf {
    portable
        .split('/')
        .fold(root.to_path_buf(), |path, component| path.join(component))
}
