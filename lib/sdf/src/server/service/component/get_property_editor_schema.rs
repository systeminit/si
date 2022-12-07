use axum::extract::Query;
use axum::Json;
use dal::property_editor::schema::PropertyEditorSchema;
use dal::{Component, ComponentId, StandardModel, Visibility};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

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

    let is_component_in_tenancy = Component::is_in_tenancy(&ctx, request.component_id).await?;
    let is_component_in_visibility = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .is_some();
    if is_component_in_tenancy && !is_component_in_visibility {
        return Err(ComponentError::InvalidVisibility);
    }

    let component = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(ComponentError::ComponentNotFound)?;
    let schema_variant_id = *component
        .schema_variant(&ctx)
        .await?
        .ok_or(ComponentError::SchemaNotFound)?
        .id();
    let prop_edit_schema =
        PropertyEditorSchema::for_schema_variant(&ctx, schema_variant_id).await?;

    Ok(Json(prop_edit_schema))
}
