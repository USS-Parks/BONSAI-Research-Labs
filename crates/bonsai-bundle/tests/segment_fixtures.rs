use bonsai_bundle::{
    RecoveryOutcome, SegmentError, SegmentWriter, recover_open_segment, validate_bundle,
    validate_segment,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const MAXIMUM_FRAME_SIZE: u32 = 64;
const HEADER_LEN: usize = 60;
const FRAME_PREFIX_LEN: usize = 12;

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
fn committed_fixture_matrix_has_exact_deterministic_outcomes() {
    let matrix: FixtureMatrix = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/event-segments/v1/expected-outcomes.json"
    )))
    .expect("fixture matrix must decode");
    assert_eq!(matrix.format, "bonsai.event-segment/v1");
    for case in matrix.cases {
        let actual = run_case(&case.name);
        assert_eq!(actual, case.expected, "fixture {}", case.name);
    }
}

#[test]
fn append_only_writer_rejects_oversize_before_writing_and_publishes_once() {
    let directory = TempDir::new().expect("temporary bundle");
    let mut writer = SegmentWriter::create(directory.path(), 0, 4).expect("writer");
    let error = writer.append(b"12345").expect_err("oversize must fail");
    assert_eq!(error.code(), "SEGMENT_FRAME_TOO_LARGE");
    writer.append(b"1234").expect("bounded frame");
    let summary = writer.finalize().expect("finalize");
    assert_eq!(summary.frame_count, 1);
    assert_eq!(
        validate_bundle(directory.path()).expect("valid bundle"),
        vec![summary]
    );
    assert!(!open_path(directory.path(), 0).exists());
}

#[test]
fn recovery_never_salvages_a_partial_frame() {
    let directory = TempDir::new().expect("temporary bundle");
    let mut writer =
        SegmentWriter::create(directory.path(), 0, MAXIMUM_FRAME_SIZE).expect("writer");
    writer.append(b"complete").expect("frame");
    drop(writer);
    let path = open_path(directory.path(), 0);
    let original = fs::read(&path).expect("staged bytes");
    fs::write(&path, &original[..original.len() - 1]).expect("truncate staged copy");
    let error = recover_open_segment(&path).expect_err("partial recovery must fail");
    assert_eq!(error.code(), "SEGMENT_FRAME_TRUNCATED");
    assert_eq!(
        fs::read(&path).expect("unchanged staged bytes"),
        &original[..original.len() - 1]
    );
}

fn run_case(name: &str) -> String {
    match name {
        "valid" => valid_case(),
        "truncated_header" => validation_error(|bytes| bytes.truncate(HEADER_LEN - 1)),
        "truncated_frame" => {
            validation_error(|bytes| bytes.truncate(HEADER_LEN + FRAME_PREFIX_LEN + 2))
        }
        "checksum_corruption" => {
            validation_error(|bytes| bytes[HEADER_LEN + FRAME_PREFIX_LEN] ^= 0x01)
        }
        "segment_checksum_corruption" => validation_error(|bytes| {
            let segment_checksum_offset = bytes.len() - 64;
            bytes[segment_checksum_offset] ^= 0x01;
        }),
        "oversized_frame" => validation_error(|bytes| {
            let length_offset = HEADER_LEN + 8;
            bytes[length_offset..length_offset + 4]
                .copy_from_slice(&(MAXIMUM_FRAME_SIZE + 1).to_le_bytes());
        }),
        "duplicate_sequence" => duplicate_sequence_case(),
        "non_monotonic_sequence" => non_monotonic_sequence_case(),
        "recover_complete_open" => recover_complete_open_case(),
        "recover_finalized_open" => recover_finalized_open_case(),
        "recover_partial_open" => recover_partial_open_case(),
        unknown => panic!("unknown fixture case: {unknown}"),
    }
}

