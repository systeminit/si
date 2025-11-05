use axum::extract::Path;
use dal::{
    ChangeSetId,
    WorkspacePk,
};
use telemetry::prelude::*;

use super::IndexResult;
use crate::extract::EddaClient;

pub async fn rebuild_change_set_index(
    EddaClient(edda_client): EddaClient,
    Path((workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> IndexResult<()> {
    request_rebuild(&edda_client, workspace_pk, change_set_id).await
}

#[instrument(
    level = "info",
    name = "sdf.index.request_rebuild",
    skip_all,
    fields(
        si.workspace.id = %workspace_pk,
        si.change_set.id = %change_set_id,
        si.edda_request.id = Empty
    )
)]
async fn request_rebuild(
    edda_client: &edda_client::EddaClient,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
) -> IndexResult<()> {
    let span = Span::current();
    let request_id = edda_client
        .rebuild_for_change_set(workspace_pk, change_set_id)
        .await?;
    span.record("si.edda_request.id", request_id.to_string());
    Ok(())
}
