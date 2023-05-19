use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use super::{AttrFuncInputSpec, FuncUniqueId, SpecError};

/// MapKeyFuncSpecs track custom functions set on keys to a map
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct MapKeyFuncSpec {
    #[builder(setter(into))]
    pub key: String,
    #[builder(setter(into))]
    pub func_unique_id: FuncUniqueId,
    #[builder(setter(each(name = "input"), into), default)]
    pub inputs: Vec<AttrFuncInputSpec>,
}

impl MapKeyFuncSpec {
    pub fn builder() -> MapKeyFuncSpecBuilder {
        MapKeyFuncSpecBuilder::default()
    }
}
