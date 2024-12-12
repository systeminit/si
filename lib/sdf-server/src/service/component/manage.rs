use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::{ChangeSet, Component, ComponentId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use crate::{
    extract::{v1::AccessBuilder, HandlerContext, PosthogClient},
    routes::AppError,
    service::force_change_set_response::ForceChangeSetResponse,
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ManageComponentRequest {
    pub manager_component_id: ComponentId,
    pub managed_component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn manage(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(ManageComponentRequest {
        manager_component_id,
        managed_component_id,
        visibility,
    }): Json<ManageComponentRequest>,
) -> Result<ForceChangeSetResponse<()>, AppError> {
    let mut ctx = builder.build(request_ctx.build(visibility)).await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let edge =
        Component::manage_component(&ctx, manager_component_id, managed_component_id).await?;

    WsEvent::connection_upserted(&ctx, edge.into())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "manage_component",
        serde_json::json!({
            "how": "/component/manage",
            "manager_component_id": manager_component_id,
            "managed_component_id": managed_component_id,
        }),
    );

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
