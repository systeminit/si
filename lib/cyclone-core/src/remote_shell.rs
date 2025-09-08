use serde::{
    Deserialize,
    Serialize,
};
use telemetry::prelude::*;
use telemetry_utils::metric;

use crate::CycloneRequestable;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteShellRequest {
    pub execution_id: String,
    pub image: Option<String>,
    pub env_vars: std::collections::HashMap<String, String>,
    pub working_dir: Option<String>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoteShellResultSuccess {
    pub execution_id: String,
    pub session_id: String,
    pub container_id: String,
    pub connection_info: RemoteShellConnectionInfo,
    pub status: RemoteShellStatus,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoteShellConnectionInfo {
    pub nats_subject: String,
    pub stdin_subject: String,
    pub stdout_subject: String,
    pub stderr_subject: String,
    pub control_subject: String,
}

#[remain::sorted]
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum RemoteShellStatus {
    Active,
    Error,
    Terminated,
}

impl CycloneRequestable for RemoteShellRequest {
    type Response = RemoteShellResultSuccess;

    fn execution_id(&self) -> &str {
        &self.execution_id
    }

    fn kind(&self) -> &str {
        "remoteShell"
    }

    fn websocket_path(&self) -> &str {
        "/execute/remote-shell"
    }

    fn inc_run_metric(&self) {
        metric!(counter.function_run.remote_shell = 1);
    }

    fn dec_run_metric(&self) {
        metric!(counter.function_run.remote_shell = -1);
    }
}