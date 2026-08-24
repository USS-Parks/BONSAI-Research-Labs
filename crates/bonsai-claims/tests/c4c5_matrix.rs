use bonsai_claims::ClaimVerdict;
use bonsai_claims::c4c5::{CycleEvidence, adjudicate_c4_c5};
use bonsai_claims::matrix::{EvidenceCell, generate_claim_matrix, m3_seed_matrix};

#[allow(clippy::fn_params_excessive_bools, clippy::too_many_arguments)]
fn record(
    id: &str,
    c3: ClaimVerdict,
    families: u64,
    forward: bool,
    backward: bool,
    shortcut: bool,
    multiplicity: bool,
    mixed: bool,
    growth: bool,
    energy: u8,
    generations: u64,
    gain: Option<i64>,
) -> CycleEvidence {
    CycleEvidence {
        case_id: id.to_owned(),
        c3,
        families,
        forward_construction: forward,
        backward_credit: backward,
        one_score_only: shortcut,
        one_seed_only: false,
        one_family_only: false,
        artifact_count_only: false,
        scaled_compute: false,
        multiplicity_corrected: multiplicity,
        comparator_mixed: mixed,
        uncontrolled_growth: growth,
        energy_tier: energy,
        generations,
        net_gain: gain,
    }
}

fn corpus() -> Vec<CycleEvidence> {
    vec![
        record(
            "passable",
            ClaimVerdict::Pass,
            3,
            true,
            true,
            false,
            true,
            false,
            false,
            2,
            2,
            Some(4),
        ),
        record(
            "c3_missing",
            ClaimVerdict::Indeterminate,
            3,
            true,
            true,
            false,
            true,
            false,
            false,
            2,
            2,
            Some(4),
        ),
        record(
            "shortcut",
            ClaimVerdict::Pass,
            3,
            true,
            true,
            true,
            true,
            false,
            false,
            2,
            2,
            Some(4),
        ),
        record(
            "mixed",
            ClaimVerdict::Pass,
            3,
            true,
            true,
            false,
            true,
            true,
            false,
            2,
            2,
            Some(4),
        ),
        record(
            "multiplicity",
            ClaimVerdict::Pass,
            3,
            true,
            true,
            false,
            false,
            false,
            false,
            2,
            2,
            Some(4),
        ),
        record(
            "energy_e0",
            ClaimVerdict::Pass,
            3,
            true,
            true,
            false,
            true,
            false,
            false,
            0,
            2,
            Some(4),
        ),
        record(
            "growth",
            ClaimVerdict::Pass,
            3,
            true,
            true,
            false,
            true,
            false,
            true,
            2,
            2,
            Some(4),
        ),
    ]
}

#[test]
fn adversarial_c4_c5_fixtures_cannot_pass() {
    let table = adjudicate_c4_c5(&corpus()).expect("c4c5");
    let row = |id: &str| {
        table
            .rows
            .iter()
            .find(|row| row.case_id == id)
            .expect("row")
    };
    assert_eq!(row("passable").c4, ClaimVerdict::Pass);
    assert_eq!(row("passable").c5, ClaimVerdict::Pass);
    assert_eq!(row("c3_missing").c4, ClaimVerdict::Indeterminate);
    assert_eq!(row("shortcut").c4, ClaimVerdict::Fail);
    assert_eq!(row("mixed").c4, ClaimVerdict::Fail);
    assert_eq!(row("multiplicity").c4, ClaimVerdict::Fail);
    assert_eq!(row("energy_e0").c5, ClaimVerdict::Fail);
    assert_eq!(row("growth").c5, ClaimVerdict::Fail);
}

#[test]
fn uncovered_or_pass_without_hash_fails_evidence_check() {
    let matrix = generate_claim_matrix(m3_seed_matrix()).expect("seed");
    assert!(matrix.uncovered.is_empty());
    assert!(matrix.cells.iter().all(|cell| cell.status == "not-run"));
    let mut incomplete = m3_seed_matrix();
    incomplete.pop();
    assert!(generate_claim_matrix(incomplete).is_err());
    assert!(
        generate_claim_matrix(vec![EvidenceCell {
            claim: "C0".to_owned(),
            requirement: "x".to_owned(),
            rules: vec!["r".to_owned()],
            metrics: vec!["m".to_owned()],
            fixtures: vec!["f".to_owned()],
            status: "claimed".to_owned(),
            verdict: ClaimVerdict::Pass,
            evidence_sha256: None,
        }])
        .is_err()
    );
}
