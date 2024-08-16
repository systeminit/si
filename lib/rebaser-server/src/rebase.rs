use dal::change_set::{ChangeSet, ChangeSetError, ChangeSetId};
use dal::workspace_snapshot::WorkspaceSnapshotError;
use dal::{
    DalContext, TransactionsError, Workspace, WorkspaceError, WorkspacePk, WorkspaceSnapshot,
    WsEvent, WsEventError,
};
use si_events::rebase_batch_address::RebaseBatchAddress;
use si_events::WorkspaceSnapshotAddress;
use si_layer_cache::activities::rebase::RebaseStatus;
use si_layer_cache::activities::ActivityRebaseRequest;
use si_layer_cache::event::LayeredEventMetadata;
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
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("workspace pk expected but was none")]
    WorkspacePkExpected,
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

    let corrected_updates = to_rebase_workspace_snapshot
        .correct_transforms(
            rebase_batch.updates().to_vec(),
            message
                .payload
                .from_change_set_id
                .is_some_and(|from_id| from_id != to_rebase_change_set.id.into()),
        )
        .await?;
    debug!("corrected transforms: {:?}", start.elapsed());

    to_rebase_workspace_snapshot
        .perform_updates(&corrected_updates)
        .await?;

    debug!("updates complete: {:?}", start.elapsed());

    if !corrected_updates.is_empty() {
        // Once all updates have been performed, we can write out, mark everything as recently seen
        // and update the pointer.
        to_rebase_workspace_snapshot.write(ctx).await?;
        debug!("snapshot written: {:?}", start.elapsed());
        to_rebase_change_set
            .update_pointer(ctx, to_rebase_workspace_snapshot.id().await)
            .await?;

        debug!("pointer updated: {:?}", start.elapsed());
    }
    let updates_count = rebase_batch.updates().len();
    span.record("si.updates.count", updates_count.to_string());

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

    if let Some(workspace) = Workspace::get_by_pk(
        ctx,
        &ctx.tenancy()
            .workspace_pk()
            .ok_or(RebaseError::WorkspacePkExpected)?,
    )
    .await?
    {
        if workspace.default_change_set_id() == to_rebase_change_set.id
            && *workspace.pk() != WorkspacePk::NONE
        {
            let all_open_change_sets = ChangeSet::list_open(ctx).await?;
            for target_change_set in all_open_change_sets.into_iter().filter(|cs| {
                cs.id != workspace.default_change_set_id() && cs.id != to_rebase_change_set.id
            }) {
                let ctx_clone = ctx.clone();
                let rebase_batch_address = message.payload.rebase_batch_address;
                tokio::task::spawn(async move {
                    debug!(
                        "replaying batch {} onto {} from {}",
                        rebase_batch_address, target_change_set.id, to_rebase_change_set.id
                    );

                    if let Err(err) = replay_changes(
                        &ctx_clone,
                        to_rebase_change_set.id,
                        target_change_set.id,
                        rebase_batch_address,
                    )
                    .await
                    {
                        error!(
                            err = ?err,
                            "error replaying rebase batch {} changes onto {}",
                            rebase_batch_address,
                            target_change_set.id
                        );
                    }
                });
            }
        }
    }

    if let Some(source_change_set_id) = message.payload.from_change_set_id {
        let mut event = WsEvent::change_set_applied(
            ctx,
            source_change_set_id.into(),
            message.payload.to_rebase_change_set_id.into(),
            None,
        )
        .await?;
        event.set_workspace_pk(message.metadata.tenancy.workspace_pk.into_raw_id().into());
        event.set_change_set_id(Some(message.payload.to_rebase_change_set_id.into()));
        event.publish_immediately(ctx).await?;
    }

    Ok(RebaseStatus::Success {
        updates_performed: message.payload.rebase_batch_address,
    })
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

async fn replay_changes(
    ctx: &DalContext,
    current_change_set_id: ChangeSetId,
    target_change_set_id: ChangeSetId,
    rebase_batch_address: RebaseBatchAddress,
) -> RebaseResult<()> {
    let metadata = LayeredEventMetadata::new(
        si_events::Tenancy::new(
            ctx.tenancy()
                .workspace_pk()
                .unwrap_or(WorkspacePk::NONE)
                .into(),
            target_change_set_id.into(),
        ),
        si_events::Actor::System,
    );

    ctx.layer_db()
        .activity()
        .rebase()
        .rebase_from_change_set(
            target_change_set_id.into(),
            rebase_batch_address,
            current_change_set_id.into(),
            metadata,
        )
        .await?;

    Ok(())
}
