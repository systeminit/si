use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::{ChangeSet, Component, ComponentId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use crate::{
    extract::{v1::AccessBuilder, HandlerContext, PosthogClient},
    service::force_change_set_response::ForceChangeSetResponse,
    track,
};

use super::ComponentResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UnmanageComponentRequest {
    pub manager_component_id: ComponentId,
    pub managed_component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn unmanage(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(UnmanageComponentRequest {
        manager_component_id,
        managed_component_id,
        visibility,
    }): Json<UnmanageComponentRequest>,
) -> ComponentResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder.build(request_ctx.build(visibility)).await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    Component::unmanage_component(&ctx, manager_component_id, managed_component_id).await?;

    WsEvent::manages_edge_deleted(&ctx, manager_component_id, managed_component_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "unmanage_component",
        serde_json::json!({
            "how": "/component/unmanage",
            "manager_component_id": manager_component_id,
            "managed_component_id": managed_component_id,
        }),
    );

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
