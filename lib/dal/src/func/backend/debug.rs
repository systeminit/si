use async_trait::async_trait;
use serde::{
    Deserialize,
    Serialize,
};
use veritech_client::{
    BeforeFunction,
    ComponentView,
    DebugRequest,
    DebugResultSuccess,
    FunctionResult,
};

use super::ExtractPayload;
use crate::func::backend::{
    FuncBackendResult,
    FuncDispatch,
    FuncDispatchContext,
};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct FuncBackendDebugArgs {
    pub component: ComponentView,
    pub debug_input: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct FuncBackendDebug {
    request: DebugRequest,
    context: FuncDispatchContext,
}

#[async_trait]
impl FuncDispatch for FuncBackendDebug {
    type Args = FuncBackendDebugArgs;
    type Output = DebugResultSuccess;

    fn new(
        context: FuncDispatchContext,
        code_base64: &str,
        handler: &str,
        args: Self::Args,
        before: Vec<BeforeFunction>,
    ) -> Box<Self> {
        let request = DebugRequest {
            execution_id: context.func_run_id.to_string(),
            handler: handler.into(),
            code_base64: code_base64.into(),
            component: args.component,
            debug_input: args.debug_input,
            before,
        };

        Box::new(Self { context, request })
    }

    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>> {
        let (veritech, output_tx, workspace_id, change_set_id) = self.context.into_inner();
        Ok(veritech
            .execute_debug(
                output_tx,
                &self.request,
                &workspace_id.to_string(),
                &change_set_id.to_string(),
            )
            .await?)
    }
}

impl ExtractPayload for DebugResultSuccess {
    type Payload = DebugResultSuccess;

    fn extract(self) -> FuncBackendResult<Self::Payload> {
        Ok(self)
    }
}
