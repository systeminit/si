use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowResolveRequest {
    pub execution_id: String,
    pub handler: String,
    pub code_base64: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowResolveResultSuccess {
    pub execution_id: String,
    pub name: String,
    pub kind: String,

    // TODO: have a struct for these
    pub steps: serde_json::Value,
    pub args: serde_json::Value,

    // Collects the error if the function throws
    pub message: Option<String>,
}
