use axum::Json;
use serde::{Deserialize, Serialize};

use super::{WorkflowError, WorkflowResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use dal::{
    workflow::WorkflowTreeView, FuncBinding, FuncBindingReturnValue, StandardModel, Visibility,
    WorkflowPrototype, WorkflowPrototypeId, WorkflowTree,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowResolveRequest {
    pub id: WorkflowPrototypeId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowResolveResponse {
    json: String,
}

pub async fn resolve(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<WorkflowResolveRequest>,
) -> WorkflowResult<Json<WorkflowResolveResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let resolver = WorkflowPrototype::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(WorkflowError::PrototypeNotFound(request.id))?
        .resolve(&ctx)
        .await?;
    let func_binding = FuncBinding::get_by_id(&ctx, &resolver.func_binding_id())
        .await?
        .ok_or_else(|| WorkflowError::FuncBindingNotFound(resolver.func_binding_id()))?;
    let value = FuncBindingReturnValue::get_by_func_binding_id(&ctx, *func_binding.id()).await?;
    let value = value.as_ref().and_then(|v| v.value());
    let tree = WorkflowTree::deserialize(value.unwrap_or(&serde_json::Value::Null))?;
    let view = WorkflowTreeView::new(&ctx, tree).await?;

    txns.commit().await?;

    Ok(Json(WorkflowResolveResponse {
        json: serde_json::to_string_pretty(&view)?,
    }))
}
