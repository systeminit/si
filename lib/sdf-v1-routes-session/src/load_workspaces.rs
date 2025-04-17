use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::Workspace;
use serde::{Deserialize, Serialize};

use crate::SessionResult;
use sdf_core::tracking::track;
use sdf_extract::{v1::AccessBuilder, HandlerContext, PosthogClient};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoadWorkspaceResponse {
    pub workspaces: Vec<Workspace>,
}

pub async fn load_workspaces(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
) -> SessionResult<Json<LoadWorkspaceResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let workspaces = Workspace::list_for_user(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "workspace_loaded",
        serde_json::json!({}),
    );

    Ok(Json(LoadWorkspaceResponse { workspaces }))
}
