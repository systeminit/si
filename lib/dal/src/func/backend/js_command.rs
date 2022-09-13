use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use telemetry::tracing::trace;
use veritech::{CommandRunRequest, CommandRunResultSuccess, FunctionResult, OutputStream};

use crate::func::backend::{
    ExtractPayload, FuncBackendError, FuncBackendResult, FuncDispatch, FuncDispatchContext,
};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct FuncBackendJsCommandArgs(serde_json::Value);

#[derive(Debug)]
pub struct FuncBackendJsCommand {
    pub context: FuncDispatchContext,
    pub request: CommandRunRequest,
}

#[async_trait]
impl FuncDispatch for FuncBackendJsCommand {
    type Args = FuncBackendJsCommandArgs;
    type Output = CommandRunResultSuccess;

    fn new(
        context: FuncDispatchContext,
        code_base64: &str,
        handler: &str,
        args: Self::Args,
    ) -> Box<Self> {
        let request = CommandRunRequest {
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
            .execute_command_run(output_tx.clone(), &self.request)
            .await?;
        if let FunctionResult::Success(value) = &value {
            if let Some(message) = &value.error {
                output_tx
                    .send(OutputStream {
                        execution_id: self.request.execution_id,
                        stream: "return".to_owned(),
                        level: "error".to_owned(),
                        group: None,
                        data: None,
                        message: message.clone(),
                        timestamp: std::cmp::max(Utc::now().timestamp(), 0) as u64,
                    })
                    .await
                    .map_err(|_| FuncBackendError::SendError)?;
            } else {
                trace!("no message found for CommandRunResultSuccess: skipping!")
            }
        } else {
            return Err(FuncBackendError::FunctionResultCommandRun(value));
        }

        Ok(value)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandRunResult {
    pub created: HashMap<String, serde_json::Value>,
    pub updated: HashMap<String, serde_json::Value>,
    pub error: Option<String>,
}

impl ExtractPayload for CommandRunResultSuccess {
    type Payload = CommandRunResult;

    fn extract(self) -> FuncBackendResult<Self::Payload> {
        Ok(CommandRunResult {
            updated: self.updated,
            created: self.created,
            error: self.error,
        })
    }
}
