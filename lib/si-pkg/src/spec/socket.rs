use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};

use super::{AttrFuncInputSpec, FuncUniqueId, SpecError};

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
pub enum SocketSpecKind {
    Input,
    Output,
}

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
    Copy,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub enum SocketSpecArity {
    #[default]
    Many,
    One,
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct SocketSpec {
    #[builder(setter(into), default)]
    pub func_unique_id: Option<FuncUniqueId>,

    #[builder(setter(into))]
    pub kind: SocketSpecKind,

    #[builder(setter(into))]
    pub name: String,

    #[builder(setter(into), default)]
    pub arity: SocketSpecArity,

    #[builder(setter(each(name = "input"), into), default)]
    pub inputs: Vec<AttrFuncInputSpec>,

    #[builder(setter(into), default)]
    pub ui_hidden: bool,
}

impl SocketSpec {
    pub fn builder() -> SocketSpecBuilder {
        SocketSpecBuilder::default()
    }
}
