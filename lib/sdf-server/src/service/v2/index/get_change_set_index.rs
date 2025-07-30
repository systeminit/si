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
use sdf_core::index::{
    FrontEndObjectMeta,
    IndexError,
};
use telemetry::prelude::*;

use super::{
    AccessBuilder,
    IndexResult,
    request_rebuild_and_watch,
};
use crate::extract::{
    EddaClient,
    FriggStore,
    HandlerContext,
};

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
