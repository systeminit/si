use cyclone_core::CommandRunResultSuccess;
use serde::{Deserialize, Serialize};

/// This struct contains the lang-js server execution response. All fields without the
/// `#[serde(default)]` macro must be populated.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LangServerCommandRunResultSuccess {
    pub execution_id: String,
    #[serde(default)]
    pub value: Option<serde_json::Value>,
    #[serde(default)]
    pub created: Option<bool>,
    // Collects the error if the function throws
    #[serde(default)]
    pub error: Option<String>,
}

impl From<LangServerCommandRunResultSuccess> for CommandRunResultSuccess {
    fn from(value: LangServerCommandRunResultSuccess) -> Self {
        Self {
            execution_id: value.execution_id,
            error: value.error,
            value: value.value.unwrap_or(serde_json::Value::Null),
            created: value.created.unwrap_or(false),
        }
    }
}
