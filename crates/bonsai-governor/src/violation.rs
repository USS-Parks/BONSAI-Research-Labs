//! Fail-closed governor violation lifecycle and terminal evidence.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationState {
    Running,
    Warning,
    Degraded,
    HardViolation,
    Terminating,
    Completed,
    Failed,
    Recovered,
}

impl ViolationState {
    const fn permits(self, next: Self) -> bool {
        matches!(
            (self, next),
            (
                Self::Running,
                Self::Warning | Self::HardViolation | Self::Completed | Self::Failed
            ) | (
                Self::Warning,
                Self::Degraded | Self::HardViolation | Self::Completed | Self::Failed
            ) | (
                Self::Degraded,
                Self::HardViolation | Self::Completed | Self::Failed
            ) | (Self::HardViolation, Self::Terminating | Self::Failed)
                | (Self::Terminating, Self::Failed | Self::Completed)
                | (Self::Failed, Self::Recovered)
        )
    }

    const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Recovered)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FinalVerdict {
    Compliant,
    SoftDegraded,
    HardViolated,
    RecoveredFailure,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ViolationRecord {
    pub ordinal: u64,
    pub state: ViolationState,
    pub reason_code: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FailureBundle {
    pub schema: String,
    pub records: Vec<ViolationRecord>,
    pub terminal_state: ViolationState,
    pub verdict: FinalVerdict,
    pub claim_eligible: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViolationError {
    InvalidTransition,
    InvalidReason,
    Arithmetic,
    NotTerminal,
    InvalidBundle,
}

impl fmt::Display for ViolationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidTransition => "GOVERNOR_VIOLATION_TRANSITION_INVALID",
            Self::InvalidReason => "GOVERNOR_VIOLATION_REASON_INVALID",
            Self::Arithmetic => "GOVERNOR_VIOLATION_ORDINAL_OVERFLOW",
            Self::NotTerminal => "GOVERNOR_VIOLATION_NOT_TERMINAL",
            Self::InvalidBundle => "GOVERNOR_FAILURE_BUNDLE_INVALID",
        })
    }
}

impl Error for ViolationError {}

#[derive(Clone, Debug)]
pub struct ViolationMachine {
    records: Vec<ViolationRecord>,
    hard_violation_seen: bool,
    soft_violation_seen: bool,
}

