use axum::extract::Query;
use axum::Json;
use dal::{
    resource::ResourceHealth, system::UNSET_SYSTEM_ID, Component, ComponentId, StandardModel,
    SystemId, Visibility, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

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
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetComponentMetadataRequest>,
) -> ComponentResult<Json<GetComponentMetadataResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let component = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(ComponentError::NotFound)?;

    let schema = component
        .schema_with_tenancy(&ctx)
        .await?
        .ok_or(ComponentError::SchemaNotFound)?;

    let system_id = request.system_id.unwrap_or(UNSET_SYSTEM_ID);

    let qualifications =
        Component::list_qualifications_by_component_id(&ctx, *component.id(), system_id).await?;

    let qualified = qualifications
        .into_iter()
        .flat_map(|q| q.result.map(|r| r.success))
        .reduce(|q, acc| acc && q);

    let resource =
        Component::get_resource_by_component_and_system(&ctx, request.component_id, system_id)
            .await?;

    let response = GetComponentMetadataResponse {
        schema_name: schema.name().to_owned(),
        qualified,
        resource_health: resource.map(|r| r.health),
    };
    Ok(Json(response))
}
