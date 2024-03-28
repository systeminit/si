use crate::BeforeFunction;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationRequest {
    pub execution_id: String,
    pub handler: String,
    pub code_base64: String,
    pub value: Option<serde_json::Value>,
    pub validation_format: String,
    pub before: Vec<BeforeFunction>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResultSuccess {
    pub execution_id: String,
    pub error: Option<String>,
}
