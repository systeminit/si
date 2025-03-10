//! This crate validates a subset of Joi schema rules in Rust.

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    // missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;

// These are commented out until we decide to support them in any way.
// pub mod alternatives;
// pub mod date;
mod boolean;
pub mod generic;
mod number;
mod string;
#[cfg(test)]
mod test;

#[remain::sorted]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unreachable")]
    Unreachable,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(tag = "type")]
pub enum Validator {
    // Alternatives(alternatives::Validator),
    Boolean(boolean::Validator),
    // Date(date::Validator),
    Number(number::Validator),
    String(string::Validator),
}

impl Validator {
    // Takes in an optional value, validates it according to the rules, and returns a response
    // in Joi format.
    pub fn validate(
        mut self,
        value: &Option<serde_json::Value>,
    ) -> ValidateResponse<Option<serde_json::Value>> {
        let label = self.take_label().unwrap_or("value".into());
        let result = match self {
            Validator::Boolean(validator) => validator.validate(value),
            Validator::Number(validator) => validator.validate(value),
            Validator::String(validator) => validator.validate(value),
        };
        ValidateResponse {
            value: value.clone(),
            error: match result {
                Ok(()) => None,
                Err((r#type, message)) => Some(ValidateError {
                    _original: value.clone(),
                    details: vec![ValidateErrorDetails {
                        message: format!("{} {}", Self::to_json_string(&label), message),
                        r#type,
                        path: vec![],
                        context: ValidateContext {
                            label,
                            value: value.clone(),
                            extra: None,
                        },
                    }],
                }),
            },
            warning: None,
            artifacts: None,
        }
    }

    // Outputs a JSON string, with quotes and escapes
    fn to_json_string(string: &str) -> String {
        serde_json::Value::from(string).to_string()
    }

    fn take_label(&mut self) -> Option<String> {
        match self {
            Validator::Boolean(validator) => validator.base.flags.label.take(),
            Validator::Number(validator) => validator.base.flags.label.take(),
            Validator::String(validator) => validator.base.flags.label.take(),
        }
    }

    pub fn rule_names(&self) -> Vec<&'static str> {
        match self {
            Validator::Boolean(validator) => validator.rule_names(),
            Validator::Number(validator) => validator.rule_names(),
            Validator::String(validator) => validator.rule_names(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ValidateResponse<T> {
    pub value: T,
    pub error: Option<ValidateError<T>>,
    pub warning: Option<ValidateError<T>>,
    pub artifacts: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ValidateError<T> {
    pub _original: T,
    pub details: Vec<ValidateErrorDetails<T>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ValidateErrorDetails<T> {
    pub message: String,
    pub path: Vec<StringOrNumber>,
    pub r#type: String,
    pub context: ValidateContext<T>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ValidateContext<T> {
    pub label: String,
    pub value: T,
    #[serde(flatten)]
    pub extra: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum StringOrNumber {
    String(String),
    Number(usize),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Args<T> {
    pub args: T,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Options<T> {
    pub options: T,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OneOrMany<T: DeserializeOwned>(#[serde_as(as = "serde_with::OneOrMany<_>")] pub Vec<T>);

fn rule_err(r#type: impl Into<String>, message: impl Into<String>) -> (String, String) {
    (r#type.into(), message.into())
}

fn require(
    condition: bool,
    r#type: impl Into<String>,
    message: impl Into<String>,
) -> Result<(), (String, String)> {
    if condition {
        Ok(())
    } else {
        Err(rule_err(r#type, message))
    }
}
