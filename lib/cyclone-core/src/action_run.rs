use serde::{
    Deserialize,
    Serialize,
};
use telemetry::prelude::*;
use telemetry_utils::metric;

use crate::{
    BeforeFunction,
    CycloneRequestable,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionRunRequest {
    pub execution_id: String,
    pub handler: String,
    pub code_base64: String,
    pub args: serde_json::Value,
    pub before: Vec<BeforeFunction>,
}

#[remain::sorted]
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum ResourceStatus {
    Error,
    Ok,
    Warning,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActionRunResultSuccess {
    pub execution_id: String,
    pub resource_id: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub status: ResourceStatus,
    pub message: Option<String>,
    // Collects the error if the function throws
    pub error: Option<String>,
}

impl CycloneRequestable for ActionRunRequest {
    type Response = ActionRunResultSuccess;

    fn execution_id(&self) -> &str {
        &self.execution_id
    }

    fn kind(&self) -> &str {
        "actionRun"
    }

    fn websocket_path(&self) -> &str {
        "/execute/command"
    }

    fn inc_run_metric(&self) {
        metric!(counter.function_run.action = 1);
    }

    fn dec_run_metric(&self) {
        metric!(counter.function_run.action = -1);
    }
}
