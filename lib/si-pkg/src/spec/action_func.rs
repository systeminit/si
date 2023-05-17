use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};

use super::{FuncUniqueId, SpecError};

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
pub enum ActionFuncSpecKind {
    Create,
    Refresh,
    Other,
    Delete,
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct ActionFuncSpec {
    #[builder(setter(into))]
    pub func_unique_id: FuncUniqueId,

    #[builder(setter(into))]
    pub kind: ActionFuncSpecKind,
}

impl ActionFuncSpec {
    pub fn builder() -> ActionFuncSpecBuilder {
        ActionFuncSpecBuilder::default()
    }
}
