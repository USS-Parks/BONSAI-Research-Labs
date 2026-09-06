//! Generic online parameter accounting; historical manifests keep their exact v1 interpretation.
use crate::bonsai::adapter::v1::PrimitiveAccounting;
use crate::resource::WorkClass;
use serde::Deserialize;
use serde_json::{Value, json};

const MAXIMUM_UPDATE_BYTES: usize = 32 * 1024;
const MAXIMUM_WORK_PER_CLASS: u64 = 1_000_000;
const MAXIMUM_PARAMETER_TOUCHES: u64 = 8_192;
const MAXIMUM_RETAINED_STATE_BYTES: u64 = 16 * 1024 * 1024;
const MAXIMUM_SERIALIZED_STATE_BYTES: u64 = 1024 * 1024;
const LEARNER_WORK_CLASSES: [WorkClass; 7] = [
    WorkClass::Acting,
    WorkClass::Learning,
    WorkClass::FeatureGeneration,
    WorkClass::OptionLearning,
    WorkClass::ModelLearning,
    WorkClass::Planning,
    WorkClass::Curation,
];

#[derive(Clone, Debug)]
pub enum OnlineAccounting {
    Legacy,
    V1 { touches_per_update: u64 },
    V2(V2Declaration),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct V2Declaration {
    schema: String,
    feedback_signal_type: String,
    parameter_update_schema: String,
    work_per_step: Vec<WorkTariff>,
    maximum_parameter_touches_per_update: u64,
    retained_state_limit_bytes: u64,
    serialized_state_limit_bytes: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclarationV1 {
    schema: String,
    parameter_touches_per_update: u64,
    parameter_update_schema: String,
}

#[derive(Clone, Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct WorkTariff {
    work_class: WorkClass,
    amount: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct V2Update {
    schema: String,
    update: u64,
    action: u32,
    reward: i64,
    work_by_class: serde_json::Map<String, Value>,
    parameter_touches: u64,
    state_before: String,
    state_after: String,
    allocated_bytes: u64,
    serialized_bytes: u64,
    details: Value,
}

impl OnlineAccounting {
    /// Parse an explicit immutable declaration, or the established historical contract.
    ///
    /// # Errors
    /// Refuses unknown versions, malformed fields and unsupported bounds.
    pub fn from_declaration_or_legacy(value: Option<&Value>) -> Result<Self, &'static str> {
        let Some(value) = value else {
            return Ok(Self::Legacy);
        };
        let schema = value
            .get("schema")
            .and_then(Value::as_str)
            .ok_or("ACCOUNTING_DECLARATION_INVALID")?;
        match schema {
            "bonsai.online-accounting/v1" => {
                let declaration: DeclarationV1 = serde_json::from_value(value.clone())
                    .map_err(|_| "ACCOUNTING_DECLARATION_INVALID")?;
                if declaration.schema != "bonsai.online-accounting/v1"
                    || declaration.parameter_update_schema != "bonsai.parameter-update/v1"
                    || !(1..=257).contains(&declaration.parameter_touches_per_update)
                {
                    return Err("ACCOUNTING_DECLARATION_UNSUPPORTED");
                }
                Ok(Self::V1 {
                    touches_per_update: declaration.parameter_touches_per_update,
                })
            }
            "bonsai.online-accounting/v2" => {
                let declaration: V2Declaration = serde_json::from_value(value.clone())
                    .map_err(|_| "ACCOUNTING_DECLARATION_INVALID")?;
                declaration.validate()?;
                Ok(Self::V2(declaration))
            }
            _ => Err("ACCOUNTING_DECLARATION_UNSUPPORTED"),
        }
    }

    #[must_use]
    pub const fn touches_per_update(&self) -> u64 {
        match self {
            Self::Legacy => 2,
            Self::V1 { touches_per_update } => *touches_per_update,
            Self::V2(declaration) => declaration.maximum_parameter_touches_per_update,
        }
    }

    /// The v2 maximum is explicit; legacy versions retain only their historical getter.
    #[must_use]
    pub const fn maximum_parameter_touches_per_update(&self) -> Option<u64> {
        match self {
            Self::V2(declaration) => Some(declaration.maximum_parameter_touches_per_update),
            Self::Legacy | Self::V1 { .. } => None,
        }
    }

    #[must_use]
    pub const fn retained_state_limit_bytes(&self) -> Option<u64> {
        match self {
            Self::V2(declaration) => Some(declaration.retained_state_limit_bytes),
            Self::Legacy | Self::V1 { .. } => None,
        }
    }

    #[must_use]
    pub const fn serialized_state_limit_bytes(&self) -> Option<u64> {
        match self {
            Self::V2(declaration) => Some(declaration.serialized_state_limit_bytes),
            Self::Legacy | Self::V1 { .. } => None,
        }
    }

    #[must_use]
    pub const fn is_transition_feedback(&self) -> bool {
        matches!(self, Self::V2(_))
    }

    /// Return stable work-class order for one declared step.
    #[must_use]
    pub fn work_per_step(&self, actions: u64) -> Vec<(WorkClass, u64)> {
        match self {
            Self::V2(declaration) => declaration
                .work_per_step
                .iter()
                .map(|tariff| (tariff.work_class, tariff.amount))
                .collect(),
            Self::Legacy | Self::V1 { .. } => {
                vec![(WorkClass::Acting, actions), (WorkClass::Learning, 1)]
            }
        }
    }

    /// Return the v2 declaration values that an external admission authority needs.
    #[must_use]
    pub fn admission_payload(&self, total: u64) -> Option<Value> {
        let Self::V2(declaration) = self else {
            return None;
        };
        Some(json!({
            "schema": "bonsai.online-admission/v2",
            "step": total,
            "work_per_step": &declaration.work_per_step,
            "retained_state_limit_bytes": declaration.retained_state_limit_bytes,
            "serialized_state_limit_bytes": declaration.serialized_state_limit_bytes,
        }))
    }

    /// Validate counters and an update record linked to the actual chosen action/reward.
    ///
    /// # Errors
    /// Refuses missing, malformed, inconsistent, oversized or unlinked evidence.
    pub fn validate(
        &self,
        measured: &PrimitiveAccounting,
        completed: u64,
        action: u32,
        reward: i64,
    ) -> Result<(), &'static str> {
        match self {
            Self::V2(declaration) => {
                declaration.validate_measured(measured, completed, action, reward)
            }
            Self::Legacy | Self::V1 { .. } => {
                let expected = completed
                    .checked_mul(self.touches_per_update())
                    .ok_or("ACCOUNTING_OVERFLOW")?;
                if measured.environment_steps != completed
                    || measured.updates != completed
                    || measured.parameter_touches != expected
                    || measured.replay_items_retained != 0
                {
                    return Err("ACCOUNTING_INCONSISTENT");
                }
                if measured.parameter_update.is_empty() && matches!(self, Self::Legacy) {
                    return Ok(());
                }
                if measured.parameter_update.len() > MAXIMUM_UPDATE_BYTES {
                    return Err("PARAMETER_UPDATE_TOO_LARGE");
                }
                let update: Value = serde_json::from_slice(&measured.parameter_update)
                    .map_err(|_| "PARAMETER_UPDATE_INVALID")?;
                if update["schema"] != "bonsai.parameter-update/v1"
                    || update["update"].as_u64() != Some(completed)
                    || update["action"].as_u64() != Some(u64::from(action))
                    || update["reward"].as_i64() != Some(reward)
                {
                    return Err("PARAMETER_UPDATE_LINK_MISMATCH");
                }
                for key in ["before", "after"] {
                    let values = update[key].as_array().ok_or("PARAMETER_UPDATE_INVALID")?;
                    if u64::try_from(values.len()).ok() != Some(self.touches_per_update())
                        || values.iter().any(|value| !value.is_number())
                    {
                        return Err("PARAMETER_UPDATE_SHAPE_MISMATCH");
                    }
                }
                Ok(())
            }
        }
    }
}

