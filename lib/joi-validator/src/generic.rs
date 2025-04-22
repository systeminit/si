use std::fmt::Debug;

use serde::{
    Deserialize,
    Serialize,
};

use crate::require;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Validator<T, ExtraFlags> {
    // Flags that affect rules (such as "only" and "presence")
    #[serde(default)]
    pub flags: Flags<ExtraFlags>,
    // Allowed values (only checked if flags.only is set)
    pub allow: Option<Vec<T>>,
    // Disallowed values
    pub invalid: Option<Vec<T>>,

    // Preferences like whether to convert values
    // pub preferences: Option<Preferences>,

    // Descriptive stuff that's OK to pass through
    pub examples: Option<Vec<T>>,
    pub metas: Option<Vec<serde_json::Value>>,
    pub notes: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

impl<T: PartialEq + Serialize + Debug, ExtraFlags> Validator<T, ExtraFlags> {
    pub fn validate_value(&self, value: &T) -> Result<(), (String, String)> {
        self.validate_valid_values(value)
    }

    pub fn validate_presence<V>(&self, value: &Option<V>) -> Result<(), (String, String)> {
        match self.flags.presence {
            Some(Presence::Required) => require(value.is_some(), "any.required", "is required"),
            Some(Presence::Forbidden) => require(value.is_none(), "any.unknown", "is not allowed"),
            Some(Presence::Optional) | None => Ok(()),
        }
    }

    fn validate_valid_values(&self, value: &T) -> Result<(), (String, String)> {
        // Check if it's one of the allowed values
        if self.flags.only {
            if let Some(allow) = &self.allow {
                require(
                    allow.contains(value),
                    "any.only",
                    format!("must be one of {:?}", self.allow),
                )?;
            }
        }
        // Check for invalid values
        if let Some(invalid) = &self.invalid {
            require(
                !invalid.contains(value),
                "any.invalid",
                "contains an invalid value",
            )?;
        }
        Ok(())
    }

    pub fn rule_names(&self) -> Vec<&'static str> {
        let mut rule_names = vec![];
        match self.flags.presence {
            Some(Presence::Required) => rule_names.push("any.required"),
            Some(Presence::Forbidden) => rule_names.push("any.unknown"),
            Some(Presence::Optional) | None => (),
        }
        if self.flags.only {
            if let Some(_allow) = &self.allow {
                rule_names.push("any.only");
            }
        }
        if let Some(_invalid) = &self.invalid {
            rule_names.push("any.invalid");
        }
        rule_names
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Flags<ExtraFlags> {
    #[serde(default)]
    pub only: bool, // Defaults to false
    pub presence: Option<Presence>,
    // pub unit: Option<String>,

    // Label to use in error messages
    pub label: Option<String>,

    // Purely descriptive
    pub description: Option<String>,

    #[serde(flatten)]
    pub extra_flags: ExtraFlags,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum Presence {
    Forbidden,
    Optional,
    Required,
}

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// pub struct Preferences {
//     pub convert: Option<bool>,
// }
