use axum::extract::OriginalUri;
use axum::Json;
use dal::change_set_pointer::ChangeSetPointer;
use dal::{Workspace, WorkspaceError};
// use dal::ChangeSet;
use serde::{Deserialize, Serialize};

use super::{ChangeSetError, ChangeSetResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetRequest {
    pub change_set_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetResponse {
    pub change_set: ChangeSetPointer,
}

pub async fn create_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CreateChangeSetRequest>,
) -> ChangeSetResult<Json<CreateChangeSetResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let change_set_name = &request.change_set_name;

    let workspace_pk = ctx
        .tenancy()
        .workspace_pk()
        .ok_or(ChangeSetError::NoTenancySet)?;

    let workspace = Workspace::get_by_pk(&ctx, &workspace_pk)
        .await?
        .ok_or(ChangeSetError::WorkspaceNotFound(workspace_pk))?;

    let base_change_set_pointer = ChangeSetPointer::find(&ctx, workspace.default_change_set_id())
        .await?
        .ok_or(ChangeSetError::DefaultChangeSetNotFound(
            workspace.default_change_set_id(),
        ))?;

    let mut change_set_pointer = ChangeSetPointer::new(
        &ctx,
        change_set_name,
        Some(workspace.default_change_set_id()),
    )
    .await?;

    change_set_pointer
        .update_pointer(
            &ctx,
            base_change_set_pointer.workspace_snapshot_id.ok_or(
                ChangeSetError::DefaultChangeSetNoWorkspaceSnapshotPointer(
                    workspace.default_change_set_id(),
                ),
            )?,
        )
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "create_change_set",
        serde_json::json!({
                    "change_set_name": change_set_name,
        }),
    );

    ctx.commit_no_rebase().await?;

    Ok(Json(CreateChangeSetResponse {
        change_set: change_set_pointer,
    }))
}
