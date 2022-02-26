use crate::{
    key_pair::{DecryptRequest, KeyPairError},
    sensitive_container::ListSecrets,
    ComponentView, SensitiveString,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeGenerationRequest {
    pub execution_id: String,
    pub handler: String,
    pub component: ComponentView,
    pub code_base64: String,
}

impl ListSecrets for CodeGenerationRequest {
    fn list_secrets(&self) -> Result<Vec<SensitiveString>, KeyPairError> {
        self.component.list_secrets()
    }
}

impl DecryptRequest for CodeGenerationRequest {
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CodeGenerated {
    pub format: String,
    pub code: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeGenerationResultSuccess {
    pub execution_id: String,
    pub data: CodeGenerated,
    #[serde(default = "crate::timestamp")]
    pub timestamp: u64,
}
