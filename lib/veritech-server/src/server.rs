use std::{
    collections::HashMap,
    convert::Infallible,
    fmt,
    future::{
        Future,
        IntoFuture as _,
    },
    io,
    sync::Arc,
    time::Duration,
};

use futures::{
    StreamExt,
    join,
};
use naxum::{
    MessageHead,
    ServiceBuilder,
    ServiceExt as _,
    TowerServiceExt as _,
    extract::MatchedSubject,
    handler::Handler as _,
    middleware::{
        ack::{
            AckLayer,
            BackoffOnFailure,
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
use si_crypto::VeritechDecryptionKey;
use si_data_nats::{
    NatsClient,
    NatsConfig,
    Subscriber,
    async_nats,
    jetstream,
};
use si_pool_noodle::{
    KillExecutionRequest,
    PoolNoodle,
    Spec,
    instance::cyclone::{
        LocalUdsInstance,
        LocalUdsInstanceSpec,
    },
    pool_noodle::PoolNoodleConfig,
};
use telemetry::prelude::*;
use telemetry_utils::metric;
use tokio::sync::{
    Mutex,
    oneshot,
};
use tokio_util::sync::CancellationToken;
use veritech_core::{
    ExecutionId,
    GetNatsSubjectFor,
    incoming_subject,
    veritech_work_queue,
};

use crate::{
    Config,
    ServerError,
    ServerResult,
    app_state::{
        AppState,
        KillAppState,
    },
    config::CycloneSpec,
    handlers,
    heartbeat::HeartbeatApp,
};

const CONSUMER_NAME: &str = "veritech-server";

/// Server metadata, used with telemetry.
#[derive(Clone, Debug)]
pub struct ServerMetadata {
    #[allow(dead_code)]
    instance_id: String,
}

impl ServerMetadata {
    /// Returns the server's unique instance id.
    #[allow(dead_code)]
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }
}

pub struct Server {
    metadata: Arc<ServerMetadata>,
    inner: Box<dyn Future<Output = io::Result<()>> + Unpin + Send>,
    kill_inner: Box<dyn Future<Output = io::Result<()>> + Unpin + Send>,
    shutdown_token: CancellationToken,
}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("metadata", &self.metadata)
            .field("shutdown_token", &self.shutdown_token)
            .finish()
    }
}

impl Server {
    #[instrument(name = "veritech.init.from_config", level = "info", skip_all)]
    pub async fn from_config(
        config: Config,
        token: CancellationToken,
    ) -> ServerResult<(Self, Option<HeartbeatApp>)> {
        let nats = Self::connect_to_nats(config.nats()).await?;
        let mut nats_config = config.nats().clone();
        if let Some(name) = nats_config.connection_name {
            nats_config.connection_name = Some(format!("{name}-stream"));
        }
        let nats_jetstream = Self::connect_to_nats(&nats_config).await?;

        let metadata = Arc::new(ServerMetadata {
            instance_id: config.instance_id().into(),
        });

        let decryption_key = VeritechDecryptionKey::from_config(config.crypto().clone()).await?;

        let kill_senders = Arc::new(Mutex::new(HashMap::new()));

        match config.cyclone_spec() {
            CycloneSpec::LocalHttp(_spec) => {
                //
                // TODO(fnichol): Hi there! Ultimately, the Veritech server should be able to work
                // with a LocalUds Cyclone backend and a LocalHttp version. But since this involves
                // threading through some generic types which themselves have trait
                // constraints--well we can add this back in the near future... Good luck to us
                // all.
                //
                // let mut cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec> =
                //     PoolNoodle::new(spec.pool_size.into(), spec.clone(), token.clone());
                //
                // spec.clone()
                //     .setup()
                //     .await
                //     .map_err(|e| ServerError::CycloneSetupError(Box::new(e)))?;
                // cyclone_pool
                //     .start(config.healthcheck_pool())
                //     .map_err(|e| ServerError::CyclonePool(Box::new(e)))?;
                //
                // // ...
                //
                // Ok(Server {
                //     metadata,
                //     inner: inner_future,
                //     kill_inner: kill_inner_future,
                //     shutdown_token: token,
                // })

                unimplemented!("get ready for a surprise!!")
            }
            CycloneSpec::LocalUds(spec) => {
                let pool_config = PoolNoodleConfig {
                    check_health: config.healthcheck_pool(),
                    pool_size: spec.pool_size,
                    retry_limit: config.pool_get_retry_limit(),
                    shutdown_token: token.clone(),
                    spec: spec.clone(),
                    ..Default::default()
                };

                let mut cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec> =
                    PoolNoodle::new(pool_config).await;

                spec.clone()
                    .setup()
                    .await
                    .map_err(|e| ServerError::CycloneSetupError(Box::new(e)))?;
                cyclone_pool
                    .run()
                    .map_err(|e| ServerError::CyclonePool(Box::new(e)))?;

                // Reset metrics before creating the naxum apps.
                metric!(counter.veritech.handlers_doing_work = 0);
                metric!(counter.veritech.pool_exhausted = 0);

                let inner_future = Self::build_app(
                    metadata.clone(),
                    cyclone_pool,
                    Arc::new(decryption_key),
                    config.cyclone_client_execution_timeout(),
                    config.consumer_max_deliver(),
                    nats.clone(),
                    nats_jetstream.clone(),
                    kill_senders.clone(),
                    token.clone(),
                )
                .await?;

                let kill_inner_future = Self::build_kill_app(
                    metadata.clone(),
                    nats.clone(),
                    kill_senders,
                    token.clone(),
                )
                .await?;

                let maybe_heartbeat_app = if config.heartbeat_app() {
                    Some(HeartbeatApp::new(
                        nats,
                        token.clone(),
                        config.instance_id(),
                        config.heartbeat_app_sleep_duration(),
                        config.heartbeat_app_publish_timeout_duration(),
                    ))
                } else {
                    None
                };

                Ok((
                    Server {
                        metadata,
                        inner: inner_future,
                        kill_inner: kill_inner_future,
                        shutdown_token: token,
                    },
                    maybe_heartbeat_app,
                ))
            }
        }
    }

