use bonsai_contracts::bonsai::artifact::v1::{
    ArtifactBirth, ArtifactCost, ArtifactLifecycleEvent, ArtifactType, Provenance,
    artifact_lifecycle_event::Detail,
};
use bonsai_contracts::bonsai::event::v1::Availability;
use bonsai_lineage::persistent::{PersistentLimits, PersistentLineageRegistry};
use std::fs;

fn limits() -> PersistentLimits {
    PersistentLimits {
        maximum_live_artifacts: 4,
        maximum_consumers: 16,
        maximum_frame_bytes: 4096,
        database_bytes: 1024 * 1024,
        output_bytes: 8 * 1024 * 1024,
    }
}
fn event(sequence: u64) -> ArtifactLifecycleEvent {
    let provenance = Some(Provenance {
        producer_id: "BX-08".into(),
        producer_version: "1".into(),
        source_event_ids: vec![vec![9; 16]],
        method_ids: vec!["test".into()],
    });
    let detail = if sequence == 1 {
        Detail::Birth(ArtifactBirth {
            artifact_type: ArtifactType::Feature as i32,
            representation_sha256: vec![3; 32],
            parents: vec![],
            provenance,
        })
    } else {
        let mut id = vec![7; 16];
        id[..8].copy_from_slice(&sequence.to_le_bytes());
        Detail::Cost(ArtifactCost {
            cost_entry_id: id,
            counter_id: "work".into(),
            unit: "1".into(),
            amount: Some(1),
            availability: Availability::Measured as i32,
            provenance,
            estimator_id: None,
            estimator_version: None,
            unavailable_reason: None,
        })
    };
    ArtifactLifecycleEvent {
        artifact_id: vec![1; 16],
        artifact_revision_id: vec![2; 16],
        lifecycle_sequence: sequence,
        detail: Some(detail),
    }
}

#[test]
fn all_segment_and_index_boundaries_recover_the_exact_committed_prefix() {
    for boundary in [
        "before_segment_commit",
        "after_segment_commit",
        "before_index_commit",
        "after_index_commit",
    ] {
        let temp = tempfile::tempdir().expect("temp");
        let root = temp.path().join("run");
        let mut store = PersistentLineageRegistry::create(&root, limits()).expect("create");
        store.append(&event(1)).expect("birth");
        store.commit().expect("baseline");
        store.append(&event(2)).expect("pending cost");
        let result = store.commit_with(|at| {
            if at == boundary {
                Err("injected interruption".into())
            } else {
                Ok(())
            }
        });
        assert!(result.is_err());
        drop(store);
        let (mut store, report) = PersistentLineageRegistry::open(&root).expect("recover");
        let accepted = if boundary == "after_index_commit" {
            2
        } else {
            1
        };
        assert_eq!(report.committed_events, accepted);
        assert!(report.continuation);
        assert_eq!(
            store
                .artifact(&[1; 16])
                .expect("read")
                .expect("artifact")
                .next_sequence,
            accepted + 1
        );
        store
            .append(&event(accepted + 1))
            .expect("continue exact next event");
        let next = store.commit().expect("continuation");
        assert_eq!(next.events, accepted + 1);
        assert!(next.continuation);
        assert_eq!(
            report.orphan_batches,
            u64::from(boundary != "after_index_commit")
        );
    }
}

#[test]
fn committed_truncation_and_state_corruption_are_rejected() {
    for mode in ["truncated", "state", "ownership"] {
        let temp = tempfile::tempdir().expect("temp");
        let root = temp.path().join("run");
        let mut store = PersistentLineageRegistry::create(&root, limits()).expect("create");
        store.append(&event(1)).expect("birth");
        store.commit().expect("commit");
        drop(store);
        if mode == "state" {
            let db = rusqlite::Connection::open(root.join("lineage.sqlite3")).expect("db");
            db.execute("UPDATE artifacts SET state=?1", [b"{}".as_slice()])
                .expect("damage");
        } else if mode == "ownership" {
            let db = rusqlite::Connection::open(root.join("lineage.sqlite3")).expect("db");
            db.execute("UPDATE revisions SET owner=?1", [vec![8; 16]])
                .expect("damage owner");
        } else {
            let batch = fs::read_dir(root.join("segments"))
                .expect("segments")
                .next()
                .expect("batch")
                .expect("entry")
                .path();
            let path = batch.join("segment-00000000000000000000.bseg");
            let length = fs::metadata(&path).expect("metadata").len();
            fs::OpenOptions::new()
                .write(true)
                .open(path)
                .expect("open")
                .set_len(length - 1)
                .expect("truncate");
        }
        assert!(PersistentLineageRegistry::open(&root).is_err());
    }
}

