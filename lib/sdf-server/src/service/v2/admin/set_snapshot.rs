use std::sync::Arc;

use axum::{
    extract::{
        Host,
        Multipart,
        OriginalUri,
        Path,
    },
    response::Json,
};
use dal::{
    ChangeSet,
    ChangeSetId,
    WorkspacePk,
    WorkspaceSnapshotAddress,
    WorkspaceSnapshotGraph,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::Tenancy;
use telemetry::prelude::*;

use crate::{
    extract::PosthogClient,
    service::v2::admin::{
        AdminAPIError,
        AdminAPIResult,
        AdminUserContext,
    },
    track_no_ctx,
};

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct SetSnapshotResponse {
    workspace_snapshot_address: WorkspaceSnapshotAddress,
}

#[instrument(
    name = "admin.set_snapshot",
    level = "info",
    skip_all,
    fields(
        si.change_set.id = %change_set_id,
        si.workspace.id = %workspace_id,
        si.workspace_snapshot.address = Empty,
    ),
)]
pub async fn set_snapshot(
    AdminUserContext(mut ctx): AdminUserContext,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((workspace_id, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    mut multipart: Multipart,
) -> AdminAPIResult<Json<SetSnapshotResponse>> {
    ctx.update_tenancy(Tenancy::new(workspace_id));

    let span = current_span_for_instrument_at!("info");

    let mut change_set = ChangeSet::get_by_id(&ctx, change_set_id).await?;

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
        "si.workspace_snapshot.address",
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
        workspace_id,
        change_set_id,
        "admin.set_snapshot",
        serde_json::json!({
            "workspace_snapshot_address": workspace_snapshot_address.to_string(),
        }),
    );

    Ok(Json(SetSnapshotResponse {
        workspace_snapshot_address,
    }))
}
