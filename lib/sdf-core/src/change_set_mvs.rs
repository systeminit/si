use std::time::Duration;

use dal::{
    ChangeSetId,
    WorkspacePk,
    WorkspaceSnapshotAddress,
};
use futures_lite::StreamExt;
use telemetry::prelude::*;
use thiserror::Error;

const WATCH_INDEX_TIMEOUT: Duration = Duration::from_secs(30);

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetMvsError {
    #[error("Edda client error: {0}")]
    EddaClient(#[from] edda_client::ClientError),
    #[error("Frigg error: {0}")]
    Frigg(#[from] frigg::FriggError),
}

pub type Result<T> = std::result::Result<T, ChangeSetMvsError>;

#[instrument(
    level = "info",
    name = "sdf.change_set.create_index_for_new_change_set_and_watch",
    skip_all,
    fields(
        si.edda_request.id = Empty
    )
)]
pub async fn create_index_for_new_change_set_and_watch(
    frigg: &frigg::FriggStore,
    edda_client: &edda_client::EddaClient,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
    base_change_set_id: ChangeSetId,
    to_snapshot_address: WorkspaceSnapshotAddress,
) -> Result<bool> {
    let span = Span::current();
    let mut watch = frigg
        .watch_change_set_index(workspace_pk, change_set_id)
        .await?;
    let request_id = edda_client
        .new_change_set(
            workspace_pk,
            change_set_id,
            base_change_set_id,
            to_snapshot_address,
        )
        .await?;
    span.record("si.edda_request.id", request_id.to_string());

    let timeout = WATCH_INDEX_TIMEOUT;
    tokio::select! {
        _ = tokio::time::sleep(timeout) => {
            info!("timed out waiting for new change set index to be created");
            Ok(false)
        },
        _ = watch.next() => Ok(true)
    }
}
