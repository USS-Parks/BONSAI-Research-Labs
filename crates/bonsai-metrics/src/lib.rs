//! Version-pinned deterministic metric registry and rational-value engine.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

pub mod behavior;
pub mod resources;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MetricKey {
    pub id: String,
    pub version: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricDirection {
    HigherIsBetter,
    LowerIsBetter,
    Neutral,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum MetricWindow {
    Lifetime,
    PerStep,
    Rolling { duration_steps: u64 },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum MetricFormula {
    Input,
    Sum,
    Difference,
    Ratio,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MetricSpec {
    pub key: MetricKey,
    pub formula: MetricFormula,
    pub unit: String,
    pub window: MetricWindow,
    pub direction: MetricDirection,
    pub inputs: Vec<MetricKey>,
    pub availability_rule: String,
    pub claim_uses: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricAvailability {
    Available,
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RationalValue {
    pub numerator: i64,
    pub denominator: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MetricValue {
    pub key: MetricKey,
    pub unit: String,
    pub availability: MetricAvailability,
    pub value: Option<RationalValue>,
    pub detail_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedMetricTable {
    pub schema: String,
    pub rows: Vec<MetricValue>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetricError {
    Identity,
    Duplicate,
    Formula,
    MissingInput,
    VersionMismatch,
    Cycle,
    Value,
    Arithmetic,
}

impl fmt::Display for MetricError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "METRIC_IDENTITY_INVALID",
            Self::Duplicate => "METRIC_DUPLICATE",
            Self::Formula => "METRIC_FORMULA_INVALID",
            Self::MissingInput => "METRIC_INPUT_MISSING",
            Self::VersionMismatch => "METRIC_INPUT_VERSION_MISMATCH",
            Self::Cycle => "METRIC_DEPENDENCY_CYCLE",
            Self::Value => "METRIC_VALUE_INVALID",
            Self::Arithmetic => "METRIC_ARITHMETIC_FAILED",
        })
    }
}

impl Error for MetricError {}

pub struct MetricRegistry {
    specs: BTreeMap<MetricKey, MetricSpec>,
    order: Vec<MetricKey>,
}

impl MetricRegistry {
    /// Validate a complete registry and derive its stable dependency order.
    ///
    /// # Errors
    ///
    /// Rejects malformed identities/formulas, duplicate keys, missing or
    /// mis-versioned dependencies, and cycles.
    pub fn new(specs: Vec<MetricSpec>) -> Result<Self, MetricError> {
        let mut indexed = BTreeMap::new();
        for spec in specs {
            validate_spec(&spec)?;
            if indexed.insert(spec.key.clone(), spec).is_some() {
                return Err(MetricError::Duplicate);
            }
        }
        for spec in indexed.values() {
            for dependency in &spec.inputs {
                if !indexed.contains_key(dependency) {
                    if indexed.keys().any(|key| key.id == dependency.id) {
                        return Err(MetricError::VersionMismatch);
                    }
                    return Err(MetricError::MissingInput);
                }
            }
        }
        let order = dependency_order(&indexed)?;
        Ok(Self {
            specs: indexed,
            order,
        })
    }

    /// Compute every registered metric in stable key order.
    ///
    /// Missing source values propagate explicit unavailability through derived
    /// metrics. Unregistered inputs are rejected.
    ///
    /// # Errors
    ///
    /// Returns a stable error for invalid input values or checked arithmetic.
    pub fn compute(
        &self,
        inputs: &BTreeMap<MetricKey, Option<RationalValue>>,
    ) -> Result<DerivedMetricTable, MetricError> {
        if inputs.keys().any(|key| {
            self.specs
                .get(key)
                .is_none_or(|spec| spec.formula != MetricFormula::Input)
        }) {
            return Err(MetricError::MissingInput);
        }
        let mut values: BTreeMap<MetricKey, Option<RationalValue>> = BTreeMap::new();
        for key in &self.order {
            let spec = &self.specs[key];
            let value = match spec.formula {
                MetricFormula::Input => inputs.get(key).cloned().flatten(),
                MetricFormula::Sum => combine(spec, &values, sum_values)?,
                MetricFormula::Difference => combine(spec, &values, difference_values)?,
                MetricFormula::Ratio => combine(spec, &values, ratio_values)?,
            };
            if value.as_ref().is_some_and(|value| value.denominator == 0) {
                return Err(MetricError::Value);
            }
            values.insert(key.clone(), value.map(normalize));
        }
        let rows = self
            .specs
            .values()
            .map(|spec| {
                let value = values.get(&spec.key).cloned().flatten();
                MetricValue {
                    key: spec.key.clone(),
                    unit: spec.unit.clone(),
                    availability: if value.is_some() {
                        MetricAvailability::Available
                    } else {
                        MetricAvailability::Unavailable
                    },
                    detail_code: value
                        .is_none()
                        .then(|| "METRIC_INPUT_UNAVAILABLE".to_owned()),
                    value,
                }
            })
            .collect();
        Ok(DerivedMetricTable {
            schema: "bonsai.derived-metric-table/v1".to_owned(),
            rows,
        })
    }
}

fn validate_spec(spec: &MetricSpec) -> Result<(), MetricError> {
    if spec.key.id.is_empty()
        || spec.key.version.is_empty()
        || spec.unit.is_empty()
        || spec.availability_rule != "all_inputs_required"
        || matches!(spec.window, MetricWindow::Rolling { duration_steps: 0 })
    {
        return Err(MetricError::Identity);
    }
    let input_count = spec.inputs.len();
    let valid = match spec.formula {
        MetricFormula::Input => input_count == 0,
        MetricFormula::Sum => input_count > 0,
        MetricFormula::Difference | MetricFormula::Ratio => input_count == 2,
    };
    if !valid || spec.inputs.iter().collect::<BTreeSet<_>>().len() != input_count {
        return Err(MetricError::Formula);
    }
    Ok(())
}

fn dependency_order(
    specs: &BTreeMap<MetricKey, MetricSpec>,
) -> Result<Vec<MetricKey>, MetricError> {
    fn visit(
        key: &MetricKey,
        specs: &BTreeMap<MetricKey, MetricSpec>,
        active: &mut BTreeSet<MetricKey>,
        done: &mut BTreeSet<MetricKey>,
        order: &mut Vec<MetricKey>,
    ) -> Result<(), MetricError> {
        if done.contains(key) {
            return Ok(());
        }
        if !active.insert(key.clone()) {
            return Err(MetricError::Cycle);
        }
        for dependency in &specs[key].inputs {
            visit(dependency, specs, active, done, order)?;
        }
        active.remove(key);
        done.insert(key.clone());
        order.push(key.clone());
        Ok(())
    }
    let mut active = BTreeSet::new();
    let mut done = BTreeSet::new();
    let mut order = Vec::new();
    for key in specs.keys() {
        visit(key, specs, &mut active, &mut done, &mut order)?;
    }
    Ok(order)
}

fn combine(
    spec: &MetricSpec,
    values: &BTreeMap<MetricKey, Option<RationalValue>>,
    operation: fn(&[RationalValue]) -> Result<RationalValue, MetricError>,
) -> Result<Option<RationalValue>, MetricError> {
    let collected = spec
        .inputs
        .iter()
        .map(|key| values.get(key).cloned().flatten())
        .collect::<Option<Vec<_>>>();
    collected.map(|values| operation(&values)).transpose()
}

fn sum_values(values: &[RationalValue]) -> Result<RationalValue, MetricError> {
    values.iter().try_fold(
        RationalValue {
            numerator: 0,
            denominator: 1,
        },
        |total, value| add(&total, value),
    )
}

fn difference_values(values: &[RationalValue]) -> Result<RationalValue, MetricError> {
    let negative = values[1]
        .numerator
        .checked_neg()
        .ok_or(MetricError::Arithmetic)?;
    add(
        &values[0],
        &RationalValue {
            numerator: negative,
            denominator: values[1].denominator,
        },
    )
}

fn ratio_values(values: &[RationalValue]) -> Result<RationalValue, MetricError> {
    if values[1].numerator == 0 {
        return Err(MetricError::Arithmetic);
    }
    let sign = if values[1].numerator < 0 { -1 } else { 1 };
    let numerator = values[0]
        .numerator
        .checked_mul(i64::from(sign))
        .and_then(|value| value.checked_mul(i64::try_from(values[1].denominator).ok()?))
        .ok_or(MetricError::Arithmetic)?;
    let denominator = values[0]
        .denominator
        .checked_mul(values[1].numerator.unsigned_abs())
        .ok_or(MetricError::Arithmetic)?;
    Ok(RationalValue {
        numerator,
        denominator,
    })
}

fn add(left: &RationalValue, right: &RationalValue) -> Result<RationalValue, MetricError> {
    let left_denominator = i64::try_from(left.denominator).map_err(|_| MetricError::Arithmetic)?;
    let right_denominator =
        i64::try_from(right.denominator).map_err(|_| MetricError::Arithmetic)?;
    let numerator = left
        .numerator
        .checked_mul(right_denominator)
        .and_then(|value| value.checked_add(right.numerator.checked_mul(left_denominator)?))
        .ok_or(MetricError::Arithmetic)?;
    let denominator = left
        .denominator
        .checked_mul(right.denominator)
        .ok_or(MetricError::Arithmetic)?;
    Ok(RationalValue {
        numerator,
        denominator,
    })
}

fn normalize(mut value: RationalValue) -> RationalValue {
    let mut left = value.numerator.unsigned_abs();
    let mut right = value.denominator;
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    let divisor = left.max(1);
    value.numerator /= i64::try_from(divisor).unwrap_or(1);
    value.denominator /= divisor;
    value
}

#[cfg(test)]
mod tests {
    use super::{
        MetricDirection, MetricError, MetricFormula, MetricKey, MetricRegistry, MetricSpec,
        MetricWindow, RationalValue,
    };
    use std::collections::BTreeMap;

    fn key(id: &str) -> MetricKey {
        MetricKey {
            id: id.to_owned(),
            version: "1.0".to_owned(),
        }
    }

    fn spec(id: &str, formula: MetricFormula, inputs: Vec<MetricKey>) -> MetricSpec {
        MetricSpec {
            key: key(id),
            formula,
            unit: "count".to_owned(),
            window: MetricWindow::Lifetime,
            direction: MetricDirection::Neutral,
            inputs,
            availability_rule: "all_inputs_required".to_owned(),
            claim_uses: vec!["C0".to_owned()],
        }
    }

    #[test]
    fn golden_registry_is_deterministic_and_version_pinned() {
        let registry = MetricRegistry::new(vec![
            spec("reward_total", MetricFormula::Input, vec![]),
            spec("steps", MetricFormula::Input, vec![]),
            spec(
                "reward_rate",
                MetricFormula::Ratio,
                vec![key("reward_total"), key("steps")],
            ),
        ])
        .expect("registry");
        let inputs = BTreeMap::from([
            (
                key("reward_total"),
                Some(RationalValue {
                    numerator: 12,
                    denominator: 1,
                }),
            ),
            (
                key("steps"),
                Some(RationalValue {
                    numerator: 3,
                    denominator: 1,
                }),
            ),
        ]);
        let expected =
            serde_json::to_vec(&registry.compute(&inputs).expect("table")).expect("json");
        for _ in 0..100 {
            assert_eq!(
                serde_json::to_vec(&registry.compute(&inputs).expect("table")).expect("json"),
                expected
            );
        }
        assert!(
            String::from_utf8(expected)
                .expect("utf8")
                .contains("\"numerator\":4")
        );
    }

    #[test]
    fn cycles_and_wrong_versions_fail_closed() {
        assert_eq!(
            MetricRegistry::new(vec![
                spec("a", MetricFormula::Sum, vec![key("b")]),
                spec("b", MetricFormula::Sum, vec![key("a")]),
            ])
            .err(),
            Some(MetricError::Cycle)
        );
        let mut wrong = key("source");
        wrong.version = "2.0".to_owned();
        assert_eq!(
            MetricRegistry::new(vec![
                spec("source", MetricFormula::Input, vec![]),
                spec("derived", MetricFormula::Sum, vec![wrong]),
            ])
            .err(),
            Some(MetricError::VersionMismatch)
        );
    }

    #[test]
    fn unavailable_inputs_propagate_without_zero() {
        let registry = MetricRegistry::new(vec![
            spec("a", MetricFormula::Input, vec![]),
            spec("b", MetricFormula::Input, vec![]),
            spec("sum", MetricFormula::Sum, vec![key("a"), key("b")]),
        ])
        .expect("registry");
        let table = registry
            .compute(&BTreeMap::from([(
                key("a"),
                Some(RationalValue {
                    numerator: 1,
                    denominator: 1,
                }),
            )]))
            .expect("table");
        let sum = table
            .rows
            .iter()
            .find(|row| row.key.id == "sum")
            .expect("sum");
        assert!(sum.value.is_none());
        assert_eq!(sum.detail_code.as_deref(), Some("METRIC_INPUT_UNAVAILABLE"));
    }
}
