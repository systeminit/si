use crate::{
    key_pair::{DecryptRequest, KeyPairError},
    sensitive_container::ListSecrets,
    CodeGenerated, ComponentView, SensitiveString,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QualificationCheckRequest {
    pub execution_id: String,
    pub handler: String,
    pub component: QualificationCheckComponent,
    pub code_base64: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QualificationCheckComponent {
    pub data: ComponentView,
    pub parents: Vec<ComponentView>,
    pub codes: Vec<CodeGenerated>,
}

impl DecryptRequest for QualificationCheckRequest {
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

impl ListSecrets for QualificationCheckRequest {
    fn list_secrets(&self) -> Result<Vec<SensitiveString>, KeyPairError> {
        let mut secrets = self.component.data.list_secrets()?;
        for component in &self.component.parents {
            secrets.extend(component.list_secrets()?);
        }
        Ok(secrets)
    }
}

// Note: these map 1:1 to the DAL qualificationsubcheck data in the qualification view.
//       perhaps they should live permanently here, and be exported via veritech?
//       for now I'm duplicating, so we can experiement with it.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum QualificationSubCheckStatus {
    Success,
    Failure,
    Unknown,
}

impl Default for QualificationSubCheckStatus {
    fn default() -> Self {
        QualificationSubCheckStatus::Unknown
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct QualificationSubCheck {
    pub description: String,
    pub status: QualificationSubCheckStatus,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QualificationCheckResultSuccess {
    pub execution_id: String,
    pub qualified: bool,
    pub title: Option<String>,
    pub link: Option<String>,
    pub sub_checks: Option<Vec<QualificationSubCheck>>,
    pub message: Option<String>,
    #[serde(default = "crate::timestamp")]
    pub timestamp: u64,
}
