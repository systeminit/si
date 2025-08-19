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
    AttrFuncInputSpec,
    HasUniqueId,
    SpecError,
};

/// SiPropFuncs track custom functions for for props created for all schema variants and not part
/// of the domain tree (which varies for each variant). Currently these are the props under the
/// /root/si Object and also some props in the resource Object that are also invariant across
/// schema variants but which can have custom functions
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
pub enum SiPropFuncSpecKind {
    Color,
    Name,
    ResourcePayload,
}

impl SiPropFuncSpecKind {
    pub fn prop_path(&self) -> Vec<&'static str> {
        match self {
            Self::Name => vec!["root", "si", "name"],
            Self::Color => vec!["root", "si", "color"],
            Self::ResourcePayload => vec!["root", "resource", "payload"],
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
    pub func_unique_id: String,
    #[builder(setter(into), default)]
    #[serde(default)]
    pub unique_id: Option<String>,
    #[builder(setter(into), default)]
    #[serde(default)]
    pub deleted: bool,

    #[builder(setter(each(name = "input"), into), default)]
    pub inputs: Vec<AttrFuncInputSpec>,
}

impl HasUniqueId for SiPropFuncSpec {
    fn unique_id(&self) -> Option<&str> {
        self.unique_id.as_deref()
    }
}

impl SiPropFuncSpec {
    pub fn builder() -> SiPropFuncSpecBuilder {
        SiPropFuncSpecBuilder::default()
    }
}
