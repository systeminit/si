use serde::{Deserialize, Serialize};

use crate::ComponentView;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationRequest {
    pub execution_id: String,
    pub handler: String,
    pub component: ComponentView,
    pub code_base64: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationResultSuccess {
    pub execution_id: String,
    pub success: bool,
    pub recommended_actions: Vec<String>,
    pub message: Option<String>,
}
