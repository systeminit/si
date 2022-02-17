use crate::ComponentView;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeGenerationRequest {
    pub execution_id: String,
    pub handler: String,
    pub component: ComponentView,
    pub code_base64: String,
}

impl CodeGenerationRequest {
    pub fn deserialize_from_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn serialize_to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CodeGenerated {
    pub format: String,
    pub code: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CodeGenerationResultSuccess {
    pub execution_id: String,
    pub timestamp: u64,
    pub data: CodeGenerated,
}
