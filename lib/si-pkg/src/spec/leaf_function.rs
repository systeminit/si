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

#[remain::sorted]
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
pub enum LeafKind {
    CodeGeneration,
    Qualification,
}

#[remain::sorted]
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
pub enum LeafInputLocation {
    Code,
    DeletedAt,
    Domain,
    Resource,
    Secrets,
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct LeafFunctionSpec {
    #[builder(setter(into))]
    pub func_unique_id: String,

    #[builder(setter(into))]
    pub leaf_kind: LeafKind,

    #[builder(setter(into), default)]
    #[serde(default)]
    pub unique_id: Option<String>,

    #[builder(setter(into), default)]
    #[serde(default)]
    pub deleted: bool,

    #[builder(setter(into), default)]
    pub inputs: Vec<LeafInputLocation>,
}

impl HasUniqueId for LeafFunctionSpec {
    fn unique_id(&self) -> Option<&str> {
        self.unique_id.as_deref()
    }
}

impl LeafFunctionSpec {
    pub fn builder() -> LeafFunctionSpecBuilder {
        LeafFunctionSpecBuilder::default()
    }

    pub fn anonymize(&mut self) {
        self.unique_id = None;
    }
}
