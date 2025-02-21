use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{generic, Args, Limit};

pub type Validator = generic::Validator<DateTime<Utc>, Rule, Flags>;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(tag = "name")]
pub enum Rule {
    Greater(Args<Limit<DateTime<Utc>>>),
    Less(Args<Limit<DateTime<Utc>>>),
    Max(Args<Limit<DateTime<Utc>>>),
    Min(Args<Limit<DateTime<Utc>>>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Flags {
    pub format: Option<Format>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Format {
    Iso,        // Joi.date().iso()
    Javascript, // Joi.date().timestamp()
}
