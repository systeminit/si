use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use veritech::{FunctionResult, ResourceSyncRequest, ResourceSyncResultSuccess};

use crate::func::backend::{ExtractPayload, FuncBackendResult, FuncDispatch, FuncDispatchContext};
use crate::ComponentView;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct FuncBackendJsResourceSyncArgs {
    pub component: ComponentView,
}

#[derive(Debug)]
pub struct FuncBackendJsResourceSync {
    context: FuncDispatchContext,
    request: ResourceSyncRequest,
}

#[async_trait]
impl FuncDispatch for FuncBackendJsResourceSync {
    type Args = FuncBackendJsResourceSyncArgs;
    type Output = ResourceSyncResultSuccess;

    fn new(
        context: FuncDispatchContext,
        code_base64: &str,
        handler: &str,
        args: Self::Args,
    ) -> Box<Self> {
        let request = ResourceSyncRequest {
            // Once we start tracking the state of these executions, then this id will be useful,
            // but for now it's passed along and back, and is opaue
            execution_id: "rodrigosantoro".to_string(),
            handler: handler.into(),
            component: args.component.into(),
            code_base64: code_base64.into(),
        };

        Box::new(Self { context, request })
    }

    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>> {
        let (veritech, output_tx) = self.context.into_inner();
        let value = veritech
            .execute_resource_sync(output_tx, &self.request)
            .await?;
        Ok(value)
    }
}

impl ExtractPayload for ResourceSyncResultSuccess {
    type Payload = Self;

    fn extract(self) -> Self::Payload {
        self
    }
}
