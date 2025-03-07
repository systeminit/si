use std::{
    collections::HashMap,
    convert::Infallible,
    fmt,
    future::{Future, IntoFuture as _},
    io::{self},
    sync::Arc,
    time::Duration,
};

use futures::{join, StreamExt};
use naxum::{
    extract::MatchedSubject,
    handler::Handler as _,
    middleware::{
        ack::AckLayer,
        matched_subject::{ForSubject, MatchedSubjectLayer},
        trace::TraceLayer,
    },
    response::{IntoResponse, Response},
    MessageHead, ServiceBuilder, ServiceExt as _, TowerServiceExt as _,
};
use si_crypto::VeritechDecryptionKey;
use si_data_nats::{
    async_nats::{self},
    jetstream, NatsClient, NatsConfig, State, Subscriber,
};
use si_pool_noodle::{
    instance::cyclone::{LocalUdsInstance, LocalUdsInstanceSpec},
    pool_noodle::PoolNoodleConfig,
    KillExecutionRequest, PoolNoodle, Spec,
};
use telemetry::prelude::*;
use telemetry_utils::metric;
use tokio::sync::{oneshot, Mutex};
use tokio_util::sync::CancellationToken;
use veritech_core::{incoming_subject, veritech_work_queue, ExecutionId, GetNatsSubjectFor};

use crate::{
    app_state::{AppState, KillAppState},
    config::CycloneSpec,
    handlers, Config, ServerError, ServerResult,
};

mod pause_resume_stream;

pub use pause_resume_stream::{PauseResumeController, PauseResumeStream};

