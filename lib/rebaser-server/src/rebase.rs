use dal::change_set::{ChangeSet, ChangeSetError, ChangeSetId};
use dal::workspace_snapshot::graph::ConflictsAndUpdates;
use dal::workspace_snapshot::vector_clock::VectorClockId;
use dal::workspace_snapshot::WorkspaceSnapshotError;
use dal::{DalContext, TransactionsError, WorkspacePk, WorkspaceSnapshot, WsEventError};
use si_events::rebase_batch_address::RebaseBatchAddress;
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
    #[error("missing rebase batch {0}")]
    MissingRebaseBatch(RebaseBatchAddress),
    #[error("to_rebase snapshot has no recently seen vector clock for its change set {0}")]
    MissingVectorClockForChangeSet(ChangeSetId),
    #[error("snapshot has no recently seen vector clock for any change set")]
    MissingVectorClockForSnapshot,
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

#[instrument(name = "rebase.perform_rebase", level = "info", skip_all, fields(
    si.change_set.id = Empty,
    si.workspace.id = Empty,
    si.conflicts = Empty,
    si.updates = Empty,
    si.conflicts.count = Empty,
    si.updates.count = Empty,
))]
pub async fn perform_rebase(
    ctx: &mut DalContext,
    message: &ActivityRebaseRequest,
) -> RebaseResult<RebaseStatus> {
    let span = Span::current();
    span.record(
        "si.change_set.id",
        &message.metadata.tenancy.change_set_id.to_string(),
    );
    span.record(
        "si.workspace.id",
        &message.metadata.tenancy.workspace_pk.to_string(),
    );
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

    let rebase_batch = ctx
        .layer_db()
        .rebase_batch()
        .read_wait_for_memory(&message.payload.rebase_batch_address)
        .await?
        .ok_or(RebaseError::MissingRebaseBatch(
            message.payload.rebase_batch_address,
        ))?;

    debug!(
        "to_rebase_address: {}, rebase_batch_address: {}",
        to_rebase_workspace_snapshot_address, message.payload.rebase_batch_address
    );
    debug!("after snapshot fetch and parse: {:?}", start.elapsed());

    // Choose the most recent vector clock for the to_rebase change set for conflict detection
    let to_rebase_vector_clock_id = to_rebase_workspace_snapshot
        .max_recently_seen_clock_id(Some(to_rebase_change_set.id))
        .await?
        .ok_or(RebaseError::MissingVectorClockForChangeSet(
            to_rebase_change_set.id,
        ))?;

    let conflicts_and_updates = ConflictsAndUpdates {
        ..Default::default()
    };

    // If there are conflicts, immediately assemble a reply message that conflicts were found.
    // Otherwise, we can perform updates and assemble a "success" reply message.
    let message: RebaseStatus = if conflicts_and_updates.conflicts.is_empty() {
        to_rebase_workspace_snapshot
            .perform_updates(to_rebase_vector_clock_id, rebase_batch.updates())
            .await?;

        debug!("updates complete: {:?}", start.elapsed());

        if !rebase_batch.updates().is_empty() {
            // Once all updates have been performed, we can write out, mark everything as recently seen
            // and update the pointer.
            let workspace_pk = ctx.tenancy().workspace_pk().unwrap_or(WorkspacePk::NONE);
            let vector_clock_id = VectorClockId::new(
                to_rebase_change_set.id.into_inner(),
                workspace_pk.into_inner(),
            );

            to_rebase_workspace_snapshot
                .collapse_vector_clocks(ctx)
                .await?;

            to_rebase_workspace_snapshot
                .write(ctx, vector_clock_id)
                .await?;
            debug!("snapshot written: {:?}", start.elapsed());
            to_rebase_change_set
                .update_pointer(ctx, to_rebase_workspace_snapshot.id().await)
                .await?;

            debug!("pointer updated: {:?}", start.elapsed());
        }
        let updates_count = rebase_batch.updates().len();
        //let updates_performed = serde_json::to_value(rebase_batch.updates())?.to_string();

        //span.record("si.updates", updates_performed.clone());
        span.record("si.updates.count", updates_count.to_string());
        RebaseStatus::Success {
            updates_performed: message.payload.rebase_batch_address,
        }
    } else {
        let conflicts_count = conflicts_and_updates.conflicts.len();
        let conflicts_found = serde_json::to_value(conflicts_and_updates.conflicts)?.to_string();
        span.record("si.conflicts", conflicts_found.clone());
        span.record("si.conflicts.count", conflicts_count.to_string());
        RebaseStatus::ConflictsFound {
            conflicts_found,
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
            // TODO: RebaseBatch eviction?
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
