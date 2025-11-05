use std::time::Duration;

use axum::{
    Json,
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
};
use dal::{
    ChangeSetId,
    WorkspacePk,
};
use futures_lite::StreamExt;
use sdf_core::index::{
    FrontEndObjectMeta,
    IndexError,
};
use telemetry::prelude::*;

use super::{
    AccessBuilder,
    IndexResult,
};
use crate::extract::{
    EddaClient,
    FriggStore,
    HandlerContext,
};

const WATCH_INDEX_TIMEOUT: Duration = Duration::from_secs(4);

pub async fn get_change_set_index(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    FriggStore(frigg): FriggStore,
    EddaClient(edda_client): EddaClient,
    Path((workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> IndexResult<impl IntoResponse> {
    let _ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let index = match frigg
        .get_change_set_index(workspace_pk, change_set_id)
        .await?
    {
        Some((index, _kv_revision)) => index,
        None => {
            info!(
                "Index not found for change_set {}; attempting full build",
                change_set_id,
            );
            if !request_rebuild_and_watch(&frigg, &edda_client, workspace_pk, change_set_id).await?
            {
                // Return 202 Accepted with the same response body if the build didn't succeed in time
                // to let the caller know the create succeeded, we're just waiting on downstream work
                return Ok((StatusCode::ACCEPTED, Json(None)));
            }
            frigg
                .get_change_set_index(workspace_pk, change_set_id)
                .await?
                .map(|i| i.0)
                .ok_or(IndexError::IndexNotFoundAfterFreshBuild(
                    workspace_pk,
                    change_set_id,
                ))?
        }
    };

    Ok((
        StatusCode::OK,
        Json(Some(FrontEndObjectMeta {
            workspace_snapshot_address: index.clone().id,
            index_checksum: index.clone().checksum,

            front_end_object: index,
        })),
    ))
}

#[instrument(
    level = "info",
    name = "sdf.index.request_rebuild_and_watch",
    skip_all,
    fields(
        si.workspace.id = %workspace_pk,
        si.change_set.id = %change_set_id,
        si.edda_request.id = Empty
    )
)]
async fn request_rebuild_and_watch(
    frigg: &frigg::FriggStore,
    edda_client: &edda_client::EddaClient,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
) -> IndexResult<bool> {
    let span = Span::current();
    let mut watch = frigg
        .watch_change_set_index(workspace_pk, change_set_id)
        .await?;
    let request_id = edda_client
        .rebuild_for_change_set(workspace_pk, change_set_id)
        .await?;
    span.record("si.edda_request.id", request_id.to_string());

    let timeout = WATCH_INDEX_TIMEOUT;
    tokio::select! {
        _ = tokio::time::sleep(timeout) => {
            info!("timed out waiting for new index to be rebuilt");
            Ok(false)
        },
        _ = watch.next() => Ok(true)
    }
}
