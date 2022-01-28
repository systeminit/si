use axum::extract::Query;
use axum::Json;
use dal::system::UNSET_ID_VALUE;
use dal::{
    Component, ComponentId, HistoryActor, ResourceView, StandardModel, SystemId, Tenancy,
    Visibility, Workspace, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{Authorization, NatsTxn, PgRwTxn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetResourceRequest {
    pub component_id: ComponentId,
    pub system_id: Option<SystemId>,
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
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    Query(request): Query<GetResourceRequest>,
    Authorization(claim): Authorization,
) -> ComponentResult<Json<GetResourceResponse>> {
    let txn = txn.start().await?;
    let nats = nats.start().await?;

    let billing_account_tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    // Note(paulo): We only need to write here because currently we don't have a propper place to create a Resource for a Component + System
    let history_actor = HistoryActor::User(claim.user_id);
    let workspace = Workspace::get_by_id(
        &txn,
        &billing_account_tenancy,
        &request.visibility,
        &request.workspace_id,
    )
    .await?
    .ok_or(ComponentError::InvalidRequest)?;
    let tenancy = Tenancy::new_workspace(vec![*workspace.id()]);

    let resource = Component::get_resource_by_component_and_system(
        &txn,
        &nats,
        &tenancy,
        &request.visibility,
        &history_actor,
        request.component_id,
        request.system_id.unwrap_or_else(|| UNSET_ID_VALUE.into()),
    )
    .await?;

    txn.commit().await?;
    nats.commit().await?;
    Ok(Json(GetResourceResponse { resource }))
}
