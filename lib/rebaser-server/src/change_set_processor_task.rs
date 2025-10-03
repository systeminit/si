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
    StatusCode,
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

/// Only allow 50 actions to be run at once With a 60 second rebaser deadline,
/// this means that actions will begin to timeout once the average rebase loop
/// hits 1200ms (or 600ms for Destroy actions, which rebase twice) and 50 are
/// enqueued at once. An even better strategy would be to adapt this dynamically
/// by measuring rebase time on head per workspace, so that we we're always
/// throttling by how many concurrent actions could fit inside 60,000ms.
const DEFAULT_ACTION_CONCURRENCY_LIMIT: Option<usize> = Some(50);

#[remain::sorted]
#[derive(Debug, Error)]
pub(crate) enum ChangeSetProcessorTaskError {
    /// When a naxum-based service encounters an I/O error
    #[error("naxum error: {0}")]
    Naxum(#[source] std::io::Error),
    /// When the app encounters an error that should be retried by restarting the task again
    #[error("task app aborted processing")]
    TaskAborted,
}

type Error = ChangeSetProcessorTaskError;

type Result<T> = result::Result<T, ChangeSetProcessorTaskError>;

pub(crate) struct ChangeSetProcessorTask {
    _metadata: Arc<ServerMetadata>,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    internal_token: CancellationToken,
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
        snapshot_eviction_grace_period: Duration,
    ) -> Self {
        let connection_metadata = nats.metadata_clone();

        let prefix = nats.metadata().subject_prefix().map(|s| s.to_owned());

        // Cancellation token used to internally trigger a graceful shutdown of this naxum app
        let internal_token = CancellationToken::new();

        let state = AppState::new(
            workspace_id,
            change_set_id,
            nats,
            edda,
            ctx_builder,
            run_dvu_notify,
            server_tracker,
            features,
            snapshot_eviction_grace_period,
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
                    .on_success(OnSuccessRemoveMessage::new(stream.clone()))
                    .on_failure(OnFailureHandleMessage::new(
                        stream,
                        internal_token.clone(),
                        dead_letter_queue,
                    )),
            )
            .service(handlers::default.with_state(state))
            .map_response(Response::into_response);

        let inner =
            naxum::serve_with_incoming_limit(inactive_aware_incoming, app.into_make_service(), 1)
                .with_graceful_shutdown(graceful_shutdown_signal(
                    task_token,
                    quiesced_token,
                    internal_token.clone(),
                ));

        let inner_fut = inner.into_future();

        Self {
            _metadata: metadata,
            workspace_id,
            change_set_id,
            internal_token,
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

        if self.internal_token.is_cancelled() {
            // If the app internalled cancelled itself, it's due to a message processing error that
            // should be retried--and importantly not skipped over
            Err(Error::TaskAborted)
        } else {
            // Otherwise, this is a clean task shutdown
            Ok(())
        }
    }
}

struct QuiescedCaptured {
    instance_id: String,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    quiesced_notify: Arc<Notify>,
}

#[derive(Clone, Debug)]
struct OnSuccessRemoveMessage {
    stream: jetstream::stream::Stream,
}

impl OnSuccessRemoveMessage {
    fn new(stream: jetstream::stream::Stream) -> Self {
        Self { stream }
    }
}

impl jetstream_post_process::OnSuccess for OnSuccessRemoveMessage {
    fn call(
        &mut self,
        head: Arc<naxum::Head>,
        info: Arc<jetstream_post_process::Info>,
        _status: StatusCode,
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
struct OnFailureHandleMessage {
    stream: jetstream::stream::Stream,
    internal_token: CancellationToken,
    dead_letter_queue: DeadLetterQueue,
}

impl OnFailureHandleMessage {
    fn new(
        stream: jetstream::stream::Stream,
        internal_token: CancellationToken,
        dead_letter_queue: DeadLetterQueue,
    ) -> Self {
        Self {
            stream,
            internal_token,
            dead_letter_queue,
        }
    }

    async fn shutdown_to_retry_message(internal_token: CancellationToken) {
        internal_token.cancel();
    }

    async fn move_to_dlq(
        head: Arc<naxum::Head>,
        info: Arc<jetstream_post_process::Info>,
        stream: jetstream::stream::Stream,
        dead_letter_queue: DeadLetterQueue,
    ) {
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
    }
}

impl jetstream_post_process::OnFailure for OnFailureHandleMessage {
    fn call(
        &mut self,
        head: Arc<naxum::Head>,
        info: Arc<jetstream_post_process::Info>,
        maybe_status: Option<StatusCode>,
    ) -> BoxFuture<'static, ()> {
        match maybe_status.map(|sc| sc.as_u16()) {
            // - 406 "Not Acceptable" (**retry**)
            //   - api type negotiation
            //       - unsupported message type
            //       - unsupported message version
            //
            // - 415 "Unsuppported Media Type" (**retry**)
            //   - api type negotiation
            //       - unsupported content type
            //
            // - 502 "Bad Gateway" (**retry**)
            //   - `PreCommit`
            //     - failed to completely process message before commiting to change
            //
            // - 503 "Service Unavailable" (**retry**)
            //   - `TransientService`
            //     - failed to process message due to transient service error
            Some(code @ (406 | 415 | 502 | 503)) => {
                warn!(
                    subject = head.subject.as_str(),
                    stream_sequence = info.stream_sequence,
                    subject = head.subject.as_str(),
                    status_code = code,
                    concat!(
                        "error encoutered when processing message; ",
                        "message will be retried, shutting down processor task",
                    ),
                );

                Box::pin(Self::shutdown_to_retry_message(self.internal_token.clone()))
            }
            // - 400 "Bad Request" (**no retry**)
            //   - api type negotiation
            //     - required headers are missing
            //     - failed to parse content info from headers
            //     - failed to deserialize message payload (after checking type/version compat)
            //     - failed to upgrade message to current (after checking type/version compat)
            //   - `PreCommit`
            //     - change set was abandoned
            //
            // -500 "Internal Server Error" (**no retry**)
            //   - `PostCommit`
            //     - failed to completely process message after commiting to change
            Some(code @ (400 | 500)) => {
                warn!(
                    subject = head.subject.as_str(),
                    stream_sequence = info.stream_sequence,
                    subject = head.subject.as_str(),
                    status_code = code,
                    "error encoutered when processing message; moving to errored messages subject",
                );

                Box::pin(Self::move_to_dlq(
                    head,
                    info,
                    self.stream.clone(),
                    self.dead_letter_queue.clone(),
                ))
            }
            // Unexpected code
            Some(unexpected_code) => {
                // TODO(fnichol): unlcear if this is `error` or `warn`
                error!(
                    subject = head.subject.as_str(),
                    stream_sequence = info.stream_sequence,
                    subject = head.subject.as_str(),
                    status_code = unexpected_code,
                    concat!(
                        "error encoutered when processing message (unexpected status code); ",
                        "moving to errored messages subject",
                    ),
                );

                Box::pin(Self::move_to_dlq(
                    head,
                    info,
                    self.stream.clone(),
                    self.dead_letter_queue.clone(),
                ))
            }
            // No code could be determined
            None => {
                // TODO(fnichol): unlcear if this is `error` or `warn`
                error!(
                    subject = head.subject.as_str(),
                    stream_sequence = info.stream_sequence,
                    subject = head.subject.as_str(),
                    concat!(
                        "error encoutered when processing message (no status code provided); ",
                        "moving to errored messages subject",
                    ),
                );

                Box::pin(Self::move_to_dlq(
                    head,
                    info,
                    self.stream.clone(),
                    self.dead_letter_queue.clone(),
                ))
            }
        }
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

// Await conditions that would signal an app's graceful shutdown.
//
// There are 3 conditions that should result in a graceful shutdown:
//
// 1. The "task token" is cancelled, which represents a shutdown signal from the overall serivce
// 2. The "quiescence token" is cancelled, which represents a quiet period of not receiving a
//    message over a certain period
// 3. The "internal token" is cancelled, which represents an internally sourced request to shut
//    down the app
async fn graceful_shutdown_signal(
    task_token: CancellationToken,
    quiescence_token: CancellationToken,
    internal_token: CancellationToken,
) {
    tokio::select! {
        _ = task_token.cancelled() => {
            trace!("task token has received cancellation");
        }
        _ = quiescence_token.cancelled() => {
            trace!("quiescence token has received cancellation");
        }
        _ = internal_token.cancelled() => {
            trace!("internal token has received cancellation");
        }
    }
}

mod handlers {
    use std::{
        result,
        time::Instant,
    };

    use dal::{
        ChangeSet,
        Workspace,
        WorkspaceSnapshot,
        WsEvent,
        action::{
            Action,
            ActionError,
        },
        billing_publish,
        workspace_snapshot::{
            DependentValueRoot,
            dependent_value_root::DependentValueRootError,
            selector::WorkspaceSnapshotSelectorDiscriminants,
            split_snapshot::SplitSnapshot,
        },
    };
    use edda_client::EddaClient;
    use naxum::{
        extract::State,
        response::{
            IntoResponse,
            Response,
        },
    };
    use naxum_extractor_acceptable::{
        HeaderReply,
        Negotiate,
    };
    use pinga_client::api_types::{
        Container,
        SerializeContainer,
    };
    use rebaser_core::api_types::{
        ContentInfo,
        SerializeError,
        enqueue_updates_request::EnqueueUpdatesRequest,
        enqueue_updates_response::{
            EnqueueUpdatesResponse,
            EnqueueUpdatesResponseVCurrent,
            RebaseStatus,
        },
    };
    use si_data_nats::HeaderMap;
    use telemetry::prelude::*;
    use telemetry_nats::propagation;
    use telemetry_utils::metric;
    use thiserror::Error;

    use super::{
        DEFAULT_ACTION_CONCURRENCY_LIMIT,
        app_state::AppState,
    };
    use crate::rebase::{
        RebaseError,
        perform_rebase,
        send_updates_to_edda_legacy_snapshot,
        send_updates_to_edda_split_snapshot,
    };

    #[remain::sorted]
    #[derive(Debug, Error)]
    pub(crate) enum HandlerErrorSource {
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

    #[remain::sorted]
    #[derive(Debug, Error)]
    pub(crate) enum HandlerError {
        #[error("message processing error after commiting to change (delete message): {0}")]
        PostCommit(#[source] HandlerErrorSource),
        #[error("message processing error before commiting to change (retry message): {0}")]
        PreCommit(#[source] HandlerErrorSource),
        #[error("message processing error due to transient service (retry message): {0}")]
        TransientService(#[source] HandlerErrorSource),
        #[error("unprocessable message error (delete message): {0}")]
        Unprocessable(#[source] HandlerErrorSource),
    }

    impl HandlerError {
        pub(crate) fn pre_commit(err: impl Into<HandlerErrorSource>) -> Self {
            Self::PreCommit(err.into())
        }

        pub(crate) fn post_commit(err: impl Into<HandlerErrorSource>) -> Self {
            Self::PostCommit(err.into())
        }

        pub(crate) fn transient_service(err: impl Into<HandlerErrorSource>) -> Self {
            Self::TransientService(err.into())
        }

        pub(crate) fn unprocessable(err: impl Into<HandlerErrorSource>) -> Self {
            Self::Unprocessable(err.into())
        }
    }

    type Error = HandlerError;
    type ErrorSource = HandlerErrorSource;

    type Result<T> = result::Result<T, HandlerError>;

    impl IntoResponse for HandlerError {
        fn into_response(self) -> Response {
            metric!(counter.change_set_processor_task.failed_rebase = 1);

            match self {
                Self::PreCommit(err) => {
                    error!(
                        si.error.message = ?err,
                        "failed to completely process message before commiting to change",
                    );
                    Response::default_bad_gateway()
                }
                Self::PostCommit(err) => {
                    error!(
                        si.error.message = ?err,
                        "failed to completely process message after commiting to change",
                    );
                    Response::default_internal_server_error()
                }
                Self::TransientService(err) => {
                    error!(
                        si.error.message = ?err,
                        "failed to process message due to transient service error",
                    );
                    Response::default_service_unavailable()
                }
                Self::Unprocessable(err) => {
                    error!(
                        si.error.message = ?err,
                        "failed to process message which appears to be unprocessable",
                    );
                    Response::default_bad_request()
                }
            }
        }
    }

    pub(crate) async fn default(
        State(state): State<AppState>,
        HeaderReply(maybe_reply): HeaderReply,
        Negotiate(request): Negotiate<EnqueueUpdatesRequest>,
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
            snapshot_eviction_grace_period,
        } = state;
        let span = Span::current();
        span.record("si.workspace.id", workspace_id.to_string());
        span.record("si.change_set.id", change_set_id.to_string());
        let metric_label = format!("{workspace_id}:{change_set_id}");

        metric!(counter.rebaser.rebase_processing = 1, label = metric_label);

        let mut ctx = ctx_builder
            .build_for_change_set_as_system(workspace_id, change_set_id, None)
            .await
            .map_err(|err| match &err {
                // - Failed to get connection from pool or other pg pool issue
                dal::TransactionsError::PgPool(_)
                // - Failed to determine change set via db query
                | dal::TransactionsError::ChangeSet(_)
                // - Failed to determine workspace via db query
                // - Failed to determine snapshot address via db query
                // - Failed to load snapshot from layer db
                //
                | dal::TransactionsError::Workspace(_)
                => {
                    HandlerError::transient_service(err)
                }
                _ => HandlerError::pre_commit(err)
            })?;

        let rebase_batch_address_kind_result = match perform_rebase(
            &mut ctx,
            &edda,
            &request,
            &server_tracker,
            features,
            snapshot_eviction_grace_period,
        )
        .await
        {
            Ok(rebase_batch_address_kind) => Ok(rebase_batch_address_kind),
            Err(rebase_error) => match &rebase_error {
                    // - Failed to compute correct transforms. Failures here are related to the
                    //   structure of a graph snapshot and so would reliably and reproducibly fail
                    //   on additional retries.
                    RebaseError::WorkspaceSnapshot(
                        dal::WorkspaceSnapshotError::CorrectTransforms(_)
                        | dal::WorkspaceSnapshotError::CorrectTransformsSplit(_),
                    )
                    // - Failed to perform updates from correct transforms. For similar reasons as
                    //   above, these error are a result of unexpected graph snapshot structures
                    //   and would reliably and reproducibly failed on additional retries.
                    | RebaseError::PerformUpdates(_)
                    // - Change set was abandoned and will not be "un-abandoned"
                    | RebaseError::AbandonedChangeSet(_) => {
                        Err(HandlerError::unprocessable(rebase_error))
                    }
                    // - Failed to send updates to edda, but this is after commit so should *not*
                    //   retry
                    RebaseError::SendEddaUpdates(_) => {
                        Err(HandlerError::post_commit(rebase_error))
                    }
                    // - Failed to successfully commit the changes--all these errors are pre-commit
                    //   and so message should be retried
                    _ => {
                        Err(HandlerError::pre_commit(rebase_error))
                    }
                },
        };

        let maybe_post_rebase_activities_result = if rebase_batch_address_kind_result.is_ok() {
            Some(
                post_rebase_activities(workspace_id, change_set_id, &edda, run_notify, &mut ctx)
                    .await
                    .map_err(HandlerError::post_commit),
            )
        } else {
            None
        };

        // If a reply was requested, send it
        if let Some(reply) = maybe_reply {
            let response = EnqueueUpdatesResponse::new(EnqueueUpdatesResponseVCurrent {
                id: request.id,
                workspace_id: request.workspace_id,
                change_set_id: request.change_set_id,
                status: match &rebase_batch_address_kind_result {
                    Ok(rebase_batch_address_kind) => RebaseStatus::Success {
                        updates_performed: *rebase_batch_address_kind,
                    },
                    Err(handler_error) => RebaseStatus::Error {
                        message: handler_error.to_string(),
                    },
                },
            });

            let mut info = ContentInfo::from(&response);
            let (content_type, payload) = response.to_vec().map_err(Error::post_commit)?;
            info.content_type = content_type.into();

            let mut headers = HeaderMap::new();
            propagation::inject_headers(&mut headers);
            info.inject_into_headers(&mut headers);

            nats.publish_with_headers(reply, headers, payload.into())
                .await
                .map_err(|err| Error::post_commit(ErrorSource::PublishReply(err)))?;
        }

        // TODO(fnichol): hrm, is this *really* true that we've written to the change set. I mean,
        // yes but until a dvu has finished this is an incomplete view?
        let mut event = WsEvent::change_set_written(&ctx, change_set_id)
            .await
            .map_err(Error::post_commit)?;
        event.set_workspace_pk(workspace_id);
        event.set_change_set_id(Some(change_set_id));
        event
            .publish_immediately(&ctx)
            .await
            .map_err(Error::post_commit)?;

        metric!(counter.rebaser.rebase_processing = -1, label = metric_label);

        match (
            rebase_batch_address_kind_result,
            maybe_post_rebase_activities_result,
        ) {
            // If error occurred in perform rebase, return it as it's dominant error
            (Err(handler_error), _) => Err(handler_error),
            // If error occurred in post-rebase activities, return it
            (Ok(_), Some(Err(handler_error))) => Err(handler_error),
            // If no error is returned in perform rebase or post-rebase activities, return ok
            (Ok(_), None) | (Ok(_), Some(Ok(_))) => Ok(()),
        }
    }

    #[instrument(
    name = "rebase.post_rebase_activities",
    level = "info",
    skip_all,
    fields(
        si.workspace.id = %workspace_id,
        si.rebase.actions_dispatched = Empty,
        si.rebase.dispatched_actions_total = Empty,
        si.rebase.snapshot_write_time = Empty,
        si.rebase.pointer_updated_post_rebase = Empty,
        si.rebase.dispatch_actions_time = Empty,
        si.rebase.action_dependency_graph_time = Empty,
        si.rebase.dependent_value_graph_time = Empty,
    ))]
    async fn post_rebase_activities(
        workspace_id: dal::WorkspacePk,
        change_set_id: dal::ChangeSetId,
        edda: &EddaClient,
        run_notify: std::sync::Arc<tokio::sync::Notify>,
        ctx: &mut dal::DalContext,
    ) -> result::Result<(), HandlerErrorSource> {
        let start = Instant::now();
        let span = current_span_for_instrument_at!("info");

        if DependentValueRoot::roots_exist(ctx).await? {
            run_notify.notify_one();
        }

        // Dispatch eligible actions if the change set is the default for the workspace.
        // Actions are **ONLY** ever dispatched from the default change set for a workspace.
        if let Some(workspace) = Workspace::get_by_pk_opt(ctx, workspace_id).await? {
            if workspace.default_change_set_id() == ctx.visibility().change_set_id {
                let original_snapshot_address = ctx.workspace_snapshot()?.id().await;

                let mut change_set =
                    ChangeSet::get_by_id(ctx, ctx.visibility().change_set_id).await?;
                let did_actions_dispatch =
                    Action::dispatch_actions(ctx, DEFAULT_ACTION_CONCURRENCY_LIMIT).await?;
                span.record(
                    "si.rebase.dispatch_actions_time",
                    start.elapsed().as_millis(),
                );
                if did_actions_dispatch {
                    // Write out the snapshot to get the new address/id.
                    let new_snapshot_id = ctx
                        .write_snapshot()
                        .await?
                        .ok_or(dal::WorkspaceSnapshotError::WorkspaceSnapshotNotWritten)?;
                    span.record("si.rebase.snapshot_write_time", start.elapsed().as_millis());
                    // Manually update the pointer to the new address/id that reflects the new
                    // Action states.
                    change_set.update_pointer(ctx, new_snapshot_id).await?;
                    span.record(
                        "si.rebase.pointer_updated_post_rebase",
                        start.elapsed().as_millis(),
                    );

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
                    // send to edda
                    match workspace.snapshot_kind() {
                        WorkspaceSnapshotSelectorDiscriminants::LegacySnapshot => {
                            let og_legacy_snapshot =
                                WorkspaceSnapshot::find(ctx, original_snapshot_address).await?;
                            let new_legacy_snapshot =
                                WorkspaceSnapshot::find(ctx, new_snapshot_id).await?;
                            send_updates_to_edda_legacy_snapshot(
                                ctx,
                                &og_legacy_snapshot,
                                &new_legacy_snapshot,
                                edda,
                                change_set_id,
                                workspace_id,
                                span,
                            )
                            .await
                            .map_err(RebaseError::SendEddaUpdates)?;
                        }
                        WorkspaceSnapshotSelectorDiscriminants::SplitSnapshot => {
                            let og_split_snapshot =
                                SplitSnapshot::find(ctx, original_snapshot_address).await?;
                            let new_split_snapshot =
                                SplitSnapshot::find(ctx, new_snapshot_id).await?;
                            send_updates_to_edda_split_snapshot(
                                ctx,
                                &og_split_snapshot,
                                &new_split_snapshot,
                                edda,
                                change_set_id,
                                workspace_id,
                                span,
                            )
                            .await
                            .map_err(RebaseError::SendEddaUpdates)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

mod app_state {
    //! Application state for a change set processor.

    use std::{
        sync::Arc,
        time::Duration,
    };

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
        /// Grace period before a snapshot can be evicted after creation
        pub(crate) snapshot_eviction_grace_period: Duration,
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
            snapshot_eviction_grace_period: Duration,
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
                snapshot_eviction_grace_period,
            }
        }
    }
}
