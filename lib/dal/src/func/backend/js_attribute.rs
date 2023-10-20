use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use veritech_client::{
    FunctionResult, ResolverFunctionComponent, ResolverFunctionRequest,
    ResolverFunctionResponseType, ResolverFunctionResultSuccess,
};

use crate::func::backend::{ExtractPayload, FuncBackendResult, FuncDispatch, FuncDispatchContext};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct FuncBackendJsAttributeArgs {
    pub component: ResolverFunctionComponent,
    pub response_type: ResolverFunctionResponseType,
}

#[derive(Debug)]
pub struct FuncBackendJsAttribute {
    context: FuncDispatchContext,
    request: ResolverFunctionRequest,
}

#[async_trait]
impl FuncDispatch for FuncBackendJsAttribute {
    type Args = FuncBackendJsAttributeArgs;
    type Output = ResolverFunctionResultSuccess;

    fn new(
        context: FuncDispatchContext,
        code_base64: &str,
        handler: &str,
        args: Self::Args,
    ) -> Box<Self> {
        let request = ResolverFunctionRequest {
            // Once we start tracking the state of these executions, then this id will be useful,
            // but for now it's passed along and back, and is opaue
            execution_id: "tomcruise".to_string(),
            handler: handler.into(),
            component: args.component,
            response_type: args.response_type,
            code_base64: code_base64.into(),
        };

        Box::new(Self { context, request })
    }

    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>> {
        let (veritech, output_tx) = self.context.into_inner();
        let value = veritech
            .execute_resolver_function(output_tx, &self.request)
            .await?;
        let value = match value {
            FunctionResult::Failure(failure) => match &self.request.response_type {
                ResolverFunctionResponseType::Action => FunctionResult::Failure(failure),
                ResolverFunctionResponseType::Array => FunctionResult::Failure(failure),
                ResolverFunctionResponseType::Boolean => FunctionResult::Failure(failure),
                ResolverFunctionResponseType::Integer => FunctionResult::Failure(failure),
                ResolverFunctionResponseType::Identity => FunctionResult::Failure(failure),
                ResolverFunctionResponseType::Map => FunctionResult::Failure(failure),
                ResolverFunctionResponseType::Object => FunctionResult::Failure(failure),
                ResolverFunctionResponseType::String => FunctionResult::Failure(failure),
                ResolverFunctionResponseType::Unset => FunctionResult::Failure(failure),
                ResolverFunctionResponseType::Json => FunctionResult::Failure(failure),
                ResolverFunctionResponseType::Qualification => {
                    FunctionResult::Success(Self::Output {
                        execution_id: failure.execution_id,
                        data: serde_json::json!({
                            "result": "failure",
                            "message": format!("Function execution failed: {}", failure.error.message),
                        }),
                        unset: false,
                        timestamp: u64::try_from(std::cmp::max(Utc::now().timestamp(), 0))
                            .expect("timestamp not be negative"),
                    })
                }
                ResolverFunctionResponseType::CodeGeneration => {
                    FunctionResult::Success(Self::Output {
                        execution_id: failure.execution_id,
                        data: serde_json::json!({
                            "format": "json",
                            "code": "null",
                            "message": format!("Function execution failed: {}", failure.error.message),
                        }),
                        unset: false,
                        timestamp: u64::try_from(std::cmp::max(Utc::now().timestamp(), 0))
                            .expect("timestamp not be negative"),
                    })
                }
            },
            FunctionResult::Success(value) => FunctionResult::Success(value),
        };
        Ok(value)
    }
}

impl ExtractPayload for ResolverFunctionResultSuccess {
    type Payload = serde_json::Value;

    fn extract(self) -> FuncBackendResult<Self::Payload> {
        Ok(self.data)
    }
}
