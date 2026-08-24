//! Claim-to-evidence matrix generator (BV-07).

use crate::ClaimVerdict;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceCell {
    pub claim: String,
    pub requirement: String,
    pub rules: Vec<String>,
    pub metrics: Vec<String>,
    pub fixtures: Vec<String>,
    pub status: String,
    pub verdict: ClaimVerdict,
    pub evidence_sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimMatrix {
    pub schema: String,
    pub cells: Vec<EvidenceCell>,
    pub uncovered: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatrixError {
    Identity,
    Uncovered,
}

impl fmt::Display for MatrixError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "CLAIM_MATRIX_IDENTITY_INVALID",
            Self::Uncovered => "CLAIM_MATRIX_REQUIREMENT_UNCOVERED",
        })
    }
}

impl Error for MatrixError {}

const REQUIRED: [&str; 7] = ["C0", "C1", "C2", "C3", "C4", "C5", "INSTRUMENT"];

/// Map every required claim to hashed evidence or explicit not-run/indeterminate.
///
/// # Errors
///
/// Fails closed when a required claim is missing or a cell has no evidence
/// identity and is not explicitly not-run or indeterminate.
pub fn generate_claim_matrix(cells: Vec<EvidenceCell>) -> Result<ClaimMatrix, MatrixError> {
    let mut seen = BTreeSet::new();
    for cell in &cells {
        if cell.claim.is_empty() || cell.requirement.is_empty() || !seen.insert(cell.claim.as_str())
        {
            return Err(MatrixError::Identity);
        }
        let explicit = cell.status == "not-run" || cell.verdict == ClaimVerdict::Indeterminate;
        if cell.evidence_sha256.is_none() && !explicit {
            return Err(MatrixError::Uncovered);
        }
        if cell.verdict == ClaimVerdict::Pass && cell.evidence_sha256.is_none() {
            return Err(MatrixError::Uncovered);
        }
    }
    let uncovered = REQUIRED
        .into_iter()
        .filter(|claim| !seen.contains(claim))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if !uncovered.is_empty() {
        return Err(MatrixError::Uncovered);
    }
    Ok(ClaimMatrix {
        schema: "bonsai.claim-evidence-matrix/v1".to_owned(),
        cells,
        uncovered: Vec::new(),
    })
}

/// Stable SHA-256 of a fixture or rule identity.
#[must_use]
pub fn evidence_hash(identity: &str) -> String {
    let digest = Sha256::digest(identity.as_bytes());
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut encoded, "{byte:02x}").expect("hex");
    }
    encoded
}

/// Seed matrix used by M3: C0–C5 rules exist; instrument completion stays not-run.
#[must_use]
pub fn m3_seed_matrix() -> Vec<EvidenceCell> {
    [
        (
            "C0",
            "valid provenance",
            "c0c1",
            "not-run",
            ClaimVerdict::Indeterminate,
        ),
        (
            "C1",
            "enforceable budgets",
            "c0c1",
            "not-run",
            ClaimVerdict::Indeterminate,
        ),
        (
            "C2",
            "continual adaptation",
            "c2c3",
            "not-run",
            ClaimVerdict::Indeterminate,
        ),
        (
            "C3",
            "abstraction utility",
            "c2c3",
            "not-run",
            ClaimVerdict::Indeterminate,
        ),
        (
            "C4",
            "construction plus credit",
            "c4c5",
            "not-run",
            ClaimVerdict::Indeterminate,
        ),
        (
            "C5",
            "multigenerational gain",
            "c4c5",
            "not-run",
            ClaimVerdict::Indeterminate,
        ),
        (
            "INSTRUMENT",
            "instrument completion",
            "parked-m4",
            "not-run",
            ClaimVerdict::Indeterminate,
        ),
    ]
    .into_iter()
    .map(
        |(claim, requirement, rules, status, verdict)| EvidenceCell {
            claim: claim.to_owned(),
            requirement: requirement.to_owned(),
            rules: vec![rules.to_owned()],
            metrics: vec!["registry/v1".to_owned()],
            fixtures: vec![format!("{}-rules", claim.to_ascii_lowercase())],
            status: status.to_owned(),
            verdict,
            evidence_sha256: None,
        },
    )
    .collect()
}
