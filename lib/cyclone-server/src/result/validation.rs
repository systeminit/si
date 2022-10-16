use cyclone_core::ValidationResultSuccess;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LangServerValidationResultSuccess {
    pub execution_id: String,
    pub valid: bool,
    pub message: Option<String>,
}

impl From<LangServerValidationResultSuccess> for ValidationResultSuccess {
    fn from(value: LangServerValidationResultSuccess) -> Self {
        Self {
            execution_id: value.execution_id,
            valid: value.valid,
            message: value.message,
        }
    }
}
