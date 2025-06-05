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
use futures::TryStreamExt;
use naxum::{
    MessageHead,
    ServiceBuilder,
    ServiceExt as _,
    TowerServiceExt as _,
    extract::MatchedSubject,
    handler::Handler as _,
    middleware::{
        matched_subject::{
            ForSubject,
            MatchedSubjectLayer,
        },
        post_process::PostProcessLayer,
        trace::TraceLayer,
    },
    response::{
        IntoResponse,
        Response,
    },
};
use si_data_nats::{
    NatsClient,
    async_nats::jetstream::consumer::push,
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
    ServerMetadata,
    compressing_stream::CompressingStream,
};

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
        incoming: CompressingStream<push::Ordered>,
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
            .layer(PostProcessLayer::new())
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
        WorkspaceSnapshotAddress,
    };
    use frigg::FriggStore;
    use naxum::{
        Json,
        extract::State,
        response::{
            IntoResponse,
            Response,
        },
    };
    use ringmap::RingMap;
    use si_events::{
        change_batch::ChangeBatchAddress,
        workspace_snapshot::Change,
    };
    use telemetry::prelude::*;
    use telemetry_utils::metric;
    use thiserror::Error;

    use super::{
        app_state::AppState,
        materialized_view,
    };
    use crate::compressed_request::CompressedRequest;

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
        Json(request): Json<CompressedRequest>,
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
            si.edda.compressed_request.kind = request.as_ref(),
        )
    )]
    async fn handle_request(
        ctx: &DalContext,
        frigg: &FriggStore,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        request: CompressedRequest,
    ) -> Result<()> {
        match request {
            CompressedRequest::NewChangeSet {
                base_change_set_id: _,
                new_change_set_id: _,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                let index_was_copied = materialized_view::try_reuse_mv_index_for_new_change_set(
                    ctx,
                    frigg,
                    to_snapshot_address,
                )
                .await?;

                if index_was_copied {
                    process_incremental_updates(
                        ctx,
                        frigg,
                        change_set_id,
                        to_snapshot_address, // both snapshot addrs will use `to`
                        to_snapshot_address, // both snapshot addrs will use `to`
                        change_batch_addresses,
                    )
                    .await
                }
                // If we couldn't copy the index successfully, fall back to rebuild
                else {
                    materialized_view::build_all_mv_for_change_set(
                        ctx,
                        frigg,
                        None,
                        "explicit rebuild",
                    )
                    .await
                    .map_err(Into::into)
                }
            }
            CompressedRequest::Rebuild => {
                // Rebuild
                materialized_view::build_all_mv_for_change_set(ctx, frigg, None, "explicit rebuild")
                    .await
                    .map_err(Into::into)
            }
            CompressedRequest::Update {
                from_snapshot_address,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                // Index exists and current snapshot address is currently `to_snapshot_address`
                if frigg
                    .get_index(workspace_id, change_set_id)
                    .await?
                    .is_some()
                    && ctx.workspace_snapshot()?.id().await == to_snapshot_address
                {
                    process_incremental_updates(
                        ctx,
                        frigg,
                        change_set_id,
                        from_snapshot_address,
                        to_snapshot_address,
                        change_batch_addresses,
                    )
                    .await
                }
                // Current snapshot address is *not* currently `to_snapshot_address`
                else if ctx.workspace_snapshot()?.id().await != to_snapshot_address {
                    let latest_index_checksum = match frigg
                        .get_index_pointer_value(workspace_id, change_set_id)
                        .await?
                    {
                        Some((pointer, _kv_revision)) => Some(pointer.index_checksum),
                        None => None,
                    };
                    materialized_view::build_all_mv_for_change_set(
                        ctx,
                        frigg,
                        latest_index_checksum,
                        "snapshot moved",
                    )
                    .await
                    .map_err(Into::into)
                }
                // Index does not exist???
                else {
                    // todo: this is where we'd handle reusing an index from another change set if
                    // the snapshots match!
                    materialized_view::build_all_mv_for_change_set(
                        ctx,
                        frigg,
                        None,
                        "initial build",
                    )
                    .await
                    .map_err(Into::into)
                }
            }
        }
    }

    async fn process_incremental_updates(
        ctx: &DalContext,
        frigg: &FriggStore,
        change_set_id: ChangeSetId,
        from_snapshot_address: WorkspaceSnapshotAddress,
        to_snapshot_address: WorkspaceSnapshotAddress,
        change_batch_addresses: Vec<ChangeBatchAddress>,
    ) -> Result<()> {
        let mut changes = Vec::new();

        // Load all change batches and concatenate all changes from all batches
        for change_batch_address in change_batch_addresses {
            let change_batch = ctx
                .layer_db()
                .change_batch()
                .read_wait_for_memory(&change_batch_address)
                .await?
                .ok_or(HandlerError::ChangeBatchNotFound(change_batch_address))?;
            changes.extend_from_slice(change_batch.changes());
        }

        changes = deduplicate_changes(changes);

        materialized_view::build_mv_for_changes_in_change_set(
            ctx,
            frigg,
            change_set_id,
            from_snapshot_address,
            to_snapshot_address,
            &changes,
        )
        .await
        .map_err(Into::into)
    }

    fn deduplicate_changes(changes: Vec<Change>) -> Vec<Change> {
        let map: RingMap<_, _> = changes
            .into_iter()
            .map(|change| {
                (
                    (change.entity_kind, change.entity_id),
                    change.merkle_tree_hash,
                )
            })
            .collect();

        map.into_iter()
            .map(|((entity_kind, entity_id), merkle_tree_hash)| Change {
                entity_id,
                entity_kind,
                merkle_tree_hash,
            })
            .collect()
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
