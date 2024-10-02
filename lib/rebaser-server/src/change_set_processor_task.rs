use std::{
    future::{Future, IntoFuture},
    io, result,
    sync::Arc,
    time::Duration,
};

use dal::DalContextBuilder;
use futures::{future::BoxFuture, TryStreamExt};
use naxum::{
    handler::Handler as _,
    middleware::{
        post_process::{self, PostProcessLayer},
        trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    },
    ServiceExt as _,
};
use si_data_nats::{
    async_nats::jetstream::{self, consumer::push},
    NatsClient,
};
use si_events::{ChangeSetId, WorkspacePk};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::Notify;
use tokio_stream::StreamExt as _;
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;

use self::app_state::AppState;
use crate::ServerMetadata;

#[remain::sorted]
#[derive(Debug, Error)]
pub(crate) enum ChangeSetProcessorTaskError {
    /// When a naxum-based service encounters an I/O error
    #[error("naxum error: {0}")]
    Naxum(#[source] std::io::Error),
}

type Error = ChangeSetProcessorTaskError;

type Result<T> = result::Result<T, ChangeSetProcessorTaskError>;

pub(crate) struct ChangeSetProcessorTask {
    _metadata: Arc<ServerMetadata>,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    inner: Box<dyn Future<Output = io::Result<()>> + Unpin + Send>,
}

impl ChangeSetProcessorTask {
    const NAME: &'static str = "rebaser_server::change_set_processor_task";

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create(
        metadata: Arc<ServerMetadata>,
        nats: NatsClient,
        stream: jetstream::stream::Stream,
        incoming: push::Ordered,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        ctx_builder: DalContextBuilder,
        run_dvu_notify: Arc<Notify>,
        quiescent_period: Duration,
        quiesced_notify: Arc<Notify>,
        quiesced_token: CancellationToken,
        task_token: CancellationToken,
    ) -> Self {
        let state = AppState::new(
            workspace_id,
            change_set_id,
            nats,
            ctx_builder,
            run_dvu_notify,
        );

        let captured = QuiescedCaptured {
            instance_id: metadata.instance_id().to_string(),
            workspace_id,
            change_set_id,
            quiesced_notify: quiesced_notify.clone(),
        };

        let inactive_aware_incoming = incoming
            // Looks for a gap between incoming messages greater than the duration
            .timeout(quiescent_period)
            // Fire quiesced_notify which triggers a specific shutdown of the serial dvu task where
            // we *know* we want to remove the task from the set of work.
            .inspect_err(move |_elapsed| {
                let QuiescedCaptured {
                    instance_id,
                    workspace_id,
                    change_set_id,
                    quiesced_notify,
                } = &captured;

                debug!(
                    service.instance.id = instance_id,
                    si.workspace.id = %workspace_id,
                    si.change_set.id = %change_set_id,
                    "rate of requests has become inactive, triggering a quiesced shutdown",
                );
                // Notify the serial dvu task that we want to shutdown due to a quiet period
                quiesced_notify.notify_one();
            })
            // Continue processing messages as normal until the Naxum app's graceful shutdown is
            // triggered. This means we turn the stream back from a stream of
            // `Result<Result<Message, _>, Elapsed>` into `Result<Message, _>`
            .filter_map(|maybe_elapsed_item| maybe_elapsed_item.ok());

        let app = ServiceBuilder::new()
            .layer(
                TraceLayer::new()
                    .make_span_with(DefaultMakeSpan::new().level(Level::TRACE))
                    .on_request(DefaultOnRequest::new().level(Level::TRACE))
                    .on_response(DefaultOnResponse::new().level(Level::TRACE)),
            )
            .layer(PostProcessLayer::new().on_success(DeleteMessageOnSuccess::new(stream)))
            .service(handlers::default.with_state(state));

        let inner =
            naxum::serve_with_incoming_limit(inactive_aware_incoming, app.into_make_service(), 1)
                .with_graceful_shutdown(graceful_shutdown_signal(task_token, quiesced_token));

        let inner_fut = inner.into_future();

        Self {
            _metadata: metadata,
            workspace_id,
            change_set_id,
            inner: Box::new(inner_fut),
        }
    }

