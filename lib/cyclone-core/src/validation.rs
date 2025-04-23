use serde::{
    Deserialize,
    Serialize,
};
use telemetry::prelude::*;
use telemetry_utils::metric;

use crate::{
    BeforeFunction,
    request::CycloneRequestable,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationRequest {
    pub execution_id: String,
    pub handler: String,
    pub code_base64: String,
    pub value: Option<serde_json::Value>,
    pub validation_format: String,
    pub before: Vec<BeforeFunction>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResultSuccess {
    pub execution_id: String,
    pub error: Option<String>,
}

impl CycloneRequestable for ValidationRequest {
    type Response = ValidationResultSuccess;

    fn execution_id(&self) -> &str {
        &self.execution_id
    }

    fn kind(&self) -> &str {
        "validation"
    }

    fn websocket_path(&self) -> &str {
        "/execute/validation"
    }

    fn inc_run_metric(&self) {
        metric!(counter.function_run.validation = 1);
    }

    fn dec_run_metric(&self) {
        metric!(counter.function_run.validation = -1);
    }
}
