//! A micro-service which processes a stream of rebaser requests for a given change set in a
//! workspace.

use std::{
    fmt,
    future::{Future, IntoFuture},
    io,
    sync::Arc,
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

use crate::{ServerError as Error, ServerMetadata, ServerResult};

use self::app_state::AppState;

pub mod app_state;
pub mod handlers;

/// A micro service which processes a stream of rebaser requests for a given change set in a
/// workspace.
pub struct ChangeSetRequestsTask {
    metadata: Arc<ServerMetadata>,
    inner: Box<dyn Future<Output = io::Result<()>> + Unpin + Send>,
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
    const NAME: &'static str = "Rebaser::ChangeSetRequestsTask";

    /// Creates and returns a runnable [`ChangeSetRequestsTask`].
    pub fn create(
        metadata: Arc<ServerMetadata>,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        incoming: jetstream::consumer::pull::Stream,
        ctx_builder: DalContextBuilder,
        shutdown_token: CancellationToken,
    ) -> Self {
        let state = AppState::new(workspace_id, change_set_id, ctx_builder);

        let app = ServiceBuilder::new()
            .concurrency_limit(1)
            .layer(
                TraceLayer::new()
                    .make_span_with(DefaultMakeSpan::new().level(Level::TRACE))
                    .on_request(DefaultOnRequest::new().level(Level::TRACE))
                    .on_response(DefaultOnResponse::new().level(Level::TRACE)),
            )
            .layer(AckLayer::new())
            .service(handlers::process_request.with_state(state));

        let inner = naxum::serve(incoming, app.into_make_service())
            .with_graceful_shutdown(naxum::wait_on_cancelled(shutdown_token.clone()));

        Self {
            metadata,
            inner: Box::new(inner.into_future()),
            shutdown_token,
        }
    }

    /// Runs the service to completion or until the first internal error is encountered.
    #[inline]
    pub async fn run(self) {
        if let Err(err) = self.try_run().await {
            error!(task = Self::NAME, error = ?err, "error while running main loop");
        }
    }

    /// Runs the service to completion, returning its result (i.e. whether it successful or an
    /// internal error was encountered).
    pub async fn try_run(self) -> ServerResult<()> {
        self.inner.await.map_err(Error::Naxum)?;
        debug!(task = Self::NAME, "main loop shutdown complete");
        Ok(())
    }
}
