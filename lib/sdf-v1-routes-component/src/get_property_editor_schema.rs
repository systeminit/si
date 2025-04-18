use axum::{extract::Query, Json};
use dal::{property_editor::schema::PropertyEditorSchema, Component, ComponentId, Visibility};
use sdf_extract::{v1::AccessBuilder, HandlerContext};
use serde::{Deserialize, Serialize};

use super::ComponentResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetPropertyEditorSchemaRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetPropertyEditorSchemaResponse = PropertyEditorSchema;

pub async fn get_property_editor_schema(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetPropertyEditorSchemaRequest>,
) -> ComponentResult<Json<GetPropertyEditorSchemaResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let schema_variant =
        Component::schema_variant_for_component_id(&ctx, request.component_id).await?;
    let maybe_resource = Component::resource_by_id(&ctx, request.component_id).await?;

    let prop_edit_schema =
        PropertyEditorSchema::assemble(&ctx, schema_variant.id(), maybe_resource.is_some()).await?;

    Ok(Json(prop_edit_schema))
}
