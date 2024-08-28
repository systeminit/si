//! A micro-service which processes a stream of rebaser requests for a given change set in a
//! workspace.

use std::{
    fmt,
    future::{Future, IntoFuture},
    io,
    sync::Arc,
    time::Duration,
};

use dal::DalContextBuilder;
use naxum::handler::Handler;
use naxum::middleware::ack::AckLayer;
use naxum::middleware::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use naxum::ServiceExt;
use si_data_nats::async_nats::jetstream;
use si_events::{ChangeSetId, WorkspacePk};
use telemetry::prelude::*;
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;

use crate::{
    dvu_debouncer_task::DvuDebouncerTask, ServerError as Error, ServerMetadata, ServerResult,
};

use self::app_state::AppState;

pub mod app_state;
pub mod handlers;

/// A micro service which processes a stream of rebaser requests for a given change set in a
/// workspace.
pub struct ChangeSetRequestsTask {
    metadata: Arc<ServerMetadata>,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    inner: Box<dyn Future<Output = io::Result<()>> + Unpin + Send>,
    debouncer_task: DvuDebouncerTask,
    shutdown_token: CancellationToken,
}

impl fmt::Debug for ChangeSetRequestsTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChangeSetRequestsTask")
            .field("metadata", &self.metadata)
            .field("shutdown_token", &self.shutdown_token)
            .finish_non_exhaustive()
    }
}

impl ChangeSetRequestsTask {
    const NAME: &'static str = "rebaser_server::change_set_requests_task";

    /// Creates and returns a runnable [`ChangeSetRequestsTask`].
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        metadata: Arc<ServerMetadata>,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        incoming: jetstream::consumer::pull::Stream,
        kv: jetstream::kv::Store,
        ctx_builder: DalContextBuilder,
        shutdown_token: CancellationToken,
        dvu_interval: Duration,
    ) -> ServerResult<Self> {
        let debouncer_task = DvuDebouncerTask::create(
            metadata.instance_id().to_owned(),
            kv,
            workspace_id,
            change_set_id,
            ctx_builder.clone(),
            dvu_interval,
        )?;

        let state = AppState::new(workspace_id, change_set_id, ctx_builder);

        let app = ServiceBuilder::new()
            .layer(
                TraceLayer::new()
                    .make_span_with(DefaultMakeSpan::new().level(Level::TRACE))
                    .on_request(DefaultOnRequest::new().level(Level::TRACE))
                    .on_response(DefaultOnResponse::new().level(Level::TRACE)),
            )
            .layer(AckLayer::new())
            .service(handlers::process_request.with_state(state));

        let inner = naxum::serve_with_incoming_limit(incoming, app.into_make_service(), 1)
            .with_graceful_shutdown(naxum::wait_on_cancelled(shutdown_token.clone()));

        Ok(Self {
            metadata,
            workspace_id,
            change_set_id,
            inner: Box::new(inner.into_future()),
            debouncer_task,
            shutdown_token,
        })
    }

    /// Runs the service to completion or until the first internal error is encountered.
    #[inline]
    pub async fn run(self) {
        let workspace_id = self.workspace_id;
        let change_set_id = self.change_set_id;

        if let Err(err) = self.try_run().await {
            error!(
                task = Self::NAME,
                si.workspace.id = %workspace_id,
                si.change_set.id = %change_set_id,
                error = ?err,
                "error while running loop",
            );
        }
    }

    /// Runs the service to completion, returning its result (i.e. whether it successful or an
    /// internal error was encountered).
    pub async fn try_run(self) -> ServerResult<()> {
        // Spawn the task to run alongside the app and setup a drop guard for the task so that it
        // shuts down if the app errors or crashes
        let debouncer_task_drop_guard = self.debouncer_task.cancellation_token().drop_guard();
        let debouncer_task_handle = tokio::spawn(self.debouncer_task.run());

        self.inner.await.map_err(Error::Naxum)?;

        // Perform clean shutdown of task by cancelling it and awaiting its shutdown
        debouncer_task_drop_guard.disarm().cancel();
        debouncer_task_handle
            .await
            .map_err(|_err| Error::DvuDebouncerTaskJoin)?;

        debug!(
            task = Self::NAME,
            si.workspace.id = %self.workspace_id,
            si.change_set.id = %self.change_set_id,
            "shutdown complete",
        );
        Ok(())
    }
}
