use axum::{
    extract::{
        Host,
        OriginalUri,
        Path,
    },
    response::Response,
};
use base64::prelude::*;
use dal::{
    ChangeSet,
    ChangeSetId,
    WorkspacePk,
};
use hyper::{
    Body,
    header,
};
use si_db::Tenancy;

use crate::{
    extract::PosthogClient,
    service::v2::admin::{
        AdminAPIError,
        AdminAPIResult,
        AdminUserContext,
    },
    track_no_ctx,
};

pub async fn get_snapshot(
    AdminUserContext(mut ctx): AdminUserContext,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((workspace_id, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> AdminAPIResult<Response<Body>> {
    ctx.update_tenancy(Tenancy::new(workspace_id));

    let change_set = ChangeSet::get_by_id(&ctx, change_set_id).await?;

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
        workspace_id,
        change_set_id,
        "admin.get_snapshot",
        serde_json::json!({
            "workspace_snapshot_address": snap_addr.to_string(),
        }),
    );

    Ok(response)
}
