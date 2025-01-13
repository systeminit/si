use super::SpecError;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Builder, Clone, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
#[ts(export)]
pub struct PositionSpec {
    #[builder(setter(into))]
    pub x: String,
    #[builder(setter(into))]
    pub y: String,
    #[builder(setter(into))]
    pub width: Option<String>,
    #[builder(setter(into))]
    pub height: Option<String>,
}

impl PositionSpec {
    #[must_use]
    pub fn builder() -> PositionSpecBuilder {
        PositionSpecBuilder::default()
    }
}
