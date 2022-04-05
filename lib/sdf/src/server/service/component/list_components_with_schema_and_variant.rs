use axum::extract::Query;
use axum::Json;
use dal::{
    Component, ComponentId, LabelEntry, LabelList, SchemaId, SchemaVariantId, StandardModel,
    Visibility, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsWithSchemaAndVariantRequest {
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsWithSchemaAndVariantItem {
    pub component_id: ComponentId,
    pub schema_variant_id: SchemaVariantId,
    pub schema_id: SchemaId,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsWithSchemaAndVariantResponse {
    pub list: LabelList<ListComponentsWithSchemaAndVariantItem>,
}

pub async fn list_components_with_schema_and_variant(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListComponentsWithSchemaAndVariantRequest>,
) -> ComponentResult<Json<ListComponentsWithSchemaAndVariantResponse>> {
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

        let value = ListComponentsWithSchemaAndVariantItem {
            component_id: *component.id(),
            schema_variant_id: *schema_variant.id(),
            schema_id: *schema.id(),
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
    let response = ListComponentsWithSchemaAndVariantResponse { list };
    Ok(Json(response))
}
