use derive_builder::Builder;
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    AttrFuncInputSpec,
    HasUniqueId,
    SpecError,
};
use crate::SchemaVariantSpecPropRoot;

/// RootPropFuncs track custom functions for for props that are immediate children of the root.
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct RootPropFuncSpec {
    #[builder(setter(into))]
    pub prop: SchemaVariantSpecPropRoot,
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

impl HasUniqueId for RootPropFuncSpec {
    fn unique_id(&self) -> Option<&str> {
        self.unique_id.as_deref()
    }
}

impl RootPropFuncSpec {
    pub fn builder() -> RootPropFuncSpecBuilder {
        RootPropFuncSpecBuilder::default()
    }
}
