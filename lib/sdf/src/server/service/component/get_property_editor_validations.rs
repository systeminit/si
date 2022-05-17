use axum::extract::Query;
use axum::Json;
use dal::property_editor::PropertyEditorValidations;
use dal::{ComponentId, SystemId, Visibility};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetPropertyEditorValidationsRequest {
    pub component_id: ComponentId,
    pub system_id: SystemId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetPropertyEditorValidationsResponse = PropertyEditorValidations;

pub async fn get_property_editor_validations(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetPropertyEditorValidationsRequest>,
) -> ComponentResult<Json<GetPropertyEditorValidationsResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let prop_edit_validations =
        PropertyEditorValidations::for_component(&ctx, request.component_id, request.system_id)
            .await?;

    txns.commit().await?;

    Ok(Json(prop_edit_validations))
}
