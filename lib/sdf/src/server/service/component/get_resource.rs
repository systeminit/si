use axum::extract::Query;
use axum::Json;
use dal::{Component, ComponentId, ResourceView, SystemId, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetResourceRequest {
    pub component_id: ComponentId,
    pub system_id: SystemId,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetResourceResponse {
    pub resource: ResourceView,
}

pub async fn get_resource(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetResourceRequest>,
) -> ComponentResult<Json<GetResourceResponse>> {
    if request.system_id.is_none() {
        return Err(ComponentError::SystemIdRequired);
    }

    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let resource = Component::get_resource_by_component_and_system(
        &ctx,
        request.component_id,
        request.system_id,
    )
    .await?
    .ok_or(ComponentError::ResourceNotFound(
        request.component_id,
        request.system_id,
    ))?;

    txns.commit().await?;
    Ok(Json(GetResourceResponse { resource }))
}
