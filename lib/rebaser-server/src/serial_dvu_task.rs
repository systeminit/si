use std::{result, sync::Arc};

use dal::DalContextBuilder;
use si_events::{ChangeSetId, WorkspacePk};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use crate::ServerMetadata;

#[remain::sorted]
#[derive(Debug, Error)]
pub(crate) enum SerialDvuTaskError {
    /// Error when using a DAL context
    #[error("dal context transaction error: {0}")]
    DalContext(#[from] dal::TransactionsError),
}

type Result<T> = result::Result<T, SerialDvuTaskError>;

pub(crate) struct SerialDvuTask {
    metadata: Arc<ServerMetadata>,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    ctx_builder: DalContextBuilder,
    run_notify: Arc<Notify>,
    token: CancellationToken,
}

impl SerialDvuTask {
    const NAME: &'static str = "rebaser_server::serial_dvu_task";

    pub(crate) fn create(
        metadata: Arc<ServerMetadata>,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        ctx_builder: DalContextBuilder,
        run_notify: Arc<Notify>,
        token: CancellationToken,
    ) -> Self {
        Self {
            metadata,
            workspace_id,
            change_set_id,
            ctx_builder,
            run_notify,
            token,
        }
    }

    pub(crate) async fn try_run(self) -> Result<()> {
        loop {
            tokio::select! {
                biased;

                // Signal to run a DVU has fired
                _ = self.run_notify.notified() => {
                    debug!(
                        task = Self::NAME,
                        service.instance.id = self.metadata.instance_id(),
                        si.workspace.id = %self.workspace_id,
                        si.change_set.id = %self.change_set_id,
                        "notified, preparing dvu run",
                    );
                    self.run_dvu().await?;
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
                    break;
                }
            }
        }

        debug!(
            task = Self::NAME,
            service.instance.id = self.metadata.instance_id(),
            si.workspace.id = %self.workspace_id,
            si.change_set.id = %self.change_set_id,
            "shutdown complete",
        );
        Ok(())
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
        let builder = self.ctx_builder.clone();
        let ctx = builder
            .build_for_change_set_as_system(self.workspace_id, self.change_set_id, None)
            .await?;

        ctx.enqueue_dependent_values_update().await?;
        ctx.blocking_commit_no_rebase().await?;

        Ok(())
    }
}