    #[inline]
    pub async fn run(self) {
        if let Err(err) = self.try_run().await {
            error!(si.error.message = ?err, "error while running veritech main loop");
        }
    }

    pub async fn try_run(self) -> ServerResult<()> {
        let (inner_result, kill_inner_result) =
            join!(tokio::spawn(self.inner), tokio::spawn(self.kill_inner));

        // NOTE(nick,fletcher): we need to cancel before going to the kill app in case the stream
        // has closed, but the cancellation token has not been cancelled. This is a valid way to
        // close and we should handle it more gracefully in the future, but explicitly calling the
        // "cancel" method is sufficient here.
        match inner_result? {
            Ok(()) => self.shutdown_token.cancel(),
            Err(err) => return Err(ServerError::Naxum(err)),
        }
        kill_inner_result?.map_err(ServerError::Naxum)?;

        info!("veritech main loop shutdown complete");
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn build_app(
        metadata: Arc<ServerMetadata>,
        cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
        decryption_key: Arc<VeritechDecryptionKey>,
        cyclone_client_execution_timeout: Duration,
        consumer_max_deliver: i64,
        nats: NatsClient,
        nats_jetstream: NatsClient,
        kill_senders: Arc<Mutex<HashMap<ExecutionId, oneshot::Sender<()>>>>,
        token: CancellationToken,
    ) -> ServerResult<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
        let connection_metadata = nats_jetstream.metadata_clone();

        // Take the *active* subject prefix from the connected NATS client
        let prefix = nats_jetstream
            .metadata()
            .subject_prefix()
            .map(|s| s.to_owned());

        let incoming = {
            let context = jetstream::new(nats_jetstream);
            veritech_work_queue(&context, prefix.as_deref())
                .await?
                .create_consumer(Self::incoming_consumer_config(
                    prefix.as_deref(),
                    consumer_max_deliver,
                ))
                .await?
                .messages()
                .await?
        };

        let admission_semaphore = cyclone_pool.admission_semaphore();

        let state = AppState::new(
            metadata,
            cyclone_pool,
            decryption_key,
            cyclone_client_execution_timeout,
            nats,
            kill_senders,
        );

        let app = ServiceBuilder::new()
            .layer(
                MatchedSubjectLayer::new()
                    .for_subject(VeritechForSubject::with_prefix(prefix.as_deref())),
            )
            .layer(
                TraceLayer::new()
                    .make_span_with(
                        telemetry_nats::NatsMakeSpan::builder(connection_metadata).build(),
                    )
                    .on_response(telemetry_nats::NatsOnResponse::new()),
            )
            .layer(AckLayer::new().on_failure(BackoffOnFailure::new(consumer_max_deliver)))
            .service(handlers::process_request.with_state(state))
            .map_response(Response::into_response);

        let inner = naxum::serve_with_external_semaphore(
            incoming,
            app.into_make_service(),
            admission_semaphore,
        )
        .with_graceful_shutdown(naxum::wait_on_cancelled(token));

        Ok(Box::new(inner.into_future()))
    }

