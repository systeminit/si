use crate::func::backend::{ExtractPayload, FuncBackendResult, FuncDispatch, FuncDispatchContext};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use veritech_client::{BeforeFunction, FunctionResult, ValidationRequest, ValidationResultSuccess};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct FuncBackendJsAttributeArgs {
    pub value: Option<serde_json::Value>,
    pub validation_format: String,
}

#[derive(Debug, Clone)]
pub struct FuncBackendValidation {
    context: FuncDispatchContext,
    request: ValidationRequest,
}

#[async_trait]
impl FuncDispatch for FuncBackendValidation {
    type Args = FuncBackendJsAttributeArgs;
    type Output = ValidationResultSuccess;

    fn new(
        context: FuncDispatchContext,
        _code_base64: &str,
        _handler: &str,
        args: Self::Args,
        _before: Vec<BeforeFunction>,
    ) -> Box<Self> {
        let request = ValidationRequest {
            execution_id: "guarabyra".to_string(),
            value: args.value,
            validation_format: args.validation_format,
            handler: "".to_string(),
            code_base64: "".to_string(),
            before: vec![],
        };

        Box::new(Self { context, request })
    }

    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>> {
        let (veritech, output_tx) = self.context.into_inner();
        let value = veritech
            .execute_validation(output_tx.clone(), &self.request)
            .await?;
        Ok(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ValidationRunResult {
    pub error: Option<String>,
}

impl ExtractPayload for ValidationResultSuccess {
    type Payload = ValidationRunResult;

    fn extract(self) -> FuncBackendResult<Self::Payload> {
        Ok(ValidationRunResult { error: self.error })
    }
}
