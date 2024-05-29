//! A per-changeset task to debounce dependent values updates

use dal::{
    ChangeSet, ChangeSetError, ChangeSetStatus, DalContextBuilder, Tenancy, TransactionsError,
    Visibility, WorkspaceSnapshotError,
};
use si_events::{ChangeSetId, WorkspacePk};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    select,
    time::{interval, Duration},
};
use tokio_util::sync::CancellationToken;

/// DvuDebouncer error type
#[derive(Error, Debug)]
pub enum DvuDebouncerError {
    /// A Change set error
    #[error("change set: {0}")]
    ChangeSet(#[from] ChangeSetError),
    /// A transactions error
    #[error("Transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    /// Workspace Snapshot Error
    #[error("workspace snapshot: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

/// DvuDebouncer result type
type DvuDebouncerResult<T> = Result<T, DvuDebouncerError>;

/// The DVU debouncer
#[derive(Clone, Debug)]
pub struct DvuDebouncer {
    cancellation_token: CancellationToken,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    ctx_builder: DalContextBuilder,
    dvu_interval: Duration,
}

impl DvuDebouncer {
    /// Create a new dvu debouncer task
    pub fn new(
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        cancellation_token: CancellationToken,
        ctx_builder: DalContextBuilder,
        dvu_interval: Duration,
    ) -> Self {
        let debouncer = Self {
            cancellation_token,
            workspace_id,
            change_set_id,
            ctx_builder,
            dvu_interval,
        };

        let debouncer_clone = debouncer.clone();

        tokio::task::spawn(async { ticker(debouncer_clone).await });

        debouncer
    }

    async fn run_dvu_if_values_pending(&self) -> DvuDebouncerResult<()> {
        let mut builder = self.ctx_builder.clone();
        // We shouldn't need to migrate any snapshots in the dvu debouncer. SDF
        // should do that when it reads the snapshot. Once rewritten, the
        // snapshot will already be migrated when we get here.
        builder.set_no_auto_migrate_snapshots();
        let mut ctx = builder.build_default().await?;

        ctx.update_visibility_deprecated(Visibility::new(self.change_set_id.into_inner().into()));
        ctx.update_tenancy(Tenancy::new(self.workspace_id.into_inner().into()));

        if let Some(change_set) =
            ChangeSet::find(&ctx, self.change_set_id.into_inner().into()).await?
        {
            if !matches!(
                change_set.status,
                ChangeSetStatus::Open
                    | ChangeSetStatus::NeedsApproval
                    | ChangeSetStatus::NeedsAbandonApproval
            ) {
                debug!("change set no longer open, not enqueuing dependent values updates");
                return Ok(());
            }
        }

        if let Err(err) = ctx.update_snapshot_to_visibility().await {
            if err.is_unmigrated_snapshot_error() {
                debug!("Snapshot not yet migrated. Not attempting dvu");
            } else {
                Err(err)?
            }
        }

        if ctx
            .workspace_snapshot()?
            .has_dependent_value_roots()
            .await?
        {
            info!(
                "enqueuing dependent_values_update for {}",
                self.change_set_id
            );
            ctx.enqueue_dependent_values_update().await?;
            ctx.blocking_commit_no_rebase().await?;
        }

        Ok(())
    }
}

async fn ticker(debouncer: DvuDebouncer) {
    info!(
        "booting dvu task for {} with a {:?} interval",
        &debouncer.change_set_id, debouncer.dvu_interval
    );

    let mut ticker = interval(debouncer.dvu_interval);

    loop {
        select! {
            _ = debouncer.cancellation_token.cancelled() => {
                info!("DVU debouncer: value task received cancellation message");

                return;
            }
            _ = ticker.tick() => {
                // This will block, in which case we'll just run again on the next tick
                if let Err(err) = debouncer.run_dvu_if_values_pending().await {
                    error!(error = ?err, "Attempt to run dependent values update job failed for changeset {}", debouncer.change_set_id);
                }
            }
        }
    }
}
