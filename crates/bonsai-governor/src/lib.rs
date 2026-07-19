//! Deterministic external budget arithmetic and scope accounting.

#![forbid(unsafe_code)]

use bonsai_contracts::resource::{BudgetScope, WorkClass};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt;

pub mod decision;
pub mod supervisor;
pub mod violation;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CounterKey {
    pub counter_id: String,
    pub unit: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TypedAmount {
    pub key: CounterKey,
    pub amount: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetLimit {
    pub limit_id: String,
    pub work_class: WorkClass,
    pub scope: BudgetScope,
    pub key: CounterKey,
    pub soft_limit: u64,
    pub hard_limit: u64,
    pub rolling_window_ns: Option<u64>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LimitProjection {
    WithinSoft,
    SoftExceeded,
    HardExceeded,
    MeasurementUnavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeProjection {
    pub limit_id: String,
    pub scope: BudgetScope,
    pub consumed_before: Option<u64>,
    pub requested: u64,
    pub soft_limit: u64,
    pub hard_limit: u64,
    pub projected: Option<u64>,
    pub state: LimitProjection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TimedCharge {
    monotonic_time_ns: u64,
    amount: u64,
}

#[derive(Clone, Debug, Default)]
struct CounterAccount {
    per_event: u64,
    per_step: u64,
    lifetime: u64,
    rolling: VecDeque<TimedCharge>,
}

#[derive(Clone, Debug, Default)]
pub struct BudgetAccounts {
    counters: HashMap<(WorkClass, CounterKey), CounterAccount>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BudgetArithmeticError {
    Identity,
    Unit,
    Limit,
    Window,
    TimeRegression,
    Overflow,
    MissingLimit,
}

impl fmt::Display for BudgetArithmeticError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "BUDGET_IDENTITY_INVALID",
            Self::Unit => "BUDGET_UNIT_INVALID",
            Self::Limit => "BUDGET_LIMIT_INVALID",
            Self::Window => "BUDGET_WINDOW_INVALID",
            Self::TimeRegression => "BUDGET_TIME_REGRESSION",
            Self::Overflow => "BUDGET_ARITHMETIC_OVERFLOW",
            Self::MissingLimit => "BUDGET_LIMIT_NOT_FOUND",
        })
    }
}

impl Error for BudgetArithmeticError {}

impl BudgetLimit {
    /// Validate typed thresholds and scope-specific rolling-window fields.
    ///
    /// # Errors
    ///
    /// Returns a stable error for malformed identity, unit, threshold, or
    /// rolling-window declarations.
    pub fn validate(&self) -> Result<(), BudgetArithmeticError> {
        if self.limit_id.is_empty() || self.key.counter_id.is_empty() {
            return Err(BudgetArithmeticError::Identity);
        }
        if self.key.unit.is_empty() {
            return Err(BudgetArithmeticError::Unit);
        }
        if self.hard_limit == 0 || self.soft_limit > self.hard_limit {
            return Err(BudgetArithmeticError::Limit);
        }
        match (self.scope, self.rolling_window_ns) {
            (BudgetScope::RollingWindow, Some(duration)) if duration > 0 => Ok(()),
            (BudgetScope::PerEvent | BudgetScope::PerStep | BudgetScope::Lifetime, None) => Ok(()),
            _ => Err(BudgetArithmeticError::Window),
        }
    }
}

impl BudgetAccounts {
    pub fn begin_event(&mut self) {
        for account in self.counters.values_mut() {
            account.per_event = 0;
        }
    }

    pub fn begin_step(&mut self) {
        for account in self.counters.values_mut() {
            account.per_event = 0;
            account.per_step = 0;
        }
    }

    /// Project one typed request through every matching nested scope.
    ///
    /// Missing measured state is represented explicitly and never interpreted
    /// as zero. Projection does not mutate the accounts.
    ///
    /// # Errors
    ///
    /// Returns a stable error for invalid/missing limits, time regression, or
    /// overflow.
    pub fn project(
        &mut self,
        work_class: WorkClass,
        request: &TypedAmount,
        monotonic_time_ns: u64,
        limits: &[BudgetLimit],
        measurement_available: bool,
    ) -> Result<Vec<ScopeProjection>, BudgetArithmeticError> {
        validate_request(request)?;
        let matching = limits
            .iter()
            .filter(|limit| limit.work_class == work_class && limit.key == request.key)
            .collect::<Vec<_>>();
        if matching.is_empty() {
            return Err(BudgetArithmeticError::MissingLimit);
        }
        let account = self
            .counters
            .entry((work_class, request.key.clone()))
            .or_default();
        prune_rolling(account, monotonic_time_ns, matching.as_slice())?;
        matching
            .into_iter()
            .map(|limit| {
                limit.validate()?;
                if !measurement_available {
                    return Ok(ScopeProjection {
                        limit_id: limit.limit_id.clone(),
                        scope: limit.scope,
                        consumed_before: None,
                        requested: request.amount,
                        soft_limit: limit.soft_limit,
                        hard_limit: limit.hard_limit,
                        projected: None,
                        state: LimitProjection::MeasurementUnavailable,
                    });
                }
                let consumed = consumed_for(
                    account,
                    limit.scope,
                    monotonic_time_ns,
                    limit.rolling_window_ns,
                )?;
                let projected = consumed
                    .checked_add(request.amount)
                    .ok_or(BudgetArithmeticError::Overflow)?;
                let state = if projected > limit.hard_limit {
                    LimitProjection::HardExceeded
                } else if projected > limit.soft_limit {
                    LimitProjection::SoftExceeded
                } else {
                    LimitProjection::WithinSoft
                };
                Ok(ScopeProjection {
                    limit_id: limit.limit_id.clone(),
                    scope: limit.scope,
                    consumed_before: Some(consumed),
                    requested: request.amount,
                    soft_limit: limit.soft_limit,
                    hard_limit: limit.hard_limit,
                    projected: Some(projected),
                    state,
                })
            })
            .collect()
    }

    /// Commit an admitted typed request to every accounting scope exactly once.
    ///
    /// # Errors
    ///
    /// Returns a stable error for an invalid request, time regression, or
    /// arithmetic overflow. No field is changed unless every sum is safe.
    pub fn commit(
        &mut self,
        work_class: WorkClass,
        request: &TypedAmount,
        monotonic_time_ns: u64,
    ) -> Result<(), BudgetArithmeticError> {
        validate_request(request)?;
        let account = self
            .counters
            .entry((work_class, request.key.clone()))
            .or_default();
        if account
            .rolling
            .back()
            .is_some_and(|charge| monotonic_time_ns < charge.monotonic_time_ns)
        {
            return Err(BudgetArithmeticError::TimeRegression);
        }
        let event = account
            .per_event
            .checked_add(request.amount)
            .ok_or(BudgetArithmeticError::Overflow)?;
        let step = account
            .per_step
            .checked_add(request.amount)
            .ok_or(BudgetArithmeticError::Overflow)?;
        let lifetime = account
            .lifetime
            .checked_add(request.amount)
            .ok_or(BudgetArithmeticError::Overflow)?;
        account.per_event = event;
        account.per_step = step;
        account.lifetime = lifetime;
        account.rolling.push_back(TimedCharge {
            monotonic_time_ns,
            amount: request.amount,
        });
        Ok(())
    }
}

fn validate_request(request: &TypedAmount) -> Result<(), BudgetArithmeticError> {
    if request.key.counter_id.is_empty() || request.amount == 0 {
        return Err(BudgetArithmeticError::Identity);
    }
    if request.key.unit.is_empty() {
        return Err(BudgetArithmeticError::Unit);
    }
    Ok(())
}

fn prune_rolling(
    account: &mut CounterAccount,
    now_ns: u64,
    limits: &[&BudgetLimit],
) -> Result<(), BudgetArithmeticError> {
    if account
        .rolling
        .back()
        .is_some_and(|charge| now_ns < charge.monotonic_time_ns)
    {
        return Err(BudgetArithmeticError::TimeRegression);
    }
    let longest = limits
        .iter()
        .filter_map(|limit| limit.rolling_window_ns)
        .max();
    if let Some(duration) = longest {
        while account
            .rolling
            .front()
            .is_some_and(|charge| now_ns.saturating_sub(charge.monotonic_time_ns) >= duration)
        {
            account.rolling.pop_front();
        }
    }
    Ok(())
}

fn consumed_for(
    account: &CounterAccount,
    scope: BudgetScope,
    now_ns: u64,
    rolling_window_ns: Option<u64>,
) -> Result<u64, BudgetArithmeticError> {
    match scope {
        BudgetScope::PerEvent => Ok(account.per_event),
        BudgetScope::PerStep => Ok(account.per_step),
        BudgetScope::Lifetime => Ok(account.lifetime),
        BudgetScope::RollingWindow => {
            let duration = rolling_window_ns.ok_or(BudgetArithmeticError::Window)?;
            account
                .rolling
                .iter()
                .filter(|charge| now_ns.saturating_sub(charge.monotonic_time_ns) < duration)
                .try_fold(0_u64, |sum, charge| {
                    sum.checked_add(charge.amount)
                        .ok_or(BudgetArithmeticError::Overflow)
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BudgetAccounts, BudgetArithmeticError, BudgetLimit, CounterKey, LimitProjection,
        TypedAmount,
    };
    use bonsai_contracts::resource::{BudgetScope, WorkClass};

    fn key() -> CounterKey {
        CounterKey {
            counter_id: "cpu_time".to_owned(),
            unit: "ns".to_owned(),
        }
    }

    fn limits() -> Vec<BudgetLimit> {
        [
            ("event", BudgetScope::PerEvent, None),
            ("step", BudgetScope::PerStep, None),
            ("window", BudgetScope::RollingWindow, Some(10)),
            ("lifetime", BudgetScope::Lifetime, None),
        ]
        .into_iter()
        .map(|(id, scope, rolling_window_ns)| BudgetLimit {
            limit_id: id.to_owned(),
            work_class: WorkClass::Acting,
            scope,
            key: key(),
            soft_limit: 5,
            hard_limit: 10,
            rolling_window_ns,
        })
        .collect()
    }

    #[test]
    fn boundary_equality_is_inside_and_next_unit_exceeds() {
        for amount in 1..=11 {
            let mut accounts = BudgetAccounts::default();
            let request = TypedAmount { key: key(), amount };
            let projections = accounts
                .project(WorkClass::Acting, &request, 1, &limits(), true)
                .expect("projection");
            let expected = if amount <= 5 {
                LimitProjection::WithinSoft
            } else if amount <= 10 {
                LimitProjection::SoftExceeded
            } else {
                LimitProjection::HardExceeded
            };
            assert!(
                projections
                    .iter()
                    .all(|projection| projection.state == expected)
            );
        }
    }

    #[test]
    fn nested_scopes_reset_and_rolling_window_expires_exactly() {
        let mut accounts = BudgetAccounts::default();
        let request = TypedAmount {
            key: key(),
            amount: 4,
        };
        accounts
            .commit(WorkClass::Acting, &request, 1)
            .expect("first charge");
        accounts.begin_event();
        let projected = accounts
            .project(WorkClass::Acting, &request, 2, &limits(), true)
            .expect("nested projection");
        assert_eq!(projected[0].consumed_before, Some(0));
        assert_eq!(projected[1].consumed_before, Some(4));
        assert_eq!(projected[2].consumed_before, Some(4));
        assert_eq!(projected[3].consumed_before, Some(4));

        let expired = accounts
            .project(WorkClass::Acting, &request, 11, &limits(), true)
            .expect("expired window");
        assert_eq!(expired[2].consumed_before, Some(0));
        accounts.begin_step();
        let reset = accounts
            .project(WorkClass::Acting, &request, 11, &limits(), true)
            .expect("step reset");
        assert_eq!(reset[0].consumed_before, Some(0));
        assert_eq!(reset[1].consumed_before, Some(0));
        assert_eq!(reset[3].consumed_before, Some(4));
    }

    #[test]
    fn unavailable_measurement_and_overflow_fail_closed() {
        let mut accounts = BudgetAccounts::default();
        let request = TypedAmount {
            key: key(),
            amount: 1,
        };
        let unavailable = accounts
            .project(WorkClass::Acting, &request, 1, &limits(), false)
            .expect("explicit unavailable projection");
        assert!(unavailable.iter().all(|projection| {
            projection.state == LimitProjection::MeasurementUnavailable
                && projection.projected.is_none()
        }));

        accounts
            .commit(
                WorkClass::Acting,
                &TypedAmount {
                    key: key(),
                    amount: u64::MAX,
                },
                1,
            )
            .expect("maximum first charge");
        assert_eq!(
            accounts.commit(WorkClass::Acting, &request, 2),
            Err(BudgetArithmeticError::Overflow)
        );
    }

    #[test]
    fn overlapping_rolling_windows_use_their_own_duration() {
        let mut accounts = BudgetAccounts::default();
        let request = TypedAmount {
            key: key(),
            amount: 3,
        };
        accounts
            .commit(WorkClass::Acting, &request, 1)
            .expect("charge");
        let rolling = [
            BudgetLimit {
                limit_id: "short".to_owned(),
                work_class: WorkClass::Acting,
                scope: BudgetScope::RollingWindow,
                key: key(),
                soft_limit: 5,
                hard_limit: 10,
                rolling_window_ns: Some(5),
            },
            BudgetLimit {
                limit_id: "long".to_owned(),
                work_class: WorkClass::Acting,
                scope: BudgetScope::RollingWindow,
                key: key(),
                soft_limit: 5,
                hard_limit: 10,
                rolling_window_ns: Some(10),
            },
        ];
        let projected = accounts
            .project(WorkClass::Acting, &request, 6, &rolling, true)
            .expect("overlapping windows");
        assert_eq!(projected[0].consumed_before, Some(0));
        assert_eq!(projected[1].consumed_before, Some(3));
    }
}
