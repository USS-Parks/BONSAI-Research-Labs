//! Generic online parameter accounting; historical manifests keep their exact v1 interpretation.
use crate::bonsai::adapter::v1::PrimitiveAccounting;
use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Copy, Debug)]
pub struct OnlineAccounting {
    touches_per_update: u64,
    require_parameter_update: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Declaration {
    schema: String,
    parameter_touches_per_update: u64,
    parameter_update_schema: String,
}

impl OnlineAccounting {
    /// Parse an explicit immutable declaration, or the established historical contract.
    ///
    /// # Errors
    /// Refuses unknown versions, malformed fields and unsupported bounds.
    pub fn from_declaration_or_legacy(value: Option<&Value>) -> Result<Self, &'static str> {
        let Some(value) = value else {
            return Ok(Self {
                touches_per_update: 2,
                require_parameter_update: false,
            });
        };
        let declaration: Declaration =
            serde_json::from_value(value.clone()).map_err(|_| "ACCOUNTING_DECLARATION_INVALID")?;
        if declaration.schema != "bonsai.online-accounting/v1"
            || declaration.parameter_update_schema != "bonsai.parameter-update/v1"
            || !(1..=257).contains(&declaration.parameter_touches_per_update)
        {
            return Err("ACCOUNTING_DECLARATION_UNSUPPORTED");
        }
        Ok(Self {
            touches_per_update: declaration.parameter_touches_per_update,
            require_parameter_update: true,
        })
    }

    #[must_use]
    pub const fn touches_per_update(self) -> u64 {
        self.touches_per_update
    }

    /// Validate counters and an update record linked to the actual chosen action/reward.
    ///
    /// # Errors
    /// Refuses missing, malformed, inconsistent, oversized or unlinked evidence.
    pub fn validate(
        self,
        measured: &PrimitiveAccounting,
        completed: u64,
        action: u32,
        reward: i64,
    ) -> Result<(), &'static str> {
        let expected = completed
            .checked_mul(self.touches_per_update)
            .ok_or("ACCOUNTING_OVERFLOW")?;
        if measured.environment_steps != completed
            || measured.updates != completed
            || measured.parameter_touches != expected
            || measured.replay_items_retained != 0
        {
            return Err("ACCOUNTING_INCONSISTENT");
        }
        if measured.parameter_update.is_empty() && !self.require_parameter_update {
            return Ok(());
        }
        if measured.parameter_update.len() > 32 * 1024 {
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
            if u64::try_from(values.len()).ok() != Some(self.touches_per_update)
                || values.iter().any(|value| !value.is_number())
            {
                return Err("PARAMETER_UPDATE_SHAPE_MISMATCH");
            }
        }
        Ok(())
    }
}
