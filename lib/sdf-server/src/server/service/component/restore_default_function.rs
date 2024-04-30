use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use axum::extract::OriginalUri;
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
    OriginalUri(original_uri): OriginalUri,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<RestoreDefaultFunctionRequest>,
) -> ComponentResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    AttributeValue::use_default_prototype(&ctx, request.attribute_value_id).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "restore_default_function",
        serde_json::json!({
            "how": "/component/restore_default_function",
            "attribute_value_id": request.attribute_value_id.clone(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
