use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::service::diagram::DiagramResult;
use axum::extract::OriginalUri;
use axum::response::IntoResponse;
use axum::Json;
use dal::component::frame::Frame;
use dal::{ChangeSet, ComponentId, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DetachComponentRequest {
    pub child_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn detach_component_from_frame(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Json(request): Json<DetachComponentRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    Frame::orphan_child(&ctx, request.child_id).await?;

    // track(
    //     &posthog_client,
    //     &ctx,
    //     &original_uri,
    //     "detach_component_from_frame",
    //     serde_json::json!({
    //         "child_component_id": &request.component_id,
    //         "parent_component_ids": &request.parent_ids,
    //     }),
    // );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
