use axum::extract::Query;
use axum::Json;
use dal::property_editor::values::PropertyEditorValues;
use dal::{ComponentId, Visibility};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

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

    // TODO(nick): restore functionality.
    // let is_component_in_tenancy = Component::is_in_tenancy(&ctx, request.component_id).await?;
    // let is_component_in_visibility = Component::get_by_id(&ctx, &request.component_id)
    //     .await?
    //     .is_some();
    // if is_component_in_tenancy && !is_component_in_visibility {
    //     return Err(ComponentError::InvalidVisibility);
    // }
    //

    let prop_edit_values = PropertyEditorValues::assemble(&ctx, request.component_id).await?;

    // TODO(nick): this is temporary since main uses a serialized payload from the summary table.
    let prop_edit_values = serde_json::to_value(prop_edit_values)?;

    Ok(Json(prop_edit_values))
}