    async fn build_kill_app(
        metadata: Arc<ServerMetadata>,
        nats: NatsClient,
        kill_senders: Arc<Mutex<HashMap<ExecutionId, oneshot::Sender<()>>>>,
        token: CancellationToken,
    ) -> ServerResult<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
        let connection_metadata = nats.metadata_clone();

        let incoming = {
            let prefix = nats.metadata().subject_prefix().map(|s| s.to_owned());
            Self::kill_subscriber(&nats, prefix.as_deref())
                .await?
                .map(|msg| msg.into_parts().0)
                // Core NATS subscriptions are a stream of `Option<Message>` so we convert this
                // into a stream of `Option<Result<Message, Infallible>>`
                .map(Ok::<_, Infallible>)
        };

        let state = KillAppState::new(metadata, nats, kill_senders);

        let app = ServiceBuilder::new()
            .layer(
                TraceLayer::new()
                    .make_span_with(
                        telemetry_nats::NatsMakeSpan::builder(connection_metadata).build(),
                    )
                    .on_response(telemetry_nats::NatsOnResponse::new()),
            )
            .service(handlers::process_kill_request.with_state(state))
            .map_response(Response::into_response);

        let inner = naxum::serve(incoming, app.into_make_service())
            .with_graceful_shutdown(naxum::wait_on_cancelled(token));

        Ok(Box::new(inner.into_future()))
    }

    // NOTE(nick,fletcher): it's a little funky that the prefix is taken from the nats client, but
    // we ask for both here. We don't want users to forget the prefix, so being explicit is
    // helpful, but maybe we can change this in the future.
    async fn kill_subscriber(nats: &NatsClient, prefix: Option<&str>) -> ServerResult<Subscriber> {
        // we have to make a dummy request here to get the nats subject from it
        let dummy_request = KillExecutionRequest {
            execution_id: "".into(),
        };
        let subject = dummy_request.nats_subject(prefix, None, None);
        nats.subscribe(subject.clone())
            .await
            .map_err(|err| ServerError::NatsSubscribe(subject, err))
    }

    #[instrument(name = "veritech.init.connect_to_nats", level = "info", skip_all)]
    async fn connect_to_nats(config: &NatsConfig) -> ServerResult<NatsClient> {
        let client = NatsClient::new(config)
            .await
            .map_err(ServerError::NatsClient)?;
        debug!("successfully connected nats client");
        Ok(client)
    }

    #[inline]
    fn incoming_consumer_config(
        subject_prefix: Option<&str>,
        max_deliver: i64,
    ) -> async_nats::jetstream::consumer::pull::Config {
        async_nats::jetstream::consumer::pull::Config {
            durable_name: Some(CONSUMER_NAME.to_owned()),
            filter_subject: incoming_subject(subject_prefix).to_string(),
            max_deliver,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug)]
struct VeritechForSubject {
    prefix: Option<()>,
}

impl VeritechForSubject {
    fn with_prefix(prefix: Option<&str>) -> Self {
        Self {
            prefix: prefix.map(|_p| ()),
        }
    }
}

impl<R> ForSubject<R> for VeritechForSubject
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
                    Some(kind),
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
                    let matched = format!("{prefix}.{p1}.{p2}.:workspace_id.:change_set_id.{kind}");
                    req.extensions_mut().insert(MatchedSubject::from(matched));
                };
            }
            None => {
                if let (
                    Some(p1),
                    Some(p2),
                    Some(_workspace_id),
                    Some(_change_set_id),
                    Some(kind),
                    None,
                ) = (
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                ) {
                    let matched = format!("{p1}.{p2}.:workspace_id.:change_set_id.{kind}");
                    req.extensions_mut().insert(MatchedSubject::from(matched));
                };
            }
        }
    }
}
