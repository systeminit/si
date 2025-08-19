use derive_builder::Builder;
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    HasUniqueId,
    SpecError,
};

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct ManagementFuncSpec {
    #[builder(setter(into))]
    pub func_unique_id: String,

    #[builder(setter(into))]
    pub name: String,

    #[builder(setter(into), default)]
    pub description: Option<String>,
}

impl HasUniqueId for ManagementFuncSpec {
    fn unique_id(&self) -> Option<&str> {
        Some(&self.func_unique_id) // Always has a value
    }
}

impl ManagementFuncSpec {
    pub fn builder() -> ManagementFuncSpecBuilder {
        ManagementFuncSpecBuilder::default()
    }

    pub fn anonymize(&mut self) {}
}
