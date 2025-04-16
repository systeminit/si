use axum::{extract::Query, Json};
use dal::{property_editor::values::PropertyEditorValues, ComponentId, Visibility};
use sdf_extract::{v1::AccessBuilder, HandlerContext};
use serde::{Deserialize, Serialize};

use super::ComponentResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetPropertyEditorValuesRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetPropertyEditorValuesResponse = PropertyEditorValues;

pub async fn get_property_editor_values(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetPropertyEditorValuesRequest>,
) -> ComponentResult<Json<serde_json::Value>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let prop_edit_values = PropertyEditorValues::assemble(&ctx, request.component_id).await?;

    let prop_edit_values = serde_json::to_value(prop_edit_values)?;

    Ok(Json(prop_edit_values))
}
