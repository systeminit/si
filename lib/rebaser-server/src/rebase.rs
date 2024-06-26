use dal::change_set::{ChangeSet, ChangeSetError, ChangeSetId};
use dal::workspace_snapshot::vector_clock::VectorClockId;
use dal::workspace_snapshot::WorkspaceSnapshotError;
use dal::{DalContext, TransactionsError, WorkspaceSnapshot, WsEventError};
use si_events::WorkspaceSnapshotAddress;
use si_layer_cache::activities::rebase::RebaseStatus;
use si_layer_cache::activities::ActivityRebaseRequest;
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::time::Instant;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum RebaseError {
    #[error("workspace snapshot error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("layerdb error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("missing change set")]
    MissingChangeSet(ChangeSetId),
    #[error("missing workspace snapshot for change set ({0}) (the change set likely isn't pointing at a workspace snapshot)")]
    MissingWorkspaceSnapshotForChangeSet(ChangeSetId),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

type RebaseResult<T> = Result<T, RebaseError>;

#[instrument(name = "perform_rebase", level = "debug", skip_all, fields())]
pub async fn perform_rebase(
    ctx: &mut DalContext,
    message: &ActivityRebaseRequest,
) -> RebaseResult<RebaseStatus> {
    let start = Instant::now();
    // Gather everything we need to detect conflicts and updates from the inbound message.
    let mut to_rebase_change_set =
        ChangeSet::find(ctx, message.payload.to_rebase_change_set_id.into())
            .await?
            .ok_or(RebaseError::MissingChangeSet(
                message.payload.to_rebase_change_set_id.into(),
            ))?;
    let to_rebase_workspace_snapshot_address =
        to_rebase_change_set.workspace_snapshot_address.ok_or(
            RebaseError::MissingWorkspaceSnapshotForChangeSet(to_rebase_change_set.id),
        )?;
    debug!("before snapshot fetch and parse: {:?}", start.elapsed());
    let to_rebase_workspace_snapshot =
        WorkspaceSnapshot::find(ctx, to_rebase_workspace_snapshot_address).await?;
    let onto_workspace_snapshot: WorkspaceSnapshot =
        WorkspaceSnapshot::find(ctx, message.payload.onto_workspace_snapshot_address).await?;
    info!(
        "to_rebase_id: {}, onto_id: {}",
        to_rebase_workspace_snapshot_address,
        onto_workspace_snapshot.id().await
    );
    debug!("after snapshot fetch and parse: {:?}", start.elapsed());

    // Perform the conflicts and updates detection.
    let onto_vector_clock_id: VectorClockId = message.payload.onto_vector_clock_id.into();
    let conflicts_and_updates = to_rebase_workspace_snapshot
        .detect_conflicts_and_updates(
            to_rebase_change_set.vector_clock_id(),
            &onto_workspace_snapshot,
            onto_vector_clock_id,
        )
        .await?;
    info!(
        "count: conflicts ({}) and updates ({}), {:?}",
        conflicts_and_updates.conflicts.len(),
        conflicts_and_updates.updates.len(),
        start.elapsed()
    );

    // If there are conflicts, immediately assemble a reply message that conflicts were found.
    // Otherwise, we can perform updates and assemble a "success" reply message.
    let message: RebaseStatus = if conflicts_and_updates.conflicts.is_empty() {
        to_rebase_workspace_snapshot
            .perform_updates(
                &to_rebase_change_set,
                &onto_workspace_snapshot,
                conflicts_and_updates.updates.as_slice(),
            )
            .await?;
        info!("updates complete: {:?}", start.elapsed());

        if !conflicts_and_updates.updates.is_empty() {
            // Once all updates have been performed, we can write out, mark everything as recently seen
            // and update the pointer.
            to_rebase_workspace_snapshot
                .write(ctx, to_rebase_change_set.vector_clock_id())
                .await?;
            info!("snapshot written: {:?}", start.elapsed());
            to_rebase_change_set
                .update_pointer(ctx, to_rebase_workspace_snapshot.id().await)
                .await?;
            info!("pointer updated: {:?}", start.elapsed());
        }

        RebaseStatus::Success {
            updates_performed: serde_json::to_value(conflicts_and_updates.updates)?.to_string(),
        }
    } else {
        RebaseStatus::ConflictsFound {
            conflicts_found: serde_json::to_value(conflicts_and_updates.conflicts)?.to_string(),
            updates_found_and_skipped: serde_json::to_value(conflicts_and_updates.updates)?
                .to_string(),
        }
    };

    info!("rebase performed: {:?}", start.elapsed());

    // Before replying to the requester, we must commit.
    ctx.commit_no_rebase().await?;

    {
        let ictx = ctx.clone();
        tokio::spawn(async move {
            if let Err(error) =
                evict_unused_snapshots(&ictx, &to_rebase_workspace_snapshot_address).await
            {
                error!(?error, "Eviction error: {:?}", error);
            }
            if let Err(error) =
                evict_unused_snapshots(&ictx, &to_rebase_workspace_snapshot.id().await).await
            {
                error!(?error, "Eviction error: {:?}", error);
            }
            if let Err(error) =
                evict_unused_snapshots(&ictx, &onto_workspace_snapshot.id().await).await
            {
                error!(?error, "Eviction error: {:?}", error);
            }
        });
    }

    Ok(message)
}

pub(crate) async fn evict_unused_snapshots(
    ctx: &DalContext,
    workspace_snapshot_address: &WorkspaceSnapshotAddress,
) -> RebaseResult<()> {
    if !ChangeSet::workspace_snapshot_address_in_use(ctx, workspace_snapshot_address).await? {
        ctx.layer_db()
            .workspace_snapshot()
            .evict(
                workspace_snapshot_address,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;
    }
    Ok(())
}
