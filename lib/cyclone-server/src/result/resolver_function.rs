use cyclone_core::ResolverFunctionResultSuccess;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LangServerResolverFunctionResultSuccess {
    pub execution_id: String,
    #[serde(default)]
    pub data: Value,
    pub unset: bool,
}

impl From<LangServerResolverFunctionResultSuccess> for ResolverFunctionResultSuccess {
    fn from(value: LangServerResolverFunctionResultSuccess) -> Self {
        Self {
            execution_id: value.execution_id,
            data: value.data,
            unset: value.unset,
            timestamp: crate::timestamp(),
        }
    }
}
