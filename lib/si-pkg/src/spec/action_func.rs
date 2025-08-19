use derive_builder::Builder;
use serde::{
    Deserialize,
    Serialize,
};
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};

use super::{
    HasUniqueId,
    SpecError,
};

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
    Update,
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct ActionFuncSpec {
    #[builder(setter(into))]
    pub func_unique_id: String,

    #[builder(setter(into), default)]
    pub name: Option<String>,

    #[builder(setter(into))]
    pub kind: ActionFuncSpecKind,

    #[builder(setter(into), default)]
    #[serde(default)]
    pub unique_id: Option<String>,

    #[builder(setter(into), default)]
    #[serde(default)]
    pub deleted: bool,
}

impl HasUniqueId for ActionFuncSpec {
    fn unique_id(&self) -> Option<&str> {
        self.unique_id.as_deref()
    }
}

impl ActionFuncSpec {
    pub fn builder() -> ActionFuncSpecBuilder {
        ActionFuncSpecBuilder::default()
    }

    pub fn anonymize(&mut self) {
        self.unique_id = None;
    }
}
