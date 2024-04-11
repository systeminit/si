use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{response::IntoResponse, Json};
use dal::{AttributeValue, AttributeValueId, ChangeSet, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreDefaultFunctionRequest {
    pub attribute_value_id: AttributeValueId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn restore_default_function(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<RestoreDefaultFunctionRequest>,
) -> ComponentResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    AttributeValue::use_default_prototype(&ctx, request.attribute_value_id).await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
