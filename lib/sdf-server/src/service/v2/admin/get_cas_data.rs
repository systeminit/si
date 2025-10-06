use std::{
    collections::BTreeMap,
    sync::Arc,
};

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
    ChangeSetId,
    ContentHash,
    WorkspacePk,
    layer_db_types::ContentTypes,
    slow_rt,
};
use hyper::{
    Body,
    header,
};
use si_db::Tenancy;
use si_layer_cache::db;

use crate::{
    extract::PosthogClient,
    service::v2::admin::{
        AdminAPIError,
        AdminAPIResult,
        AdminUserContext,
    },
    track_no_ctx,
};

pub async fn get_cas_data(
    AdminUserContext(mut ctx): AdminUserContext,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((workspace_id, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> AdminAPIResult<Response<Body>> {
    ctx.update_tenancy(Tenancy::new(workspace_id));
    ctx.update_visibility_and_snapshot_to_visibility(change_set_id)
        .await?;

    let ctx_clone = ctx.clone();
    let cas_data = slow_rt::spawn(async move {
        let mut cas_data = BTreeMap::new();
        for node in ctx_clone.workspace_snapshot()?.nodes().await? {
            for cas_addr in node.content_store_hashes() {
                if cas_data.contains_key(&cas_addr) {
                    continue;
                }

                if let Some(cas_obj) = ctx_clone.layer_db().cas().read(&cas_addr).await? {
                    cas_data.insert(cas_addr, cas_obj);
                }
            }
        }

        Ok::<BTreeMap<ContentHash, Arc<ContentTypes>>, AdminAPIError>(cas_data)
    })?
    .await??;

    let base64 = tokio::task::spawn_blocking(move || {
        let (serialized, _) = db::serialize::to_vec(&cas_data)?;
        Ok::<String, AdminAPIError>(BASE64_STANDARD.encode(&serialized))
    })
    .await??;

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
        "admin.get_cas_data",
        serde_json::json!({}),
    );

    Ok(response)
}
