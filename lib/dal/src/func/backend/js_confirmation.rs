use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use veritech_client::{
    ComponentView, ConfirmationRequest, ConfirmationResultSuccess, FunctionResult, OutputStream,
};

use crate::func::backend::{
    ExtractPayload, FuncBackendError, FuncBackendResult, FuncDispatch, FuncDispatchContext,
};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct FuncBackendJsConfirmationArgs {
    pub component: ComponentView,
}

#[derive(Debug)]
pub struct FuncBackendJsConfirmation {
    context: FuncDispatchContext,
    request: ConfirmationRequest,
}

#[async_trait]
impl FuncDispatch for FuncBackendJsConfirmation {
    type Args = FuncBackendJsConfirmationArgs;
    type Output = ConfirmationResultSuccess;

    fn new(
        context: FuncDispatchContext,
        code_base64: &str,
        handler: &str,
        args: Self::Args,
    ) -> Box<Self> {
        let request = ConfirmationRequest {
            // Once we start tracking the state of these executions, then this id will be useful,
            // but for now it's passed along and back, and is opaue
            execution_id: "seltonmello".to_string(),
            handler: handler.into(),
            code_base64: code_base64.to_owned(),
            component: args.component,
        };

        Box::new(Self { context, request })
    }

    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>> {
        let (veritech, output_tx) = self.context.into_inner();
        let value = veritech
            .execute_confirmation(output_tx.clone(), &self.request)
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfirmationResult {
    pub success: bool,
    pub recommended_actions: Vec<String>,
    pub message: Option<String>,
}

impl ExtractPayload for ConfirmationResultSuccess {
    type Payload = ConfirmationResult;

    fn extract(self) -> FuncBackendResult<Self::Payload> {
        Ok(ConfirmationResult {
            success: self.success,
            recommended_actions: self.recommended_actions,
            message: self.message,
        })
    }
}
