use std::sync::Arc;

use axum::{
    extract::{Host, Multipart, OriginalUri, Path},
    response::Json,
};

use telemetry::prelude::*;

use dal::{ChangeSet, ChangeSetId, WorkspacePk, WorkspaceSnapshotAddress, WorkspaceSnapshotGraph};
use serde::{Deserialize, Serialize};

use crate::server::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    tracking::track_no_ctx,
};

use super::{AdminAPIError, AdminAPIResult};

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct SetSnapshotResponse {
    workspace_snapshot_address: WorkspaceSnapshotAddress,
}

#[instrument(name = "admin.set_snapshot", skip_all)]
pub async fn set_snapshot(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    mut multipart: Multipart,
) -> AdminAPIResult<Json<SetSnapshotResponse>> {
    let span = Span::current();
    span.record("si.workspace.id", workspace_pk.to_string());
    span.record("si.change_set.id", change_set_id.to_string());

    let ctx = builder.build_head(access_builder).await?;

    let mut change_set = ChangeSet::find(&ctx, change_set_id)
        .await?
        .ok_or(AdminAPIError::ChangeSetNotFound(change_set_id))?;

    let snapshot_data = Arc::new(
        multipart
            .next_field()
            .await?
            .ok_or(AdminAPIError::NoMultipartData)?
            .bytes()
            .await?,
    );

    let data_clone = snapshot_data.clone();
    let (workspace_snapshot_address, _) = tokio::task::spawn_blocking(move || {
        let uploaded_address = WorkspaceSnapshotAddress::new(&data_clone);
        // We do this to make sure the uploaded snapshot is valid
        let graph: Arc<WorkspaceSnapshotGraph> =
            si_layer_cache::db::serialize::from_bytes(&data_clone)?;
        Ok::<(WorkspaceSnapshotAddress, Arc<WorkspaceSnapshotGraph>), AdminAPIError>((
            uploaded_address,
            graph,
        ))
    })
    .await??;

    span.record(
        "workspace_snapshot_address",
        workspace_snapshot_address.to_string(),
    );

    // Our compression algorithm appears to be non-deterministic, so we write
    // exactly the bytes we received, so that the snapshot address matches the
    // blake3 hash seen locally. Otherwise it makes it seem like we're writing a
    // different snapshot than the one we uploaded
    ctx.layer_db()
        .workspace_snapshot()
        .write_bytes_to_durable_storage(&workspace_snapshot_address, &snapshot_data)
        .await?;

    change_set
        .update_pointer(&ctx, workspace_snapshot_address)
        .await?;

    ctx.commit_no_rebase().await?;

    track_no_ctx(
        &posthog_client,
        &original_uri,
        &host_name,
        ctx.history_actor().distinct_id(),
        Some(workspace_pk.to_string()),
        Some(change_set_id.to_string()),
        "admin.set_snapshot",
        serde_json::json!({
            "workspace_snapshot_address": workspace_snapshot_address.to_string(),
        }),
    );

    Ok(Json(SetSnapshotResponse {
        workspace_snapshot_address,
    }))
}