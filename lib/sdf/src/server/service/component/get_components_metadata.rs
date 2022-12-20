use axum::extract::Query;
use axum::Json;
use dal::{
    qualification::QualificationSubCheckStatus, Component, ComponentId, StandardModel, Visibility,
    WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetComponentsMetadataRequest {
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ComponentMetadata {
    pub schema_name: String,
    pub schema_link: Option<String>,
    pub qualified: Option<bool>,
    pub component_id: ComponentId,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetComponentsMetadataResponse {
    pub data: Vec<ComponentMetadata>,
}

pub async fn get_components_metadata(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetComponentsMetadataRequest>,
) -> ComponentResult<Json<GetComponentsMetadataResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let components = Component::list(&ctx).await?;
    let mut metadata = Vec::with_capacity(components.len());

    // Note: this is slow, we should have a better way of doing this
    for component in components {
        let schema = component
            .schema(&ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;

        let qualifications = Component::list_qualifications(&ctx, *component.id()).await?;

        let qualified = qualifications
            .into_iter()
            .map(|q| {
                q.result
                    .map(|r| r.status == QualificationSubCheckStatus::Success)
            })
            .reduce(|q, acc| acc.and_then(|acc| q.map(|q| acc && q)))
            .and_then(|opt| opt);

        metadata.push(ComponentMetadata {
            schema_name: schema.name().to_owned(),
            schema_link: component
                .schema_variant(&ctx)
                .await?
                .and_then(|v| v.link().map(ToOwned::to_owned)),
            qualified,
            component_id: *component.id(),
        });
    }
    Ok(Json(GetComponentsMetadataResponse { data: metadata }))
}
