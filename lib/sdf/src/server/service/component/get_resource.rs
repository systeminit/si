use axum::extract::Query;
use axum::Json;
use dal::system::UNSET_SYSTEM_ID;
use dal::{Component, ComponentId, ResourceView, SystemId, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{PgRwTxn, Tenancy};

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
    Query(request): Query<GetResourceRequest>,
    Tenancy(_write_tenancy, read_tenancy): Tenancy,
) -> ComponentResult<Json<GetResourceResponse>> {
    let txn = txn.start().await?;

    let system_id = request.system_id.unwrap_or(UNSET_SYSTEM_ID);
    let resource = Component::get_resource_by_component_and_system(
        &txn,
        &read_tenancy,
        &request.visibility,
        request.component_id,
        system_id,
    )
    .await?
    .ok_or(ComponentError::ResourceNotFound(
        request.component_id,
        system_id,
    ))?;

    txn.commit().await?;
    Ok(Json(GetResourceResponse { resource }))
}
