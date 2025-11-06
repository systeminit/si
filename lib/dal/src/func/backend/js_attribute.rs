use async_trait::async_trait;
use chrono::Utc;
use serde::{
    Deserialize,
    Serialize,
};
use veritech_client::{
    BeforeFunction,
    FunctionResult,
    ResolverFunctionComponent,
    ResolverFunctionRequest,
    ResolverFunctionResponseType,
    ResolverFunctionResultSuccess,
};

use crate::func::backend::{
    ExtractPayload,
    FuncBackendResult,
    FuncDispatch,
    FuncDispatchContext,
};

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
        before: Vec<BeforeFunction>,
    ) -> Box<Self> {
        let request = ResolverFunctionRequest {
            execution_id: context.func_run_id.to_string(),
            handler: handler.into(),
            component: args.component,
            response_type: args.response_type,
            code_base64: code_base64.into(),
            before,
        };

        Box::new(Self { context, request })
    }

    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>> {
        let (veritech, output_tx, workspace_id, change_set_id) = self.context.into_inner();
        let value = veritech
            .execute_resolver_function(
                output_tx,
                &self.request,
                &workspace_id.to_string(),
                &change_set_id.to_string(),
            )
            .await?;
        let value = match value {
            FunctionResult::Failure(failure) => match &self.request.response_type {
                ResolverFunctionResponseType::Action
                | ResolverFunctionResponseType::Array
                | ResolverFunctionResponseType::Boolean
                | ResolverFunctionResponseType::Integer
                | ResolverFunctionResponseType::Float
                | ResolverFunctionResponseType::Identity
                | ResolverFunctionResponseType::Map
                | ResolverFunctionResponseType::Object
                | ResolverFunctionResponseType::String
                | ResolverFunctionResponseType::Unset
                | ResolverFunctionResponseType::Void
                | ResolverFunctionResponseType::Management
                | ResolverFunctionResponseType::Debug
                | ResolverFunctionResponseType::Json => FunctionResult::Failure(failure),
                ResolverFunctionResponseType::Qualification => {
                    FunctionResult::Success(Self::Output {
                        execution_id: failure.execution_id().to_owned(),
                        data: serde_json::json!({
                            "result": "failure",
                            "message": format!("Function execution failed: {}", failure.error().message),
                        }),
                        unset: false,
                        timestamp: u64::try_from(std::cmp::max(Utc::now().timestamp(), 0))
                            .expect("timestamp not be negative"),
                    })
                }
                ResolverFunctionResponseType::CodeGeneration => {
                    FunctionResult::Success(Self::Output {
                        execution_id: failure.execution_id().to_owned(),
                        data: serde_json::json!({
                            "format": "json",
                            "code": "null",
                            "message": format!("Function execution failed: {}", failure.error().message),
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
