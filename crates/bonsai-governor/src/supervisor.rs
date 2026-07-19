//! Basic portable supervised budget loop for one primitive agent.

use crate::violation::{FailureBundle, ViolationError, ViolationMachine, ViolationState};
use bonsai_contracts::resource::DecisionOutcome;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BasicCounter {
    CpuTime,
    Memory,
    Storage,
    Latency,
    WorkItems,
}

const REQUIRED_COUNTERS: [BasicCounter; 5] = [
    BasicCounter::CpuTime,
    BasicCounter::Memory,
    BasicCounter::Storage,
    BasicCounter::Latency,
    BasicCounter::WorkItems,
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BasicLimit {
    pub counter: BasicCounter,
    pub unit: String,
    pub per_step_hard: u64,
    pub lifetime_hard: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StepUsage {
    pub values: BTreeMap<BasicCounter, u64>,
}

impl StepUsage {
    #[must_use]
    pub fn complete(
        cpu_time_ns: u64,
        memory_bytes: u64,
        storage_bytes: u64,
        latency_ns: u64,
        work_items: u64,
    ) -> Self {
        Self {
            values: BTreeMap::from([
                (BasicCounter::CpuTime, cpu_time_ns),
                (BasicCounter::Memory, memory_bytes),
                (BasicCounter::Storage, storage_bytes),
                (BasicCounter::Latency, latency_ns),
                (BasicCounter::WorkItems, work_items),
            ]),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetEvent {
    pub ordinal: u64,
    pub step: u64,
    pub counter: BasicCounter,
    pub unit: String,
    pub per_step_value: u64,
    pub lifetime_before: u64,
    pub lifetime_projected: u64,
    pub outcome: DecisionOutcome,
    pub reason_code: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetRunReport {
    pub schema: String,
    pub steps_completed: u64,
    pub events: Vec<BudgetEvent>,
    pub terminal: FailureBundle,
    pub c1_budget_eligible: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SupervisorError {
    Policy,
    CounterUnavailable,
    Arithmetic,
    Terminated,
    Lifecycle,
}

impl fmt::Display for SupervisorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Policy => "BASIC_SUPERVISOR_POLICY_INVALID",
            Self::CounterUnavailable => "BASIC_SUPERVISOR_COUNTER_UNAVAILABLE",
            Self::Arithmetic => "BASIC_SUPERVISOR_ARITHMETIC_FAILED",
            Self::Terminated => "BASIC_SUPERVISOR_ALREADY_TERMINATED",
            Self::Lifecycle => "BASIC_SUPERVISOR_LIFECYCLE_FAILED",
        })
    }
}

impl Error for SupervisorError {}

impl From<ViolationError> for SupervisorError {
    fn from(_: ViolationError) -> Self {
        Self::Lifecycle
    }
}

pub struct BasicSupervisor {
    limits: BTreeMap<BasicCounter, BasicLimit>,
    available: BTreeSet<BasicCounter>,
    lifetime: BTreeMap<BasicCounter, u64>,
    events: Vec<BudgetEvent>,
    steps_completed: u64,
    lifecycle: ViolationMachine,
    terminated: bool,
}

impl BasicSupervisor {
    /// Construct a supervisor with exactly the five M1 portable counters.
    ///
    /// # Errors
    ///
    /// Rejects missing/duplicate counters, zero limits, invalid units, and a
    /// per-step limit greater than its lifetime limit.
    pub fn new(
        limits: Vec<BasicLimit>,
        available: BTreeSet<BasicCounter>,
    ) -> Result<Self, SupervisorError> {
        let mut indexed = BTreeMap::new();
        for limit in limits {
            if limit.unit.is_empty()
                || limit.per_step_hard == 0
                || limit.per_step_hard > limit.lifetime_hard
                || indexed.insert(limit.counter, limit).is_some()
            {
                return Err(SupervisorError::Policy);
            }
        }
        if indexed.keys().copied().collect::<Vec<_>>() != REQUIRED_COUNTERS {
            return Err(SupervisorError::Policy);
        }
        Ok(Self {
            limits: indexed,
            available,
            lifetime: BTreeMap::new(),
            events: Vec::new(),
            steps_completed: 0,
            lifecycle: ViolationMachine::new(),
            terminated: false,
        })
    }

    /// Run one primitive step after fail-closed availability and work preflight.
    ///
    /// The closure is never called when a required counter is unavailable or
    /// the requested work-item charge already exceeds its hard budget. A
    /// measured post-step overage terminates the loop while preserving every
    /// earlier event.
    ///
    /// # Errors
    ///
    /// Returns a stable error for missing counters, arithmetic failure, or an
    /// already terminated supervisor. Post-step budget violations are evidence,
    /// not API errors.
    pub fn run_step<F>(
        &mut self,
        requested_work_items: u64,
        execute: F,
    ) -> Result<bool, SupervisorError>
    where
        F: FnOnce() -> StepUsage,
    {
        if self.terminated {
            return Err(SupervisorError::Terminated);
        }
        if self.available != BTreeSet::from(REQUIRED_COUNTERS) {
            self.terminate_preflight(
                BasicCounter::WorkItems,
                requested_work_items,
                "REQUIRED_COUNTER_UNAVAILABLE",
            )?;
            return Err(SupervisorError::CounterUnavailable);
        }
        let step = self
            .steps_completed
            .checked_add(1)
            .ok_or(SupervisorError::Arithmetic)?;
        if self.preflight_work(step, requested_work_items)? {
            return Ok(false);
        }

        let usage = execute();
        if usage.values.keys().copied().collect::<Vec<_>>() != REQUIRED_COUNTERS {
            self.terminate_preflight(
                BasicCounter::WorkItems,
                requested_work_items,
                "POST_STEP_COUNTER_MISSING",
            )?;
            return Err(SupervisorError::CounterUnavailable);
        }
        let mut projected = BTreeMap::new();
        let mut violated = Vec::new();
        for counter in REQUIRED_COUNTERS {
            let amount = usage.values[&counter];
            let before = self.lifetime.get(&counter).copied().unwrap_or(0);
            let total = before
                .checked_add(amount)
                .ok_or(SupervisorError::Arithmetic)?;
            let limit = &self.limits[&counter];
            if amount > limit.per_step_hard || total > limit.lifetime_hard {
                violated.push(counter);
            }
            projected.insert(counter, (amount, before, total));
        }
        for counter in REQUIRED_COUNTERS {
            let (amount, before, total) = projected[&counter];
            let limit = &self.limits[&counter];
            let over = violated.contains(&counter);
            self.events.push(BudgetEvent {
                ordinal: self.events.len() as u64,
                step,
                counter,
                unit: limit.unit.clone(),
                per_step_value: amount,
                lifetime_before: before,
                lifetime_projected: total,
                outcome: if over {
                    DecisionOutcome::Terminate
                } else {
                    DecisionOutcome::Admit
                },
                reason_code: if over {
                    "POST_STEP_HARD_LIMIT_EXCEEDED".to_owned()
                } else {
                    "POST_STEP_WITHIN_LIMITS".to_owned()
                },
            });
        }
        self.lifetime.extend(
            projected
                .into_iter()
                .map(|(counter, (_, _, total))| (counter, total)),
        );
        self.steps_completed = step;
        if violated.is_empty() {
            Ok(true)
        } else {
            self.lifecycle.transition(
                ViolationState::HardViolation,
                "POST_STEP_HARD_LIMIT_EXCEEDED",
            )?;
            self.terminated = true;
            Ok(false)
        }
    }

    /// Close the loop and return all prior evidence plus the terminal verdict.
    ///
    /// # Errors
    ///
    /// Returns a stable error only if lifecycle settlement fails.
    pub fn finish(self) -> Result<BudgetRunReport, SupervisorError> {
        let terminal = self.lifecycle.finish()?;
        let c1_budget_eligible = terminal.claim_eligible
            && self.available == BTreeSet::from(REQUIRED_COUNTERS)
            && self
                .events
                .iter()
                .all(|event| event.outcome == DecisionOutcome::Admit);
        Ok(BudgetRunReport {
            schema: "bonsai.basic-budget-run/v1".to_owned(),
            steps_completed: self.steps_completed,
            events: self.events,
            terminal,
            c1_budget_eligible,
        })
    }

    fn preflight_work(&mut self, step: u64, requested: u64) -> Result<bool, SupervisorError> {
        let counter = BasicCounter::WorkItems;
        let before = self.lifetime.get(&counter).copied().unwrap_or(0);
        let projected = before
            .checked_add(requested)
            .ok_or(SupervisorError::Arithmetic)?;
        let limit = &self.limits[&counter];
        if requested <= limit.per_step_hard && projected <= limit.lifetime_hard {
            return Ok(false);
        }
        self.events.push(BudgetEvent {
            ordinal: self.events.len() as u64,
            step,
            counter,
            unit: limit.unit.clone(),
            per_step_value: requested,
            lifetime_before: before,
            lifetime_projected: projected,
            outcome: DecisionOutcome::Reject,
            reason_code: "PREFLIGHT_HARD_LIMIT_EXCEEDED".to_owned(),
        });
        self.lifecycle.transition(
            ViolationState::HardViolation,
            "PREFLIGHT_HARD_LIMIT_EXCEEDED",
        )?;
        self.terminated = true;
        Ok(true)
    }

    fn terminate_preflight(
        &mut self,
        counter: BasicCounter,
        amount: u64,
        reason: &str,
    ) -> Result<(), SupervisorError> {
        let limit = &self.limits[&counter];
        self.events.push(BudgetEvent {
            ordinal: self.events.len() as u64,
            step: self.steps_completed.saturating_add(1),
            counter,
            unit: limit.unit.clone(),
            per_step_value: amount,
            lifetime_before: self.lifetime.get(&counter).copied().unwrap_or(0),
            lifetime_projected: 0,
            outcome: DecisionOutcome::Reject,
            reason_code: reason.to_owned(),
        });
        self.lifecycle
            .transition(ViolationState::HardViolation, reason)?;
        self.terminated = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{BasicCounter, BasicLimit, BasicSupervisor, StepUsage, SupervisorError};
    use bonsai_contracts::resource::DecisionOutcome;
    use std::cell::Cell;
    use std::collections::BTreeSet;

    fn limits() -> Vec<BasicLimit> {
        [
            (BasicCounter::CpuTime, "ns", 10, 20),
            (BasicCounter::Memory, "byte", 10, 20),
            (BasicCounter::Storage, "byte", 10, 20),
            (BasicCounter::Latency, "ns", 10, 20),
            (BasicCounter::WorkItems, "count", 10, 20),
        ]
        .into_iter()
        .map(|(counter, unit, per_step_hard, lifetime_hard)| BasicLimit {
            counter,
            unit: unit.to_owned(),
            per_step_hard,
            lifetime_hard,
        })
        .collect()
    }

    fn available() -> BTreeSet<BasicCounter> {
        BTreeSet::from(super::REQUIRED_COUNTERS)
    }

    #[test]
    fn under_budget_primitive_run_is_c1_budget_eligible() {
        let mut supervisor = BasicSupervisor::new(limits(), available()).expect("policy");
        for _ in 0..2 {
            assert!(
                supervisor
                    .run_step(5, || StepUsage::complete(5, 5, 5, 5, 5))
                    .expect("step")
            );
        }
        let report = supervisor.finish().expect("report");
        assert!(report.c1_budget_eligible);
        assert_eq!(report.steps_completed, 2);
        assert_eq!(report.events.len(), 10);
    }

    #[test]
    fn preflight_work_overage_is_denied_without_agent_call() {
        let called = Cell::new(false);
        let mut supervisor = BasicSupervisor::new(limits(), available()).expect("policy");
        assert!(
            !supervisor
                .run_step(11, || {
                    called.set(true);
                    StepUsage::complete(1, 1, 1, 1, 1)
                })
                .expect("denial evidence")
        );
        assert!(!called.get());
        let report = supervisor.finish().expect("report");
        assert_eq!(report.events[0].outcome, DecisionOutcome::Reject);
        assert!(!report.c1_budget_eligible);
    }

    #[test]
    fn each_measured_overage_terminates_and_preserves_prior_evidence() {
        for exceeded in super::REQUIRED_COUNTERS {
            let mut supervisor = BasicSupervisor::new(limits(), available()).expect("policy");
            supervisor
                .run_step(1, || StepUsage::complete(1, 1, 1, 1, 1))
                .expect("first step");
            let mut values = StepUsage::complete(1, 1, 1, 1, 1);
            values.values.insert(exceeded, 11);
            assert!(!supervisor.run_step(1, || values).expect("overage"));
            let report = supervisor.finish().expect("report");
            assert_eq!(report.steps_completed, 2);
            assert_eq!(report.events.len(), 10);
            assert!(
                report.events[..5]
                    .iter()
                    .all(|event| event.outcome == DecisionOutcome::Admit)
            );
            assert!(!report.c1_budget_eligible);
        }
    }

    #[test]
    fn missing_counter_rejects_before_agent_call() {
        let called = Cell::new(false);
        let mut present = available();
        present.remove(&BasicCounter::CpuTime);
        let mut supervisor = BasicSupervisor::new(limits(), present).expect("policy");
        assert_eq!(
            supervisor.run_step(1, || {
                called.set(true);
                StepUsage::complete(1, 1, 1, 1, 1)
            }),
            Err(SupervisorError::CounterUnavailable)
        );
        assert!(!called.get());
        assert!(!supervisor.finish().expect("report").c1_budget_eligible);
    }
}
