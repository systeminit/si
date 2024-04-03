use dal::change_set::{ChangeSet, ChangeSetError, ChangeSetId};
use dal::workspace_snapshot::vector_clock::VectorClockId;
use dal::workspace_snapshot::WorkspaceSnapshotError;
use dal::{
    DalContext, Tenancy, TransactionsError, Visibility, WorkspacePk, WorkspaceSnapshot, WsEvent,
    WsEventError,
};
use si_layer_cache::activities::rebase::RebaseStatus;
use si_layer_cache::activities::AckRebaseRequest;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::time::Instant;
use ulid::Ulid;

#[remain::sorted]
#[derive(Debug, Error)]
pub(crate) enum RebaseError {
    #[error("workspace snapshot error: {0}")]
    ChangeSet(#[from] ChangeSetError),
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

pub(crate) async fn perform_rebase(
    ctx: &mut DalContext,
    message: &AckRebaseRequest,
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
    info!("before snapshot fetch and parse: {:?}", start.elapsed());
    let to_rebase_workspace_snapshot =
        WorkspaceSnapshot::find(ctx, to_rebase_workspace_snapshot_address).await?;
    let onto_workspace_snapshot: WorkspaceSnapshot =
        WorkspaceSnapshot::find(ctx, message.payload.onto_workspace_snapshot_address).await?;
    info!(
        "to_rebase_id: {}, onto_id: {}",
        to_rebase_workspace_snapshot_address,
        onto_workspace_snapshot.id().await
    );
    info!("after snapshot fetch and parse: {:?}", start.elapsed());

    // Let NATS know we are still working
    let _ = message
        .ack_with(si_layer_cache::activities::AckKind::Progress)
        .await;

    // Perform the conflicts and updates detection.
    let onto_vector_clock_id: VectorClockId = message.payload.onto_vector_clock_id.into();
    let (conflicts, updates) = to_rebase_workspace_snapshot
        .detect_conflicts_and_updates(
            to_rebase_change_set.vector_clock_id(),
            &onto_workspace_snapshot,
            onto_vector_clock_id,
        )
        .await?;
    info!(
        "count: conflicts ({}) and updates ({}), {:?}",
        conflicts.len(),
        updates.len(),
        start.elapsed()
    );

    // If there are conflicts, immediately assemble a reply message that conflicts were found.
    // Otherwise, we can perform updates and assemble a "success" reply message.
    let message: RebaseStatus = if conflicts.is_empty() {
        // TODO(nick): store the offset with the change set.
        to_rebase_workspace_snapshot
            .perform_updates(
                &to_rebase_change_set,
                &onto_workspace_snapshot,
                updates.as_slice(),
            )
            .await?;
        info!("updates complete: {:?}", start.elapsed());

        if !updates.is_empty() {
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
            updates_performed: serde_json::to_value(updates)?.to_string(),
        }
    } else {
        RebaseStatus::ConflictsFound {
            conflicts_found: serde_json::to_value(conflicts)?.to_string(),
            updates_found_and_skipped: serde_json::to_value(updates)?.to_string(),
        }
    };

    info!("rebase performed: {:?}", start.elapsed());

    // Before replying to the requester, we must commit.
    ctx.commit_no_rebase().await?;

    let change_set_ulid: Ulid = to_rebase_change_set.id.into();

    let to_rebase_ctx = ctx
        .clone_with_new_visibility(Visibility::new(change_set_ulid.into()))
        .clone_with_new_tenancy(Tenancy::new(
            to_rebase_change_set
                .workspace_id
                .unwrap_or(WorkspacePk::NONE),
        ));

    WsEvent::change_set_written(&to_rebase_ctx)
        .await?
        .publish_immediately(&to_rebase_ctx)
        .await?;

    Ok(message)
}
