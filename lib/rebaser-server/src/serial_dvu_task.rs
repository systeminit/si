use std::{result, sync::Arc};

use dal::{ChangeSet, ChangeSetError, ChangeSetStatus, DalContextBuilder};
use si_events::{ChangeSetId, WorkspacePk};
use telemetry::prelude::*;
use telemetry_utils::metric;
use thiserror::Error;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use crate::{ServerMetadata, Shutdown};

#[remain::sorted]
#[derive(Debug, Error)]
pub(crate) enum SerialDvuTaskError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    /// Error when using a DAL context
    #[error("dal context transaction error: {0}")]
    DalContext(#[from] dal::TransactionsError),
    /// When failing to do an operation using the [`WorkspaceSnapshot`]
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
}

type Result<T> = result::Result<T, SerialDvuTaskError>;

pub(crate) struct SerialDvuTask {
    metadata: Arc<ServerMetadata>,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    ctx_builder: DalContextBuilder,
    run_notify: Arc<Notify>,
    quiesced_notify: Arc<Notify>,
    quiesced_token: CancellationToken,
    token: CancellationToken,
}

impl SerialDvuTask {
    const NAME: &'static str = "rebaser_server::serial_dvu_task";

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create(
        metadata: Arc<ServerMetadata>,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        ctx_builder: DalContextBuilder,
        run_notify: Arc<Notify>,
        quiesced_notify: Arc<Notify>,
        quiesced_token: CancellationToken,
        token: CancellationToken,
    ) -> Self {
        Self {
            metadata,
            workspace_id,
            change_set_id,
            ctx_builder,
            run_notify,
            quiesced_notify,
            quiesced_token,
            token,
        }
    }

    pub(crate) async fn try_run(self) -> Result<Shutdown> {
        metric!(counter.serial_dvu_task.change_set_in_progress = 1);

        let shutdown_cause = loop {
            tokio::select! {
                biased;

                // Signal to run a DVU has fired
                _ = self.run_notify.notified() => {
                    info!(
                        task = Self::NAME,
                        service.instance.id = self.metadata.instance_id(),
                        si.workspace.id = %self.workspace_id,
                        si.change_set.id = %self.change_set_id,
                        "notified, preparing dvu run",
                    );
                    self.run_dvu().await?;
                }
                // Signal to shutdown from a quiet period has fired
                _ = self.quiesced_notify.notified() => {
                    info!(
                        task = Self::NAME,
                        service.instance.id = self.metadata.instance_id(),
                        si.workspace.id = %self.workspace_id,
                        si.change_set.id = %self.change_set_id,
                        "quiesced notified, starting to shut down",
                    );

                    // Fire the quiesced_token so that the processing task immediately stops
                    // processing additional requests
                    self.quiesced_token.cancel();

                    break Shutdown::Quiesced;
                }
                // Cancellation token has fired, time to shut down
                _ = self.token.cancelled() => {
                    info!(
                        task = Self::NAME,
                        service.instance.id = self.metadata.instance_id(),
                        si.workspace.id = %self.workspace_id,
                        si.change_set.id = %self.change_set_id,
                        "received cancellation, shutting down",
                    );
                    break Shutdown::Graceful;
                }
            }
        };

        info!(
            task = Self::NAME,
            cause = ?shutdown_cause,
            service.instance.id = self.metadata.instance_id(),
            si.workspace.id = %self.workspace_id,
            si.change_set.id = %self.change_set_id,
            "shutdown complete",
        );
        metric!(counter.serial_dvu_task.change_set_in_progress = -1);

        Ok(shutdown_cause)
    }

    #[instrument(
        name = "serial_dvu_task.run_dvu",
        level = "info",
        skip_all,
        fields(
            service.instance.id = self.metadata.instance_id(),
            si.change_set.id = %self.change_set_id,
            si.workspace.id = %self.workspace_id,
        ),
    )]
    async fn run_dvu(&self) -> Result<()> {
        metric!(counter.serial_dvu_task.dvu_running = 1);

        let builder = self.ctx_builder.clone();
        let ctx = builder
            .build_for_change_set_as_system(self.workspace_id, self.change_set_id, None)
            .await?;
        let change_set = ChangeSet::get_by_id(&ctx, ctx.change_set_id()).await?;
        if change_set.status == ChangeSetStatus::Abandoned {
            warn!("Trying to enqueue DVU for abandoned change set. Returning early.");
            return Ok(());
        }

        ctx.enqueue_dependent_values_update().await?;
        ctx.blocking_commit_no_rebase().await?;
        metric!(counter.serial_dvu_task.dvu_running = -1);

        Ok(())
    }
}
