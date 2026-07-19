//! Deterministic admission decisions derived from immutable projections.

use crate::{LimitProjection, ScopeProjection, TypedAmount};
use bonsai_contracts::resource::{BudgetScope, DecisionOutcome, WorkClass};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionPolicyReference {
    pub policy_id: String,
    pub policy_version: String,
    pub canonical_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionInput {
    pub decision_id: String,
    pub monotonic_time_ns: u64,
    pub policy: DecisionPolicyReference,
    pub work_class: WorkClass,
    pub request: TypedAmount,
    pub projections: Vec<ScopeProjection>,
    pub next_rolling_release_ns: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ThrottleAllocation {
    pub counter_id: String,
    pub unit: String,
    pub admitted_amount: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionBody {
    pub schema: String,
    pub decision_id: String,
    pub monotonic_time_ns: u64,
    pub policy: DecisionPolicyReference,
    pub work_class: WorkClass,
    pub request: TypedAmount,
    pub projections: Vec<ScopeProjection>,
    pub outcome: DecisionOutcome,
    pub reason_code: String,
    pub defer_until_monotonic_time_ns: Option<u64>,
    pub throttle: Option<ThrottleAllocation>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionEvidence {
    pub body: DecisionBody,
    pub canonical_sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionError {
    Identity,
    Policy,
    Time,
    Projection,
    Arithmetic,
    Serialization,
}

impl fmt::Display for DecisionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "GOVERNOR_DECISION_IDENTITY_INVALID",
            Self::Policy => "GOVERNOR_DECISION_POLICY_INVALID",
            Self::Time => "GOVERNOR_DECISION_TIME_INVALID",
            Self::Projection => "GOVERNOR_DECISION_PROJECTION_INVALID",
            Self::Arithmetic => "GOVERNOR_DECISION_ARITHMETIC_FAILED",
            Self::Serialization => "GOVERNOR_DECISION_SERIALIZATION_FAILED",
        })
    }
}

impl Error for DecisionError {}

/// Produce canonical deterministic decision evidence.
///
/// Outcome precedence is missing hard measurement, hard overage, rolling soft
/// deferral, non-rolling soft throttle, then admission. The caller cannot
/// override the derived outcome or reason.
///
/// # Errors
///
/// Returns a stable error for malformed identity/policy/time, incomplete or
/// contradictory projections, arithmetic failure, or serialization failure.
pub fn decide(input: DecisionInput) -> Result<DecisionEvidence, DecisionError> {
    validate_input(&input)?;
    let mut projections = input.projections;
    projections.sort_by(|left, right| left.limit_id.cmp(&right.limit_id));

    let (outcome, reason_code, defer_until, throttle) = if projections
        .iter()
        .any(|projection| projection.state == LimitProjection::MeasurementUnavailable)
    {
        (
            DecisionOutcome::Reject,
            "HARD_COUNTER_UNAVAILABLE",
            None,
            None,
        )
    } else if projections
        .iter()
        .any(|projection| projection.state == LimitProjection::HardExceeded)
    {
        (DecisionOutcome::Reject, "HARD_LIMIT_EXCEEDED", None, None)
    } else if projections.iter().any(|projection| {
        projection.scope == BudgetScope::RollingWindow
            && projection.state == LimitProjection::SoftExceeded
    }) {
        let release = input
            .next_rolling_release_ns
            .filter(|release| *release > input.monotonic_time_ns)
            .ok_or(DecisionError::Time)?;
        (
            DecisionOutcome::Defer,
            "ROLLING_SOFT_LIMIT_DEFERRED",
            Some(release),
            None,
        )
    } else if projections
        .iter()
        .any(|projection| projection.state == LimitProjection::SoftExceeded)
    {
        let admitted_amount = projections
            .iter()
            .filter(|projection| projection.state == LimitProjection::SoftExceeded)
            .filter_map(|projection| {
                projection
                    .consumed_before
                    .map(|consumed| projection.soft_limit.saturating_sub(consumed))
            })
            .min()
            .unwrap_or(0);
        if admitted_amount == 0 || admitted_amount >= input.request.amount {
            return Err(DecisionError::Arithmetic);
        }
        (
            DecisionOutcome::Throttle,
            "SOFT_LIMIT_THROTTLED",
            None,
            Some(ThrottleAllocation {
                counter_id: input.request.key.counter_id.clone(),
                unit: input.request.key.unit.clone(),
                admitted_amount,
            }),
        )
    } else {
        (DecisionOutcome::Admit, "WITHIN_ALL_LIMITS", None, None)
    };

    let body = DecisionBody {
        schema: "bonsai.governor-decision/v1".to_owned(),
        decision_id: input.decision_id,
        monotonic_time_ns: input.monotonic_time_ns,
        policy: input.policy,
        work_class: input.work_class,
        request: input.request,
        projections,
        outcome,
        reason_code: reason_code.to_owned(),
        defer_until_monotonic_time_ns: defer_until,
        throttle,
    };
    let canonical = serde_json::to_vec(&body).map_err(|_| DecisionError::Serialization)?;
    Ok(DecisionEvidence {
        body,
        canonical_sha256: sha256_hex(&canonical),
    })
}

impl DecisionEvidence {
    /// Serialize the evidence with stable struct field order.
    ///
    /// # Errors
    ///
    /// Returns a stable error if serialization unexpectedly fails.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DecisionError> {
        serde_json::to_vec(self).map_err(|_| DecisionError::Serialization)
    }
}

