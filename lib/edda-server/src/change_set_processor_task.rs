use std::{
    future::{
        Future,
        IntoFuture,
    },
    io,
    result,
    sync::Arc,
    time::{
        Duration,
        Instant,
    },
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
use tokio::{
    sync::{
        Notify,
        watch,
    },
    time,
};
use tokio_stream::StreamExt as _;
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};

use self::app_state::AppState;
use crate::{
    ServerMetadata,
    api_types::change_set_request::{
        ChangeSetRequest,
        CompressedChangeSetRequest,
    },
    compressing_stream::CompressingStream,
    updates::EddaUpdates,
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
    const NAME: &'static str = "edda_server::change_set_processor_task";

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create(
        metadata: Arc<ServerMetadata>,
        nats: NatsClient,
        incoming: CompressingStream<push::Ordered, ChangeSetRequest, CompressedChangeSetRequest>,
        frigg: FriggStore,
        edda_updates: EddaUpdates,
        parallel_build_limit: usize,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        ctx_builder: DalContextBuilder,
        quiescent_period: Duration,
        quiesced_notify: Arc<Notify>,
        quiesced_token: CancellationToken,
        last_compressing_heartbeat_rx: watch::Receiver<Instant>,
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
            edda_updates,
            parallel_build_limit,
            ctx_builder,
            server_tracker,
        );

        // Set up a check interval that ideally fires more often than the quiescent period
        let check_interval =
            time::interval(quiescent_period.checked_div(10).unwrap_or(quiescent_period));

        let captured = QuiescedCaptured {
            instance_id_str: metadata.instance_id().to_string().into_boxed_str(),
            workspace_id_str: workspace_id.to_string().into_boxed_str(),
            change_set_id_str: change_set_id.to_string().into_boxed_str(),
            quiesced_notify: quiesced_notify.clone(),
            last_compressing_heartbeat_rx,
        };
        let inactive_aware_incoming = incoming
            // Frequency at which we check for a quiet period
            .timeout_repeating(check_interval)
            // Fire quiesced_notify which triggers a specific shutdown of the serial dvu task where
            // we *know* we want to remove the task from the set of work.
            .inspect_err(move |_elapsed| {
                let QuiescedCaptured {
                    instance_id_str,
                    workspace_id_str,
                    change_set_id_str,
                    quiesced_notify,
                    last_compressing_heartbeat_rx,
                } = &captured;

                let last_heartbeat_elapsed = last_compressing_heartbeat_rx.borrow().elapsed();

                debug!(
                    service.instance.id = instance_id_str,
                    si.workspace.id = workspace_id_str,
                    si.change_set.id = change_set_id_str,
                    last_heartbeat_elapsed = last_heartbeat_elapsed.as_secs(),
                    quiescent_period = quiescent_period.as_secs(),
                );

                if last_heartbeat_elapsed > quiescent_period {
                    debug!(
                        service.instance.id = instance_id_str,
                        si.workspace.id = workspace_id_str,
                        si.change_set.id = change_set_id_str,
                        "rate of requests has become inactive, triggering a quiesced shutdown",
                    );
                    // Notify the serial dvu task that we want to shutdown due to a quiet period
                    quiesced_notify.notify_one();
                }
            })
            // Continue processing messages as normal until the Naxum app's graceful shutdown is
            // triggered. This means we turn the stream back from a stream of
            // `Result<Result<Message, _>, Elapsed>` into `Result<Message, _>`
            .filter_map(|maybe_elapsed_item| maybe_elapsed_item.ok());

        let app = ServiceBuilder::new()
            .layer(MatchedSubjectLayer::new().for_subject(
                EddaChangeSetRequestsForSubject::with_prefix(prefix.as_deref()),
            ))
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
    instance_id_str: Box<str>,
    workspace_id_str: Box<str>,
    change_set_id_str: Box<str>,
    quiesced_notify: Arc<Notify>,
    last_compressing_heartbeat_rx: watch::Receiver<Instant>,
}

#[derive(Clone, Debug)]
struct EddaChangeSetRequestsForSubject {
    prefix: Option<()>,
}

impl EddaChangeSetRequestsForSubject {
    fn with_prefix(prefix: Option<&str>) -> Self {
        Self {
            prefix: prefix.map(|_p| ()),
        }
    }
}

