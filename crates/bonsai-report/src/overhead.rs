//! Paired no/minimal/full instrumentation overhead (BV-10).

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

const THROUGHPUT_CEILING_PPM: u64 = 50_000;
const LATENCY_CEILING_PPM: u64 = 100_000;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OverheadPair {
    pub series: String,
    pub none: u64,
    pub minimal: u64,
    pub full: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OverheadReport {
    pub schema: String,
    pub throughput_ppm: u64,
    pub latency_ppm: u64,
    pub cpu_ppm: u64,
    pub memory_ppm: u64,
    pub storage_ppm: u64,
    pub energy_ppm: Option<u64>,
    pub within_d11: bool,
    pub evidence_disabled: bool,
    pub detail_code: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverheadError {
    Identity,
    DisabledEvidence,
}

impl fmt::Display for OverheadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "OVERHEAD_IDENTITY_INVALID",
            Self::DisabledEvidence => "OVERHEAD_EVIDENCE_DISABLED",
        })
    }
}

impl Error for OverheadError {}

/// Quantify paired overhead. Disabling required evidence cannot pass D-11.
///
/// # Errors
///
/// Rejects missing series or an attempt to disable required evidence.
pub fn accept_overhead(
    pairs: &[OverheadPair],
    evidence_disabled: bool,
) -> Result<OverheadReport, OverheadError> {
    if evidence_disabled {
        return Err(OverheadError::DisabledEvidence);
    }
    let throughput = find(pairs, "throughput")?;
    let latency = find(pairs, "p95_latency")?;
    let cpu = find(pairs, "cpu")?;
    let memory = find(pairs, "memory")?;
    let storage = find(pairs, "storage")?;
    let energy = pairs.iter().find(|pair| pair.series == "energy");
    let throughput_ppm = delta_ppm(throughput.none, throughput.full);
    let latency_ppm = delta_ppm(latency.none, latency.full);
    let within_d11 = throughput_ppm <= THROUGHPUT_CEILING_PPM && latency_ppm <= LATENCY_CEILING_PPM;
    Ok(OverheadReport {
        schema: "bonsai.overhead-acceptance/v1".to_owned(),
        throughput_ppm,
        latency_ppm,
        cpu_ppm: delta_ppm(cpu.none, cpu.full),
        memory_ppm: delta_ppm(memory.none, memory.full),
        storage_ppm: delta_ppm(storage.none, storage.full),
        energy_ppm: energy.map(|pair| delta_ppm(pair.none, pair.full)),
        within_d11,
        evidence_disabled: false,
        detail_code: if within_d11 {
            "OVERHEAD_WITHIN_D11".to_owned()
        } else {
            "OVERHEAD_EXCEEDS_D11".to_owned()
        },
    })
}

fn find<'a>(pairs: &'a [OverheadPair], series: &str) -> Result<&'a OverheadPair, OverheadError> {
    pairs
        .iter()
        .find(|pair| pair.series == series)
        .ok_or(OverheadError::Identity)
}

fn delta_ppm(none: u64, full: u64) -> u64 {
    if none == 0 {
        return 1_000_000;
    }
    full.abs_diff(none).saturating_mul(1_000_000) / none
}
