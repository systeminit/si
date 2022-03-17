use axum::Json;
use dal::context::{HandlerContext, ServicesContext};
use dal::system::UNSET_ID_VALUE;
use dal::{
    Component, ComponentId, HistoryActor, StandardModel, SystemId, Tenancy, Visibility, Workspace,
    WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{Authorization, EncryptionKey, Nats, PgPool, Veritech};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyncResourceRequest {
    pub component_id: ComponentId,
    pub system_id: Option<SystemId>,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyncResourceResponse {
    pub success: bool,
}

pub async fn sync_resource(
    PgPool(pg_pool): PgPool,
    Nats(nats_conn): Nats,
    Veritech(veritech): Veritech,
    EncryptionKey(encryption_key): EncryptionKey,
    Authorization(claim): Authorization,
    Json(request): Json<SyncResourceRequest>,
) -> ComponentResult<Json<SyncResourceResponse>> {
    // This gets extracted as a function parameter
    let services_context = ServicesContext::new(pg_pool, nats_conn, veritech, encryption_key);

    // Building owned pg_conn and setup ctx builder--this could likely become a macro call?
    let (builder, mut pg_conn) = services_context.builder_and_pg_conn().await?;
    let (pg_txn, nats_txn) = services_context.transactions(&mut pg_conn).await?;

    // Determine our tenancies
    let billing_account_tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    let history_actor: HistoryActor = HistoryActor::from(claim.user_id);
    let workspace = Workspace::get_by_id(
        &pg_txn,
        &billing_account_tenancy,
        &request.visibility,
        &request.workspace_id,
    )
    .await?
    .ok_or(ComponentError::InvalidRequest)?;
    let tenancy = Tenancy::new_workspace(vec![*workspace.id()]);

    // Build the final DalContext used by the rest of the handler function
    let ctx = builder.build(
        HandlerContext {
            read_tenancy: tenancy.clone(),
            write_tenancy: tenancy.clone(),
            visibility: request.visibility,
            history_actor,
        },
        &pg_txn,
        &nats_txn,
    );

    let component = Component::get_by_id(
        ctx.pg_txn(),
        ctx.read_tenancy(),
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
