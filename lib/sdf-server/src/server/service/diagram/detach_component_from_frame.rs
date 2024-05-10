use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::diagram::DiagramResult;
use axum::extract::OriginalUri;
use axum::response::IntoResponse;
use axum::Json;
use dal::component::frame::Frame;
use dal::diagram::SummaryDiagramComponent;
use dal::{ChangeSet, Component, ComponentId, Visibility, WsEvent};
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
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<DetachComponentRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    Frame::orphan_child(&ctx, request.child_id).await?;

    let component: Component = Component::get_by_id(&ctx, request.child_id).await?;
    let payload: SummaryDiagramComponent =
        SummaryDiagramComponent::assemble(&ctx, &component).await?;
    WsEvent::component_updated(&ctx, payload)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "detach_component_from_frame",
        serde_json::json!({
            "how": "/diagram/detach_component_from_frame",
            "child_id": &request.child_id,
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