const CONSUMER_NAME: &str = "veritech-server";
const CONSUMER_MAX_DELIVERY: i64 = 5;

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
    ) -> ServerResult<(Self, impl Future<Output = ()>)> {
        let nats = Self::connect_to_nats(&config).await?;

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
                metric!(counter.veritech.pause_resume_stream.missing_heartbeat = 0);
                metric!(counter.veritech.pause_resume_stream.paused = 0);
                metric!(counter.veritech.pause_resume_stream.sleep_done = 0);
                metric!(counter.veritech.pause_resume_stream.subscribing = 0);
                metric!(counter.veritech.pause_resume_stream.subscribed = 0);
                metric!(counter.veritech.pause_resume_stream.stream_error = 0);

                let nats = Arc::new(Mutex::new(nats));

                // let inner_future = if config.exclude_pause_resume_stream_wrapper() {
                info!("building app without pause resume stream wrapper");
                let inner_future = Self::build_app_without_pause_resume_stream(
                    metadata.clone(),
                    config.concurrency_limit(),
                    cyclone_pool,
                    Arc::new(decryption_key),
                    config.cyclone_client_execution_timeout(),
                    nats.clone(),
                    kill_senders.clone(),
                    token.clone(),
                )
                .await?;
                // } else {
                //     info!("building app with pause resume stream wrapper");
                //     Self::build_app(
                //         metadata.clone(),
                //         config.concurrency_limit(),
                //         cyclone_pool,
                //         Arc::new(decryption_key),
                //         config.cyclone_client_execution_timeout(),
                //         nats.clone(),
                //         kill_senders.clone(),
                //         token.clone(),
                //         config.pause_duration(),
                //         config.reconnect_backoff_duration(),
                //     )
                //     .await?
                // };

                let kill_inner_future = Self::build_kill_app(
                    metadata.clone(),
                    nats.clone(),
                    kill_senders,
                    token.clone(),
                )
                .await?;

                let internal_heartbeat_inner_future =
                    Self::run_internal_heartbeat_app(nats, config.nats().to_owned(), token.clone());

                Ok((
                    Server {
                        metadata,
                        inner: inner_future,
                        kill_inner: kill_inner_future,
                        shutdown_token: token,
                    },
                    internal_heartbeat_inner_future,
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

        inner_result?.map_err(ServerError::Naxum)?;
        kill_inner_result?.map_err(ServerError::Naxum)?;

        info!("veritech main loop shutdown complete");
        Ok(())
    }

    async fn run_internal_heartbeat_app(
        nats: Arc<Mutex<NatsClient>>,
        nats_config: NatsConfig,
        token: CancellationToken,
    ) {
        info!("resetting internal heartbeat app metrics and running app...");
        metric!(counter.veritech.internal_heartbeat.loop_iteration = 0);
        metric!(
            counter
                .veritech
                .internal_heartbeat
                .client_lock_failure
                .for_publish = 0
        );
        metric!(
            counter
                .veritech
                .internal_heartbeat
                .client_lock_failure
                .for_connection_state = 0
        );
        metric!(counter.veritech.internal_heartbeat.publish.success = 0);
        metric!(counter.veritech.internal_heartbeat.publish.error = 0);
        metric!(counter.veritech.internal_heartbeat.publish.timeout = 0);
        metric!(counter.veritech.internal_heartbeat.new_client.success = 0);
        metric!(counter.veritech.internal_heartbeat.new_client.error = 0);
        metric!(counter.veritech.internal_heartbeat.new_client.timeout = 0);
        metric!(
            counter
                .veritech
                .internal_heartbeat
                .connection_state
                .connected = 0
        );
        metric!(
            counter
                .veritech
                .internal_heartbeat
                .connection_state
                .disconnected = 0
        );
        metric!(counter.veritech.internal_heartbeat.connection_state.pending = 0);

        let empty_byte_array: Vec<u8> = Vec::new();
        let sleep_seconds = 5;
        let publish_timeout_seconds = 5;
        let create_client_timeout_seconds = 20;
        let client_lock_timeout_seconds = 10;

        // Don't try to publish again if we haven't force reconnected.
        let mut needs_new_client = false;

        loop {
            metric!(counter.veritech.internal_heartbeat.loop_iteration = 1);
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(sleep_seconds)) => {
                    info!(%sleep_seconds, "internal heartbeat: finished sleeping!");

                    if needs_new_client {
                        warn!(
                            "internal heartbeat: creating new client due to internal heartbeat publish timeout..."
                        );
                        match tokio::time::timeout(
                            Duration::from_secs(create_client_timeout_seconds),
                            NatsClient::new_for_veritech(&nats_config),
                        )
                        .await
                        {
                            Ok(new_client_result) => match new_client_result {
                                Ok(new_nats) => {
                                    warn!("internal heartbeat: created new client!");
                                    metric!(counter.veritech.internal_heartbeat.new_client.success = 1);
                                    let mut guard = nats.lock().await;
                                    *guard = new_nats;
                                    needs_new_client = false;
                                }
                                Err(err) => {
                                    error!(si.error.message = ?err, "internal heartbeat: new client error");
                                    metric!(counter.veritech.internal_heartbeat.new_client.error = 1);
                                }
                            },
                            Err(err) => {
                                error!(si.error.message = ?err, %create_client_timeout_seconds, "internal heartbeat: new client timeout");
                                metric!(counter.veritech.internal_heartbeat.new_client.timeout = 1);
                            }
                        }
                    } else {
                        info!("internal heartbeat: acquiring lock for publishing heartbeat...");
                        let lock = match tokio::time::timeout(
                            Duration::from_secs(client_lock_timeout_seconds),
                            nats.lock(),
                        )
                        .await
                        {
                            Ok(lock) => lock,
                            Err(err) => {
                                error!(si.error.message = ?err, "internal heartbeat: could not acquire lock for nats client");
                                metric!(counter.veritech.internal_heartbeat.client_lock_failure.for_publish = 1);
                                continue;
                            }
                        };

                        info!("internal heartbeat: publishing heartbeat...");
                        match tokio::time::timeout(
                            Duration::from_secs(publish_timeout_seconds),
                            lock.publish(
                                "veritech.internal_heartbeat",
                                empty_byte_array.to_owned().into(),
                            ),
                        )
                        .await
                        {
                            Ok(publish_result) => match publish_result {
                                Ok(()) => {
                                    metric!(counter.veritech.internal_heartbeat.publish.success = 1);
                                }
                                Err(err) => {
                                    error!(si.error.message = ?err, "internal heartbeat: publish error");
                                    metric!(counter.veritech.internal_heartbeat.publish.error = 1);
                                }
                            },
                            Err(err) => {
                                error!(si.error.message = ?err, %publish_timeout_seconds, "internal heartbeat: publish timeout");
                                metric!(counter.veritech.internal_heartbeat.publish.timeout = 1);
                                needs_new_client = true;
                            }
                        }

                        drop(lock);
                    }

                    info!("internal heartbeat: acquiring lock for getting connection state...");
                    let lock = match tokio::time::timeout(
                        Duration::from_secs(client_lock_timeout_seconds),
                        nats.lock(),
                    )
                    .await
                    {
                        Ok(lock) => lock,
                        Err(err) => {
                            error!(si.error.message = ?err, "internal heartbeat: could not acquire lock for nats client");
                            metric!(counter.veritech.internal_heartbeat.client_lock_failure.for_connection_state = 1);
                            continue;
                        }
                    };

                    info!("internal heartbeat: getting connection state... (has an internal read lock!)");
                    let state = match lock.connection_state() {
                        State::Connected => {
                            metric!(
                                counter
                                    .veritech
                                    .internal_heartbeat
                                    .connection_state
                                    .connected = 1
                            );
                            State::Connected
                        }
                        State::Disconnected => {
                            metric!(
                                counter
                                    .veritech
                                    .internal_heartbeat
                                    .connection_state
                                    .disconnected = 1
                            );
                            State::Disconnected
                        }
                        State::Pending => {
                            metric!(counter.veritech.internal_heartbeat.connection_state.pending = 1);
                            State::Pending
                        }
                    };
                    drop(lock);
                    info!(?state, "internal heartbeat: got connection state!");
                }
                _ = token.cancelled() => {
                    info!("internal heartbeat: shutting down internal heartbeat app");
                    break;
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn build_app_without_pause_resume_stream(
        metadata: Arc<ServerMetadata>,
        concurrency_limit: usize,
        cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
        decryption_key: Arc<VeritechDecryptionKey>,
        cyclone_client_execution_timeout: Duration,
        nats: Arc<Mutex<NatsClient>>,
        kill_senders: Arc<Mutex<HashMap<ExecutionId, oneshot::Sender<()>>>>,
        token: CancellationToken,
    ) -> ServerResult<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
        let connection_metadata = nats.lock().await.metadata_clone();

        // Take the *active* subject prefix from the connected NATS client
        let prefix = nats
            .lock()
            .await
            .metadata()
            .subject_prefix()
            .map(|s| s.to_owned());

        let incoming = {
            let context = jetstream::new(nats.lock().await.clone());
            veritech_work_queue(&context, prefix.as_deref())
                .await?
                .create_consumer(Self::incoming_consumer_config(prefix.as_deref()))
                .await?
                // TODO(nick): disable jetstream heartbeat from Jarret's recommendation
                // .stream()
                // .heartbeat(Duration::from_secs(0))
                .messages()
                .await?
        };

        let state = AppState::new(
            metadata,
            cyclone_pool,
            decryption_key,
            cyclone_client_execution_timeout,
            nats,
            kill_senders,
            None,
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
            .layer(AckLayer::new())
            .service(handlers::process_request.with_state(state))
            .map_response(Response::into_response);

        let inner =
            naxum::serve_with_incoming_limit(incoming, app.into_make_service(), concurrency_limit)
                .with_graceful_shutdown(naxum::wait_on_cancelled(token));

        Ok(Box::new(inner.into_future()))
    }

    // #[allow(clippy::too_many_arguments)]
    // async fn build_app(
    //     metadata: Arc<ServerMetadata>,
    //     concurrency_limit: usize,
    //     cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    //     decryption_key: Arc<VeritechDecryptionKey>,
    //     cyclone_client_execution_timeout: Duration,
    //     nats: NatsClient,
    //     kill_senders: Arc<Mutex<HashMap<ExecutionId, oneshot::Sender<()>>>>,
    //     token: CancellationToken,
    //     pause_duration: Duration,
    //     reconnect_backoff_duration: Duration,
    // ) -> ServerResult<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
    //     let connection_metadata = nats.metadata_clone();

    //     // Take the *active* subject prefix from the connected NATS client
    //     let prefix = nats.metadata().subject_prefix().map(|s| s.to_owned());

    //     let incoming = {
    //         let context = jetstream::new(nats.clone());
    //         PauseResumeStream::new(
    //             veritech_work_queue(&context, prefix.as_deref())
    //                 .await?
    //                 .create_consumer(Self::incoming_consumer_config(prefix.as_deref()))
    //                 .await?,
    //             pause_duration,
    //             reconnect_backoff_duration,
    //         )
    //     };

    //     let state = AppState::new(
    //         metadata,
    //         cyclone_pool,
    //         decryption_key,
    //         cyclone_client_execution_timeout,
    //         nats,
    //         kill_senders,
    //         Some(incoming.controller()),
    //     );

    //     let app = ServiceBuilder::new()
    //         .layer(
    //             MatchedSubjectLayer::new()
    //                 .for_subject(VeritechForSubject::with_prefix(prefix.as_deref())),
    //         )
    //         .layer(
    //             TraceLayer::new()
    //                 .make_span_with(
    //                     telemetry_nats::NatsMakeSpan::builder(connection_metadata).build(),
    //                 )
    //                 .on_response(telemetry_nats::NatsOnResponse::new()),
    //         )
    //         .layer(AckLayer::new())
    //         .service(handlers::process_request.with_state(state))
    //         .map_response(Response::into_response);

    //     let inner =
    //         naxum::serve_with_incoming_limit(incoming, app.into_make_service(), concurrency_limit)
    //             .with_graceful_shutdown(naxum::wait_on_cancelled(token));

    //     Ok(Box::new(inner.into_future()))
    // }

    async fn build_kill_app(
        metadata: Arc<ServerMetadata>,
        nats: Arc<Mutex<NatsClient>>,
        kill_senders: Arc<Mutex<HashMap<ExecutionId, oneshot::Sender<()>>>>,
        token: CancellationToken,
    ) -> ServerResult<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
        let connection_metadata = nats.lock().await.metadata_clone();

        let incoming = {
            let prefix = nats
                .lock()
                .await
                .metadata()
                .subject_prefix()
                .map(|s| s.to_owned());
            Self::kill_subscriber(nats.clone(), prefix.as_deref())
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
    async fn kill_subscriber(
        nats: Arc<Mutex<NatsClient>>,
        prefix: Option<&str>,
    ) -> ServerResult<Subscriber> {
        // we have to make a dummy request here to get the nats subject from it
        let dummy_request = KillExecutionRequest {
            execution_id: "".into(),
        };
        let subject = dummy_request.nats_subject(prefix, None, None);
        nats.lock()
            .await
            .subscribe(subject.clone())
            .await
            .map_err(|err| ServerError::NatsSubscribe(subject, err))
    }

    #[instrument(name = "veritech.init.connect_to_nats", level = "info", skip_all)]
    async fn connect_to_nats(config: &Config) -> ServerResult<NatsClient> {
        let client = NatsClient::new_for_veritech(config.nats())
            .await
            .map_err(ServerError::NatsClient)?;
        debug!("successfully connected nats client");
        Ok(client)
    }

    #[inline]
    fn incoming_consumer_config(
        subject_prefix: Option<&str>,
    ) -> async_nats::jetstream::consumer::pull::Config {
        async_nats::jetstream::consumer::pull::Config {
            durable_name: Some(CONSUMER_NAME.to_owned()),
            filter_subject: incoming_subject(subject_prefix).to_string(),
            max_deliver: CONSUMER_MAX_DELIVERY,
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