fn validate_input(input: &DecisionInput) -> Result<(), DecisionError> {
    if input.decision_id.is_empty() || input.request.key.counter_id.is_empty() {
        return Err(DecisionError::Identity);
    }
    if input.monotonic_time_ns == 0 {
        return Err(DecisionError::Time);
    }
    if input.policy.policy_id.is_empty()
        || input.policy.policy_version.is_empty()
        || input.policy.canonical_sha256.len() != 64
        || !input
            .policy
            .canonical_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(DecisionError::Policy);
    }
    if input.projections.is_empty()
        || input.projections.iter().any(|projection| {
            projection.requested != input.request.amount
                || projection.limit_id.is_empty()
                || projection.hard_limit == 0
                || projection.soft_limit > projection.hard_limit
                || match projection.state {
                    LimitProjection::MeasurementUnavailable => {
                        projection.consumed_before.is_some() || projection.projected.is_some()
                    }
                    LimitProjection::WithinSoft => {
                        projection
                            .projected
                            .is_none_or(|projected| projected > projection.soft_limit)
                            || projection.consumed_before.is_none()
                    }
                    LimitProjection::SoftExceeded => {
                        projection.projected.is_none_or(|projected| {
                            projected <= projection.soft_limit || projected > projection.hard_limit
                        }) || projection.consumed_before.is_none()
                    }
                    LimitProjection::HardExceeded => {
                        projection
                            .projected
                            .is_none_or(|projected| projected <= projection.hard_limit)
                            || projection.consumed_before.is_none()
                    }
                }
        })
    {
        return Err(DecisionError::Projection);
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::{DecisionError, DecisionInput, DecisionPolicyReference, decide};
    use crate::{CounterKey, LimitProjection, ScopeProjection, TypedAmount};
    use bonsai_contracts::resource::{BudgetScope, DecisionOutcome, WorkClass};

    fn input(state: LimitProjection, scope: BudgetScope) -> DecisionInput {
        let unavailable = state == LimitProjection::MeasurementUnavailable;
        let consumed = match state {
            LimitProjection::WithinSoft => 4,
            LimitProjection::SoftExceeded => 8,
            LimitProjection::HardExceeded => 18,
            LimitProjection::MeasurementUnavailable => 0,
        };
        DecisionInput {
            decision_id: "decision-1".to_owned(),
            monotonic_time_ns: 100,
            policy: DecisionPolicyReference {
                policy_id: "policy-1".to_owned(),
                policy_version: "1.0".to_owned(),
                canonical_sha256: "11".repeat(32),
            },
            work_class: WorkClass::Acting,
            request: TypedAmount {
                key: CounterKey {
                    counter_id: "cpu_time".to_owned(),
                    unit: "ns".to_owned(),
                },
                amount: 4,
            },
            projections: vec![ScopeProjection {
                limit_id: "limit-1".to_owned(),
                scope,
                consumed_before: (!unavailable).then_some(consumed),
                requested: 4,
                soft_limit: 10,
                hard_limit: 20,
                projected: (!unavailable).then_some(consumed + 4),
                state,
            }],
            next_rolling_release_ns: (scope == BudgetScope::RollingWindow).then_some(150),
        }
    }

    #[test]
    fn identical_inputs_produce_byte_identical_decisions() {
        let expected = decide(input(LimitProjection::WithinSoft, BudgetScope::PerStep))
            .expect("decision")
            .canonical_bytes()
            .expect("bytes");
        for _ in 0..100 {
            let actual = decide(input(LimitProjection::WithinSoft, BudgetScope::PerStep))
                .expect("decision")
                .canonical_bytes()
                .expect("bytes");
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn each_admission_outcome_has_a_stable_reason_and_shape() {
        let admit =
            decide(input(LimitProjection::WithinSoft, BudgetScope::PerStep)).expect("admit");
        assert_eq!(admit.body.outcome, DecisionOutcome::Admit);

        let throttle =
            decide(input(LimitProjection::SoftExceeded, BudgetScope::PerStep)).expect("throttle");
        assert_eq!(throttle.body.outcome, DecisionOutcome::Throttle);
        assert_eq!(
            throttle.body.throttle.expect("allocation").admitted_amount,
            2
        );

        let defer = decide(input(
            LimitProjection::SoftExceeded,
            BudgetScope::RollingWindow,
        ))
        .expect("defer");
        assert_eq!(defer.body.outcome, DecisionOutcome::Defer);
        assert_eq!(defer.body.defer_until_monotonic_time_ns, Some(150));

        let reject =
            decide(input(LimitProjection::HardExceeded, BudgetScope::Lifetime)).expect("reject");
        assert_eq!(reject.body.outcome, DecisionOutcome::Reject);
    }

    #[test]
    fn missing_hard_counter_rejects_before_work() {
        let missing = decide(input(
            LimitProjection::MeasurementUnavailable,
            BudgetScope::Lifetime,
        ))
        .expect("missing measurement decision");
        assert_eq!(missing.body.outcome, DecisionOutcome::Reject);
        assert_eq!(missing.body.reason_code, "HARD_COUNTER_UNAVAILABLE");
        assert!(missing.body.throttle.is_none());
        assert!(missing.body.defer_until_monotonic_time_ns.is_none());
    }

    #[test]
    fn contradictory_projection_is_rejected() {
        let mut contradictory = input(LimitProjection::WithinSoft, BudgetScope::PerStep);
        contradictory.projections[0].projected = Some(12);
        assert_eq!(decide(contradictory), Err(DecisionError::Projection));
    }
}
