use super::{SessionError, SessionResult};
use crate::server::extract::{AccessBuilder, Authorization, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::Workspace;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoadWorkspaceResponse {
    pub workspace: Workspace,
}

pub async fn load_workspace(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Authorization(claim): Authorization,
) -> SessionResult<Json<LoadWorkspaceResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let workspace = Workspace::get_by_pk(&ctx, &claim.workspace_pk)
        .await?
        .ok_or(SessionError::InvalidWorkspace(claim.workspace_pk))?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "workspace_loaded",
        serde_json::json!({}),
    );

    Ok(Json(LoadWorkspaceResponse { workspace }))
}
