//! Platform-neutral resource measurement contracts.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

pub mod calibration;
pub mod clock;
pub mod portable;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    WallTime,
    MonotonicTime,
    CpuTime,
    Memory,
    Storage,
    IoRead,
    IoWrite,
    ProcessCount,
    AcceleratorTime,
    OperationCount,
    Energy,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceScope {
    AgentProcess,
    AgentProcessTree,
    AgentStorage,
    Observer,
    AcceleratorDevice,
    System,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SampleStatus {
    Measured,
    Estimated,
    Unavailable,
    Error,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceValue {
    pub amount: u64,
    pub unit: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SampleProvenance {
    pub collector_id: String,
    pub collector_version: String,
    pub backend_id: String,
    pub method: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SampleUncertainty {
    pub absolute: u64,
    pub parts_per_million: u32,
    pub basis: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceSample {
    pub counter_id: String,
    pub kind: ResourceKind,
    pub scope: ResourceScope,
    pub status: SampleStatus,
    pub value: Option<ResourceValue>,
    pub sampled_monotonic_time_ns: u64,
    pub resolution: Option<ResourceValue>,
    pub uncertainty: Option<SampleUncertainty>,
    pub provenance: SampleProvenance,
    pub estimator_id: Option<String>,
    pub detail_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceSampleBatch {
    pub schema: String,
    pub samples: Vec<ResourceSample>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CounterDescriptor {
    pub counter_id: &'static str,
    pub kind: ResourceKind,
    pub scope: ResourceScope,
    pub unit: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectorDescriptor {
    pub collector_id: &'static str,
    pub counters: &'static [CounterDescriptor],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceSampleError {
    Batch,
    Identity,
    Time,
    Value,
    Resolution,
    Provenance,
    Estimator,
    Detail,
    DuplicateCounter,
    Coverage,
}

impl fmt::Display for ResourceSampleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Batch => "RESOURCE_SAMPLE_BATCH_INVALID",
            Self::Identity => "RESOURCE_SAMPLE_IDENTITY_INVALID",
            Self::Time => "RESOURCE_SAMPLE_TIME_INVALID",
            Self::Value => "RESOURCE_SAMPLE_VALUE_INVALID",
            Self::Resolution => "RESOURCE_SAMPLE_RESOLUTION_INVALID",
            Self::Provenance => "RESOURCE_SAMPLE_PROVENANCE_INVALID",
            Self::Estimator => "RESOURCE_SAMPLE_ESTIMATOR_INVALID",
            Self::Detail => "RESOURCE_SAMPLE_DETAIL_INVALID",
            Self::DuplicateCounter => "RESOURCE_SAMPLE_DUPLICATE_COUNTER",
            Self::Coverage => "RESOURCE_SAMPLE_COVERAGE_INVALID",
        })
    }
}

impl Error for ResourceSampleError {}

impl ResourceSample {
    /// Validate one sample without interpreting an absent counter as zero.
    ///
    /// # Errors
    ///
    /// Returns a stable error for contradictory status, value, resolution,
    /// uncertainty, estimator, detail, timestamp, or provenance fields.
    pub fn validate(&self) -> Result<(), ResourceSampleError> {
        if !valid_id(&self.counter_id) {
            return Err(ResourceSampleError::Identity);
        }
        if self.sampled_monotonic_time_ns == 0 {
            return Err(ResourceSampleError::Time);
        }
        if !valid_id(&self.provenance.collector_id)
            || !valid_version(&self.provenance.collector_version)
            || !valid_id(&self.provenance.backend_id)
            || self.provenance.method.is_empty()
        {
            return Err(ResourceSampleError::Provenance);
        }

        match self.status {
            SampleStatus::Measured => {
                validate_available_value(self)?;
                if self.estimator_id.is_some() || self.detail_code.is_some() {
                    return Err(ResourceSampleError::Estimator);
                }
            }
            SampleStatus::Estimated => {
                validate_available_value(self)?;
                if self.estimator_id.as_deref().is_none_or(|id| !valid_id(id))
                    || self.uncertainty.is_none()
                    || self.detail_code.is_some()
                {
                    return Err(ResourceSampleError::Estimator);
                }
            }
            SampleStatus::Unavailable | SampleStatus::Error => {
                if self.value.is_some()
                    || self.resolution.is_some()
                    || self.uncertainty.is_some()
                    || self.estimator_id.is_some()
                {
                    return Err(ResourceSampleError::Value);
                }
                if self
                    .detail_code
                    .as_deref()
                    .is_none_or(|code| !valid_code(code))
                {
                    return Err(ResourceSampleError::Detail);
                }
            }
        }
        Ok(())
    }
}

impl ResourceSampleBatch {
    /// Validate a non-empty, versioned batch with one outcome per counter.
    ///
    /// # Errors
    ///
    /// Returns a stable error for the batch or its first invalid sample.
    pub fn validate(&self) -> Result<(), ResourceSampleError> {
        if self.schema != "bonsai.resource-sample-batch/v1" || self.samples.is_empty() {
            return Err(ResourceSampleError::Batch);
        }
        let mut counters = BTreeSet::new();
        for sample in &self.samples {
            sample.validate()?;
            if !counters.insert(sample.counter_id.as_str()) {
                return Err(ResourceSampleError::DuplicateCounter);
            }
        }
        Ok(())
    }
}

pub trait ResourceCollector {
    type Error: Error;

    fn descriptor(&self) -> &'static CollectorDescriptor;

    /// Collect one explicit outcome for every advertised counter.
    ///
    /// # Errors
    ///
    /// Returns the backend's bounded collection failure. Individual counter
    /// unavailability and read errors belong in a successful sample batch.
    fn sample(&mut self, monotonic_time_ns: u64) -> Result<ResourceSampleBatch, Self::Error>;
}

/// Prove that a collector returned exactly one explicit outcome for every
/// advertised counter and did not change its kind, scope, or unit.
///
/// # Errors
///
/// Returns a stable coverage error for omitted, invented, or contradictory
/// counter results, after validating the batch itself.
pub fn validate_collector_output(
    descriptor: &CollectorDescriptor,
    batch: &ResourceSampleBatch,
) -> Result<(), ResourceSampleError> {
    batch.validate()?;
    if descriptor.collector_id.is_empty()
        || descriptor.counters.len() != batch.samples.len()
        || descriptor.counters.is_empty()
    {
        return Err(ResourceSampleError::Coverage);
    }
    for expected in descriptor.counters {
        let Some(actual) = batch
            .samples
            .iter()
            .find(|sample| sample.counter_id == expected.counter_id)
        else {
            return Err(ResourceSampleError::Coverage);
        };
        if actual.kind != expected.kind
            || actual.scope != expected.scope
            || actual.provenance.collector_id != descriptor.collector_id
            || actual
                .value
                .as_ref()
                .is_some_and(|value| value.unit != expected.unit)
            || actual
                .resolution
                .as_ref()
                .is_some_and(|value| value.unit != expected.unit)
        {
            return Err(ResourceSampleError::Coverage);
        }
    }
    Ok(())
}

fn validate_available_value(sample: &ResourceSample) -> Result<(), ResourceSampleError> {
    let value = sample.value.as_ref().ok_or(ResourceSampleError::Value)?;
    if value.unit.is_empty() {
        return Err(ResourceSampleError::Value);
    }
    let resolution = sample
        .resolution
        .as_ref()
        .ok_or(ResourceSampleError::Resolution)?;
    if resolution.amount == 0 || resolution.unit != value.unit {
        return Err(ResourceSampleError::Resolution);
    }
    if let Some(uncertainty) = &sample.uncertainty
        && (uncertainty.basis.is_empty() || uncertainty.parts_per_million > 1_000_000)
    {
        return Err(ResourceSampleError::Value);
    }
    Ok(())
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

fn valid_version(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    (2..=3).contains(&parts.len())
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

fn valid_code(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

#[cfg(test)]
mod tests {
    use super::{
        CollectorDescriptor, CounterDescriptor, ResourceKind, ResourceSample, ResourceSampleBatch,
        ResourceSampleError, ResourceScope, ResourceValue, SampleProvenance, SampleStatus,
        SampleUncertainty, validate_collector_output,
    };

    fn sample(status: SampleStatus, amount: Option<u64>) -> ResourceSample {
        ResourceSample {
            counter_id: "process.cpu_time".to_owned(),
            kind: ResourceKind::CpuTime,
            scope: ResourceScope::AgentProcessTree,
            status,
            value: amount.map(|amount| ResourceValue {
                amount,
                unit: "ns".to_owned(),
            }),
            sampled_monotonic_time_ns: 100,
            resolution: amount.map(|_| ResourceValue {
                amount: 1,
                unit: "ns".to_owned(),
            }),
            uncertainty: None,
            provenance: SampleProvenance {
                collector_id: "fixture".to_owned(),
                collector_version: "1.0".to_owned(),
                backend_id: "portable".to_owned(),
                method: "fixture counter".to_owned(),
            },
            estimator_id: None,
            detail_code: None,
        }
    }

    #[test]
    fn zero_is_a_measured_value_not_missingness() {
        let measured_zero = sample(SampleStatus::Measured, Some(0));
        assert_eq!(measured_zero.validate(), Ok(()));
        assert_eq!(measured_zero.value.as_ref().expect("value").amount, 0);

        let mut unavailable = sample(SampleStatus::Unavailable, None);
        unavailable.detail_code = Some("COUNTER_UNSUPPORTED".to_owned());
        assert_eq!(unavailable.validate(), Ok(()));
        assert_ne!(measured_zero, unavailable);
    }

    #[test]
    fn all_four_backend_outcomes_are_explicit_and_validated() {
        let measured = sample(SampleStatus::Measured, Some(7));
        let mut estimated = sample(SampleStatus::Estimated, Some(7));
        estimated.estimator_id = Some("portable-estimator".to_owned());
        estimated.uncertainty = Some(SampleUncertainty {
            absolute: 2,
            parts_per_million: 250_000,
            basis: "calibration".to_owned(),
        });
        let mut unavailable = sample(SampleStatus::Unavailable, None);
        unavailable.detail_code = Some("COUNTER_UNSUPPORTED".to_owned());
        let mut error = sample(SampleStatus::Error, None);
        error.detail_code = Some("COUNTER_READ_FAILED".to_owned());

        for outcome in [&measured, &estimated, &unavailable, &error] {
            assert_eq!(outcome.validate(), Ok(()));
        }
    }

    #[test]
    fn contradictory_missingness_and_duplicate_counters_fail() {
        let mut unavailable_with_zero = sample(SampleStatus::Unavailable, Some(0));
        unavailable_with_zero.detail_code = Some("COUNTER_UNSUPPORTED".to_owned());
        assert_eq!(
            unavailable_with_zero.validate(),
            Err(ResourceSampleError::Value)
        );

        let measured = sample(SampleStatus::Measured, Some(1));
        let batch = ResourceSampleBatch {
            schema: "bonsai.resource-sample-batch/v1".to_owned(),
            samples: vec![measured.clone(), measured],
        };
        assert_eq!(batch.validate(), Err(ResourceSampleError::DuplicateCounter));
    }

    #[test]
    fn collector_coverage_requires_one_explicit_outcome_per_advertised_counter() {
        static COUNTERS: [CounterDescriptor; 2] = [
            CounterDescriptor {
                counter_id: "process.cpu_time",
                kind: ResourceKind::CpuTime,
                scope: ResourceScope::AgentProcessTree,
                unit: "ns",
            },
            CounterDescriptor {
                counter_id: "process.energy",
                kind: ResourceKind::Energy,
                scope: ResourceScope::AgentProcessTree,
                unit: "uJ",
            },
        ];
        let descriptor = CollectorDescriptor {
            collector_id: "fixture",
            counters: &COUNTERS,
        };
        let cpu = sample(SampleStatus::Measured, Some(0));
        let mut energy = sample(SampleStatus::Unavailable, None);
        energy.counter_id = "process.energy".to_owned();
        energy.kind = ResourceKind::Energy;
        energy.detail_code = Some("COUNTER_UNSUPPORTED".to_owned());
        let complete = ResourceSampleBatch {
            schema: "bonsai.resource-sample-batch/v1".to_owned(),
            samples: vec![cpu.clone(), energy],
        };
        assert_eq!(validate_collector_output(&descriptor, &complete), Ok(()));

        let incomplete = ResourceSampleBatch {
            schema: "bonsai.resource-sample-batch/v1".to_owned(),
            samples: vec![cpu],
        };
        assert_eq!(
            validate_collector_output(&descriptor, &incomplete),
            Err(ResourceSampleError::Coverage)
        );
    }
}
