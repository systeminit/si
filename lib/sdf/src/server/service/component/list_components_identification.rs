use axum::extract::Query;
use axum::Json;
use dal::{
    node::Node, resource::ResourceView, ComponentId, DiagramKind, LabelEntry, LabelList, Resource,
    SchemaId, SchemaVariantId, StandardModel, SystemId, Visibility, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::schema::SchemaError;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsIdentificationRequest {
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsIdentificationItem {
    pub component_id: ComponentId,
    pub schema_variant_id: SchemaVariantId,
    pub schema_id: SchemaId,
    pub schema_name: String,
    pub diagram_kind: DiagramKind,
    pub schema_variant_name: String,
    pub resource: Option<ResourceView>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsIdentificationResponse {
    pub list: LabelList<ListComponentsIdentificationItem>,
}

pub async fn list_components_identification(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListComponentsIdentificationRequest>,
) -> ComponentResult<Json<ListComponentsIdentificationResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let nodes = Node::list(&ctx).await?;

    let mut label_entries = Vec::with_capacity(nodes.len());
    for node in &nodes {
        let component = match node.component(&ctx).await? {
            Some(component) => component,
            None => continue,
        };
        let resource = Resource::get_by_component_and_system(&ctx, *component.id(), SystemId::NONE)
            .await?
            .map(ResourceView::new);

        let schema_variant = component
            .schema_variant(&ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;
        let schema = component
            .schema(&ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let diagram_kind = schema
            .diagram_kind()
            .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;

        let value = ListComponentsIdentificationItem {
            component_id: *component.id(),
            schema_variant_id: *schema_variant.id(),
            schema_variant_name: schema_variant.name().to_owned(),
            schema_id: *schema.id(),
            schema_name: schema.name().to_owned(),
            diagram_kind,
            resource,
        };
        label_entries.push(LabelEntry {
            label: component.name(&ctx).await?,
            value,
        });
    }
    let list = LabelList::from(label_entries);
    let response = ListComponentsIdentificationResponse { list };
    Ok(Json(response))
}
