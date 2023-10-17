use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use super::SpecError;

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
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
