use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use dal::component::frame::Frame;
use dal::{ChangeSet, ComponentId, Visibility, WsEvent};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

use super::DiagramResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFrameConnectionRequest {
    pub child_id: ComponentId,
    pub parent_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

/// Connect a child [`Component`](dal::Component) to a parent [`Component`](dal::Component).
/// detaching any existing parents first and creating a change set if on head.
pub async fn connect_component_to_frame(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CreateFrameConnectionRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    // Connect children to parent through frame edge
    Frame::upsert_parent(&ctx, request.child_id, request.parent_id).await?;

    WsEvent::component_updated(&ctx, request.child_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "connect_component_to_frame",
        serde_json::json!({
            "how": "/diagram/connect_component_to_frame",
            "child_id": request.child_id,
            "parent_id": request.parent_id,
            "change_set_id": ctx.change_set_id(),
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response
        .header("content-type", "application/json")
        .body("{}".to_owned())?)
}
