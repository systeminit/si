use axum::extract::Query;
use axum::Json;
use dal::{
    resource::ResourceHealth, system::UNSET_SYSTEM_ID, Component, ComponentId, StandardModel,
    SystemId, Visibility, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{PgRoTxn, Tenancy};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetComponentMetadataRequest {
    pub component_id: ComponentId,
    pub system_id: Option<SystemId>,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetComponentMetadataResponse {
    pub schema_name: String,
    pub qualified: Option<bool>,
    pub resource_health: Option<ResourceHealth>,
}

pub async fn get_component_metadata(
    mut txn: PgRoTxn,
    Query(request): Query<GetComponentMetadataRequest>,
    Tenancy(_write_tenancy, read_tenancy): Tenancy,
) -> ComponentResult<Json<GetComponentMetadataResponse>> {
    let txn = txn.start().await?;

    let component = Component::get_by_id(
        &txn,
        &(&read_tenancy).into(),
        &request.visibility,
        &request.component_id,
    )
    .await?
    .ok_or(ComponentError::NotFound)?;

    let schema = component
        .schema_with_tenancy(&txn, &(&read_tenancy).into(), &request.visibility)
        .await?
        .ok_or(ComponentError::SchemaNotFound)?;

    let system_id = request.system_id.unwrap_or(UNSET_SYSTEM_ID);

    let qualifications = Component::list_qualifications_by_component_id(
        &txn,
        &(&read_tenancy).into(),
        &request.visibility,
        *component.id(),
        system_id,
    )
    .await?;

    let qualified = qualifications
        .into_iter()
        .flat_map(|q| q.result.map(|r| r.success))
        .reduce(|q, acc| acc && q);

    let resource = Component::get_resource_by_component_and_system(
        &txn,
        &read_tenancy,
        &request.visibility,
        request.component_id,
        system_id,
    )
    .await?;

    let response = GetComponentMetadataResponse {
        schema_name: schema.name().to_owned(),
        qualified,
        resource_health: resource.map(|r| r.health),
    };
    Ok(Json(response))
}
