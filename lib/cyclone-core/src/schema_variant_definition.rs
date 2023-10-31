use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantDefinitionRequest {
    pub execution_id: String,
    pub handler: String,
    pub code_base64: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantDefinitionResultSuccess {
    pub execution_id: String,
    pub definition: serde_json::Value,
    // Collects the error if the function throws
    #[serde(default)]
    pub error: Option<String>,
}
