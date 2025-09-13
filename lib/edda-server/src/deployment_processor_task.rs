use std::{
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
use tokio_stream::StreamExt;
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};

use self::app_state::AppState;
use crate::{
    ServerMetadata,
    api_types::deployment_request::{
        CompressedDeploymentRequest,
        DeploymentRequest,
    },
    compressing_stream::CompressingStream,
    updates::EddaUpdates,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub(crate) enum DeploymentProcessorTaskError {
    /// When a naxum-based service encounters an I/O error
    #[error("naxum error: {0}")]
    Naxum(#[source] std::io::Error),
}

type Error = DeploymentProcessorTaskError;

type Result<T> = result::Result<T, DeploymentProcessorTaskError>;

pub(crate) struct DeploymentProcessorTask {
    _metadata: Arc<ServerMetadata>,
    inner: Box<dyn Future<Output = io::Result<()>> + Unpin + Send>,
}

impl DeploymentProcessorTask {
    const NAME: &'static str = "edda_server::deployment_processor_task";

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create(
        metadata: Arc<ServerMetadata>,
        nats: NatsClient,
        incoming: CompressingStream<push::Ordered, DeploymentRequest, CompressedDeploymentRequest>,
        frigg: FriggStore,
        edda_updates: EddaUpdates,
        parallel_build_limit: usize,
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
                    quiesced_notify,
                    last_compressing_heartbeat_rx,
                } = &captured;

                let last_heartbeat_elapsed = last_compressing_heartbeat_rx.borrow().elapsed();

                debug!(
                    service.instance.id = instance_id_str,
                    last_heartbeat_elapsed = last_heartbeat_elapsed.as_secs(),
                    quiescent_period = quiescent_period.as_secs(),
                );

                if last_heartbeat_elapsed > quiescent_period {
                    debug!(
                        service.instance.id = instance_id_str,
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
                EddaDeploymentRequestsForSubject::with_prefix(prefix.as_deref()),
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
            inner: Box::new(inner_fut),
        }
    }

    pub(crate) async fn try_run(self) -> Result<()> {
        self.inner.await.map_err(Error::Naxum)?;
        metric!(counter.deployment_processor_task.deployment_task = -1);

        debug!(task = Self::NAME, "shutdown complete",);
        Ok(())
    }
}

struct QuiescedCaptured {
    instance_id_str: Box<str>,
    quiesced_notify: Arc<Notify>,
    last_compressing_heartbeat_rx: watch::Receiver<Instant>,
}

#[derive(Clone, Debug)]
struct EddaDeploymentRequestsForSubject {
    prefix: Option<()>,
}

impl EddaDeploymentRequestsForSubject {
    fn with_prefix(prefix: Option<&str>) -> Self {
        Self {
            prefix: prefix.map(|_p| ()),
        }
    }
}

impl<R> ForSubject<R> for EddaDeploymentRequestsForSubject
where
    R: MessageHead,
{
    fn call(&mut self, req: &mut naxum::Message<R>) {
        let mut parts = req.subject().split('.');

        match self.prefix {
            Some(_) => {
                if let (Some(prefix), Some(p1), Some(p2), Some(p3), None) = (
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                ) {
                    let matched = format!("{prefix}.{p1}.{p2}.{p3}");
                    req.extensions_mut().insert(MatchedSubject::from(matched));
                };
            }
            None => {
                if let (Some(p1), Some(p2), Some(p3), None) =
                    (parts.next(), parts.next(), parts.next(), parts.next())
                {
                    let matched = format!("{p1}.{p2}.{p3}");
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

    use dal::DalContext;
    use frigg::FriggStore;
    use naxum::{
        Json,
        extract::State,
        response::{
            IntoResponse,
            Response,
        },
    };
    use si_data_nats::Subject;
    use telemetry::prelude::*;
    use telemetry_utils::metric;
    use thiserror::Error;

    use super::app_state::AppState;
    use crate::{
        api_types::deployment_request::CompressedDeploymentRequest,
        materialized_view,
        updates::EddaUpdates,
    };

    #[remain::sorted]
    #[derive(Debug, Error)]
    pub(crate) enum HandlerError {
        /// When failing to create a DAL context
        #[error("error creating a dal ctx: {0}")]
        DalTransactions(#[from] dal::TransactionsError),
        #[error("materialized view error: {0}")]
        MaterializedView(#[from] materialized_view::MaterializedViewError),
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
        Json(request): Json<CompressedDeploymentRequest>,
    ) -> Result<()> {
        let AppState {
            nats: _,
            frigg,
            edda_updates,
            parallel_build_limit,
            ctx_builder,
            server_tracker: _,
        } = state;
        let ctx = ctx_builder.build_default(None).await?;

        process_request(
            &ctx,
            &frigg,
            &edda_updates,
            parallel_build_limit,
            subject,
            request,
        )
        .await
    }

    #[instrument(
        // Will be renamed to: `edda.requests.deployment process`
        name = "edda.requests.deployment.process",
        level = "info",
        skip_all,
        fields(
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
            si.edda.compressed_request.kind = request.as_ref(),
            si.edda.src_requests.count = request.src_requests_count(),
        )
    )]
    async fn process_request(
        ctx: &DalContext,
        frigg: &FriggStore,
        edda_updates: &EddaUpdates,
        parallel_build_limit: usize,
        subject: Subject,
        request: CompressedDeploymentRequest,
    ) -> Result<()> {
        let span = current_span_for_instrument_at!("info");

        let otel_name = {
            let mut parts = subject.as_str().split('.');
            match (parts.next(), parts.next(), parts.next(), parts.next()) {
                (Some(p1), Some(p2), Some(p3), None) => {
                    format!("{p1}.{p2}.{p3} process")
                }
                _ => format!("{} process", subject.as_str()),
            }
        };

        span.record("messaging.destination", subject.as_str());
        span.record("otel.name", otel_name.as_str());

        match request {
            CompressedDeploymentRequest::Rebuild { .. } => {
                materialized_view::build_mvs_for_deployment(
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
            CompressedDeploymentRequest::RebuildSpecific {
                src_requests_count: _,
                schema_ids,
                schema_variant_ids,
                module_ids,
            } => materialized_view::build_mvs_for_deployment(
                ctx,
                frigg,
                edda_updates,
                parallel_build_limit,
                Some((schema_ids, schema_variant_ids, module_ids)),
                "specific rebuild",
            )
            .await
            .map_err(Into::into),
        }
        .inspect(|_| span.record_ok())
        .map_err(|err| span.record_err(err))
    }
}

mod app_state {
    //! Application state for a deployment processor.

    use dal::DalContextBuilder;
    use frigg::FriggStore;
    use si_data_nats::NatsClient;
    use tokio_util::task::TaskTracker;

    use crate::updates::EddaUpdates;

    /// Application state.
    #[derive(Clone, Debug)]
    pub(crate) struct AppState {
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
        pub(crate) fn new(
            nats: NatsClient,
            frigg: FriggStore,
            edda_updates: EddaUpdates,
            parallel_build_limit: usize,
            ctx_builder: DalContextBuilder,
            server_tracker: TaskTracker,
        ) -> Self {
            Self {
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
