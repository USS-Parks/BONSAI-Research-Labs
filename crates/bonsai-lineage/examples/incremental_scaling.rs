//! Bounded diagnostic for BX-07; each invocation owns one isolated workload.
use bonsai_contracts::bonsai::artifact::v1::artifact_lifecycle_event::Detail;
use bonsai_contracts::bonsai::artifact::v1::{
    ArtifactBirth, ArtifactCost, ArtifactLifecycleEvent, ArtifactRevision, ArtifactType,
    LineageRelation, ParentReference, Provenance,
};
use bonsai_contracts::bonsai::event::v1::Availability;
use bonsai_contracts::lineage::LineageValidationError;
use bonsai_lineage::ArtifactLifecycleRegistry;
use bonsai_platform::portable::collect_process_tree;
use serde_json::json;
use std::time::Instant;

fn id(value: u64) -> Vec<u8> {
    let mut value_bytes = vec![1; 16];
    value_bytes[..8].copy_from_slice(&value.to_le_bytes());
    value_bytes
}
fn provenance() -> Provenance {
    Provenance {
        producer_id: "BX-07-scaling".into(),
        producer_version: "1.0".into(),
        source_event_ids: vec![id(9_000_000)],
        method_ids: vec!["incremental-scaling/v1".into()],
    }
}
fn parent(value: u64) -> ParentReference {
    ParentReference {
        artifact_id: id(value),
        artifact_revision_id: id(1_000_000 + value),
        relation: LineageRelation::DerivedFrom as i32,
    }
}
fn birth(value: u64, parents: Vec<ParentReference>) -> ArtifactLifecycleEvent {
    ArtifactLifecycleEvent {
        artifact_id: id(value),
        artifact_revision_id: id(1_000_000 + value),
        lifecycle_sequence: 1,
        detail: Some(Detail::Birth(ArtifactBirth {
            artifact_type: ArtifactType::Feature as i32,
            representation_sha256: vec![1; 32],
            provenance: Some(provenance()),
            parents,
        })),
    }
}
fn cost(sequence: u64) -> ArtifactLifecycleEvent {
    ArtifactLifecycleEvent {
        artifact_id: id(1),
        artifact_revision_id: id(1_000_001),
        lifecycle_sequence: sequence,
        detail: Some(Detail::Cost(ArtifactCost {
            cost_entry_id: id(2_000_000 + sequence),
            counter_id: "work_items".into(),
            unit: "1".into(),
            amount: Some(1),
            availability: Availability::Measured as i32,
            estimator_id: None,
            estimator_version: None,
            unavailable_reason: None,
            provenance: Some(provenance()),
        })),
    }
}
fn event(case: &str, index: u64) -> ArtifactLifecycleEvent {
    match case {
        "history" if index > 1 => cost(index),
        "chain" if index > 1 => birth(index, vec![parent(index - 1)]),
        "branch" if index == 3 => birth(index, vec![parent(2)]),
        "branch" if index > 3 => birth(index, vec![parent(index - 1), parent(index - 2)]),
        _ => birth(index, vec![]),
    }
}
fn probe(case: &str, size: u64) -> ArtifactLifecycleEvent {
    if case == "history" {
        return cost(size + 1);
    }
    if case == "roots" {
        return birth(size + 1, vec![]);
    }
    ArtifactLifecycleEvent {
        artifact_id: id(1),
        artifact_revision_id: id(3_000_000),
        lifecycle_sequence: 2,
        detail: Some(Detail::Revision(ArtifactRevision {
            previous_revision_id: id(1_000_001),
            representation_sha256: vec![2; 32],
            parents: vec![parent(size)],
            provenance: Some(provenance()),
        })),
    }
}
fn run() -> Result<(), String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let [case, size] = args.as_slice() else {
        return Err("expected CASE SIZE".into());
    };
    let size = size.parse::<u64>().map_err(|error| error.to_string())?;
    if !["history", "roots", "chain", "branch"].contains(&case.as_str())
        || ![1000, 10_000, 100_000].contains(&size)
    {
        return Err("unsupported bounded scaling workload".into());
    }
    let pid = std::process::id();
    let before = collect_process_tree(pid).map_err(|error| error.to_string())?;
    let mut registry = ArtifactLifecycleRegistry::new();
    let began = Instant::now();
    for index in 1..=size {
        let (result, work) = registry.apply_measured(event(case, index));
        result.map_err(|error| error.to_string())?;
        if work.events != 1 || work.whole_graph_scans != 0 || work.graph_clones != 0 {
            return Err("prefix replay entered incremental construction".into());
        }
    }
    let build_ns = began.elapsed().as_nanos();
    let after_build = collect_process_tree(pid).map_err(|error| error.to_string())?;
    let candidate = probe(case, size);
    let began = Instant::now();
    let (result, work) = registry.apply_measured(candidate);
    let probe_ns = began.elapsed().as_nanos();
    let expected = if case == "chain" {
        Err(LineageValidationError::LineageCycle)
    } else {
        Ok(())
    };
    if result != expected
        || work.events != 1
        || work.whole_graph_scans != 0
        || work.graph_clones != 0
    {
        return Err("probe result or prefix-work invariant failed".into());
    }
    let expected_ancestry = match case.as_str() {
        "chain" => size,
        "branch" => size - 1,
        _ => 0,
    };
    if work.ancestry_nodes != expected_ancestry {
        return Err("probe visited unexpected ancestry".into());
    }
    let after_probe = collect_process_tree(pid).map_err(|error| error.to_string())?;
    println!(
        "{}",
        json!({
            "format":"bonsai.lineage-scaling/v1","case":case,"prefix_events":size,
            "os":std::env::consts::OS,"architecture":std::env::consts::ARCH,"debug_assertions":cfg!(debug_assertions),
            "build_ns":build_ns,"probe_ns":probe_ns,
            "rss_before_bytes":before.resident_memory_bytes,"rss_after_build_bytes":after_build.resident_memory_bytes,
            "rss_after_probe_bytes":after_probe.resident_memory_bytes,
            "rss_semantics":after_probe.memory_semantics,"rss_scope":"current process-tree snapshots; not peak RSS",
            "probe_accepted":result.is_ok(),"probe_error":result.err().map(|error|error.to_string()),
            "validation_work":work,
            "retention_scope":"full accepted event/history state remains retained; bounded persistence is BX-08",
            "complexity_scope":"validation probes and visited ancestry, excluding map internals, payload copy and storage append"
        })
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
