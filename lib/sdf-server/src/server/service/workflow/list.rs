use super::{WorkflowError, WorkflowResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{
    Component, ComponentId, Schema, SchemaVariant, StandardModel, Visibility, WorkflowPrototype,
    WorkflowPrototypeId,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListWorkflowsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListedWorkflowComponentView {
    id: ComponentId,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListedWorkflowView {
    id: WorkflowPrototypeId,
    title: String,
    description: Option<String>,
    link: Option<String>,
    components: Vec<ListedWorkflowComponentView>,
    schema_name: Option<String>,
    schema_variant_name: Option<String>,
}

pub type ListWorkflowsResponse = Vec<ListedWorkflowView>;

pub async fn list(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListWorkflowsRequest>,
) -> WorkflowResult<Json<ListWorkflowsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let prototypes = WorkflowPrototype::list(&ctx).await?;
    let mut views = Vec::with_capacity(prototypes.len());
    for proto in prototypes {
        let components = if proto.context().component_id.is_some() {
            let component = Component::get_by_id(&ctx, &proto.context().component_id)
                .await?
                .ok_or_else(|| WorkflowError::ComponentNotFound(proto.context().component_id))?;
            vec![ListedWorkflowComponentView {
                id: *component.id(),
                name: component.name(&ctx).await?,
            }]
        } else {
            let mut components = Vec::new();
            if proto.context().schema_variant_id.is_some() {
                for component in
                    Component::list_for_schema_variant(&ctx, proto.context().schema_variant_id)
                        .await?
                {
                    components.push(ListedWorkflowComponentView {
                        id: *component.id(),
                        name: component.name(&ctx).await?,
                    });
                }
            }
            components
        };

        let schema_name = if proto.context().schema_id.is_some() {
            let schema = Schema::get_by_id(&ctx, &proto.context().schema_id)
                .await?
                .ok_or_else(|| WorkflowError::SchemaNotFound(proto.context().schema_id))?;
            Some(schema.name().to_owned())
        } else {
            None
        };

        let schema_variant_name = if proto.context().schema_variant_id.is_some() {
            let schema_variant = SchemaVariant::get_by_id(&ctx, &proto.context().schema_variant_id)
                .await?
                .ok_or_else(|| {
                    WorkflowError::SchemaVariantNotFound(proto.context().schema_variant_id)
                })?;
            Some(schema_variant.name().to_owned())
        } else {
            None
        };

        views.push(ListedWorkflowView {
            id: proto.id().to_owned(),
            title: proto.title().to_owned(),
            description: proto.description().map(Into::into),
            link: proto.link().map(Into::into),
            components,
            schema_name,
            schema_variant_name,
        });
    }

    Ok(Json(views))
}
