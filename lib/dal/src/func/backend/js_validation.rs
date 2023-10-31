use crate::func::backend::{
    ExtractPayload, FuncBackendError, FuncBackendResult, FuncDispatch, FuncDispatchContext,
};
use crate::validation::{ValidationError, ValidationErrorKind};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use veritech_client::{FunctionResult, OutputStream, ValidationRequest, ValidationResultSuccess};

#[derive(Debug, Clone)]
pub struct FuncBackendJsValidation {
    context: FuncDispatchContext,
    request: ValidationRequest,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendJsValidationArgs {
    pub value: Value,
}

impl FuncBackendJsValidationArgs {
    pub fn new(value: Value) -> Self {
        Self { value }
    }
}

#[async_trait]
impl FuncDispatch for FuncBackendJsValidation {
    type Args = FuncBackendJsValidationArgs;
    type Output = ValidationResultSuccess;

    fn new(
        context: FuncDispatchContext,
        code_base64: &str,
        handler: &str,
        args: Self::Args,
    ) -> Box<Self> {
        let request = ValidationRequest {
            execution_id: "johnwick".to_string(),
            handler: handler.into(),
            code_base64: code_base64.to_owned(),
            value: args.value,
        };

        Box::new(Self { context, request })
    }

    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>> {
        let (veritech, output_tx) = self.context.into_inner();
        let value = veritech
            .execute_validation(output_tx.clone(), &self.request)
            .await?;
        let value = match value {
            FunctionResult::Failure(failure) => {
                output_tx
                    .send(OutputStream {
                        execution_id: failure.execution_id.clone(),
                        stream: "return".to_owned(),
                        level: "info".to_owned(),
                        group: None,
                        message: failure.error.message.clone(),
                        timestamp: std::cmp::max(Utc::now().timestamp(), 0) as u64,
                    })
                    .await
                    .map_err(|_| FuncBackendError::SendError)?;

                FunctionResult::Success(Self::Output {
                    execution_id: failure.execution_id,
                    valid: false,
                    message: Some(failure.error.message.clone()),
                })
            }
            FunctionResult::Success(value) => {
                if let Some(message) = &value.message {
                    output_tx
                        .send(OutputStream {
                            execution_id: self.request.execution_id,
                            stream: "return".to_owned(),
                            level: "info".to_owned(),
                            group: None,
                            message: message.clone(),
                            timestamp: std::cmp::max(Utc::now().timestamp(), 0) as u64,
                        })
                        .await
                        .map_err(|_| FuncBackendError::SendError)?;
                }
                FunctionResult::Success(value)
            }
        };

        Ok(value)
    }
}

impl ExtractPayload for ValidationResultSuccess {
    type Payload = Option<Vec<ValidationError>>;

    fn extract(self) -> FuncBackendResult<Self::Payload> {
        if self.valid {
            Ok(None)
        } else {
            Ok(Some(vec![ValidationError {
                kind: ValidationErrorKind::JsValidation,
                message: self.message.unwrap_or_else(|| "unknown error".to_string()),
                level: None,
                link: None,
            }]))
        }
    }
}