    pub(crate) async fn try_run(self) -> Result<()> {
        self.inner.await.map_err(Error::Naxum)?;

        debug!(
            task = Self::NAME,
            si.workspace.id = %self.workspace_id,
            si.change_set.id = %self.change_set_id,
            "shutdown complete",
        );
        Ok(())
    }
}

struct QuiescedCaptured {
    instance_id: String,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    quiesced_notify: Arc<Notify>,
}

#[derive(Clone, Debug)]
struct DeleteMessageOnSuccess {
    stream: jetstream::stream::Stream,
}

impl DeleteMessageOnSuccess {
    fn new(stream: jetstream::stream::Stream) -> Self {
        Self { stream }
    }
}

impl post_process::OnSuccess for DeleteMessageOnSuccess {
    fn call(
        &mut self,
        head: Arc<naxum::Head>,
        info: Arc<post_process::Info>,
    ) -> BoxFuture<'static, ()> {
        let stream = self.stream.clone();

        Box::pin(async move {
            trace!("deleting message on success");
            if let Err(err) = stream.delete_message(info.stream_sequence).await {
                warn!(
                    si.error.message = ?err,
                    subject = head.subject.as_str(),
                    "failed to delete the message",
                );
            }
        })
    }
}

// Await either a graceful shutdown signal from the task or an inactive request stream trigger.
async fn graceful_shutdown_signal(
    task_token: CancellationToken,
    quiescence_token: CancellationToken,
) {
    tokio::select! {
        _ = task_token.cancelled() => {}
        _ = quiescence_token.cancelled() => {}
    }
}

mod handlers {
    use std::result;

    use dal::{ChangeSet, Workspace, WorkspaceSnapshot, WsEvent};
    use naxum::{
        extract::State,
        response::{IntoResponse, Response},
    };
    use rebaser_core::{
        api_types::{
            enqueue_updates_request::EnqueueUpdatesRequest,
            enqueue_updates_response::{
                v1::RebaseStatus, EnqueueUpdatesResponse, EnqueueUpdatesResponseVCurrent,
            },
            ApiWrapper, SerializeError,
        },
        ContentInfo,
    };
    use si_data_nats::HeaderMap;
    use telemetry::prelude::*;
    use thiserror::Error;

    use crate::{
        extract::{ApiTypesNegotiate, HeaderReply},
        rebase::{perform_rebase, RebaseError},
    };

    use super::app_state::AppState;

