use std::{
    future::{
        Future,
        IntoFuture,
    },
    io,
    result,
    sync::Arc,
    time::Duration,
};

use dal::DalContextBuilder;
use edda_client::EddaClient;
use futures::{
    TryStreamExt,
    future::BoxFuture,
};
use nats_dead_letter_queue::DeadLetterQueue;
use naxum::{
    MessageHead,
    ServiceBuilder,
    ServiceExt as _,
    TowerServiceExt as _,
    extract::MatchedSubject,
    handler::Handler as _,
    middleware::{
        jetstream_post_process::{
            self,
            JetstreamPostProcessLayer,
        },
        matched_subject::{
            ForSubject,
            MatchedSubjectLayer,
        },
        trace::TraceLayer,
    },
    response::{
        IntoResponse,
        Response,
    },
};
use si_data_nats::{
    NatsClient,
    async_nats::jetstream::{
        self,
        consumer::push,
    },
};
use si_events::{
    ChangeSetId,
    WorkspacePk,
};
use telemetry::prelude::*;
use telemetry_utils::metric;
use thiserror::Error;
use tokio::sync::Notify;
use tokio_stream::StreamExt as _;
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};

use self::app_state::AppState;
use crate::{
    Features,
    ServerMetadata,
};

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
        dead_letter_queue: DeadLetterQueue,
        incoming: push::Ordered,
        edda: EddaClient,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        ctx_builder: DalContextBuilder,
        run_dvu_notify: Arc<Notify>,
        quiescent_period: Duration,
        quiesced_notify: Arc<Notify>,
        quiesced_token: CancellationToken,
        task_token: CancellationToken,
        server_tracker: TaskTracker,
        features: Features,
    ) -> Self {
        let connection_metadata = nats.metadata_clone();

        let prefix = nats.metadata().subject_prefix().map(|s| s.to_owned());

        let state = AppState::new(
            workspace_id,
            change_set_id,
            nats,
            edda,
            ctx_builder,
            run_dvu_notify,
            server_tracker,
            features,
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
                MatchedSubjectLayer::new()
                    .for_subject(RebaserRequestsForSubject::with_prefix(prefix.as_deref())),
            )
            .layer(
                TraceLayer::new()
                    .make_span_with(
                        telemetry_nats::NatsMakeSpan::builder(connection_metadata).build(),
                    )
                    .on_response(telemetry_nats::NatsOnResponse::new()),
            )
            .layer(
                JetstreamPostProcessLayer::new()
                    .on_success(DeleteMessageOnSuccess::new(stream.clone()))
                    .on_failure(MoveMessageOnFailure::new(stream, dead_letter_queue)),
            )
            .service(handlers::default.with_state(state))
            .map_response(Response::into_response);

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
        metric!(counter.change_set_processor_task.change_set_task = -1);

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

impl jetstream_post_process::OnSuccess for DeleteMessageOnSuccess {
    fn call(
        &mut self,
        head: Arc<naxum::Head>,
        info: Arc<jetstream_post_process::Info>,
    ) -> BoxFuture<'static, ()> {
        let stream = self.stream.clone();

        Box::pin(async move {
            debug!("deleting message on success");
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

#[derive(Clone, Debug)]
struct MoveMessageOnFailure {
    stream: jetstream::stream::Stream,
    dead_letter_queue: DeadLetterQueue,
}

impl MoveMessageOnFailure {
    fn new(stream: jetstream::stream::Stream, dead_letter_queue: DeadLetterQueue) -> Self {
        Self {
            stream,
            dead_letter_queue,
        }
    }
}

impl jetstream_post_process::OnFailure for MoveMessageOnFailure {
    fn call(
        &mut self,
        head: Arc<naxum::Head>,
        info: Arc<jetstream_post_process::Info>,
    ) -> BoxFuture<'static, ()> {
        let stream = self.stream.clone();
        let dead_letter_queue = self.dead_letter_queue.clone();

        Box::pin(async move {
            error!(
                subject = head.subject.as_str(),
                stream_sequence = info.stream_sequence,
                subject = head.subject.as_str(),
                "error encoutered when processing message; moving to errored messages subject",
            );

            // Fetch the message associated with the error
            let msg = match stream.get_raw_message(info.stream_sequence).await {
                Ok(msg) => msg,
                Err(err) => {
                    error!(
                        si.error.message = ?err,
                        subject = head.subject.as_str(),
                        "failed to read errored message from stream",
                    );
                    return;
                }
            };

            // Publish copy of errored message to the dead letter queues stream
            if let Err(err) = dead_letter_queue
                .publish_with_headers(msg.subject, msg.headers, msg.payload)
                .await
            {
                error!(
                    si.error.message = ?err,
                    src_subject = head.subject.as_str(),
                    "failed to re-publish errored message to dead letter queue",
                );
                return;
            }

            // Delete errored message from original stream location
            if let Err(err) = stream.delete_message(info.stream_sequence).await {
                error!(
                    si.error.message = ?err,
                    subject = head.subject.as_str(),
                    "failed to delete the message",
                );
            }
        })
    }
}

#[derive(Clone, Debug)]
struct RebaserRequestsForSubject {
    prefix: Option<()>,
}

impl RebaserRequestsForSubject {
    fn with_prefix(prefix: Option<&str>) -> Self {
        Self {
            prefix: prefix.map(|_p| ()),
        }
    }
}

impl<R> ForSubject<R> for RebaserRequestsForSubject
where
    R: MessageHead,
{
    fn call(&mut self, req: &mut naxum::Message<R>) {
        let mut parts = req.subject().split('.');

        match self.prefix {
            Some(_) => {
                if let (
                    Some(prefix),
                    Some(p1),
                    Some(p2),
                    Some(_workspace_id),
                    Some(_change_set_id),
                    None,
                ) = (
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                ) {
                    let matched = format!("{prefix}.{p1}.{p2}.:workspace_id.:change_set_id");
                    req.extensions_mut().insert(MatchedSubject::from(matched));
                };
            }
            None => {
                if let (Some(p1), Some(p2), Some(_workspace_id), Some(_change_set_id), None) = (
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                ) {
                    let matched = format!("{p1}.{p2}.:workspace_id.:change_set_id");
                    req.extensions_mut().insert(MatchedSubject::from(matched));
                };
            }
        }
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

    use dal::{
        ChangeSet,
        Workspace,
        WsEvent,
        action::{
            Action,
            ActionError,
        },
        billing_publish,
        workspace_snapshot::{
            DependentValueRoot,
            dependent_value_root::DependentValueRootError,
        },
    };
    use naxum::{
        extract::State,
        response::{
            IntoResponse,
            Response,
        },
    };
    use rebaser_core::api_types::{
        ApiWrapper,
        ContentInfo,
        SerializeError,
        enqueue_updates_request::EnqueueUpdatesRequest,
        enqueue_updates_response::{
            EnqueueUpdatesResponse,
            EnqueueUpdatesResponseVCurrent,
            v1::RebaseStatus,
        },
    };
    use si_data_nats::HeaderMap;
    use telemetry::prelude::*;
    use telemetry_nats::propagation;
    use telemetry_utils::metric;
    use thiserror::Error;

    use super::app_state::AppState;
    use crate::{
        extract::{
            ApiTypesNegotiate,
            HeaderReply,
        },
        rebase::{
            RebaseError,
            perform_rebase,
        },
    };

    #[remain::sorted]
    #[derive(Debug, Error)]
    pub(crate) enum HandlerError {
        /// Failures related to Actions
        #[error("action error: {0}")]
        Action(#[from] ActionError),
        /// Failures related to ChangeSets
        #[error("Change set error: {0}")]
        ChangeSet(#[from] dal::ChangeSetError),
        #[error("compute executor error: {0}")]
        ComputeExecutor(#[from] dal::DedicatedExecutorError),
        /// When failing to create a DAL context
        #[error("error creating a dal ctx: {0}")]
        DalTransactions(#[from] dal::TransactionsError),
        #[error("dependent value root error: {0}")]
        DependentValueRoot(#[from] DependentValueRootError),
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
            metric!(counter.change_set_processor_task.failed_rebase = 1);
            // TODO(fnichol): there are different responses, esp. for expected interrupted
            error!(si.error.message = ?self, "failed to process message");
            Response::default_internal_server_error()
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
            edda,
            ctx_builder,
            run_notify,
            server_tracker,
            features,
        } = state;
        let mut ctx = ctx_builder
            .build_for_change_set_as_system(workspace_id, change_set_id, None)
            .await?;

        let span = Span::current();
        span.record("si.workspace.id", workspace_id.to_string());
        span.record("si.change_set.id", change_set_id.to_string());

        let rebase_status = perform_rebase(&mut ctx, &edda, &request, &server_tracker, features)
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

        let maybe_post_rebase_activities_result =
            if matches!(rebase_status, RebaseStatus::Success { .. }) {
                Some(post_rebase_activities(workspace_id, run_notify, &ctx).await)
            } else {
                None
            };

        // If a reply was requested, send it
        if let Some(reply) = maybe_reply {
            let response = EnqueueUpdatesResponse::new_current(EnqueueUpdatesResponseVCurrent {
                id: request.id,
                workspace_id: request.workspace_id,
                change_set_id: request.change_set_id,
                status: rebase_status,
            });

            let info = ContentInfo::from(&response);

            let mut headers = HeaderMap::new();
            propagation::inject_headers(&mut headers);
            info.inject_into_headers(&mut headers);

            nats.publish_with_headers(reply, headers, response.to_vec()?.into())
                .await
                .map_err(Error::PublishReply)?;
        }

        // TODO(fnichol): hrm, is this *really* true that we've written to the change set. I mean,
        // yes but until a dvu has finished this is an incomplete view?
        let mut event = WsEvent::change_set_written(&ctx, change_set_id).await?;
        event.set_workspace_pk(workspace_id);
        event.set_change_set_id(Some(change_set_id));
        event.publish_immediately(&ctx).await?;

        match maybe_post_rebase_activities_result {
            // If no error is returned in post-rebase activities, return ok
            Some(Ok(_)) | None => Ok(()),
            // Otherwise, if error is returned in post-rebase activities, return it
            Some(Err(post_rebase_activities_err)) => Err(post_rebase_activities_err),
        }
    }

    async fn post_rebase_activities(
        workspace_id: dal::WorkspacePk,
        run_notify: std::sync::Arc<tokio::sync::Notify>,
        ctx: &dal::DalContext,
    ) -> Result<()> {
        if DependentValueRoot::roots_exist(ctx).await? {
            run_notify.notify_one();
        }

        // Dispatch eligible actions if the change set is the default for the workspace.
        // Actions are **ONLY** ever dispatched from the default change set for a workspace.
        if let Some(workspace) = Workspace::get_by_pk_opt(ctx, workspace_id).await? {
            if workspace.default_change_set_id() == ctx.visibility().change_set_id {
                let mut change_set =
                    ChangeSet::get_by_id(ctx, ctx.visibility().change_set_id).await?;
                if Action::dispatch_actions(ctx).await? {
                    // Write out the snapshot to get the new address/id.
                    let new_snapshot_id = ctx
                        .write_snapshot()
                        .await?
                        .ok_or(dal::WorkspaceSnapshotError::WorkspaceSnapshotNotWritten)?;
                    // Manually update the pointer to the new address/id that reflects the new
                    // Action states.
                    change_set.update_pointer(ctx, new_snapshot_id).await?;

                    if let Err(err) =
                        billing_publish::for_head_change_set_pointer_update(ctx, &change_set).await
                    {
                        error!(
                            si.error.message = ?err,
                            "Failed to publish billing for change set pointer update on HEAD",
                        );
                    }

                    // No need to send the request over to the rebaser as we are the rebaser.
                    ctx.commit_no_rebase().await?;
                }
            }
        }

        Ok(())
    }
}

mod app_state {
    //! Application state for a change set processor.

    use std::sync::Arc;

    use dal::DalContextBuilder;
    use edda_client::EddaClient;
    use si_data_nats::NatsClient;
    use si_events::{
        ChangeSetId,
        WorkspacePk,
    };
    use tokio::sync::Notify;
    use tokio_util::task::TaskTracker;

    use crate::Features;

    /// Application state.
    #[derive(Clone, Debug)]
    pub(crate) struct AppState {
        /// Workspace ID for the task
        pub(crate) workspace_id: WorkspacePk,
        /// Change set ID for the task
        pub(crate) change_set_id: ChangeSetId,
        /// NATS client
        pub(crate) nats: NatsClient,
        /// An "edda" client
        pub(crate) edda: EddaClient,
        /// DAL context builder for each processing request
        pub(crate) ctx_builder: DalContextBuilder,
        /// Signal to run a DVU job
        pub(crate) run_notify: Arc<Notify>,
        /// A task tracker for server-level tasks that can outlive the lifetime of a change set
        /// processor task
        pub(crate) server_tracker: TaskTracker,
        /// Static feature gate on new mv behavior
        pub(crate) features: Features,
    }

    impl AppState {
        /// Creates a new [`AppState`].
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn new(
            workspace_id: WorkspacePk,
            change_set_id: ChangeSetId,
            nats: NatsClient,
            edda: EddaClient,
            ctx_builder: DalContextBuilder,
            run_notify: Arc<Notify>,
            server_tracker: TaskTracker,
            features: Features,
        ) -> Self {
            Self {
                workspace_id,
                change_set_id,
                nats,
                edda,
                ctx_builder,
                run_notify,
                server_tracker,
                features,
            }
        }
    }
}
