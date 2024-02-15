use dal::change_set_pointer::{ChangeSetPointer, ChangeSetPointerError, ChangeSetPointerId};
use dal::workspace_snapshot::vector_clock::VectorClockId;
use dal::workspace_snapshot::WorkspaceSnapshotError;
use dal::{DalContext, TransactionsError, WorkspaceSnapshot};
use rebaser_core::{ReplyRebaseMessage, RequestRebaseMessage};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::time::Instant;

#[remain::sorted]
#[derive(Debug, Error)]
pub(crate) enum RebaseError {
    #[error("workspace snapshot error: {0}")]
    ChangeSetPointer(#[from] ChangeSetPointerError),
    #[error("missing change set pointer")]
    MissingChangeSetPointer(ChangeSetPointerId),
    #[error("missing workspace snapshot for change set ({0}) (the change set likely isn't pointing at a workspace snapshot)")]
    MissingWorkspaceSnapshotForChangeSet(ChangeSetPointerId),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

type RebaseResult<T> = Result<T, RebaseError>;

pub(crate) async fn perform_rebase(
    ctx: &mut DalContext,
    message: RequestRebaseMessage,
) -> RebaseResult<ReplyRebaseMessage> {
    let start = Instant::now();
    // Gather everything we need to detect conflicts and updates from the inbound message.
    let mut to_rebase_change_set =
        ChangeSetPointer::find(ctx, message.to_rebase_change_set_id.into())
            .await?
            .ok_or(RebaseError::MissingChangeSetPointer(
                message.to_rebase_change_set_id.into(),
            ))?;
    let to_rebase_workspace_snapshot_id = to_rebase_change_set.workspace_snapshot_id.ok_or(
        RebaseError::MissingWorkspaceSnapshotForChangeSet(to_rebase_change_set.id),
    )?;
    info!("before snapshot fetch and parse: {:?}", start.elapsed());
    let mut to_rebase_workspace_snapshot =
        WorkspaceSnapshot::find(ctx, to_rebase_workspace_snapshot_id).await?;
    let mut onto_workspace_snapshot: WorkspaceSnapshot =
        WorkspaceSnapshot::find(ctx, message.onto_workspace_snapshot_id.into()).await?;
    info!(
        "to_rebase_id: {}, onto_id: {}",
        to_rebase_workspace_snapshot_id,
        onto_workspace_snapshot.id()
    );
    info!("after snapshot fetch and parse: {:?}", start.elapsed());

    // Perform the conflicts and updates detection.
    let onto_vector_clock_id: VectorClockId = message.onto_vector_clock_id.into();
    let (conflicts, updates) = to_rebase_workspace_snapshot
        .detect_conflicts_and_updates(
            to_rebase_change_set.vector_clock_id(),
            &mut onto_workspace_snapshot,
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
    let message: ReplyRebaseMessage = if conflicts.is_empty() {
        // TODO(nick): store the offset with the change set.
        to_rebase_workspace_snapshot.perform_updates(
            &to_rebase_change_set,
            &mut onto_workspace_snapshot,
            updates.as_slice(),
        )?;
        info!("updates complete: {:?}", start.elapsed());

        if !updates.is_empty() {
            // Once all updates have been performed, we can write out, mark everything as recently seen
            // and update the pointer.
            to_rebase_workspace_snapshot
                .write(ctx, to_rebase_change_set.vector_clock_id())
                .await?;
            info!("snapshot written: {:?}", start.elapsed());
            to_rebase_change_set
                .update_pointer(ctx, to_rebase_workspace_snapshot.id())
                .await?;
            info!("pointer updated: {:?}", start.elapsed());
        }

        ReplyRebaseMessage::Success {
            updates_performed: serde_json::to_value(updates)?,
        }
    } else {
        ReplyRebaseMessage::ConflictsFound {
            conflicts_found: serde_json::to_value(conflicts)?,
            updates_found_and_skipped: serde_json::to_value(updates)?,
        }
    };

    info!("rebase performed: {:?}", start.elapsed());

    // Before replying to the requester, we must commit.
    ctx.commit_no_rebase().await?;

    Ok(message)
}
