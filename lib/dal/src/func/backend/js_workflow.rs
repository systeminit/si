use async_trait::async_trait;
use chrono::Utc;
use veritech::{
    FunctionResult, OutputStream, WorkflowResolveRequest, WorkflowResolveResultSuccess,
};

use crate::func::backend::{
    ExtractPayload, FuncBackendError, FuncBackendResult, FuncDispatch, FuncDispatchContext,
};
use crate::WorkflowView;

pub type FuncBackendJsWorkflowArgs = serde_json::Value;

#[derive(Debug)]
pub struct FuncBackendJsWorkflow {
    pub context: FuncDispatchContext,
    pub request: WorkflowResolveRequest,
}

#[async_trait]
impl FuncDispatch for FuncBackendJsWorkflow {
    type Args = FuncBackendJsWorkflowArgs;
    type Output = WorkflowResolveResultSuccess;

    fn new(
        context: FuncDispatchContext,
        code_base64: &str,
        handler: &str,
        args: Self::Args,
    ) -> Box<Self> {
        let request = WorkflowResolveRequest {
            // Once we start tracking the state of these executions, then this id will be useful,
            // but for now it's passed along and back, and is opaue
            execution_id: "danielfurlanjsworkflow".to_string(),
            handler: handler.into(),
            code_base64: code_base64.into(),
            args,
        };

        Box::new(Self { context, request })
    }

    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>> {
        let (veritech, output_tx) = self.context.into_inner();
        let value = veritech
            .execute_workflow_resolve(output_tx.clone(), &self.request)
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

impl ExtractPayload for WorkflowResolveResultSuccess {
    type Payload = WorkflowView;

    fn extract(self) -> FuncBackendResult<Self::Payload> {
        Ok(WorkflowView::new(
            self.name,
            serde_json::from_value(serde_json::Value::String(self.kind))?,
            serde_json::from_value(self.steps)?,
            serde_json::from_value(self.args)?,
        ))
    }
}
