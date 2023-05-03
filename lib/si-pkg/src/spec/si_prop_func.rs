use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};

use super::{AttrFuncInputSpec, FuncUniqueId, SpecError};

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
pub enum SiPropFuncSpecKind {
    Name,
    Color,
}

impl SiPropFuncSpecKind {
    pub fn prop_path(&self) -> Vec<&'static str> {
        match self {
            Self::Name => vec!["root", "si", "name"],
            Self::Color => vec!["root", "si", "color"],
        }
    }
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct SiPropFuncSpec {
    #[builder(setter(into))]
    pub kind: SiPropFuncSpecKind,
    #[builder(setter(into))]
    pub func_unique_id: FuncUniqueId,
    #[builder(setter(each(name = "input"), into), default)]
    pub inputs: Vec<AttrFuncInputSpec>,
}

impl SiPropFuncSpec {
    pub fn builder() -> SiPropFuncSpecBuilder {
        SiPropFuncSpecBuilder::default()
    }
}
