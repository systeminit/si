use super::SpecError;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};
use ts_rs::TS;

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
    TS,
)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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
    TS,
)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum LeafInputLocation {
    Code,
    DeletedAt,
    Domain,
    Resource,
    Secrets,
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
#[ts(export)]
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

impl LeafFunctionSpec {
    pub fn builder() -> LeafFunctionSpecBuilder {
        LeafFunctionSpecBuilder::default()
    }
}
