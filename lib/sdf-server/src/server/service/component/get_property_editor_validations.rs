use axum::extract::{Json, Query};
use dal::{ComponentId, Visibility};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetPropertyEditorValidationsRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

// TODO(nick): restore original type.
// pub type GetPropertyEditorValidationsResponse = PropertyEditorValidations;

pub type GetPropertyEditorValidationsResponse = Vec<()>;

pub async fn get_property_editor_validations(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetPropertyEditorValidationsRequest>,
) -> ComponentResult<Json<GetPropertyEditorValidationsResponse>> {
    let _ctx = builder.build(request_ctx.build(request.visibility)).await?;

    // let is_component_in_tenancy = Component::is_in_tenancy(&ctx, request.component_id).await?;
    // let is_component_in_visibility = Component::get_by_id(&ctx, &request.component_id)
    //     .await?
    //     .is_some();
    // if is_component_in_tenancy && !is_component_in_visibility {
    //     return Err(ComponentError::InvalidVisibility);
    // }
    //
    // let prop_edit_validations =
    //     PropertyEditorValidations::for_component(&ctx, request.component_id).await?;

    // TODO(nick): restore functionality.
    let prop_edit_validations = Vec::new();

    Ok(Json(prop_edit_validations))
}
