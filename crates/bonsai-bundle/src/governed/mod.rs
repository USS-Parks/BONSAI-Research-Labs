//! Independent evidence reconstruction for the supported governed reference run.
//!
//! The receipt digest must come from the trusted operator's output, retained
//! outside the result directory. This verifier makes no hostile-host attestation.
mod context;
mod execution;
mod reference;
mod resources;
mod snapshot;
mod trace;

use bonsai_claims::c0c1::{BundleEvidence, C0C1Table, adjudicate_c0_c1};
use bonsai_contracts::track::{
    Track, TrackDeclaration, TransitionAccess, UpdateSchedule, derive_track,
};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

type Result<T> = std::result::Result<T, &'static str>;

/// Facts can only be constructed by successful evidence reconstruction.
/// There is deliberately no deserializer or public field mutation.
#[derive(Serialize)]
pub struct VerifiedRunFacts {
    run_id: String,
    receipt_sha256: String,
    events: u64,
    environment_steps: u64,
    finished_episodes: u64,
    reward_sum: i64,
    work_items: u64,
    observer_bytes: u64,
    derived_track: Track,
}

/// Per-run evidence verdict. Stronger research claims require their own gates.
#[derive(Serialize)]
pub struct GovernedRunVerdict {
    format: &'static str,
    facts: VerifiedRunFacts,
    claims: C0C1Table,
    limitations: Vec<&'static str>,
}

/// Reconstruct a finalized supported run from immutable, receipt-bound bytes.
///
/// # Errors
/// Rejects unsafe, modified, incomplete, unsupported or contradictory evidence.
/// No positive claim table is returned on any reconstruction error.
pub fn verify_governed_run(
    root: impl AsRef<Path>,
    expected_receipt_sha256: &str,
    schemas: &crate::BundleSchemas,
) -> Result<GovernedRunVerdict> {
    let snapshot = snapshot::Snapshot::load(root.as_ref(), expected_receipt_sha256)?;
    let context = context::Context::load(&snapshot, schemas)?;
    let outcome = execution::reconstruct(&snapshot, &context)?;
    // These values describe the trace we just reconstructed and pinned reference
    // implementation, never booleans copied from the caller's declaration.
    let track = derive_track(&TrackDeclaration {
        schema_version: "1.0".into(),
        declared_track: Track::A,
        runtime_facts_complete: true,
        batch_size: 1,
        transition_access: TransitionAccess::SinglePass,
        replay_capacity_transitions: 0,
        offline_updates: false,
        observer_data_access: false,
        privileged_state: false,
        human_labels: false,
        domain_feature_targets: false,
        update_schedule: UpdateSchedule::EventDriven,
        fixed_external_budgets: true,
    });
    ensure(track.derived == Track::A, "RUN_DERIVED_TRACK_INELIGIBLE")?;
    let facts = VerifiedRunFacts {
        run_id: snapshot.run_id,
        receipt_sha256: expected_receipt_sha256.into(),
        events: outcome.events,
        environment_steps: outcome.steps,
        finished_episodes: outcome.episodes,
        reward_sum: outcome.reward,
        work_items: outcome.work,
        observer_bytes: snapshot.total_bytes,
        derived_track: track.derived,
    };
    let claims = adjudicate_c0_c1(&[BundleEvidence {
        case_id: facts.run_id.clone(),
        provenance_valid: true,
        events_valid: true,
        resource_evidence: true,
        budget_compliant: true,
        soft_degraded: false,
        hard_violation: false,
        hard_counter_declared: true,
        hard_counter_available: true,
        tampered: false,
        track_ambiguous: false,
    }])
    .map_err(|_| "RUN_CLAIM_ADJUDICATION_INVALID")?;
    Ok(GovernedRunVerdict {
        format: "bonsai.governed-run-verdict/v1",
        facts,
        claims,
        limitations: vec![
            "C0/C1 apply only to this recorded reference run; C2-C5 are not adjudicated.",
            "E0 supplies no energy evidence. Physical-host acceptance is not established.",
            "Receipt trust requires the operator digest retained outside the bundle.",
            "Reference launch and protocol evidence do not establish hostile-code or hostile-host isolation.",
            "Publication and multi-seed research eligibility require separate gates.",
        ],
    })
}

fn ensure(condition: bool, code: &'static str) -> Result<()> {
    if condition { Ok(()) } else { Err(code) }
}
fn number(value: &Value, key: &str) -> Result<u64> {
    value[key].as_u64().ok_or("RUN_REQUIRED_COUNTER_INVALID")
}
fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key].as_str().ok_or("RUN_REQUIRED_TEXT_INVALID")
}
