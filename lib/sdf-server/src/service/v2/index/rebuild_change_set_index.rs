use axum::extract::Path;
use dal::{ChangeSetId, WorkspacePk};

use crate::extract::EddaClient;

use super::{IndexResult, request_rebuild};

pub async fn rebuild_change_set_index(
    EddaClient(edda_client): EddaClient,
    Path((workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> IndexResult<()> {
    request_rebuild(&edda_client, workspace_pk, change_set_id).await
}
