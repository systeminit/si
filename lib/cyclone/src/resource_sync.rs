use crate::{
    key_pair::{DecryptRequest, KeyPairError},
    sensitive_container::ListSecrets,
    ComponentView, SensitiveString,
};
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
    fn list_secrets(&self) -> Result<Vec<SensitiveString>, KeyPairError> {
        self.component.list_secrets()
    }
}

impl DecryptRequest for ResourceSyncRequest {
    fn decrypt_request(self) -> Result<serde_json::Value, KeyPairError> {
        let mut value = serde_json::to_value(&self)?;

        match value.pointer_mut("/component") {
            Some(v) => *v = self.component.decrypt_request()?,
            None => {
                return Err(KeyPairError::JSONPointerNotFound(
                    value,
                    "/component".to_owned(),
                ))
            }
        }
        Ok(value)
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
