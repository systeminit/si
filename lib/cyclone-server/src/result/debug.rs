use cyclone_core::DebugResultSuccess;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LangServerDebugResultSuccess {
    pub execution_id: String,
    pub output: serde_json::Value,
    pub error: Option<String>,
}

impl From<LangServerDebugResultSuccess> for DebugResultSuccess {
    fn from(value: LangServerDebugResultSuccess) -> Self {
        Self {
            execution_id: value.execution_id,
            output: value.output,
            error: value.error,
        }
    }
}
