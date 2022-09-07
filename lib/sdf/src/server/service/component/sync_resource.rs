use axum::Json;
use dal::{Component, ComponentId, StandardModel, SystemId, Visibility};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyncResourceRequest {
    pub component_id: ComponentId,
    pub system_id: Option<SystemId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyncResourceResponse {
    pub success: bool,
}

pub async fn sync_resource(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SyncResourceRequest>,
) -> ComponentResult<Json<SyncResourceResponse>> {
    let system_id = request.system_id.unwrap_or_else(|| SystemId::from(-1));
    if system_id.is_none() {
        return Err(ComponentError::SystemIdRequired);
    }

    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let component = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(ComponentError::ComponentNotFound)?;
    component.sync_resource(&ctx, system_id).await?;

    ctx.commit().await?;

    Ok(Json(SyncResourceResponse { success: true }))
}
