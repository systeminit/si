use axum::{
    extract::Path,
    response::{IntoResponse, Response},
};
use base64::prelude::*;
use dal::{ChangeSet, ChangeSetId, WorkspacePk};
use hyper::{header, Body};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::{AdminAPIError, AdminAPIResult};

pub async fn get_snapshot(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> AdminAPIResult<impl IntoResponse> {
    let ctx = builder.build_head(access_builder).await?;

    let change_set = ChangeSet::find(&ctx, change_set_id)
        .await?
        .ok_or(AdminAPIError::ChangeSetNotFound(change_set_id))?;

    let snap_addr = change_set.workspace_snapshot_address.ok_or(
        AdminAPIError::WorkspaceSnapshotAddressNotFound(change_set_id),
    )?;

    let bytes = ctx
        .layer_db()
        .workspace_snapshot()
        .read_bytes_from_durable_storage(&snap_addr)
        .await?
        .ok_or(AdminAPIError::WorkspaceSnapshotNotFound(
            snap_addr,
            change_set_id,
        ))?;

    let base64 = BASE64_STANDARD.encode(bytes);

    let body = Body::from(base64);

    let response = Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(body)?;

    Ok(response)
}
