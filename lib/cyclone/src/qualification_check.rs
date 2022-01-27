use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QualificationCheckRequest {
    pub execution_id: String,
    pub handler: String,
    pub component: QualificationCheckComponent,
    pub code_base64: String,
}

impl QualificationCheckRequest {
    pub fn deserialize_from_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn serialize_to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QualificationCheckComponent {
    pub name: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct QualificationCheckResultSuccess {
    pub execution_id: String,
    pub qualified: bool,
    pub message: Option<String>,
    pub timestamp: u64,
}
