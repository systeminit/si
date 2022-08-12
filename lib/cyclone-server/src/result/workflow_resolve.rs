use cyclone_core::WorkflowResolveResultSuccess;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LangServerWorkflowResolveResultSuccess {
    pub execution_id: String,
    pub name: String,
    pub kind: String,

    // TODO: have a struct for these
    pub steps: serde_json::Value,
    #[serde(default)]
    pub args: serde_json::Value,

    // Collects the error if the function throws
    #[serde(default)]
    pub message: Option<String>,
}

impl From<LangServerWorkflowResolveResultSuccess> for WorkflowResolveResultSuccess {
    fn from(value: LangServerWorkflowResolveResultSuccess) -> Self {
        Self {
            execution_id: value.execution_id,
            name: value.name,
            kind: value.kind,
            steps: value.steps,
            args: value.args,
            message: value.message,
        }
    }
}
