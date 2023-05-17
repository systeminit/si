use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionRunRequest {
    pub execution_id: String,
    pub handler: String,
    pub code_base64: String,
    pub args: serde_json::Value,
}

#[remain::sorted]
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum ResourceStatus {
    Error,
    Ok,
    Warning,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionRunResultSuccess {
    pub execution_id: String,
    pub payload: Option<serde_json::Value>,
    pub status: ResourceStatus,
    pub message: Option<String>,
    // Collects the error if the function throws
    pub error: Option<String>,
}
