//! Resource-policy and governor-decision contract validation.

use crate::bonsai::governor::v1::{
    BudgetScope as WireBudgetScope, DecisionOutcome as WireDecisionOutcome, GovernorDecisionEvent,
    ObservationBasis, ObservedBudgetState, PolicyReference, WorkClass as WireWorkClass,
    WorkRequest,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetScope {
    PerEvent,
    PerStep,
    RollingWindow,
    Lifetime,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkClass {
    Acting,
    Learning,
    FeatureGeneration,
    OptionLearning,
    ModelLearning,
    Planning,
    Curation,
    Environment,
    Observer,
}

const WORK_CLASSES: [WorkClass; 9] = [
    WorkClass::Acting,
    WorkClass::Learning,
    WorkClass::FeatureGeneration,
    WorkClass::OptionLearning,
    WorkClass::ModelLearning,
    WorkClass::Planning,
    WorkClass::Curation,
    WorkClass::Environment,
    WorkClass::Observer,
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BasisRequirement {
    Measured,
    Estimated,
    MeasuredOrEstimated,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionOutcome {
    Admit,
    Defer,
    Throttle,
    Reject,
    Terminate,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LimitState {
    WithinSoft,
    SoftExceeded,
    HardExceeded,
    BasisUnavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RollingWindow {
    pub duration_ns: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceLimit {
    pub limit_id: String,
    pub work_class: WorkClass,
    pub scope: BudgetScope,
    pub counter_id: String,
    pub unit: String,
    pub soft_limit: u64,
    pub hard_limit: u64,
    pub basis_requirement: BasisRequirement,
    pub rolling_window: Option<RollingWindow>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkClassAllocation {
    pub work_class: WorkClass,
    pub allocation_weight: u64,
    pub limit_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRule {
    pub reason_code: String,
    pub limit_state: LimitState,
    pub outcome: DecisionOutcome,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourcePolicy {
    pub schema_version: String,
    pub policy_id: String,
    pub policy_version: String,
    pub resource_profile_id: String,
    pub limits: Vec<ResourceLimit>,
    pub work_class_allocations: Vec<WorkClassAllocation>,
    pub decision_rules: Vec<DecisionRule>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourcePolicyValidationError {
    Identity,
    Version,
    Limit,
    ScopeCoverage,
    Allocation,
    DecisionRule,
}

impl fmt::Display for ResourcePolicyValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "RESOURCE_POLICY_IDENTITY_INVALID",
            Self::Version => "RESOURCE_POLICY_VERSION_INVALID",
            Self::Limit => "RESOURCE_POLICY_LIMIT_INVALID",
            Self::ScopeCoverage => "RESOURCE_POLICY_SCOPE_COVERAGE_INVALID",
            Self::Allocation => "RESOURCE_POLICY_ALLOCATION_INVALID",
            Self::DecisionRule => "RESOURCE_POLICY_DECISION_RULE_INVALID",
        })
    }
}

impl Error for ResourcePolicyValidationError {}

/// Validate semantic invariants that JSON Schema cannot express alone.
///
/// # Errors
///
/// Returns a stable error when identities, limits, allocation coverage, or
/// decision rules are incomplete or contradictory.
pub fn validate_resource_policy(
    policy: &ResourcePolicy,
) -> Result<(), ResourcePolicyValidationError> {
    if policy.schema_version != "1.0" || !valid_uuid_text(&policy.policy_id) {
        return Err(ResourcePolicyValidationError::Identity);
    }
    if !valid_version(&policy.policy_version) {
        return Err(ResourcePolicyValidationError::Version);
    }

    let mut limit_by_id = HashMap::new();
    let mut scopes = HashSet::new();
    for limit in &policy.limits {
        if limit.limit_id.is_empty()
            || limit.counter_id.is_empty()
            || limit.unit.is_empty()
            || limit.hard_limit == 0
            || limit.soft_limit > limit.hard_limit
            || limit_by_id.insert(limit.limit_id.as_str(), limit).is_some()
        {
            return Err(ResourcePolicyValidationError::Limit);
        }
        match (limit.scope, &limit.rolling_window) {
            (BudgetScope::RollingWindow, Some(window)) if window.duration_ns > 0 => {}
            (BudgetScope::PerEvent | BudgetScope::PerStep | BudgetScope::Lifetime, None) => {}
            _ => return Err(ResourcePolicyValidationError::Limit),
        }
        scopes.insert(limit.scope);
    }
    if scopes.len() != 4 {
        return Err(ResourcePolicyValidationError::ScopeCoverage);
    }

    let mut allocated_classes = HashSet::new();
    let mut referenced_limits = HashSet::new();
    for allocation in &policy.work_class_allocations {
        if allocation.allocation_weight == 0
            || allocation.limit_ids.is_empty()
            || !allocated_classes.insert(allocation.work_class)
        {
            return Err(ResourcePolicyValidationError::Allocation);
        }
        for limit_id in &allocation.limit_ids {
            let Some(limit) = limit_by_id.get(limit_id.as_str()) else {
                return Err(ResourcePolicyValidationError::Allocation);
            };
            if limit.work_class != allocation.work_class || !referenced_limits.insert(limit_id) {
                return Err(ResourcePolicyValidationError::Allocation);
            }
        }
    }
    if allocated_classes != HashSet::from(WORK_CLASSES)
        || referenced_limits.len() != policy.limits.len()
    {
        return Err(ResourcePolicyValidationError::Allocation);
    }

    let mut reason_codes = HashSet::new();
    let mut outcomes = HashSet::new();
    for rule in &policy.decision_rules {
        if !valid_reason_code(&rule.reason_code) || !reason_codes.insert(rule.reason_code.as_str())
        {
            return Err(ResourcePolicyValidationError::DecisionRule);
        }
        outcomes.insert(rule.outcome);
    }
    let expected_outcomes = HashSet::from([
        DecisionOutcome::Admit,
        DecisionOutcome::Defer,
        DecisionOutcome::Throttle,
        DecisionOutcome::Reject,
        DecisionOutcome::Terminate,
    ]);
    if outcomes != expected_outcomes {
        return Err(ResourcePolicyValidationError::DecisionRule);
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionReconstruction {
    pub policy: PolicyReference,
    pub observed_state: ObservedBudgetState,
    pub requested_work: WorkRequest,
    pub reason_code: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GovernorDecisionValidationError {
    Identity,
    PolicyReference,
    RequestedWork,
    ObservedState,
    ObservationBasis,
    ReasonCode,
    OutcomeAction,
}

impl fmt::Display for GovernorDecisionValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "GOVERNOR_DECISION_IDENTITY_INVALID",
            Self::PolicyReference => "GOVERNOR_POLICY_REFERENCE_INVALID",
            Self::RequestedWork => "GOVERNOR_REQUESTED_WORK_INVALID",
            Self::ObservedState => "GOVERNOR_OBSERVED_STATE_INVALID",
            Self::ObservationBasis => "GOVERNOR_OBSERVATION_BASIS_INVALID",
            Self::ReasonCode => "GOVERNOR_REASON_CODE_INVALID",
            Self::OutcomeAction => "GOVERNOR_OUTCOME_ACTION_INVALID",
        })
    }
}

impl Error for GovernorDecisionValidationError {}

/// Validate one governor-decision event and extract its reconstruction inputs.
///
/// # Errors
///
/// Returns a stable error if the event omits or contradicts any policy,
/// observation, request, basis, reason, or outcome-action field.
pub fn reconstruct_governor_decision(
    decision: &GovernorDecisionEvent,
) -> Result<DecisionReconstruction, GovernorDecisionValidationError> {
    if !valid_uuid_bytes(&decision.decision_id)
        || !valid_uuid_bytes(&decision.run_id)
        || decision.monotonic_time_ns == 0
    {
        return Err(GovernorDecisionValidationError::Identity);
    }
    let policy = decision
        .policy
        .as_ref()
        .ok_or(GovernorDecisionValidationError::PolicyReference)?;
    if !valid_uuid_bytes(&policy.policy_id)
        || !valid_version(&policy.policy_version)
        || policy.canonical_sha256.len() != 32
        || policy.canonical_sha256.iter().all(|byte| *byte == 0)
    {
        return Err(GovernorDecisionValidationError::PolicyReference);
    }
    let request = decision
        .requested_work
        .as_ref()
        .ok_or(GovernorDecisionValidationError::RequestedWork)?;
    let request_class = WireWorkClass::try_from(request.work_class)
        .map_err(|_| GovernorDecisionValidationError::RequestedWork)?;
    if request_class == WireWorkClass::Unspecified
        || !valid_uuid_bytes(&request.work_item_id)
        || request.resources.is_empty()
    {
        return Err(GovernorDecisionValidationError::RequestedWork);
    }
    let mut requested_resources = HashMap::new();
    for resource in &request.resources {
        if resource.counter_id.is_empty()
            || resource.unit.is_empty()
            || resource.amount == 0
            || requested_resources
                .insert(
                    (resource.counter_id.as_str(), resource.unit.as_str()),
                    resource.amount,
                )
                .is_some()
        {
            return Err(GovernorDecisionValidationError::RequestedWork);
        }
    }

    let observed = decision
        .observed_state
        .as_ref()
        .ok_or(GovernorDecisionValidationError::ObservedState)?;
    if observed.observations.is_empty()
        || observed
            .triggering_event_id
            .as_ref()
            .is_some_and(|id| !valid_uuid_bytes(id))
    {
        return Err(GovernorDecisionValidationError::ObservedState);
    }
    let mut observed_limits = HashMap::new();
    for observation in &observed.observations {
        validate_observation(
            observation,
            observed.triggering_event_id.as_deref(),
            decision.monotonic_time_ns,
            request_class,
        )?;
        if observed_limits
            .insert(observation.limit_id.as_str(), observation)
            .is_some()
        {
            return Err(GovernorDecisionValidationError::ObservedState);
        }
    }
    if decision.affected_limit_ids.is_empty() {
        return Err(GovernorDecisionValidationError::ObservedState);
    }
    let mut affected = HashSet::new();
    for limit_id in &decision.affected_limit_ids {
        let Some(observation) = observed_limits.get(limit_id.as_str()) else {
            return Err(GovernorDecisionValidationError::ObservedState);
        };
        if !affected.insert(limit_id.as_str())
            || !requested_resources
                .contains_key(&(observation.counter_id.as_str(), observation.unit.as_str()))
        {
            return Err(GovernorDecisionValidationError::ObservedState);
        }
    }
    if !valid_reason_code(&decision.reason_code) {
        return Err(GovernorDecisionValidationError::ReasonCode);
    }
    validate_outcome_action(decision, &requested_resources)?;

    Ok(DecisionReconstruction {
        policy: policy.clone(),
        observed_state: observed.clone(),
        requested_work: request.clone(),
        reason_code: decision.reason_code.clone(),
    })
}

fn validate_observation(
    observation: &crate::bonsai::governor::v1::BudgetObservation,
    triggering_event_id: Option<&[u8]>,
    monotonic_time_ns: u64,
    request_class: WireWorkClass,
) -> Result<(), GovernorDecisionValidationError> {
    let scope = WireBudgetScope::try_from(observation.scope)
        .map_err(|_| GovernorDecisionValidationError::ObservedState)?;
    let work_class = WireWorkClass::try_from(observation.work_class)
        .map_err(|_| GovernorDecisionValidationError::ObservedState)?;
    if scope == WireBudgetScope::Unspecified
        || work_class != request_class
        || observation.limit_id.is_empty()
        || observation.counter_id.is_empty()
        || observation.unit.is_empty()
        || (scope == WireBudgetScope::PerEvent && triggering_event_id.is_none())
    {
        return Err(GovernorDecisionValidationError::ObservedState);
    }
    match (
        scope,
        observation.window_start_monotonic_time_ns,
        observation.window_duration_ns,
    ) {
        (WireBudgetScope::RollingWindow, Some(start), Some(duration))
            if start <= monotonic_time_ns && duration > 0 => {}
        (
            WireBudgetScope::PerEvent | WireBudgetScope::PerStep | WireBudgetScope::Lifetime,
            None,
            None,
        ) => {}
        _ => return Err(GovernorDecisionValidationError::ObservedState),
    }
    let basis = ObservationBasis::try_from(observation.basis)
        .map_err(|_| GovernorDecisionValidationError::ObservationBasis)?;
    match basis {
        ObservationBasis::Measured => {
            if observation.consumed.is_none()
                || observation.estimator_id.is_some()
                || observation.estimator_version.is_some()
                || observation.unavailable_reason.is_some()
            {
                return Err(GovernorDecisionValidationError::ObservationBasis);
            }
        }
        ObservationBasis::Estimated => {
            if observation.consumed.is_none()
                || observation
                    .estimator_id
                    .as_deref()
                    .is_none_or(str::is_empty)
                || observation
                    .estimator_version
                    .as_deref()
                    .is_none_or(|version| !valid_version(version))
                || observation.unavailable_reason.is_some()
            {
                return Err(GovernorDecisionValidationError::ObservationBasis);
            }
        }
        ObservationBasis::Unavailable => {
            if observation.consumed.is_some()
                || observation.estimator_id.is_some()
                || observation.estimator_version.is_some()
                || observation
                    .unavailable_reason
                    .as_deref()
                    .is_none_or(str::is_empty)
            {
                return Err(GovernorDecisionValidationError::ObservationBasis);
            }
        }
        ObservationBasis::Unspecified => {
            return Err(GovernorDecisionValidationError::ObservationBasis);
        }
    }
    Ok(())
}

fn validate_outcome_action(
    decision: &GovernorDecisionEvent,
    requested_resources: &HashMap<(&str, &str), u64>,
) -> Result<(), GovernorDecisionValidationError> {
    let outcome = WireDecisionOutcome::try_from(decision.outcome)
        .map_err(|_| GovernorDecisionValidationError::OutcomeAction)?;
    match outcome {
        WireDecisionOutcome::Admit
        | WireDecisionOutcome::Reject
        | WireDecisionOutcome::Terminate => {
            if decision.defer_until_monotonic_time_ns.is_some()
                || !decision.throttle_allocations.is_empty()
            {
                return Err(GovernorDecisionValidationError::OutcomeAction);
            }
        }
        WireDecisionOutcome::Defer => {
            if decision
                .defer_until_monotonic_time_ns
                .is_none_or(|until| until <= decision.monotonic_time_ns)
                || !decision.throttle_allocations.is_empty()
            {
                return Err(GovernorDecisionValidationError::OutcomeAction);
            }
        }
        WireDecisionOutcome::Throttle => {
            if decision.defer_until_monotonic_time_ns.is_some()
                || decision.throttle_allocations.is_empty()
            {
                return Err(GovernorDecisionValidationError::OutcomeAction);
            }
            let mut throttled = HashSet::new();
            for allocation in &decision.throttle_allocations {
                let key = (allocation.counter_id.as_str(), allocation.unit.as_str());
                let Some(requested) = requested_resources.get(&key) else {
                    return Err(GovernorDecisionValidationError::OutcomeAction);
                };
                if allocation.admitted_amount == 0
                    || allocation.admitted_amount >= *requested
                    || !throttled.insert(key)
                {
                    return Err(GovernorDecisionValidationError::OutcomeAction);
                }
            }
        }
        WireDecisionOutcome::Unspecified => {
            return Err(GovernorDecisionValidationError::OutcomeAction);
        }
    }
    Ok(())
}

fn valid_uuid_bytes(bytes: &[u8]) -> bool {
    bytes.len() == 16 && bytes.iter().any(|byte| *byte != 0)
}

fn valid_uuid_text(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => byte == b'-',
            _ => byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase(),
        })
        && value.bytes().any(|byte| byte != b'0' && byte != b'-')
}

fn valid_version(value: &str) -> bool {
    let mut count = 0;
    for part in value.split('.') {
        if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
            return false;
        }
        count += 1;
    }
    (2..=3).contains(&count)
}

fn valid_reason_code(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_uppercase())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

#[cfg(test)]
mod tests {
    use super::{
        GovernorDecisionValidationError, ResourcePolicy, reconstruct_governor_decision,
        validate_resource_policy,
    };
    use crate::bonsai::governor::v1::{
        BudgetObservation, BudgetScope, DecisionOutcome, GovernorDecisionEvent, ObservationBasis,
        ObservedBudgetState, PolicyReference, ResourceRequest, ThrottleAllocation, WorkClass,
        WorkRequest,
    };
    use prost::Message;

    #[test]
    fn frozen_resource_policy_has_complete_scopes_classes_and_outcomes() {
        let policy: ResourcePolicy = serde_json::from_str(include_str!(
            "../../../fixtures/resource-policy/v1/valid.json"
        ))
        .expect("fixture decodes");
        assert_eq!(validate_resource_policy(&policy), Ok(()));
    }

    fn decision(outcome: DecisionOutcome) -> GovernorDecisionEvent {
        let mut decision = GovernorDecisionEvent {
            decision_id: vec![1; 16],
            run_id: vec![2; 16],
            policy: Some(PolicyReference {
                policy_id: vec![3; 16],
                policy_version: "1.0.0".to_owned(),
                canonical_sha256: vec![4; 32],
            }),
            monotonic_time_ns: 200,
            requested_work: Some(WorkRequest {
                work_item_id: vec![5; 16],
                work_class: WorkClass::Acting as i32,
                resources: vec![ResourceRequest {
                    counter_id: "cpu_time_ns".to_owned(),
                    unit: "ns".to_owned(),
                    amount: 10,
                }],
            }),
            observed_state: Some(ObservedBudgetState {
                step_index: 7,
                triggering_event_id: Some(vec![6; 16]),
                observations: vec![BudgetObservation {
                    limit_id: "acting.window.cpu".to_owned(),
                    scope: BudgetScope::RollingWindow as i32,
                    work_class: WorkClass::Acting as i32,
                    counter_id: "cpu_time_ns".to_owned(),
                    unit: "ns".to_owned(),
                    consumed: Some(90),
                    basis: ObservationBasis::Estimated as i32,
                    estimator_id: Some("process-cpu-proxy".to_owned()),
                    estimator_version: Some("1.0.0".to_owned()),
                    window_start_monotonic_time_ns: Some(100),
                    window_duration_ns: Some(1_000),
                    unavailable_reason: None,
                }],
            }),
            outcome: outcome as i32,
            reason_code: reason_code(outcome).to_owned(),
            affected_limit_ids: vec!["acting.window.cpu".to_owned()],
            defer_until_monotonic_time_ns: None,
            throttle_allocations: Vec::new(),
        };
        match outcome {
            DecisionOutcome::Defer => decision.defer_until_monotonic_time_ns = Some(300),
            DecisionOutcome::Throttle => {
                decision.throttle_allocations.push(ThrottleAllocation {
                    counter_id: "cpu_time_ns".to_owned(),
                    unit: "ns".to_owned(),
                    admitted_amount: 5,
                });
            }
            DecisionOutcome::Terminate => {
                let observation = &mut decision
                    .observed_state
                    .as_mut()
                    .expect("observed state")
                    .observations[0];
                observation.consumed = None;
                observation.basis = ObservationBasis::Unavailable as i32;
                observation.estimator_id = None;
                observation.estimator_version = None;
                observation.unavailable_reason = Some("collector_permission_denied".to_owned());
            }
            DecisionOutcome::Admit | DecisionOutcome::Reject => {}
            DecisionOutcome::Unspecified => unreachable!(),
        }
        decision
    }

    const fn reason_code(outcome: DecisionOutcome) -> &'static str {
        match outcome {
            DecisionOutcome::Admit => "WITHIN_SOFT_LIMITS",
            DecisionOutcome::Defer => "ROLLING_WINDOW_RETRY",
            DecisionOutcome::Throttle => "SOFT_LIMIT_THROTTLE",
            DecisionOutcome::Reject => "HARD_LIMIT_REJECT",
            DecisionOutcome::Terminate => "MEASUREMENT_BASIS_UNAVAILABLE",
            DecisionOutcome::Unspecified => "UNSPECIFIED",
        }
    }

    #[test]
    fn all_five_outcomes_retain_exact_reconstruction_inputs() {
        for outcome in [
            DecisionOutcome::Admit,
            DecisionOutcome::Defer,
            DecisionOutcome::Throttle,
            DecisionOutcome::Reject,
            DecisionOutcome::Terminate,
        ] {
            let original = decision(outcome);
            let bytes = original.encode_to_vec();
            let decoded = GovernorDecisionEvent::decode(bytes.as_slice()).expect("wire decode");
            let reconstructed = reconstruct_governor_decision(&decoded).expect("valid decision");
            assert_eq!(reconstructed.policy, original.policy.expect("policy"));
            assert_eq!(
                reconstructed.observed_state,
                original.observed_state.expect("observed state")
            );
            assert_eq!(
                reconstructed.requested_work,
                original.requested_work.expect("requested work")
            );
            assert_eq!(reconstructed.reason_code, reason_code(outcome));
        }
    }

    #[test]
    fn estimated_basis_requires_estimator_provenance() {
        let mut decision = decision(DecisionOutcome::Admit);
        decision
            .observed_state
            .as_mut()
            .expect("observed state")
            .observations[0]
            .estimator_id = None;
        assert_eq!(
            reconstruct_governor_decision(&decision),
            Err(GovernorDecisionValidationError::ObservationBasis)
        );
    }

    #[test]
    fn outcome_specific_actions_fail_closed() {
        let mut decision = decision(DecisionOutcome::Admit);
        decision.defer_until_monotonic_time_ns = Some(300);
        assert_eq!(
            reconstruct_governor_decision(&decision),
            Err(GovernorDecisionValidationError::OutcomeAction)
        );
    }
}
