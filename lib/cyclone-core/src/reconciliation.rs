use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationRequest {
    pub execution_id: String,
    pub handler: String,
    pub code_base64: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationResultSuccess {
    pub execution_id: String,
    pub updates: HashMap<String, serde_json::Value>,
    pub actions: Vec<String>,
    pub message: Option<String>,
}
