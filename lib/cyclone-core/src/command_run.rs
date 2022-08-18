use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandRunRequest {
    pub execution_id: String,
    pub handler: String,
    pub code_base64: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandRunResultSuccess {
    pub execution_id: String,
    // Collects the error if the function throws
    pub message: Option<String>,
}
