use serde::{
    Deserialize,
    Serialize,
};
use telemetry::prelude::*;
use telemetry_utils::metric;

use crate::{
    BeforeFunction,
    ComponentView,
    CycloneRequestable,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugRequest {
    pub execution_id: String,
    pub handler: String,
    pub code_base64: String,
    pub component: ComponentView,
    pub debug_input: Option<serde_json::Value>,
    pub before: Vec<BeforeFunction>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugResultSuccess {
    pub execution_id: String,
    pub output: serde_json::Value,
    pub error: Option<String>,
}

impl CycloneRequestable for DebugRequest {
    type Response = DebugResultSuccess;

    fn execution_id(&self) -> &str {
        &self.execution_id
    }

    fn kind(&self) -> &str {
        "debug"
    }

    fn websocket_path(&self) -> &str {
        "/execute/debug"
    }

    fn inc_run_metric(&self) {
        metric!(counter.function_run.debug = 1);
    }

    fn dec_run_metric(&self) {
        metric!(counter.function_run.debug = -1);
    }
}
