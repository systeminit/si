use async_trait::async_trait;
use chrono::Utc;
use serde::{
    Deserialize,
    Serialize,
};
use telemetry::tracing::trace;
use veritech_client::{
    ActionRunRequest,
    ActionRunResultSuccess,
    BeforeFunction,
    FunctionResult,
    OutputStream,
    ResourceStatus,
};

use crate::func::backend::{
    ExtractPayload,
    FuncBackendError,
    FuncBackendResult,
    FuncDispatch,
    FuncDispatchContext,
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
        before: Vec<BeforeFunction>,
    ) -> Box<Self> {
        let request = ActionRunRequest {
            execution_id: context.func_run_id.to_string(), // RIP PAULO - GONE (from si) BUT NOT FORGOTTEN
            handler: handler.into(),
            code_base64: code_base64.into(),
            args: args.0,
            before,
        };

        Box::new(Self { context, request })
    }

    /// This private function dispatches the assembled request to veritech for execution.
    /// This is the "last hop" function in the dal before using the veritech client directly.
    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>> {
        let (veritech, output_tx, workspace_id, change_set_id) = self.context.into_inner();
        let value = veritech
            .execute_action_run(
                output_tx.clone(),
                &self.request,
                &workspace_id.to_string(),
                &change_set_id.to_string(),
            )
            .await?;
        let value = match value {
            FunctionResult::Success(value) => {
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

                FunctionResult::Success(value)
            }
            FunctionResult::Failure(failure) => {
                output_tx
                    .send(OutputStream {
                        execution_id: failure.execution_id().to_owned(),
                        stream: "return".to_owned(),
                        level: "error".to_owned(),
                        group: None,
                        message: failure.error().message.to_owned(),
                        timestamp: std::cmp::max(Utc::now().timestamp(), 0) as u64,
                    })
                    .await
                    .map_err(|_| FuncBackendError::SendError)?;

                FunctionResult::Success(Self::Output {
                    execution_id: failure.execution_id().to_owned(),
                    resource_id: None,
                    payload: self
                        .request
                        .args
                        .pointer("/properties/resource/payload")
                        .cloned(),
                    status: ResourceStatus::Error,
                    message: Some(failure.error().message.clone()),
                    error: Some(serde_json::to_string(&failure.error())?),
                })
            }
        };

        Ok(value)
    }
}

impl ExtractPayload for ActionRunResultSuccess {
    type Payload = ActionRunResultSuccess;

    fn extract(self) -> FuncBackendResult<Self::Payload> {
        Ok(self)
    }
}
