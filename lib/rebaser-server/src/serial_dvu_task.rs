use std::{result, sync::Arc};

use dal::DalContextBuilder;
use si_events::{ChangeSetId, WorkspacePk};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use crate::{ServerMetadata, Shutdown};

#[remain::sorted]
#[derive(Debug, Error)]
pub(crate) enum SerialDvuTaskError {
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
    run_dvu_notify: Arc<Notify>,
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
        run_dvu_notify: Arc<Notify>,
        quiesced_notify: Arc<Notify>,
        quiesced_token: CancellationToken,
        token: CancellationToken,
    ) -> Self {
        Self {
            metadata,
            workspace_id,
            change_set_id,
            ctx_builder,
            run_dvu_notify,
            quiesced_notify,
            quiesced_token,
            token,
        }
    }

    pub(crate) async fn try_run(self) -> Result<Shutdown> {
        // Attempt to run an initial dvu in case there are processed requests that haven't yet been
        // finished with a dvu run
        self.maybe_run_initial_dvu().await?;

        let shutdown_cause = loop {
            tokio::select! {
                biased;

                // Signal to run a DVU has fired
                _ = self.run_dvu_notify.notified() => {
                    debug!(
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
                    debug!(
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
                    debug!(
                        task = Self::NAME,
                        service.instance.id = self.metadata.instance_id(),
                        si.workspace.id = %self.workspace_id,
                        si.change_set.id = %self.change_set_id,
                        "received cancellation",
                    );

                    break Shutdown::Graceful;
                }
            }
        };

        debug!(
            task = Self::NAME,
            cause = ?shutdown_cause,
            service.instance.id = self.metadata.instance_id(),
            si.workspace.id = %self.workspace_id,
            si.change_set.id = %self.change_set_id,
            "shutdown complete",
        );
        Ok(shutdown_cause)
    }

    async fn run_dvu(&self) -> Result<()> {
        let builder = self.ctx_builder.clone();
        let ctx = builder
            .build_for_change_set_as_system(self.workspace_id.into(), self.change_set_id.into())
            .await?;

        info!(
            task = Self::NAME,
            service.instance.id = self.metadata.instance_id(),
            si.workspace.id = %self.workspace_id,
            si.change_set.id = %self.change_set_id,
            "enqueuing dependent_values_update",
        );
        ctx.enqueue_dependent_values_update().await?;
        ctx.blocking_commit_no_rebase().await?;

        Ok(())
    }

    async fn maybe_run_initial_dvu(&self) -> Result<()> {
        let builder = self.ctx_builder.clone();
        let ctx = builder
            .build_for_change_set_as_system(self.workspace_id.into(), self.change_set_id.into())
            .await?;

        if ctx
            .workspace_snapshot()?
            .has_dependent_value_roots()
            .await?
        {
            info!(
                task = Self::NAME,
                service.instance.id = self.metadata.instance_id(),
                si.workspace.id = %self.workspace_id,
                si.change_set.id = %self.change_set_id,
                "enqueuing *initial* dependent_values_update",
            );
            ctx.enqueue_dependent_values_update().await?;
            ctx.blocking_commit_no_rebase().await?;
        }

        Ok(())
    }
}
