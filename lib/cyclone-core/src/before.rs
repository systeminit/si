use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeforeFunctionRequest {
    pub execution_id: String,
    pub handler: String,
    pub code_base64: String,
    pub arg: serde_json::Value,
}
