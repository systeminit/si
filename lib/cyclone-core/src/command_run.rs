use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub updated: HashMap<String, serde_json::Value>,
    pub created: HashMap<String, serde_json::Value>,
    // Collects the error if the function throws
    pub error: Option<String>,
}
