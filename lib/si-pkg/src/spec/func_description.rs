use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use super::{FuncUniqueId, SpecError};

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct FuncDescriptionSpec {
    #[builder(setter(into))]
    pub func_unique_id: FuncUniqueId,

    #[builder(setter(into))]
    pub contents: serde_json::Value,
}

impl FuncDescriptionSpec {
    pub fn builder() -> FuncDescriptionSpecBuilder {
        FuncDescriptionSpecBuilder::default()
    }
}
