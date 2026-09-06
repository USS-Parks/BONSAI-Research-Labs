use bonsai_bundle::{
    AnalyticalTable, BlobId, DecisionRow, DerivationExpectation, DerivationSpec, EventRow,
    LineageRow, MetricRow, materialize_derivation, validate_derivation,
};
use serde::Deserialize;
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
fn committed_fixture_matrix_has_exact_derivation_outcomes() {
    let matrix: FixtureMatrix = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/derivations/v1/expected-outcomes.json"
    )))
    .expect("fixture matrix must decode");
    assert_eq!(matrix.format, "bonsai.derivation/v1");
    for case in matrix.cases {
        let actual = run_case(&case.name);
        assert_eq!(actual, case.expected, "fixture {}", case.name);
    }
}

#[test]
fn every_table_round_trips_with_exact_schema_and_provenance() {
    let directory = TempDir::new().expect("temporary derivations");
    for (index, table) in sample_tables().iter().enumerate() {
        let path = directory.path().join(format!("table-{index}.parquet"));
        let spec = spec();
        let written = materialize_derivation(&path, table, &spec).expect("materialize");
        let validated = validate_derivation(
            &path,
            &DerivationExpectation {
                kind: table.kind(),
                source_hashes: spec.source_hashes.clone(),
                producer_id: spec.producer_id.clone(),
                producer_version: spec.producer_version.clone(),
            },
        )
        .expect("validate");
        assert_eq!(validated, written);
        assert_eq!(validated.row_count, 1);
    }
}

#[test]
fn materialization_never_replaces_an_existing_derivative() {
    let directory = TempDir::new().expect("temporary derivations");
    let path = directory.path().join("events.parquet");
    let table = sample_tables().remove(0);
    materialize_derivation(&path, &table, &spec()).expect("first materialization");
    let first = std::fs::read(&path).expect("first bytes");
    materialize_derivation(&path, &table, &spec()).expect_err("no replacement");
    assert_eq!(std::fs::read(path).expect("unchanged bytes"), first);
}

fn run_case(name: &str) -> String {
    match name {
        "all_table_contracts" => all_table_contracts_case(),
        "identical_regeneration" => identical_regeneration_case(),
        "wrong_source_hash" => wrong_source_hash_case(),
        "stale_producer" => stale_producer_case(),
        unknown => panic!("unknown fixture case: {unknown}"),
    }
}

fn all_table_contracts_case() -> String {
    let directory = TempDir::new().expect("temporary derivations");
    for (index, table) in sample_tables().iter().enumerate() {
        let path = directory.path().join(format!("table-{index}.parquet"));
        materialize_derivation(&path, table, &spec()).expect("all schemas materialize");
    }
    "DERIVATION_TABLES_VALID".to_owned()
}

fn identical_regeneration_case() -> String {
    let directory = TempDir::new().expect("temporary derivations");
    let table = sample_tables().remove(0);
    let first = materialize_derivation(directory.path().join("first.parquet"), &table, &spec())
        .expect("first derivation");
    let second = materialize_derivation(directory.path().join("second.parquet"), &table, &spec())
        .expect("second derivation");
    assert_eq!(first, second);
    "DERIVATION_SEMANTICALLY_IDENTICAL".to_owned()
}

fn wrong_source_hash_case() -> String {
    let (directory, path, table) = one_derivation();
    let error = validate_derivation(
        &path,
        &DerivationExpectation {
            kind: table.kind(),
            source_hashes: vec![BlobId::digest(b"different authoritative input")],
            producer_id: "bonsai.fixture".to_owned(),
            producer_version: "1.0.0".to_owned(),
        },
    )
    .expect_err("wrong input must fail");
    drop(directory);
    error.code().to_owned()
}

fn stale_producer_case() -> String {
    let (directory, path, table) = one_derivation();
    let error = validate_derivation(
        &path,
        &DerivationExpectation {
            kind: table.kind(),
            source_hashes: spec().source_hashes,
            producer_id: "bonsai.fixture".to_owned(),
            producer_version: "2.0.0".to_owned(),
        },
    )
    .expect_err("older producer must be stale");
    drop(directory);
    error.code().to_owned()
}

fn one_derivation() -> (TempDir, std::path::PathBuf, AnalyticalTable) {
    let directory = TempDir::new().expect("temporary derivations");
    let path = directory.path().join("events.parquet");
    let table = sample_tables().remove(0);
    materialize_derivation(&path, &table, &spec()).expect("materialize");
    (directory, path, table)
}

fn spec() -> DerivationSpec {
    DerivationSpec {
        source_hashes: vec![
            BlobId::digest(b"segment zero"),
            BlobId::digest(b"metric specification"),
        ],
        producer_id: "bonsai.fixture".to_owned(),
        producer_version: "1.0.0".to_owned(),
    }
}

