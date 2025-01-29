use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use veritech_client::{
    BeforeFunction, ComponentViewWithGeometry, FunctionResult, ManagementRequest,
    ManagementResultSuccess, ThisComponent,
};

use crate::func::backend::{FuncBackendResult, FuncDispatch, FuncDispatchContext};

use super::ExtractPayload;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct FuncBackendManagementArgs {
    this_component: ThisComponent,
    components: HashMap<String, ComponentViewWithGeometry>,
    current_view: String,
    variant_socket_map: HashMap<String, usize>,
}

#[derive(Debug)]
pub struct FuncBackendManagement {
    request: ManagementRequest,
    context: FuncDispatchContext,
}

#[async_trait]
impl FuncDispatch for FuncBackendManagement {
    type Args = FuncBackendManagementArgs;
    type Output = ManagementResultSuccess;

    fn new(
        context: FuncDispatchContext,
        code_base64: &str,
        handler: &str,
        args: Self::Args,
        before: Vec<BeforeFunction>,
    ) -> Box<Self> {
        let request = ManagementRequest {
            execution_id: context.func_run_id.to_string(),
            handler: handler.into(),
            code_base64: code_base64.into(),
            this_component: args.this_component,
            components: args.components,
            current_view: args.current_view,
            variant_socket_map: args.variant_socket_map,
            before,
        };

        Box::new(Self { context, request })
    }

    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>> {
        let (veritech, output_tx, workspace_id, change_set_id) = self.context.into_inner();
        Ok(veritech
            .execute_management(
                output_tx,
                &self.request,
                &workspace_id.to_string(),
                &change_set_id.to_string(),
            )
            .await?)
    }
}

impl ExtractPayload for ManagementResultSuccess {
    type Payload = ManagementResultSuccess;

    fn extract(self) -> FuncBackendResult<Self::Payload> {
        Ok(self)
    }
}
