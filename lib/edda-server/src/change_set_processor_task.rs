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
use frigg::FriggStore;
use futures::{
    TryStreamExt,
    future::BoxFuture,
};
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
use crate::ServerMetadata;

mod materialized_view;

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
    const NAME: &'static str = "edda_server::change_set_processor_task";

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create(
        metadata: Arc<ServerMetadata>,
        nats: NatsClient,
        stream: jetstream::stream::Stream,
        incoming: push::Ordered,
        frigg: FriggStore,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        ctx_builder: DalContextBuilder,
        quiescent_period: Duration,
        quiesced_notify: Arc<Notify>,
        quiesced_token: CancellationToken,
        task_token: CancellationToken,
        server_tracker: TaskTracker,
    ) -> Self {
        let connection_metadata = nats.metadata_clone();

        let prefix = nats.metadata().subject_prefix().map(|s| s.to_owned());

        let state = AppState::new(
            workspace_id,
            change_set_id,
            nats,
            frigg,
            ctx_builder,
            server_tracker,
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
                    .for_subject(EddaRequestsForSubject::with_prefix(prefix.as_deref())),
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
                    .on_failure(DeleteMessageOnFailure::new(stream)),
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
struct DeleteMessageOnFailure {
    stream: jetstream::stream::Stream,
}

impl DeleteMessageOnFailure {
    fn new(stream: jetstream::stream::Stream) -> Self {
        Self { stream }
    }
}

impl jetstream_post_process::OnFailure for DeleteMessageOnFailure {
    fn call(
        &mut self,
        head: Arc<naxum::Head>,
        info: Arc<jetstream_post_process::Info>,
    ) -> BoxFuture<'static, ()> {
        let stream = self.stream.clone();

        Box::pin(async move {
            error!(
                stream_sequence = info.stream_sequence,
                subject = head.subject.as_str(),
                "deleting message on FAILURE"
            );
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
struct EddaRequestsForSubject {
    prefix: Option<()>,
}

impl EddaRequestsForSubject {
    fn with_prefix(prefix: Option<&str>) -> Self {
        Self {
            prefix: prefix.map(|_p| ()),
        }
    }
}

impl<R> ForSubject<R> for EddaRequestsForSubject
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
        ChangeSetId,
        DalContext,
        WorkspacePk,
    };
    use frigg::FriggStore;
    use naxum::{
        extract::State,
        response::{
            IntoResponse,
            Response,
        },
    };
    use si_events::change_batch::ChangeBatchAddress;
    use telemetry::prelude::*;
    use telemetry_utils::metric;
    use thiserror::Error;

    use super::{
        app_state::AppState,
        materialized_view,
    };
    use crate::extract::{
        ApiTypesNegotiate,
        EddaRequestKind,
    };

    #[remain::sorted]
    #[derive(Debug, Error)]
    pub(crate) enum HandlerError {
        #[error("change batch not found: {0}")]
        ChangeBatchNotFound(ChangeBatchAddress),
        /// Failures related to ChangeSets
        #[error("Change set error: {0}")]
        ChangeSet(#[from] dal::ChangeSetError),
        #[error("compute executor error: {0}")]
        ComputeExecutor(#[from] dal::DedicatedExecutorError),
        /// When failing to create a DAL context
        #[error("error creating a dal ctx: {0}")]
        DalTransactions(#[from] dal::TransactionsError),
        #[error("frigg error: {0}")]
        Frigg(#[from] frigg::Error),
        #[error("layerdb error: {0}")]
        LayerDb(#[from] si_layer_cache::LayerDbError),
        #[error("materialized view error: {0}")]
        MaterializedView(#[from] materialized_view::MaterializedViewError),
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
        ApiTypesNegotiate(request): ApiTypesNegotiate,
    ) -> Result<()> {
        let AppState {
            workspace_id,
            change_set_id,
            nats: _,
            frigg,
            ctx_builder,
            server_tracker: _,
        } = state;
        let ctx = ctx_builder
            .build_for_change_set_as_system(workspace_id, change_set_id, None)
            .await?;

        let span = current_span_for_instrument_at!("info");

        if !span.is_disabled() {
            span.record("si.workspace.id", workspace_id.to_string());
            span.record("si.change_set.id", change_set_id.to_string());
        }

        let to_process_change_set = ChangeSet::get_by_id(&ctx, change_set_id).await?;

        // We should skip any change sets that are not active
        // Active in this case is open, needs approval status, approved or rejected
        if !to_process_change_set.status.is_active() {
            debug!("Attempted to process a non-active change set. Skipping");
            return Ok(());
        }

        handle_request(&ctx, &frigg, workspace_id, change_set_id, request).await
    }

    #[instrument(
        name = "edda.change_set_processor_task.handle_request"
        level = "info",
        skip_all,
        fields(
            si.workspace.id = %workspace_id,
            si.change_set.id = %change_set_id,
            si.edda_request.id = Empty
        )
    )]
    async fn handle_request(
        ctx: &DalContext,
        frigg: &FriggStore,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        request: EddaRequestKind,
    ) -> Result<()> {
        let span = current_span_for_instrument_at!("info");

        match request {
            EddaRequestKind::Update(update_request)
                if frigg
                    .get_index(workspace_id, change_set_id)
                    .await?
                    .is_some()
                    && ctx.workspace_snapshot()?.id().await
                        == update_request.to_snapshot_address =>
            {
                if !span.is_disabled() {
                    span.record("si.edda_request.id", update_request.id.to_string());
                }
                let change_batch = ctx
                    .layer_db()
                    .change_batch()
                    .read_wait_for_memory(&update_request.change_batch_address)
                    .await?
                    .ok_or(HandlerError::ChangeBatchNotFound(
                        update_request.change_batch_address,
                    ))?;

                materialized_view::build_mv_for_changes_in_change_set(
                    ctx,
                    frigg,
                    change_set_id,
                    update_request.from_snapshot_address,
                    update_request.to_snapshot_address,
                    change_batch.changes(),
                )
                .instrument(tracing::info_span!(
                    "edda.change_set_processor_task.build_mv_for_changes_in_change_set"
                ))
                .await
                .map_err(Into::into)
            }
            EddaRequestKind::Update(update_request)
                if ctx.workspace_snapshot()?.id().await != update_request.to_snapshot_address =>
            {
                if !span.is_disabled() {
                    span.record("si.edda_request.id", update_request.id.to_string());
                }
                materialized_view::build_all_mv_for_change_set(
                    ctx,
                    frigg,
                    Some(update_request.from_snapshot_address),
                )
                .instrument(tracing::info_span!(
                    "edda.change_set_processor_task.build_all_mv_for_change_set.snapshot_moved"
                ))
                .await
                .map_err(Into::into)
            }
            EddaRequestKind::Update(update_request) => {
                if !span.is_disabled() {
                    span.record("si.edda_request.id", update_request.id.to_string());
                }
                materialized_view::build_all_mv_for_change_set(ctx, frigg, None)
                    .instrument(tracing::info_span!(
                        "edda.change_set_processor_task.build_all_mv_for_change_set.initial_build"
                    ))
                    .await
                    .map_err(Into::into)
            }
            EddaRequestKind::Rebuild(rebuild_request) => {
                if !span.is_disabled() {
                    span.record("si.edda_request.id", rebuild_request.id.to_string());
                }
                materialized_view::build_all_mv_for_change_set(ctx, frigg, None)
                    .instrument(tracing::info_span!(
                        "edda.change_set_processor_task.build_all_mv_for_change_set.explicit_rebuild"
                    ))
                    .await
                    .map_err(Into::into)
            }
            EddaRequestKind::NewChangeSet(new_change_set_request) => {
                if !span.is_disabled() {
                    span.record("si.edda_request.id", new_change_set_request.id.to_string());
                }
                materialized_view::reuse_or_rebuild_index_for_new_change_set(
                    ctx,
                    frigg,
                    new_change_set_request.to_snapshot_address,
                )
                .instrument(tracing::info_span!(
                    "edda.change_set_processor_task.new_change_set"
                ))
                .await
                .map_err(Into::into)
            }
        }
    }
}

mod app_state {
    //! Application state for a change set processor.

    use dal::DalContextBuilder;
    use frigg::FriggStore;
    use si_data_nats::NatsClient;
    use si_events::{
        ChangeSetId,
        WorkspacePk,
    };
    use tokio_util::task::TaskTracker;

    /// Application state.
    #[derive(Clone, Debug)]
    pub(crate) struct AppState {
        /// Workspace ID for the task
        pub(crate) workspace_id: WorkspacePk,
        /// Change set ID for the task
        pub(crate) change_set_id: ChangeSetId,
        /// NATS client
        #[allow(dead_code)]
        pub(crate) nats: NatsClient,
        /// Frigg store
        pub(crate) frigg: FriggStore,
        /// DAL context builder for each processing request
        pub(crate) ctx_builder: DalContextBuilder,
        /// A task tracker for server-level tasks that can outlive the lifetime of a change set
        /// processor task
        #[allow(dead_code)]
        pub(crate) server_tracker: TaskTracker,
    }

    impl AppState {
        /// Creates a new [`AppState`].
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn new(
            workspace_id: WorkspacePk,
            change_set_id: ChangeSetId,
            nats: NatsClient,
            frigg: FriggStore,
            ctx_builder: DalContextBuilder,
            server_tracker: TaskTracker,
        ) -> Self {
            Self {
                workspace_id,
                change_set_id,
                nats,
                frigg,
                ctx_builder,
                server_tracker,
            }
        }
    }
}
