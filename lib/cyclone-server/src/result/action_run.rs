use cyclone_core::{ActionRunResultSuccess, ResourceStatus};
use serde::{Deserialize, Serialize};

/// This struct contains the lang-js server execution response. All fields without the
/// `#[serde(default)]` macro must be populated.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LangServerActionRunResultSuccess {
    pub execution_id: String,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
    pub health: ResourceStatus,
    #[serde(default)]
    pub message: Option<String>,
    // Collects the error if the function throws
    #[serde(default)]
    pub error: Option<String>,
}

impl From<LangServerActionRunResultSuccess> for ActionRunResultSuccess {
    fn from(value: LangServerActionRunResultSuccess) -> Self {
        Self {
            execution_id: value.execution_id,
            error: value.error,
            status: value.health,
            message: value.message,
            payload: value.payload,
        }
    }
}
