use crate::{
    key_pair::{DecryptRequest, KeyPairError},
    sensitive_container::ListSecrets,
    ComponentView, SensitiveString,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverFunctionRequest {
    pub execution_id: String,
    pub handler: String,
    pub component: ResolverFunctionComponent,
    pub code_base64: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResolverFunctionComponent {
    pub data: ComponentView,
    pub parents: Vec<ComponentView>,
    // TODO: add widget data here (for example select's options)
}

impl ListSecrets for ResolverFunctionRequest {
    fn list_secrets(&self) -> Result<Vec<SensitiveString>, KeyPairError> {
        let mut secrets = self.component.data.list_secrets()?;
        for component in &self.component.parents {
            secrets.extend(component.list_secrets()?);
        }
        Ok(secrets)
    }
}

impl DecryptRequest for ResolverFunctionRequest {
    fn decrypt_request(self) -> Result<serde_json::Value, KeyPairError> {
        let mut value = serde_json::to_value(&self)?;

        let (component, parents) = (self.component.data, self.component.parents);

        match value.pointer_mut("/component/data") {
            Some(v) => *v = component.decrypt_request()?,
            None => {
                return Err(KeyPairError::JSONPointerNotFound(
                    value,
                    "/component/data".to_owned(),
                ))
            }
        }

        let mut decrypted_parents = Vec::with_capacity(parents.len());
        for parent in parents {
            decrypted_parents.push(parent.decrypt_request()?);
        }
        match value.pointer_mut("/component/parents") {
            Some(v) => *v = serde_json::Value::Array(decrypted_parents),
            None => {
                return Err(KeyPairError::JSONPointerNotFound(
                    value,
                    "/component/parents".to_owned(),
                ))
            }
        }
        Ok(value)
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverFunctionResultSuccess {
    pub execution_id: String,
    pub data: Value,
    pub unset: bool,
    #[serde(default = "crate::timestamp")]
    pub timestamp: u64,
}
