use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use veritech::{CommandRunRequest, CommandRunResultSuccess, FunctionResult, OutputStream};

use crate::func::backend::{
    ExtractPayload, FuncBackendError, FuncBackendResult, FuncDispatch, FuncDispatchContext,
};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct FuncBackendJsCommandArgs(Vec<serde_json::Value>);

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
        _args: Self::Args,
    ) -> Box<Self> {
        let request = CommandRunRequest {
            // Once we start tracking the state of these executions, then this id will be useful,
            // but for now it's passed along and back, and is opaue
            execution_id: "danielfurlan".to_string(),
            handler: handler.into(),
            code_base64: code_base64.into(),
        };

        Box::new(Self { context, request })
    }

    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>> {
        let (veritech, output_tx) = self.context.into_inner();
        let value = veritech
            .execute_command_run(output_tx.clone(), &self.request)
            .await?;
        match &value {
            FunctionResult::Failure(_) => {}
            FunctionResult::Success(value) => {
                if let Some(message) = &value.message {
                    output_tx
                        .send(OutputStream {
                            execution_id: self.request.execution_id,
                            stream: "return".to_owned(),
                            level: "info".to_owned(),
                            group: None,
                            data: None,
                            message: message.clone(),
                            timestamp: std::cmp::max(Utc::now().timestamp(), 0) as u64,
                        })
                        .await
                        .map_err(|_| FuncBackendError::SendError)?;
                } else {
                }
            }
        }
        Ok(value)
    }
}

impl ExtractPayload for CommandRunResultSuccess {
    type Payload = ();

    fn extract(self) -> Self::Payload {}
}
