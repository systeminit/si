use axum::Json;
use dal::{system::UNSET_ID_VALUE, Component, ComponentId, StandardModel, SystemId, Visibility};
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
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SyncResourceRequest>,
) -> ComponentResult<Json<SyncResourceResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let component = Component::get_by_id(
        ctx.pg_txn(),
        &ctx.read_tenancy().into(),
        ctx.visibility(),
        &request.component_id,
    )
    .await?
    .ok_or(ComponentError::ComponentNotFound)?;
    component
        .sync_resource(
            ctx.pg_txn(),
            ctx.nats_txn(),
            ctx.veritech().clone(),
            ctx.encryption_key(),
            ctx.history_actor(),
            request.system_id.unwrap_or_else(|| UNSET_ID_VALUE.into()),
        )
        .await?;

    txns.commit().await?;
    Ok(Json(SyncResourceResponse { success: true }))
}
