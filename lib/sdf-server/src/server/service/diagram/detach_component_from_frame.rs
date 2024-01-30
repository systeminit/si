use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::diagram::{DiagramError, DiagramResult};
use axum::extract::OriginalUri;
use axum::response::IntoResponse;
use axum::Json;
use dal::{ChangeSet, Component, ComponentId, Edge, StandardModel, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DetachComponentRequest {
    pub component_id: ComponentId,
    pub parent_component_ids: Vec<ComponentId>,
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

    let force_changeset_pk = ChangeSet::force_new(&mut ctx).await?;
    let child_comp = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?;

    let child_comp_edges = Edge::list_for_component(&ctx, *child_comp.id()).await?;
    for mut child_comp_edge in child_comp_edges {
        if request
            .parent_component_ids
            .contains(&child_comp_edge.head_component_id())
            || request
                .parent_component_ids
                .contains(&child_comp_edge.tail_component_id())
        {
            child_comp_edge.delete_and_propagate(&ctx).await?;
        }
    }

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "detach_component_from_frame",
        serde_json::json!({
            "child_component_id": &request.component_id,
            "parent_component_ids": &request.parent_component_ids,
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
