use serde::{
    Deserialize,
    Serialize,
};

use crate::CycloneRequestable;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KillExecutionRequest {
    pub execution_id: String,
}

impl CycloneRequestable for KillExecutionRequest {
    type Response = ();

    fn execution_id(&self) -> &str {
        &self.execution_id
    }

    fn kind(&self) -> &str {
        ""
    }

    fn websocket_path(&self) -> &str {
        ""
    }

    fn inc_run_metric(&self) {}

    fn dec_run_metric(&self) {}
}
