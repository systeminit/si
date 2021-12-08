use axum::{extract::Query, Json};
use dal::{StandardModel, Visibility, WorkspaceId, Component, ComponentId};
use serde::{Deserialize, Serialize};

use super::ApplicationResult;
use crate::server::extract::{Authorization, PgRoTxn, QueryWorkspaceTenancy};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetApplicationRequest {
    pub application_id: ComponentId,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetApplicationResponse = Option<Component>;

pub async fn get_application(
    mut txn: PgRoTxn,
    Authorization(_claim): Authorization,
    Query(request): Query<GetApplicationRequest>,
    QueryWorkspaceTenancy(tenancy): QueryWorkspaceTenancy,
) -> ApplicationResult<Json<GetApplicationResponse>> {
    let txn = txn.start().await?;
    let component = Component::get_by_id(&txn, &tenancy, &request.visibility, &request.application_id).await?;
    Ok(Json(component))
}
