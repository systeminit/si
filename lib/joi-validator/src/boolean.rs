use serde::Deserialize;

use crate::{
    generic,
    rule_err,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Validator {
    #[serde(default)]
    rules: Vec<Rule>,
    // #[serde(default)]
    // falsy: Vec<String>,
    // #[serde(default)]
    // truthy: Vec<String>,
    #[serde(flatten)]
    pub base: generic::Validator<bool, Flags>,
}

impl Validator {
    pub fn validate(self, value: &Option<serde_json::Value>) -> Result<(), (String, String)> {
        self.base.validate_presence(value)?;
        if let Some(value) = value {
            let value = match value {
                serde_json::Value::Bool(boolean) => Some(*boolean),
                serde_json::Value::String(string) => {
                    if string.eq_ignore_ascii_case("true") {
                        Some(true)
                    } else if string.eq_ignore_ascii_case("false") {
                        Some(false)
                    } else {
                        None
                    }
                }
                _ => None,
            }
            .ok_or(rule_err("boolean.base", "must be a boolean"))?;

            // Now that we have the boolean, validate it
            self.base.validate_value(&value)?;
            for rule in self.rules {
                rule.validate(&value)?;
            }
        }
        Ok(())
    }

    pub fn rule_names(&self) -> Vec<&'static str> {
        let mut rule_names = self.base.rule_names();
        rule_names.push("boolean.base");
        rule_names
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(tag = "name")]
enum Rule {}

impl Rule {
    fn validate(&self, _value: &bool) -> Result<(), (String, String)> {
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Flags {
    // // Set true to not check if the number is in the safe range
    // #[serde(default)]
    // sensitive: bool,
}
