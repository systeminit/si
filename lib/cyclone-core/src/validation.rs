use crate::BeforeFunctionRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationRequest {
    pub execution_id: String,
    pub handler: String,
    pub value: serde_json::Value,
    pub code_base64: String,
    pub before: Vec<BeforeFunctionRequest>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResultSuccess {
    pub execution_id: String,
    pub valid: bool,
    pub message: Option<String>,
}
