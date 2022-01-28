use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSyncRequest {
    pub execution_id: String,
    pub handler: String,
    pub component: ResourceSyncComponent,
    pub code_base64: String,
}

impl ResourceSyncRequest {
    pub fn deserialize_from_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn serialize_to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSyncComponent {
    pub name: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResourceSyncResultSuccess {
    pub execution_id: String,
    pub timestamp: u64,
    pub data: serde_json::Value,
}
