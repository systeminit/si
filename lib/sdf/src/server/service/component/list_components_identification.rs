use axum::extract::Query;
use axum::Json;
use dal::{
    Component, ComponentId, LabelEntry, LabelList, SchemaId, SchemaVariantId, SchematicKind,
    StandardModel, Visibility, WorkspaceId,
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
    pub schematic_kind: SchematicKind,
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

    let components = Component::list(&ctx).await?;
    let mut label_entries = Vec::with_capacity(components.len());
    for component in &components {
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
            schema_id: *schema.id(),
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