impl<R> ForSubject<R> for EddaChangeSetRequestsForSubject
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
                    Some(p3),
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
                    parts.next(),
                ) {
                    let matched = format!("{prefix}.{p1}.{p2}.{p3}.:workspace_id.:change_set_id");
                    req.extensions_mut().insert(MatchedSubject::from(matched));
                };
            }
            None => {
                if let (
                    Some(p1),
                    Some(p2),
                    Some(p3),
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
                    let matched = format!("{p1}.{p2}.{p3}.:workspace_id.:change_set_id");
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
    use std::{
        collections::BTreeSet,
        result,
    };

    use dal::{
        ChangeSet,
        ChangeSetId,
        DalContext,
        Schema,
        Ulid,
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
    use si_data_nats::Subject;
    use si_events::{
        change_batch::ChangeBatchAddress,
        workspace_snapshot::{
            Change,
            EntityKind,
        },
    };
    use telemetry::prelude::*;
    use telemetry_utils::metric;
    use thiserror::Error;

    use super::app_state::AppState;
    use crate::{
        api_types::change_set_request::CompressedChangeSetRequest,
        materialized_view,
        updates::EddaUpdates,
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
        #[error("schema error: {0}")]
        Schema(#[from] dal::SchemaError),
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
        subject: Subject,
        Json(request): Json<CompressedChangeSetRequest>,
    ) -> Result<()> {
        let AppState {
            workspace_id,
            change_set_id,
            nats: _,
            frigg,
            edda_updates,
            parallel_build_limit,
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

        process_request(
            &ctx,
            &frigg,
            &edda_updates,
            parallel_build_limit,
            subject,
            workspace_id,
            change_set_id,
            request,
        )
        .await
    }

    #[instrument(
        name = "edda.requests.change_set.process",
        level = "info",
        skip_all,
        fields(
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
            si.workspace.id = %workspace_id,
            si.change_set.id = %change_set_id,
            si.edda.compressed_request.kind = request.as_ref(),
            si.edda.src_requests.count = request.src_requests_count(),
        )
    )]
    #[allow(clippy::too_many_arguments)]
    async fn process_request(
        ctx: &DalContext,
        frigg: &FriggStore,
        edda_updates: &EddaUpdates,
        parallel_build_limit: usize,
        subject: Subject,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        request: CompressedChangeSetRequest,
    ) -> Result<()> {
        let span = current_span_for_instrument_at!("info");

        let otel_name = {
            let mut parts = subject.as_str().split('.');
            match (
                parts.next(),
                parts.next(),
                parts.next(),
                parts.next(),
                parts.next(),
                parts.next(),
            ) {
                (Some(p1), Some(p2), Some(p3), Some(_workspace_id), Some(_change_set_id), None) => {
                    format!("{p1}.{p2}.{p3}.:workspace_id.:change_set_id process")
                }
                _ => format!("{} process", subject.as_str()),
            }
        };

        span.record("messaging.destination", subject.as_str());
        span.record("otel.name", otel_name.as_str());

        match request {
            CompressedChangeSetRequest::NewChangeSet {
                src_requests_count: _,
                base_change_set_id: _,
                new_change_set_id: _,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                let index_was_copied = materialized_view::try_reuse_mv_index_for_new_change_set(
                    ctx,
                    frigg,
                    edda_updates,
                    to_snapshot_address,
                )
                .await
                .map_err(|err| span.record_err(err))?;

                if index_was_copied {
                    process_incremental_updates(
                        ctx,
                        frigg,
                        edda_updates,
                        parallel_build_limit,
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
                        edda_updates,
                        parallel_build_limit,
                        None,
                        "explicit rebuild",
                    )
                    .await
                    .map_err(Into::into)
                }
            }
            CompressedChangeSetRequest::Rebuild { .. } => {
                // Rebuild
                materialized_view::build_all_mv_for_change_set(
                    ctx,
                    frigg,
                    edda_updates,
                    parallel_build_limit,
                    None,
                    "explicit rebuild",
                )
                .await
                .map_err(Into::into)
            }
            CompressedChangeSetRequest::RebuildChangedDefinitions { .. } => {
                // Rebuild only change set MVs with outdated definition checksums
                // Since we're not processing incremental changes, use build_all_mv_for_change_set
                // which will detect and rebuild only outdated definitions
                materialized_view::build_all_mv_for_change_set(
                    ctx,
                    frigg,
                    edda_updates,
                    parallel_build_limit,
                    None,
                    "selective rebuild based on definition checksums",
                )
                .await
                .map_err(Into::into)
            }
            CompressedChangeSetRequest::Update {
                src_requests_count: _,
                from_snapshot_address,
                to_snapshot_address,
                change_batch_addresses,
            } => {
                // Index exists
                if frigg
                    .get_change_set_index(workspace_id, change_set_id)
                    .await
                    .map_err(|err| span.record_err(err))?
                    .is_some()
                {
                    // Always use unified approach that deduplicates work between explicit updates and changed definitions
                    let mut changes = Vec::new();

                    // Load all change batches and concatenate all changes from all batches
                    for change_batch_address in change_batch_addresses {
                        let change_batch = ctx
                            .layer_db()
                            .change_batch()
                            .read_wait_for_memory(&change_batch_address)
                            .await
                            .map_err(|err| span.record_err(err))?
                            .ok_or(HandlerError::ChangeBatchNotFound(change_batch_address))?;
                        changes.extend_from_slice(change_batch.changes());
                    }

                    changes = deduplicate_changes(changes);
                    post_process_changes(ctx, &mut changes).await?;

                    // build_mv_for_changes_in_change_set now automatically combines
                    // explicit changes and outdated definitions
                    materialized_view::build_mv_for_changes_in_change_set(
                        ctx,
                        frigg,
                        edda_updates,
                        parallel_build_limit,
                        change_set_id,
                        from_snapshot_address,
                        to_snapshot_address,
                        &changes,
                    )
                    .await
                    .map_err(Into::into)
                }
                // Index does not exist
                else {
                    // todo: this is where we'd handle reusing an index from another change set if
                    // the snapshots match!
                    let build_reason = "initial build with changed definitions";
                    materialized_view::build_all_mv_for_change_set(
                        ctx,
                        frigg,
                        edda_updates,
                        parallel_build_limit,
                        None,
                        build_reason,
                    )
                    .await
                    .map_err(Into::into)
                }
            }
        }
        .inspect(|_| span.record_ok())
        .map_err(|err| span.record_err(err))
    }

    /// In order to have correct materialized views, sometimes a change in one thing
    /// means we should report a change in some other thing, even though that thing
    /// has not actually changed. For example, if an overlay function has been added
    /// to a schema, we need to recalculate the materialized views for the schema
    /// variants under that schema.
    #[instrument(
        level = "info",
        name = "edda.requests.change_set.process.post_process_changes",
        skip_all
    )]
    async fn post_process_changes(ctx: &DalContext, changes: &mut Vec<Change>) -> Result<()> {
        let mut overlay_category_changed = false;
        let mut changed_schemas = BTreeSet::new();
        let mut changed_variants = BTreeSet::new();
        for change in changes.iter() {
            match change.entity_kind {
                EntityKind::CategoryOverlay => {
                    overlay_category_changed = true;
                }
                EntityKind::Schema => {
                    let id: Ulid = change.entity_id.into();
                    changed_schemas.insert(id);
                }
                EntityKind::SchemaVariant => {
                    let id: Ulid = change.entity_id.into();
                    changed_variants.insert(id);
                }
                _ => {}
            }
        }

        if overlay_category_changed {
            for changed_schema_id in changed_schemas {
                let variant_ids =
                    Schema::list_schema_variant_ids(ctx, changed_schema_id.into()).await?;
                for variant_id in variant_ids {
                    let variant_ulid: Ulid = variant_id.into();
                    if changed_variants.contains(&variant_ulid) {
                        continue;
                    }
                    let merkle_tree_hash = ctx
                        .workspace_snapshot()?
                        .get_node_weight(variant_id)
                        .await?
                        .merkle_tree_hash();

                    changes.push(Change {
                        entity_id: variant_ulid.into(),
                        entity_kind: EntityKind::SchemaVariant,
                        merkle_tree_hash,
                    });
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn process_incremental_updates(
        ctx: &DalContext,
        frigg: &FriggStore,
        edda_updates: &EddaUpdates,
        parallel_build_limit: usize,
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
        post_process_changes(ctx, &mut changes).await?;

        materialized_view::build_mv_for_changes_in_change_set(
            ctx,
            frigg,
            edda_updates,
            parallel_build_limit,
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

    use crate::updates::EddaUpdates;

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
        /// Publishes patch and index update messages
        pub(crate) edda_updates: EddaUpdates,
        /// Parallelism limit for materialized view builds
        pub(crate) parallel_build_limit: usize,
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
            edda_updates: EddaUpdates,
            parallel_build_limit: usize,
            ctx_builder: DalContextBuilder,
            server_tracker: TaskTracker,
        ) -> Self {
            Self {
                workspace_id,
                change_set_id,
                nats,
                frigg,
                edda_updates,
                parallel_build_limit,
                ctx_builder,
                server_tracker,
            }
        }
    }
}
