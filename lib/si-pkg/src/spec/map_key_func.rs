use super::{AttrFuncInputSpec, SpecError};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// MapKeyFuncSpecs track custom functions set on keys to a map
#[derive(Builder, Clone, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
#[ts(export)]
pub struct MapKeyFuncSpec {
    #[builder(setter(into))]
    pub key: String,
    #[builder(setter(into))]
    pub func_unique_id: String,
    #[builder(setter(each(name = "input"), into), default)]
    pub inputs: Vec<AttrFuncInputSpec>,
}

impl MapKeyFuncSpec {
    pub fn builder() -> MapKeyFuncSpecBuilder {
        MapKeyFuncSpecBuilder::default()
    }
}
