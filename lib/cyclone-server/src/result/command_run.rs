use cyclone_core::CommandRunResultSuccess;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LangServerCommandRunResultSuccess {
    pub execution_id: String,
    #[serde(default)]
    pub updated: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub created: HashMap<String, serde_json::Value>,
    // Collects the error if the function throws
    #[serde(default)]
    pub error: Option<String>,
}

impl From<LangServerCommandRunResultSuccess> for CommandRunResultSuccess {
    fn from(value: LangServerCommandRunResultSuccess) -> Self {
        Self {
            execution_id: value.execution_id,
            error: value.error,
            updated: value.updated,
            created: value.created,
        }
    }
}
