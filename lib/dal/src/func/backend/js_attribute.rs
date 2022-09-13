use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use veritech::{FunctionResult, ResolverFunctionRequest, ResolverFunctionResultSuccess};

use crate::func::backend::{
    ExtractPayload, FuncBackendError, FuncBackendResult, FuncDispatch, FuncDispatchContext,
};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct FuncBackendJsAttributeArgs {
    pub component: veritech::ResolverFunctionComponent,
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
            code_base64: code_base64.into(),
        };

        Box::new(Self { context, request })
    }

    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>> {
        let (veritech, output_tx) = self.context.into_inner();
        let value = veritech
            .execute_resolver_function(output_tx, &self.request)
            .await?;
        match &value {
            FunctionResult::Success(success) if success.unset => {
                return Err(FuncBackendError::UnexpectedUnset);
            }
            _ => {}
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
