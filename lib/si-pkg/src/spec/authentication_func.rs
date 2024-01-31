use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use super::SpecError;

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct AuthenticationFuncSpec {
    #[builder(setter(into))]
    pub func_unique_id: String,

    #[builder(setter(into), default)]
    pub name: Option<String>,

    #[builder(setter(into), default)]
    #[serde(default)]
    pub unique_id: Option<String>,

    #[builder(setter(into), default)]
    #[serde(default)]
    pub deleted: bool,
}

impl AuthenticationFuncSpec {
    pub fn builder() -> AuthenticationFuncSpecBuilder {
        AuthenticationFuncSpecBuilder::default()
    }
}
