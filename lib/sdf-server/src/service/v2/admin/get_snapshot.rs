use axum::{
    extract::{Host, OriginalUri, Path},
    response::{IntoResponse, Response},
};
use base64::prelude::*;
use dal::{ChangeSet, ChangeSetId, WorkspacePk};
use hyper::{header, Body};

use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    track_no_ctx,
};

use super::{AdminAPIError, AdminAPIResult};

pub async fn get_snapshot(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> AdminAPIResult<impl IntoResponse> {
    let ctx = builder.build_head(access_builder).await?;

    let change_set = ChangeSet::find(&ctx, change_set_id)
        .await?
        .ok_or(AdminAPIError::ChangeSetNotFound(change_set_id))?;

    let snap_addr = change_set.workspace_snapshot_address;

    let bytes = ctx
        .layer_db()
        .workspace_snapshot()
        .read_bytes_from_durable_storage(&snap_addr)
        .await?
        .ok_or(AdminAPIError::WorkspaceSnapshotNotFound(
            snap_addr,
            change_set_id,
        ))?;

    let base64 = tokio::task::spawn_blocking(|| BASE64_STANDARD.encode(bytes)).await?;

    let body = Body::from(base64);

    let response = Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(body)?;

    track_no_ctx(
        &posthog_client,
        &original_uri,
        &host_name,
        ctx.history_actor().distinct_id(),
        Some(workspace_pk.to_string()),
        Some(change_set_id.to_string()),
        "admin.get_snapshot",
        serde_json::json!({
            "workspace_snapshot_address": snap_addr.to_string(),
        }),
    );

    Ok(response)
}
