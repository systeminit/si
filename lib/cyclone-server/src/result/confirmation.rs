use cyclone_core::ConfirmationResultSuccess;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LangServerConfirmationResultSuccess {
    pub execution_id: String,
    pub success: bool,
    pub message: Option<String>,
    #[serde(default)]
    pub recommended_actions: Vec<String>,
}

impl From<LangServerConfirmationResultSuccess> for ConfirmationResultSuccess {
    fn from(value: LangServerConfirmationResultSuccess) -> Self {
        Self {
            execution_id: value.execution_id,
            success: value.success,
            message: value.message,
            recommended_actions: value.recommended_actions,
        }
    }
}
