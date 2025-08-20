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

impl HasUniqueId for AuthenticationFuncSpec {
    fn unique_id(&self) -> Option<&str> {
        self.unique_id.as_deref()
    }
}

impl AuthenticationFuncSpec {
    pub fn builder() -> AuthenticationFuncSpecBuilder {
        AuthenticationFuncSpecBuilder::default()
    }

    pub fn anonymize(&mut self) {
        self.unique_id = None;
    }
}
