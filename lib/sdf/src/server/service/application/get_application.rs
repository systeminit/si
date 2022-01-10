use axum::{extract::Query, Json};
use dal::node::NodeId;
use dal::{Component, ComponentId, StandardModel, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::ApplicationResult;
use crate::server::extract::{Authorization, PgRoTxn, QueryWorkspaceTenancy};
use crate::service::application::ApplicationError;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetApplicationRequest {
    pub application_id: ComponentId,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetApplicationResponse {
    pub application: Component,
    pub application_node_id: NodeId,
}

//pub type GetApplicationResponse = Option<Component>;

pub async fn get_application(
    mut txn: PgRoTxn,
    Authorization(_claim): Authorization,
    Query(request): Query<GetApplicationRequest>,
    QueryWorkspaceTenancy(tenancy): QueryWorkspaceTenancy,
) -> ApplicationResult<Json<GetApplicationResponse>> {
    let txn = txn.start().await?;
    let application =
        Component::get_by_id(&txn, &tenancy, &request.visibility, &request.application_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
    let application_node = application
        .node(&txn, &tenancy, &request.visibility)
        .await?
        .pop()
        .ok_or(ApplicationError::NotFound)?;
    Ok(Json(GetApplicationResponse {
        application,
        application_node_id: *application_node.id(),
    }))
}