impl ViolationMachine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            records: vec![ViolationRecord {
                ordinal: 0,
                state: ViolationState::Running,
                reason_code: "GOVERNOR_STARTED".to_owned(),
            }],
            hard_violation_seen: false,
            soft_violation_seen: false,
        }
    }

    #[must_use]
    pub fn state(&self) -> ViolationState {
        self.records
            .last()
            .map_or(ViolationState::Running, |record| record.state)
    }

    /// Append exactly one legal state transition.
    ///
    /// # Errors
    ///
    /// Rejects invalid reason codes, terminal reentry, skipped lifecycle states,
    /// and ordinal overflow.
    pub fn transition(
        &mut self,
        next: ViolationState,
        reason_code: &str,
    ) -> Result<(), ViolationError> {
        if !valid_reason(reason_code) {
            return Err(ViolationError::InvalidReason);
        }
        let current = self.state();
        if !current.permits(next) {
            return Err(ViolationError::InvalidTransition);
        }
        let ordinal = self
            .records
            .last()
            .and_then(|record| record.ordinal.checked_add(1))
            .ok_or(ViolationError::Arithmetic)?;
        self.soft_violation_seen |=
            matches!(next, ViolationState::Warning | ViolationState::Degraded);
        self.hard_violation_seen |= next == ViolationState::HardViolation;
        self.records.push(ViolationRecord {
            ordinal,
            state: next,
            reason_code: reason_code.to_owned(),
        });
        Ok(())
    }

    /// Fail and recover the evidence lifecycle without resuming governed work.
    ///
    /// # Errors
    ///
    /// Rejects recovery after an already terminal outcome.
    pub fn recover_after_fault(&mut self, reason_code: &str) -> Result<(), ViolationError> {
        if self.state().is_terminal() {
            return Err(ViolationError::InvalidTransition);
        }
        self.transition(ViolationState::Failed, reason_code)?;
        self.transition(ViolationState::Recovered, "GOVERNOR_EVIDENCE_RECOVERED")
    }

    /// Finish the current lifecycle and return a validated immutable bundle.
    ///
    /// A hard violation always passes through terminating and produces a
    /// non-eligible `hard_violated` verdict. Soft-only runs may complete but
    /// remain degraded. A recovered failure is never claim eligible.
    ///
    /// # Errors
    ///
    /// Rejects an incomplete or internally inconsistent lifecycle.
    pub fn finish(mut self) -> Result<FailureBundle, ViolationError> {
        match self.state() {
            ViolationState::HardViolation => {
                self.transition(
                    ViolationState::Terminating,
                    "HARD_LIMIT_TERMINATION_STARTED",
                )?;
                self.transition(ViolationState::Completed, "HARD_LIMIT_TERMINATED")?;
            }
            ViolationState::Terminating => {
                self.transition(ViolationState::Completed, "HARD_LIMIT_TERMINATED")?;
            }
            ViolationState::Running | ViolationState::Warning | ViolationState::Degraded => {
                self.transition(ViolationState::Completed, "GOVERNOR_RUN_COMPLETED")?;
            }
            ViolationState::Completed | ViolationState::Recovered => {}
            ViolationState::Failed => return Err(ViolationError::NotTerminal),
        }
        let terminal_state = self.state();
        let verdict = if terminal_state == ViolationState::Recovered {
            FinalVerdict::RecoveredFailure
        } else if self.hard_violation_seen {
            FinalVerdict::HardViolated
        } else if self.soft_violation_seen {
            FinalVerdict::SoftDegraded
        } else {
            FinalVerdict::Compliant
        };
        let bundle = FailureBundle {
            schema: "bonsai.governor-failure-bundle/v1".to_owned(),
            records: self.records,
            terminal_state,
            verdict,
            claim_eligible: verdict == FinalVerdict::Compliant,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Default for ViolationMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl FailureBundle {
    /// Validate transition order, terminal uniqueness, and verdict eligibility.
    ///
    /// # Errors
    ///
    /// Rejects malformed or self-contradictory failure evidence.
    pub fn validate(&self) -> Result<(), ViolationError> {
        if self.schema != "bonsai.governor-failure-bundle/v1"
            || self
                .records
                .first()
                .is_none_or(|record| record.ordinal != 0 || record.state != ViolationState::Running)
            || self.records.last().map(|record| record.state) != Some(self.terminal_state)
            || !self.terminal_state.is_terminal()
        {
            return Err(ViolationError::InvalidBundle);
        }
        for (index, record) in self.records.iter().enumerate() {
            if record.ordinal != index as u64 || !valid_reason(&record.reason_code) {
                return Err(ViolationError::InvalidBundle);
            }
        }
        if self
            .records
            .windows(2)
            .any(|pair| !pair[0].state.permits(pair[1].state))
            || self
                .records
                .iter()
                .take(self.records.len().saturating_sub(1))
                .any(|record| record.state.is_terminal())
        {
            return Err(ViolationError::InvalidBundle);
        }
        let hard = self
            .records
            .iter()
            .any(|record| record.state == ViolationState::HardViolation);
        let soft = self.records.iter().any(|record| {
            matches!(
                record.state,
                ViolationState::Warning | ViolationState::Degraded
            )
        });
        let expected = if self.terminal_state == ViolationState::Recovered {
            FinalVerdict::RecoveredFailure
        } else if hard {
            FinalVerdict::HardViolated
        } else if soft {
            FinalVerdict::SoftDegraded
        } else {
            FinalVerdict::Compliant
        };
        if self.verdict != expected || self.claim_eligible != (expected == FinalVerdict::Compliant)
        {
            return Err(ViolationError::InvalidBundle);
        }
        Ok(())
    }
}

fn valid_reason(reason: &str) -> bool {
    !reason.is_empty()
        && reason.len() <= 96
        && reason
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

#[cfg(test)]
mod tests {
    use super::{FinalVerdict, ViolationError, ViolationMachine, ViolationState};

    fn hard_path() -> Vec<(ViolationState, &'static str)> {
        vec![
            (ViolationState::Warning, "SOFT_LIMIT_WARNING"),
            (ViolationState::Degraded, "SOFT_LIMIT_DEGRADED"),
            (ViolationState::HardViolation, "HARD_LIMIT_VIOLATION"),
            (
                ViolationState::Terminating,
                "HARD_LIMIT_TERMINATION_STARTED",
            ),
            (ViolationState::Completed, "HARD_LIMIT_TERMINATED"),
        ]
    }

    #[test]
    fn normal_and_soft_paths_have_exact_verdicts() {
        let compliant = ViolationMachine::new().finish().expect("compliant");
        assert_eq!(compliant.verdict, FinalVerdict::Compliant);
        assert!(compliant.claim_eligible);

        let mut soft = ViolationMachine::new();
        soft.transition(ViolationState::Warning, "SOFT_LIMIT_WARNING")
            .expect("warning");
        soft.transition(ViolationState::Degraded, "SOFT_LIMIT_DEGRADED")
            .expect("degraded");
        let bundle = soft.finish().expect("soft finish");
        assert_eq!(bundle.verdict, FinalVerdict::SoftDegraded);
        assert!(!bundle.claim_eligible);
    }

    #[test]
    fn hard_violation_cannot_remain_claim_eligible() {
        let mut machine = ViolationMachine::new();
        machine
            .transition(ViolationState::HardViolation, "HARD_LIMIT_VIOLATION")
            .expect("hard violation");
        let bundle = machine.finish().expect("terminal bundle");
        assert_eq!(bundle.verdict, FinalVerdict::HardViolated);
        assert_eq!(bundle.terminal_state, ViolationState::Completed);
        assert!(!bundle.claim_eligible);
    }

    #[test]
    fn fault_after_every_transition_yields_one_valid_terminal_outcome() {
        let path = hard_path();
        for fault_after in 0..=path.len() {
            let mut machine = ViolationMachine::new();
            if fault_after == 0 {
                machine
                    .recover_after_fault("FAULT_INJECTED")
                    .expect("initial recovery");
            } else {
                for (index, (state, reason)) in path.iter().copied().enumerate() {
                    machine.transition(state, reason).expect("path transition");
                    if index + 1 == fault_after && !machine.state().is_terminal() {
                        machine
                            .recover_after_fault("FAULT_INJECTED")
                            .expect("recovery");
                        break;
                    }
                }
            }
            let bundle = machine.finish().expect("valid terminal bundle");
            bundle.validate().expect("bundle validates");
            assert_eq!(
                bundle
                    .records
                    .iter()
                    .filter(|record| record.state.is_terminal())
                    .count(),
                1
            );
        }
    }

    #[test]
    fn invalid_transition_and_terminal_reentry_fail_closed() {
        let mut machine = ViolationMachine::new();
        assert_eq!(
            machine.transition(ViolationState::Terminating, "SKIPPED_HARD_STATE"),
            Err(ViolationError::InvalidTransition)
        );
        let mut finished = ViolationMachine::new();
        finished
            .transition(ViolationState::Completed, "GOVERNOR_RUN_COMPLETED")
            .expect("complete");
        assert_eq!(
            finished.recover_after_fault("TOO_LATE"),
            Err(ViolationError::InvalidTransition)
        );
    }
}
