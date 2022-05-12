use axum::Json;
use dal::{
    edit_field::{EditFieldBaggage, EditFieldObjectKind},
    AttributeContext, Component, Visibility, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::{EditFieldError, EditFieldResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertFromEditFieldRequest {
    pub object_kind: EditFieldObjectKind,
    pub object_id: i64,
    pub edit_field_id: String,
    pub baggage: Option<EditFieldBaggage>,
    pub workspace_id: Option<WorkspaceId>,
    pub attribute_context: AttributeContext,
    pub key: Option<String>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertFromEditFieldResponse {
    pub success: bool,
}

/// # Panics
pub async fn insert_from_edit_field(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<InsertFromEditFieldRequest>,
) -> EditFieldResult<Json<InsertFromEditFieldResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    match request.object_kind {
        EditFieldObjectKind::ComponentProp => {
            let baggage = request.baggage.ok_or(EditFieldError::MissingBaggage)?;
            Component::insert_from_edit_field_with_baggage(
                &ctx,
                request.attribute_context,
                baggage,
                request.key,
            )
            .await?;
        }
        kind => return Err(EditFieldError::InvalidObjectKind(kind)),
    }

    txns.commit().await?;

    Ok(Json(InsertFromEditFieldResponse { success: true }))
}
