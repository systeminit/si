use axum::Json;
use dal::context::HandlerContext;
use dal::system::UNSET_ID_VALUE;
use dal::{Component, ComponentId, StandardModel, SystemId, Visibility};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{HistoryActor, ServicesContext, Tenancy};

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
    ServicesContext(services_context): ServicesContext,
    HistoryActor(history_actor): HistoryActor,
    Tenancy((write_tenancy, read_tenancy)): Tenancy,
    Json(request): Json<SyncResourceRequest>,
) -> ComponentResult<Json<SyncResourceResponse>> {
    // Building owned pg_conn and setup ctx builder--this could likely become a macro call?
    let (builder, mut pg_conn) = services_context.builder_and_pg_conn().await?;
    let (pg_txn, nats_txn) = services_context.transactions(&mut pg_conn).await?;

    // Build the final DalContext used by the rest of the handler function
    let ctx = builder.build(
        // TODO: read_tenancy, write_tenancy and history actor should be set by default to whatever the extractor gets
        HandlerContext {
            read_tenancy,
            write_tenancy,
            history_actor,
            visibility: request.visibility,
        },
        &pg_txn,
        &nats_txn,
    );

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

    pg_txn.commit().await?;
    nats_txn.commit().await?;
    Ok(Json(SyncResourceResponse { success: true }))
}
