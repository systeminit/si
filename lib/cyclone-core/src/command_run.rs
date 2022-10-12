use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandRunRequest {
    pub execution_id: String,
    pub handler: String,
    pub code_base64: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandRunResultSuccess {
    pub execution_id: String,
    pub value: serde_json::Value,
    pub created: bool,
    // Collects the error if the function throws
    pub error: Option<String>,
}
