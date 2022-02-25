use crate::{sensitive_container::ListSecrets, ComponentView, SensitiveString};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSyncRequest {
    pub execution_id: String,
    pub handler: String,
    pub component: ComponentView,
    pub code_base64: String,
}

impl ListSecrets for ResourceSyncRequest {
    fn list_secrets(&self) -> Vec<SensitiveString> {
        self.component.list_secrets()
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSyncResultSuccess {
    pub execution_id: String,
    pub data: serde_json::Value,
    #[serde(default = "crate::timestamp")]
    pub timestamp: u64,
}
