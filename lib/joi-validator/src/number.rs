use std::f64;

use serde::Deserialize;

use crate::{generic, require, rule_err, Args};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Validator {
    #[serde(default)]
    rules: Vec<Rule>,
    #[serde(flatten)]
    pub base: generic::Validator<f64, Flags>,
}

impl Validator {
    fn validate_safe(&self, value: f64) -> Result<(), (String, String)> {
        if !self.base.flags.extra_flags.r#unsafe {
            require(
                (JS_MIN_SAFE_INTEGER..=JS_MAX_SAFE_INTEGER).contains(&value),
                "number.unsafe",
                "unsafe number",
            )?;
        }
        Ok(())
    }
}

impl Validator {
    pub fn validate(self, value: &Option<serde_json::Value>) -> Result<(), (String, String)> {
        self.base.validate_presence(value)?;
        if let Some(value) = value {
            let value = match value {
                serde_json::Value::Number(number) => number.as_f64(),
                serde_json::Value::String(string) => string.trim().parse().ok(),
                _ => None,
            }
            .ok_or_else(|| rule_err("number.base", "must be a number"))?;

            // Now that we have the float, validate it
            self.validate_safe(value)?;
            self.base.validate_value(&value)?;
            for rule in self.rules {
                rule.validate(&value)?;
            }
        }
        Ok(())
    }

    pub fn rule_names(&self) -> Vec<&'static str> {
        let mut rule_names = self.base.rule_names();
        rule_names.push("number.base");
        if !self.base.flags.extra_flags.r#unsafe {
            rule_names.push("number.unsafe");
        }
        rule_names.extend(self.rules.iter().map(Rule::rule_name));
        rule_names
    }
}

const JS_MIN_SAFE_INTEGER: f64 = -9007199254740991.0;
const JS_MAX_SAFE_INTEGER: f64 = 9007199254740991.0;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(tag = "name")]
enum Rule {
    Greater(Args<Limit>),
    Integer,
    Less(Args<Limit>),
    Max(Args<Limit>),
    Min(Args<Limit>),
    // Multiple(Args<Multiple>),
    // Port,
    // Precision(Precision),
    // Sign(Args<SignArgs>),
}

impl Rule {
    fn validate(&self, value: &f64) -> Result<(), (String, String)> {
        match self {
            Rule::Integer => require(value.fract() == 0.0, "number.integer", "must be an integer"),
            Rule::Greater(rule) => require(
                value > &rule.args.limit,
                "number.greater",
                format!("must be greater than {}", rule.args.limit),
            ),
            Rule::Less(rule) => require(
                value < &rule.args.limit,
                "number.less",
                format!("must be less than {}", rule.args.limit),
            ),
            Rule::Max(rule) => require(
                value <= &rule.args.limit,
                "number.max",
                format!("must be less than or equal to {}", rule.args.limit),
            ),
            Rule::Min(rule) => require(
                value >= &rule.args.limit,
                "number.min",
                format!("must be greater than or equal to {}", rule.args.limit),
            ),
        }
    }

    fn rule_name(&self) -> &'static str {
        match self {
            Rule::Greater(_) => "number.greater",
            Rule::Integer => "number.integer",
            Rule::Less(_) => "number.less",
            Rule::Max(_) => "number.max",
            Rule::Min(_) => "number.min",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct Limit {
    limit: f64,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Flags {
    // Set true to not check if the number is in the safe range
    #[serde(default)]
    r#unsafe: bool,
}

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// struct Precision {
//     limit: u64,
// }

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// struct Multiple {
//     base: f64,
//     base_decimal_place: u64,
//     pfactor: f64,
// }

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// struct SignArgs {
//     sign: Sign,
// }

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
// enum Sign {
//     Negative,
//     Positive,
// }
