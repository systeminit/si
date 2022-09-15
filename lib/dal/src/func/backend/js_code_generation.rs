use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use veritech::{CodeGenerated, CodeGenerationRequest, CodeGenerationResultSuccess, FunctionResult};

use crate::func::backend::{ExtractPayload, FuncBackendResult, FuncDispatch, FuncDispatchContext};
use crate::ComponentView;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct FuncBackendJsCodeGenerationArgs {
    pub component: ComponentView,
}

#[derive(Debug)]
pub struct FuncBackendJsCodeGeneration {
    context: FuncDispatchContext,
    request: CodeGenerationRequest,
}

#[async_trait]
impl FuncDispatch for FuncBackendJsCodeGeneration {
    type Args = FuncBackendJsCodeGenerationArgs;
    type Output = CodeGenerationResultSuccess;

    fn new(
        context: FuncDispatchContext,
        code_base64: &str,
        handler: &str,
        args: Self::Args,
    ) -> Box<Self> {
        let request = CodeGenerationRequest {
            // Once we start tracking the state of these executions, then this id will be useful,
            // but for now it's passed along and back, and is opaue
            execution_id: "wagnermoura".to_string(),
            handler: handler.into(),
            code_base64: code_base64.into(),
            component: args.component.into(),
        };

        Box::new(Self { context, request })
    }

    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>> {
        let (veritech, output_tx) = self.context.into_inner();
        let value = veritech
            .execute_code_generation(output_tx, &self.request)
            .await?;
        Ok(value)
    }
}

impl ExtractPayload for CodeGenerationResultSuccess {
    type Payload = CodeGenerated;

    fn extract(self) -> FuncBackendResult<Self::Payload> {
        Ok(self.data)
    }
}
