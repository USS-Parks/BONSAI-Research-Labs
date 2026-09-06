use bonsai_contracts::bonsai::artifact::v1::{
    ArtifactBirth, ArtifactCost, ArtifactLifecycleEvent, ArtifactType, Provenance,
    artifact_lifecycle_event::Detail,
};
use bonsai_contracts::bonsai::event::v1::Availability;
use bonsai_lineage::persistent::{PersistentLimits, PersistentLineageRegistry};
use bonsai_platform::portable::collect_process_tree;
use serde_json::json;
use std::time::Instant;

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

fn run() -> Result<(), String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let [operation, root, value] = args.as_slice() else {
        return Err("expected OPERATION ROOT VALUE".into());
    };
    let before = collect_process_tree(std::process::id()).map_err(|e| e.to_string())?;
    let began = Instant::now();
    if operation == "recover" {
        let (store, report) = PersistentLineageRegistry::open(root)?;
        let after = collect_process_tree(std::process::id()).map_err(|e| e.to_string())?;
        println!(
            "{}",
            json!({"operation":operation,"report":report,"checkpoint":store.checkpoint()?,
            "elapsed_ns":began.elapsed().as_nanos(),"rss_before_bytes":before.resident_memory_bytes,
            "rss_after_bytes":after.resident_memory_bytes,"os":std::env::consts::OS})
        );
        return Ok(());
    }
    let mut bounds = limits();
    bounds.maximum_live_artifacts = 1;
    bounds.database_bytes = 32 * 1024 * 1024;
    bounds.output_bytes = 256 * 1024 * 1024;
    let mut store = PersistentLineageRegistry::create(root, bounds)?;
    if operation == "crash" {
        if ![
            "before_segment_commit",
            "after_segment_commit",
            "before_index_commit",
            "after_index_commit",
        ]
        .contains(&value.as_str())
        {
            return Err("unknown crash boundary".into());
        }
        store.append(&event(1))?;
        store.commit()?;
        store.append(&event(2))?;
        store.commit_with(|at| {
            if at == value {
                std::process::exit(73);
            }
            Ok(())
        })?;
        return Err("crash boundary was not reached".into());
    }
    let count = value.parse::<u64>().map_err(|e| e.to_string())?;
    if operation != "grow" || ![1000, 10_000, 100_000].contains(&count) {
        return Err("unsupported workload".into());
    }
    for sequence in 1..=count {
        store.append(&event(sequence))?;
        if sequence.is_multiple_of(1000) {
            store.commit()?;
        }
    }
    let checkpoint = store.commit()?;
    let after = collect_process_tree(std::process::id()).map_err(|e| e.to_string())?;
    println!(
        "{}",
        json!({"operation":operation,"events":count,"checkpoint":checkpoint,
        "elapsed_ns":began.elapsed().as_nanos(),"rss_before_bytes":before.resident_memory_bytes,
        "rss_after_bytes":after.resident_memory_bytes,"os":std::env::consts::OS,
        "live_artifact_cap":bounds.maximum_live_artifacts,"maximum_batch_events":1000,
        "rss_scope":"current process-tree snapshots, not peak; complete history on disk"})
    );
    Ok(())
}
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
