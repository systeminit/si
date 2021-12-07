use super::{ApplicationError, ApplicationResult};
use crate::server::extract::{Authorization, NatsTxn, PgRwTxn};
use axum::Json;
use dal::node::NodeKind;
use dal::{
    Component, HistoryActor, Node, StandardModel, Tenancy, Visibility,
    Workspace, WorkspaceId,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateApplicationRequest {
    pub name: String,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateApplicationResponse {
    pub application: Component,
}

pub async fn create_application(
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    Authorization(claim): Authorization,
    Json(request): Json<CreateApplicationRequest>,
) -> ApplicationResult<Json<CreateApplicationResponse>> {
    let txn = txn.start().await?;
    let nats = nats.start().await?;

    // Create the workspace tenancy
    let billing_account_tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    let history_actor: HistoryActor = HistoryActor::from(claim.user_id);
    let workspace = Workspace::get_by_id(
        &txn,
        &billing_account_tenancy,
        &request.visibility,
        &request.workspace_id,
    )
    .await?
    .ok_or(ApplicationError::InvalidRequest)?;

    let tenancy = Tenancy::new_workspace(vec![*workspace.id()]);
    // You can only create applications directly to head? This feels wrong, but..
    let visibility = Visibility::new_head(false);
    let component = Component::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &request.name,
    )
    .await?;
    let node = Node::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &NodeKind::Component,
    )
    .await?;
    node.set_component(&txn, &nats, &visibility, &history_actor, component.id())
        .await?;

    txn.commit().await?;
    nats.commit().await?;
    Ok(Json(CreateApplicationResponse {
        application: component,
    }))
}
