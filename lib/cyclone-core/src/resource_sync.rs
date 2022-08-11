use serde::{Deserialize, Serialize};

use crate::ComponentView;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSyncRequest {
    pub execution_id: String,
    pub handler: String,
    pub component: ComponentView,
    pub code_base64: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSyncResultSuccess {
    pub execution_id: String,
    pub data: serde_json::Value,
    pub timestamp: u64,
}