fn valid_case() -> String {
    let (directory, path) = finalized_fixture();
    let code = if validate_segment(path).is_ok() {
        "SEGMENT_VALID"
    } else {
        "SEGMENT_UNEXPECTED_ERROR"
    };
    drop(directory);
    code.to_owned()
}

fn validation_error(mutate: impl FnOnce(&mut Vec<u8>)) -> String {
    let (directory, path) = finalized_fixture();
    let mut bytes = fs::read(&path).expect("segment bytes");
    mutate(&mut bytes);
    fs::write(&path, bytes).expect("mutated fixture");
    let code = validate_segment(path)
        .expect_err("fixture must fail")
        .code()
        .to_owned();
    drop(directory);
    code
}

fn duplicate_sequence_case() -> String {
    let (directory, path) = finalized_fixture();
    fs::copy(path, directory.path().join("duplicate.bseg")).expect("duplicate fixture");
    validate_bundle(directory.path())
        .expect_err("duplicate sequence must fail")
        .code()
        .to_owned()
}

fn non_monotonic_sequence_case() -> String {
    let directory = TempDir::new().expect("temporary bundle");
    match SegmentWriter::create(directory.path(), 1, MAXIMUM_FRAME_SIZE) {
        Ok(_) => panic!("first sequence must be zero"),
        Err(error) => error.code().to_owned(),
    }
}

fn recover_complete_open_case() -> String {
    let directory = TempDir::new().expect("temporary bundle");
    let mut writer =
        SegmentWriter::create(directory.path(), 0, MAXIMUM_FRAME_SIZE).expect("writer");
    writer.append(b"fixture-event").expect("frame");
    drop(writer);
    match recover_open_segment(open_path(directory.path(), 0)).expect("recovery") {
        RecoveryOutcome::Recovered(summary) if summary.frame_count == 1 => {
            "SEGMENT_RECOVERED".to_owned()
        }
        outcome => panic!("unexpected outcome: {outcome:?}"),
    }
}

fn recover_finalized_open_case() -> String {
    let (directory, final_path) = finalized_fixture();
    let staged_path = open_path(directory.path(), 0);
    fs::copy(&final_path, &staged_path).expect("stale staged link fixture");
    match recover_open_segment(staged_path).expect("recovery") {
        RecoveryOutcome::AlreadyFinalized(summary) if summary.frame_count == 1 => {
            "SEGMENT_ALREADY_FINALIZED".to_owned()
        }
        outcome => panic!("unexpected outcome: {outcome:?}"),
    }
}

fn recover_partial_open_case() -> String {
    let directory = TempDir::new().expect("temporary bundle");
    let mut writer =
        SegmentWriter::create(directory.path(), 0, MAXIMUM_FRAME_SIZE).expect("writer");
    writer.append(b"fixture-event").expect("frame");
    drop(writer);
    let path = open_path(directory.path(), 0);
    let bytes = fs::read(&path).expect("staged bytes");
    fs::write(&path, &bytes[..bytes.len() - 1]).expect("partial frame fixture");
    recover_open_segment(path)
        .expect_err("partial recovery must fail")
        .code()
        .to_owned()
}

fn finalized_fixture() -> (TempDir, PathBuf) {
    let directory = TempDir::new().expect("temporary bundle");
    let mut writer =
        SegmentWriter::create(directory.path(), 0, MAXIMUM_FRAME_SIZE).expect("writer");
    writer.append(b"fixture-event").expect("frame");
    let summary = writer.finalize().expect("finalize");
    assert_eq!(summary.checksum.len(), Sha256::output_size());
    let path = directory.path().join("segment-00000000000000000000.bseg");
    (directory, path)
}

fn open_path(directory: &Path, sequence: u64) -> PathBuf {
    directory.join(format!("segment-{sequence:020}.open"))
}

#[allow(dead_code)]
fn assert_error_is_send_sync(error: SegmentError) {
    fn assert_send_sync<T: Send + Sync>(_: T) {}
    assert_send_sync(error);
}
