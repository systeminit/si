use cyclone_core::ReconciliationResultSuccess;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// This struct contains the lang-js server execution response. All fields without the
/// `#[serde(default)]` macro must be populated.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LangServerReconciliationResultSuccess {
    pub execution_id: String,
    pub updates: HashMap<String, serde_json::Value>,
    pub actions: Vec<String>,
    #[serde(default)]
    pub message: Option<String>,
}

impl From<LangServerReconciliationResultSuccess> for ReconciliationResultSuccess {
    fn from(value: LangServerReconciliationResultSuccess) -> Self {
        Self {
            execution_id: value.execution_id,
            updates: value.updates,
            actions: value.actions,
            message: value.message,
        }
    }
}
