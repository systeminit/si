use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use super::{FuncUniqueId, SpecError};

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct CommandFuncSpec {
    #[builder(setter(into))]
    pub func_unique_id: FuncUniqueId,
}

impl CommandFuncSpec {
    pub fn builder() -> CommandFuncSpecBuilder {
        CommandFuncSpecBuilder::default()
    }
}