#[test]
fn unsupported_checkpoint_and_output_quota_fail_without_deleting_user_data() {
    let temp = tempfile::tempdir().expect("temp");
    let sentinel = temp.path().join("user-data.txt");
    fs::write(&sentinel, b"preserve").expect("sentinel");
    let root = temp.path().join("run");
    let mut small = limits();
    small.output_bytes = small.database_bytes * 5 + 65_536 + 512;
    let mut store = PersistentLineageRegistry::create(&root, small).expect("create");
    store.append(&event(1)).expect("birth");
    store.commit().expect("baseline");
    let mut exhausted = false;
    for sequence in 2..20 {
        if store.append(&event(sequence)).is_err() {
            exhausted = true;
            break;
        }
    }
    assert!(exhausted);
    drop(store);
    let (store, report) = PersistentLineageRegistry::open(&root).expect("recover quota");
    assert_eq!(report.committed_events, 1);
    drop(store);
    let db = rusqlite::Connection::open(root.join("lineage.sqlite3")).expect("db");
    db.pragma_update(None, "user_version", 999)
        .expect("incompatible");
    drop(db);
    assert!(PersistentLineageRegistry::open(&root).is_err());
    assert_eq!(fs::read(sentinel).expect("user data"), b"preserve");
}

#[test]
fn sqlite_full_preserves_original_error_and_poisoned_state() {
    let temp = tempfile::tempdir().expect("temp");
    let root = temp.path().join("run");
    let mut bounds = limits();
    bounds.database_bytes = 65_536;
    let mut store = PersistentLineageRegistry::create(&root, bounds).expect("create");
    store.append(&event(1)).expect("birth");
    store.commit().expect("baseline");
    let mut failure = None;
    for sequence in 2..10_000 {
        if let Err(error) = store.append(&event(sequence)) {
            failure = Some(error);
            break;
        }
    }
    assert!(failure.expect("full").contains("database or disk is full"));
    assert_eq!(
        store.append(&event(2)).expect_err("poisoned"),
        "LINEAGE_SESSION_POISONED"
    );
    drop(store);
    let (_, report) = PersistentLineageRegistry::open(&root).expect("recover");
    assert_eq!(report.committed_events, 1);
}

#[test]
fn a_second_writer_cannot_open_an_active_store() {
    let temp = tempfile::tempdir().expect("temp");
    let root = temp.path().join("run");
    let store = PersistentLineageRegistry::create(&root, limits()).expect("create");
    assert!(PersistentLineageRegistry::open(&root).is_err());
    drop(store);
    assert!(PersistentLineageRegistry::open(&root).is_ok());
}

#[test]
fn retired_ancestry_remains_queryable_under_a_single_live_artifact_cap() {
    use bonsai_contracts::bonsai::artifact::v1::{
        ArtifactDisposition, ArtifactDispositionRecord, LineageRelation, ParentReference,
    };
    let temp = tempfile::tempdir().expect("temp");
    let root = temp.path().join("run");
    let mut bounds = limits();
    bounds.maximum_live_artifacts = 1;
    let mut store = PersistentLineageRegistry::create(&root, bounds).expect("create");
    let first = event(1);
    let Some(Detail::Birth(birth)) = &first.detail else {
        panic!("birth")
    };
    let mut retire = event(2);
    retire.detail = Some(Detail::Disposition(ArtifactDispositionRecord {
        disposition: ArtifactDisposition::Retired as i32,
        successor_artifact_id: None,
        provenance: birth.provenance.clone(),
    }));
    store.append(&first).expect("birth");
    let mut child = event(1);
    child.artifact_id = vec![4; 16];
    child.artifact_revision_id = vec![5; 16];
    if let Some(Detail::Birth(birth)) = &mut child.detail {
        birth.parents = vec![ParentReference {
            artifact_id: vec![1; 16],
            artifact_revision_id: vec![2; 16],
            relation: LineageRelation::DerivedFrom as i32,
        }];
    }
    assert_eq!(
        store.append(&child).expect_err("live cap"),
        "LINEAGE_LIVE_ARTIFACT_LIMIT"
    );
    store.append(&retire).expect("retire");
    store.append(&child).expect("same cap after retirement");
    store.commit().expect("commit");
    drop(store);
    let (mut store, _) = PersistentLineageRegistry::open(&root).expect("recover");
    assert!(
        store
            .is_ancestor(&[1; 16], &[4; 16])
            .expect("historical ancestry")
    );
    assert!(
        !store
            .is_ancestor(&[4; 16], &[1; 16])
            .expect("reverse ancestry")
    );
    assert_eq!(
        store.revision_owner(&[2; 16]).expect("historical owner"),
        Some(vec![1; 16])
    );
}

#[test]
fn committed_v1_checkpoint_fixture_remains_readable() {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/lineage-checkpoint/v1");
    let temporary = tempfile::tempdir().expect("temporary");
    let destination = temporary.path().join("checkpoint");
    copy_fixture(&source, &destination);
    let (store, report) = PersistentLineageRegistry::open(&destination).expect("v1 fixture");
    assert_eq!(report.committed_events, 2);
    assert_eq!(report.committed_batches, 2);
    assert!(report.continuation);
    assert_eq!(
        store
            .artifact(&[1; 16])
            .expect("read")
            .expect("artifact")
            .next_sequence,
        3
    );
}
fn copy_fixture(source: &std::path::Path, destination: &std::path::Path) {
    fs::create_dir(destination).expect("fixture directory");
    for entry in fs::read_dir(source).expect("fixture") {
        let entry = entry.expect("entry");
        let target = destination.join(entry.file_name());
        if entry.file_type().expect("type").is_dir() {
            copy_fixture(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("fixture copy");
        }
    }
}
