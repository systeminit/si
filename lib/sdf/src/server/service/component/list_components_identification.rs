use axum::extract::Query;
use axum::Json;
use dal::{
    node::Node, node::NodeId, ComponentId, LabelEntry, LabelList, SchemaId, SchemaVariantId,
    SchematicKind, StandardModel, Visibility, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

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
    pub schematic_kind: SchematicKind,
    pub schema_variant_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsIdentificationResponse {
    pub list: LabelList<ListComponentsIdentificationItem>,
}

pub async fn list_components_identification(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListComponentsIdentificationRequest>,
) -> ComponentResult<Json<ListComponentsIdentificationResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let nodes = Node::list(&ctx).await?;

    let mut label_entries = Vec::with_capacity(nodes.len());
    for node in &nodes {
        let component = match node.component(&ctx).await? {
            Some(component) => component,
            None => continue,
        };

        let schema_variant = component
            .schema_variant(&ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;
        let schema = component
            .schema(&ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;

        let value = ListComponentsIdentificationItem {
            component_id: *component.id(),
            schema_variant_id: *schema_variant.id(),
            schema_variant_name: schema_variant.name().to_owned(),
            schema_id: *schema.id(),
            schema_name: schema.name().to_owned(),
            schematic_kind: (*schema.kind()).into(),
        };
        label_entries.push(LabelEntry {
            label: component
                .find_value_by_json_pointer::<String>(&ctx, "/root/si/name")
                .await?
                .ok_or(ComponentError::ComponentNameNotFound)?,
            value,
        });
    }
    let list = LabelList::from(label_entries);
    let response = ListComponentsIdentificationResponse { list };
    Ok(Json(response))
}
