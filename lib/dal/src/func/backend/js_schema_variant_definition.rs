use crate::func::backend::{ExtractPayload, FuncBackendResult, FuncDispatch, FuncDispatchContext};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use veritech_client::{
    BeforeFunctionRequest, FunctionResult, SchemaVariantDefinitionRequest,
    SchemaVariantDefinitionResultSuccess,
};
#[derive(Debug, Clone)]
pub struct FuncBackendJsSchemaVariantDefinition {
    context: FuncDispatchContext,
    request: SchemaVariantDefinitionRequest,
}

#[async_trait]
impl FuncDispatch for FuncBackendJsSchemaVariantDefinition {
    type Args = ();
    type Output = SchemaVariantDefinitionResultSuccess;

    fn new(
        context: FuncDispatchContext,
        code_base64: &str,
        handler: &str,
        _args: Self::Args,
        _before: Vec<BeforeFunctionRequest>,
    ) -> Box<Self> {
        let request = SchemaVariantDefinitionRequest {
            execution_id: "villanelle".to_string(),
            handler: handler.into(),
            code_base64: code_base64.to_owned(),
        };

        Box::new(Self { context, request })
    }

    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>> {
        let (veritech, output_tx) = self.context.into_inner();
        let value = veritech
            .execute_schema_variant_definition(output_tx.clone(), &self.request)
            .await?;
        let value = match value {
            FunctionResult::Failure(failure) => FunctionResult::Success(Self::Output {
                execution_id: failure.execution_id,
                definition: serde_json::Value::Null,
                error: Some(failure.error.message.clone()),
            }),
            FunctionResult::Success(value) => FunctionResult::Success(value),
        };

        Ok(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SchemaVariantDefinitionRunResult {
    pub definition: serde_json::Value,
    pub error: Option<String>,
}

impl ExtractPayload for SchemaVariantDefinitionResultSuccess {
    type Payload = SchemaVariantDefinitionRunResult;

    fn extract(self) -> FuncBackendResult<Self::Payload> {
        Ok(SchemaVariantDefinitionRunResult {
            definition: self.definition,
            error: self.error,
        })
    }
}
