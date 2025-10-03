use std::collections::BTreeMap;

use axum::extract::{
    Host,
    Multipart,
    OriginalUri,
    Path,
};
use dal::{
    ChangeSetId,
    ContentHash,
    WorkspacePk,
    WorkspaceSnapshotAddress,
    layer_db_types::ContentTypes,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::Tenancy;
use si_layer_cache::db;
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
    name = "admin.upload_cas_data",
    level = "info",
    skip_all,
    fields(
        si.change_set.id = %change_set_id,
        si.workspace.id = %workspace_id,
    ),
)]
pub async fn upload_cas_data(
    AdminUserContext(mut ctx): AdminUserContext,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((workspace_id, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    mut multipart: Multipart,
) -> AdminAPIResult<()> {
    ctx.update_tenancy(Tenancy::new(workspace_id));

    let cas_data_bytes = multipart
        .next_field()
        .await?
        .ok_or(AdminAPIError::NoMultipartData)?
        .bytes()
        .await?;

    let cas_data: BTreeMap<ContentHash, ContentTypes> = tokio::task::spawn_blocking(move || {
        Ok::<_, AdminAPIError>(db::serialize::from_bytes(&cas_data_bytes)?)
    })
    .await??;

    for (cas_addr, cas_obj) in cas_data {
        if ctx.layer_db().cas().read(&cas_addr).await?.is_none() {
            let bytes = tokio::task::spawn_blocking(move || {
                Ok::<Vec<u8>, AdminAPIError>(db::serialize::to_vec(&cas_obj)?.0)
            })
            .await??;

            ctx.layer_db()
                .cas()
                .write_bytes_to_durable_storage(&cas_addr, &bytes)
                .await?;
        }
    }

    ctx.commit_no_rebase().await?;

    track_no_ctx(
        &posthog_client,
        &original_uri,
        &host_name,
        ctx.history_actor().distinct_id(),
        workspace_id,
        change_set_id,
        "admin.upload_cas_data",
        serde_json::json!({}),
    );

    Ok(())
}
