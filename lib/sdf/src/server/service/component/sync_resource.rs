use axum::Json;
use dal::system::UNSET_ID_VALUE;
use dal::{
    Component, ComponentId, HistoryActor, StandardModel, SystemId, Tenancy, Visibility, Workspace,
    WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{Authorization, NatsTxn, PgRwTxn, Veritech};

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
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    Veritech(veritech): Veritech,
    Authorization(claim): Authorization,
    Json(request): Json<SyncResourceRequest>,
) -> ComponentResult<Json<SyncResourceResponse>> {
    let txn = txn.start().await?;
    let nats = nats.start().await?;

    let billing_account_tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    let history_actor: HistoryActor = HistoryActor::from(claim.user_id);
    let workspace = Workspace::get_by_id(
        &txn,
        &billing_account_tenancy,
        &request.visibility,
        &request.workspace_id,
    )
    .await?
    .ok_or(ComponentError::InvalidRequest)?;
    let tenancy = Tenancy::new_workspace(vec![*workspace.id()]);

    let component =
        Component::get_by_id(&txn, &tenancy, &request.visibility, &request.component_id)
            .await?
            .ok_or(ComponentError::ComponentNotFound)?;
    component
        .sync_resource(
            &txn,
            &nats,
            veritech,
            &history_actor,
            request.system_id.unwrap_or_else(|| UNSET_ID_VALUE.into()),
        )
        .await?;

    txn.commit().await?;
    nats.commit().await?;
    Ok(Json(SyncResourceResponse { success: true }))
}