fn sample_tables() -> Vec<AnalyticalTable> {
    vec![
        AnalyticalTable::Events(vec![EventRow {
            run_id: "run-0001".to_owned(),
            source_id: "source-0001".to_owned(),
            source_sequence: 7,
            event_type: "bonsai.event.fixture.v1".to_owned(),
            monotonic_time_ns: 42,
            wall_time_unix_ns: None,
            payload_sha256: BlobId::digest(b"event payload"),
        }]),
        AnalyticalTable::Metrics(vec![MetricRow {
            run_id: "run-0001".to_owned(),
            metric_id: "reward-rate".to_owned(),
            metric_version: "1.0.0".to_owned(),
            value: Some(0.5),
            unit: "reward_per_step".to_owned(),
            availability: "measured".to_owned(),
            input_sha256: BlobId::digest(b"metric inputs"),
        }]),
        AnalyticalTable::Lineage(vec![LineageRow {
            artifact_id: "artifact-0001".to_owned(),
            revision: 1,
            artifact_type: "feature".to_owned(),
            parent_artifact_id: None,
            consumer_artifact_id: Some("consumer-0001".to_owned()),
            disposition: "retained".to_owned(),
        }]),
        AnalyticalTable::Decisions(vec![DecisionRow {
            run_id: "run-0001".to_owned(),
            decision_id: "decision-0001".to_owned(),
            policy_version: "resource-policy/1.0".to_owned(),
            outcome: "admit".to_owned(),
            reason_code: "WITHIN_LIMIT".to_owned(),
            observed_state_sha256: BlobId::digest(b"observed state"),
            requested_work_sha256: BlobId::digest(b"requested work"),
        }]),
    ]
}

#[test]
fn streamed_batches_preserve_all_four_table_semantics() {
    use bonsai_bundle::{DerivationStreamLimits, materialize_derivation_stream};
    let directory = TempDir::new().expect("temporary");
    for (index, table) in sample_tables().iter().enumerate() {
        let whole = repeat_table(table, 12);
        let expected = materialize_derivation(
            directory.path().join(format!("whole-{index}.parquet")),
            &whole,
            &spec(),
        )
        .expect("whole");
        let path = directory.path().join(format!("stream-{index}.parquet"));
        let actual = materialize_derivation_stream(
            &path,
            table.kind(),
            12,
            (0..4).map(|_| repeat_table(table, 3)),
            &spec(),
            DerivationStreamLimits {
                maximum_batch_rows: 3,
                maximum_batch_bytes: 8192,
                maximum_file_rows: 12,
                maximum_output_bytes: 65536,
            },
        )
        .expect("stream");
        assert_eq!(actual, expected);
        let checked = validate_derivation(
            &path,
            &DerivationExpectation {
                kind: table.kind(),
                source_hashes: spec().source_hashes,
                producer_id: spec().producer_id,
                producer_version: spec().producer_version,
            },
        )
        .expect("streamed validation");
        assert_eq!(checked, expected);
    }
}

#[test]
fn streamed_output_enforces_batch_count_and_byte_quotas() {
    use bonsai_bundle::{DerivationStreamLimits, materialize_derivation_stream};
    let directory = TempDir::new().expect("temporary");
    let table = sample_tables().remove(0);
    for (name, rows, batch_rows, bytes) in [
        ("count", 2, 2, 65536),
        ("batch", 1, 0, 65536),
        ("output", 1, 1, 32),
    ] {
        let path = directory.path().join(format!("{name}.parquet"));
        assert!(
            materialize_derivation_stream(
                &path,
                table.kind(),
                rows,
                [repeat_table(&table, 1)],
                &spec(),
                DerivationStreamLimits {
                    maximum_batch_rows: batch_rows,
                    maximum_batch_bytes: 8192,
                    maximum_file_rows: 2,
                    maximum_output_bytes: bytes
                }
            )
            .is_err()
        );
        if name == "output" {
            assert!(std::fs::metadata(path).expect("partial output").len() <= bytes);
        }
    }
}

fn repeat_table(table: &AnalyticalTable, count: usize) -> AnalyticalTable {
    match table {
        AnalyticalTable::Events(rows) => AnalyticalTable::Events(vec![rows[0].clone(); count]),
        AnalyticalTable::Metrics(rows) => AnalyticalTable::Metrics(vec![rows[0].clone(); count]),
        AnalyticalTable::Lineage(rows) => AnalyticalTable::Lineage(vec![rows[0].clone(); count]),
        AnalyticalTable::Decisions(rows) => {
            AnalyticalTable::Decisions(vec![rows[0].clone(); count])
        }
    }
}