impl V2Declaration {
    fn validate(&self) -> Result<(), &'static str> {
        if self.schema != "bonsai.online-accounting/v2"
            || self.feedback_signal_type != "bonsai.agent.causal-transition/v1"
            || self.parameter_update_schema != "bonsai.online-update/v2"
            || !(1..=MAXIMUM_PARAMETER_TOUCHES).contains(&self.maximum_parameter_touches_per_update)
            || !(1..=MAXIMUM_RETAINED_STATE_BYTES).contains(&self.retained_state_limit_bytes)
            || !(1..=MAXIMUM_SERIALIZED_STATE_BYTES).contains(&self.serialized_state_limit_bytes)
            || !(2..=LEARNER_WORK_CLASSES.len()).contains(&self.work_per_step.len())
        {
            return Err("ACCOUNTING_DECLARATION_UNSUPPORTED");
        }
        let mut prior_index = None;
        let mut has_acting = false;
        let mut has_learning = false;
        for tariff in &self.work_per_step {
            let Some(index) = LEARNER_WORK_CLASSES
                .iter()
                .position(|work_class| *work_class == tariff.work_class)
            else {
                return Err("ACCOUNTING_DECLARATION_UNSUPPORTED");
            };
            if prior_index.is_some_and(|prior| index <= prior)
                || !(1..=MAXIMUM_WORK_PER_CLASS).contains(&tariff.amount)
            {
                return Err("ACCOUNTING_DECLARATION_UNSUPPORTED");
            }
            prior_index = Some(index);
            has_acting |= tariff.work_class == WorkClass::Acting;
            has_learning |= tariff.work_class == WorkClass::Learning;
        }
        if has_acting && has_learning {
            Ok(())
        } else {
            Err("ACCOUNTING_DECLARATION_UNSUPPORTED")
        }
    }

    fn validate_measured(
        &self,
        measured: &PrimitiveAccounting,
        completed: u64,
        action: u32,
        reward: i64,
    ) -> Result<(), &'static str> {
        let expected_touches = completed
            .checked_mul(self.maximum_parameter_touches_per_update)
            .ok_or("ACCOUNTING_OVERFLOW")?;
        let tariff_total = self.work_per_step.iter().try_fold(0_u64, |total, tariff| {
            total
                .checked_add(tariff.amount)
                .ok_or("ACCOUNTING_OVERFLOW")
        })?;
        let expected_work = completed
            .checked_mul(tariff_total)
            .ok_or("ACCOUNTING_OVERFLOW")?;
        if measured.environment_steps != completed
            || measured.updates != completed
            || measured.parameter_touches > expected_touches
            || measured.work_items != expected_work
            || measured.replay_items_retained != 0
        {
            return Err("ACCOUNTING_INCONSISTENT");
        }
        if measured.parameter_update.len() > MAXIMUM_UPDATE_BYTES {
            return Err("PARAMETER_UPDATE_TOO_LARGE");
        }
        let update: V2Update = serde_json::from_slice(&measured.parameter_update)
            .map_err(|_| "PARAMETER_UPDATE_INVALID")?;
        if update.schema != self.parameter_update_schema
            || update.update != completed
            || update.action != action
            || update.reward != reward
            || update.parameter_touches != measured.parameter_touches
            || !lower_hex_64(&update.state_before)
            || !lower_hex_64(&update.state_after)
            || update.allocated_bytes == 0
            || update.allocated_bytes > self.retained_state_limit_bytes
            || update.serialized_bytes == 0
            || update.serialized_bytes > self.serialized_state_limit_bytes
            || !update.details.is_object()
        {
            return Err("PARAMETER_UPDATE_LINK_MISMATCH");
        }
        if update.work_by_class.len() != self.work_per_step.len() {
            return Err("PARAMETER_UPDATE_SHAPE_MISMATCH");
        }
        for tariff in &self.work_per_step {
            let serialized = serde_json::to_value(tariff.work_class)
                .expect("WorkClass serialization is infallible");
            let name = serialized
                .as_str()
                .expect("WorkClass serialization is a string");
            let expected = completed
                .checked_mul(tariff.amount)
                .ok_or("ACCOUNTING_OVERFLOW")?;
            if update.work_by_class.get(name).and_then(Value::as_u64) != Some(expected) {
                return Err("PARAMETER_UPDATE_SHAPE_MISMATCH");
            }
        }
        Ok(())
    }
}

fn lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