    #[remain::sorted]
    #[derive(Debug, Error)]
    pub(crate) enum HandlerError {
        /// Failures related to ChangeSets
        #[error("Change set error: {0}")]
        ChangeSet(#[from] dal::ChangeSetError),
        #[error("compute executor error: {0}")]
        ComputeExecutor(#[from] dal::DedicatedExecutorError),
        /// When failing to create a DAL context
        #[error("error creating a dal ctx: {0}")]
        DalTransactions(#[from] dal::TransactionsError),
        #[error("error publishing reply: {0}")]
        PublishReply(#[source] si_data_nats::Error),
        /// Failures related to rebasing/updating a snapshot or change set pointer.
        #[error("rebase error: {0}")]
        Rebase(#[from] RebaseError),
        #[error("error serializing: {0}")]
        Serialize(#[from] SerializeError),
        /// When failing to find the workspace
        #[error("workspace error: {0}")]
        Workspace(#[from] dal::WorkspaceError),
        /// When failing to do an operation using the [`WorkspaceSnapshot`]
        #[error("workspace snapshot error: {0}")]
        WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
        /// When failing to send a [`WsEvent`]
        #[error("failed to construct ws event: {0}")]
        WsEvent(#[from] dal::WsEventError),
    }

    type Error = HandlerError;

    type Result<T> = result::Result<T, HandlerError>;

    impl IntoResponse for HandlerError {
        fn into_response(self) -> Response {
            // TODO(fnichol): there are different responses, esp. for expected interrupted
            error!(si.error.message = ?self, "failed to process message");
            Response::internal_server_error()
        }
    }

    pub(crate) async fn default(
        State(state): State<AppState>,
        HeaderReply(maybe_reply): HeaderReply,
        ApiTypesNegotiate(request): ApiTypesNegotiate<EnqueueUpdatesRequest>,
    ) -> Result<()> {
        let AppState {
            workspace_id,
            change_set_id,
            nats,
            ctx_builder,
            run_notify,
        } = state;
        let mut ctx = ctx_builder
            .build_for_change_set_as_system(workspace_id.into(), change_set_id.into())
            .await?;

        let rebase_status = perform_rebase(&mut ctx, &request)
            .await
            .unwrap_or_else(|err| {
                error!(
                    si.error.message = ?err,
                    ?request,
                    "performing rebase failed, attempting to reply",
                );
                RebaseStatus::Error {
                    message: err.to_string(),
                }
            });

        // Dispatch eligible actions if the change set is the default for the workspace.
        // Actions are **ONLY** ever dispatched from the default change set for a workspace.
        if matches!(rebase_status, RebaseStatus::Success { .. }) {
            // If we find dependent value roots, then notify the serial dvu task to run at least
            // one more dvu
            if ctx
                .workspace_snapshot()?
                .has_dependent_value_roots()
                .await?
            {
                run_notify.notify_one();
            }

            if let Some(workspace) = Workspace::get_by_pk(&ctx, &workspace_id.into()).await? {
                if workspace.default_change_set_id() == ctx.visibility().change_set_id {
                    let mut change_set = ChangeSet::find(&ctx, ctx.visibility().change_set_id)
                        .await?
                        .ok_or(RebaseError::MissingChangeSet(change_set_id.into()))?;
                    if WorkspaceSnapshot::dispatch_actions(&ctx).await? {
                        // Write out the snapshot to get the new address/id.
                        let new_snapshot_id = ctx
                            .write_snapshot()
                            .await?
                            .ok_or(dal::WorkspaceSnapshotError::WorkspaceSnapshotNotWritten)?;
                        // Manually update the pointer to the new address/id that reflects the new
                        // Action states.
                        change_set.update_pointer(&ctx, new_snapshot_id).await?;
                        // No need to send the request over to the rebaser as we are the rebaser.
                        ctx.commit_no_rebase().await?;
                    }
                }
            }
        }

        // If a reply was requested, send it
        if let Some(reply) = maybe_reply {
            let response = EnqueueUpdatesResponse::new_current(EnqueueUpdatesResponseVCurrent {
                id: request.id,
                workspace_id: request.workspace_id,
                change_set_id: request.change_set_id,
                status: rebase_status,
            });

            let info = ContentInfo::from(&response);

            // TODO(fnichol): add span propagation
            let mut headers = HeaderMap::new();
            info.inject_into_headers(&mut headers);

            nats.publish_with_headers(reply, headers, response.to_vec()?.into())
                .await
                .map_err(Error::PublishReply)?;
        }

        // TODO(fnichol): hrm, is this *really* true that we've written to the change set. I mean,
        // yes but until a dvu has finished this is an incomplete view?
        let mut event = WsEvent::change_set_written(&ctx, change_set_id.into()).await?;
        event.set_workspace_pk(workspace_id.into());
        event.set_change_set_id(Some(change_set_id.into()));
        event.publish_immediately(&ctx).await?;

        Ok(())
    }
}

mod app_state {
    //! Application state for a change set processor.

    use std::sync::Arc;

    use dal::DalContextBuilder;
    use si_data_nats::NatsClient;
    use si_events::{ChangeSetId, WorkspacePk};
    use tokio::sync::Notify;

    /// Application state.
    #[derive(Clone, Debug)]
    pub(crate) struct AppState {
        /// Workspace ID for the task
        pub(crate) workspace_id: WorkspacePk,
        /// Change set ID for the task
        pub(crate) change_set_id: ChangeSetId,
        /// NATS Jetstream context
        pub(crate) nats: NatsClient,
        /// DAL context builder for each processing request
        pub(crate) ctx_builder: DalContextBuilder,
        /// Signal to run a DVU job
        pub(crate) run_notify: Arc<Notify>,
    }

    impl AppState {
        /// Creates a new [`AppState`].
        pub(crate) fn new(
            workspace_id: WorkspacePk,
            change_set_id: ChangeSetId,
            nats: NatsClient,
            ctx_builder: DalContextBuilder,
            run_notify: Arc<Notify>,
        ) -> Self {
            Self {
                workspace_id,
                change_set_id,
                nats,
                ctx_builder,
                run_notify,
            }
        }
    }
}
