use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use telemetry::tracing::trace;
use veritech_client::{
    ActionRunRequest, ActionRunResultSuccess, FunctionResult, OutputStream, ResourceStatus,
};

use crate::func::backend::{
    ExtractPayload, FuncBackendError, FuncBackendResult, FuncDispatch, FuncDispatchContext,
};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct FuncBackendJsActionArgs(serde_json::Value);

#[derive(Debug)]
pub struct FuncBackendJsAction {
    pub context: FuncDispatchContext,
    pub request: ActionRunRequest,
}

#[async_trait]
impl FuncDispatch for FuncBackendJsAction {
    type Args = FuncBackendJsActionArgs;
    type Output = ActionRunResultSuccess;

    fn new(
        context: FuncDispatchContext,
        code_base64: &str,
        handler: &str,
        args: Self::Args,
    ) -> Box<Self> {
        let request = ActionRunRequest {
            // Once we start tracking the state of these executions, then this id will be useful,
            // but for now it's passed along and back, and is opaue
            execution_id: "ayrtonsennajscommand".to_string(),
            handler: handler.into(),
            code_base64: code_base64.into(),
            args: serde_json::to_value(args).unwrap(),
        };

        Box::new(Self { context, request })
    }

    /// This private function dispatches the assembled request to veritech for execution.
    /// This is the "last hop" function in the dal before using the veritech client directly.
    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>> {
        let (veritech, output_tx) = self.context.into_inner();
        let value = veritech
            .execute_action_run(output_tx.clone(), &self.request)
            .await?;
        if let FunctionResult::Success(value) = &value {
            if let Some(message) = &value.error {
                output_tx
                    .send(OutputStream {
                        execution_id: self.request.execution_id,
                        stream: "return".to_owned(),
                        level: "error".to_owned(),
                        group: None,
                        message: message.clone(),
                        timestamp: std::cmp::max(Utc::now().timestamp(), 0) as u64,
                    })
                    .await
                    .map_err(|_| FuncBackendError::SendError)?;
            } else {
                trace!("no message found for ActionRunResultSuccess: skipping!")
            }
        } else {
            return Err(FuncBackendError::FunctionResultActionRun(value));
        }

        Ok(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ActionRunResult {
    pub status: ResourceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub message: Option<String>,
    // Note: we might benefit from adding the metadata here, but it's unused and takes a lot of boilerplate in the root_prop definition
    #[serde(default)]
    pub logs: Vec<String>,
    pub last_synced: Option<String>,
}

impl ExtractPayload for ActionRunResultSuccess {
    type Payload = ActionRunResult;

    fn extract(self) -> FuncBackendResult<Self::Payload> {
        Ok(ActionRunResult {
            payload: self.payload,
            status: self.status,
            message: self.message.or(self.error),
            logs: Default::default(),
            last_synced: Some(Utc::now().to_rfc3339()),
        })
    }
}
