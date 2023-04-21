use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};

use super::{FuncUniqueId, SpecError};

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct WorkflowSpec {
    #[builder(setter(into))]
    pub func_unique_id: FuncUniqueId,

    #[builder(setter(into))]
    pub title: String,

    #[builder(setter(each(name = "action"), into), default)]
    pub actions: Vec<ActionSpec>,
}

impl WorkflowSpec {
    pub fn builder() -> WorkflowSpecBuilder {
        WorkflowSpecBuilder::default()
    }
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Eq,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
    Copy,
)]
#[serde(rename_all = "camelCase")]
pub enum ActionSpecKind {
    Create,
    Refresh,
    Other,
    Destroy,
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct ActionSpec {
    #[builder(setter(into))]
    pub name: String,

    #[builder(setter(into))]
    pub kind: ActionSpecKind,
}

impl ActionSpec {
    pub fn builder() -> ActionSpecBuilder {
        ActionSpecBuilder::default()
    }
}
