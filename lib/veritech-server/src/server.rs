use chrono::Utc;

use deadpool_cyclone::CycloneClient;
use deadpool_cyclone::Spec;
use deadpool_cyclone::{
    instance::cyclone::LocalUdsInstance, instance::cyclone::LocalUdsInstanceSpec, ActionRunRequest,
    ActionRunResultSuccess, FunctionResult, FunctionResultFailure, FunctionResultFailureError,
    PoolNoodle, ProgressMessage, ReconciliationRequest, ReconciliationResultSuccess,
    ResolverFunctionRequest, ResolverFunctionResultSuccess, SchemaVariantDefinitionRequest,
    SchemaVariantDefinitionResultSuccess, ValidationRequest, ValidationResultSuccess,
};
use futures::{channel::oneshot, join, StreamExt};
use nats_subscriber::Request;
use si_data_nats::NatsClient;
use std::{io, sync::Arc};
use telemetry::prelude::*;
use thiserror::Error;

use tokio::{
    signal::unix,
    sync::{broadcast, mpsc},
};

use crate::{config::CycloneSpec, Config, FunctionSubscriber, Publisher, PublisherError};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("action run error: {0}")]
    ActionRun(#[from] deadpool_cyclone::ExecutionError<ActionRunResultSuccess>),
    #[error("cyclone error: {0}")]
    Cyclone(#[from] deadpool_cyclone::ClientError),
    #[error("cyclone pool error: {0}")]
    CyclonePool(#[source] Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error("cyclone progress error: {0}")]
    CycloneProgress(#[source] Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error("cyclone spec setup error: {0}")]
    CycloneSetupError(#[source] Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error("cyclone spec builder error: {0}")]
    CycloneSpec(#[source] Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error("error connecting to nats: {0}")]
    NatsConnect(#[source] si_data_nats::NatsError),
    #[error("no reply mailbox found")]
    NoReplyMailboxFound,
    #[error(transparent)]
    Publisher(#[from] PublisherError),
    #[error(transparent)]
    Reconciliation(#[from] deadpool_cyclone::ExecutionError<ReconciliationResultSuccess>),
    #[error(transparent)]
    ResolverFunction(#[from] deadpool_cyclone::ExecutionError<ResolverFunctionResultSuccess>),
    #[error(transparent)]
    SchemaVariantDefinition(
        #[from] deadpool_cyclone::ExecutionError<SchemaVariantDefinitionResultSuccess>,
    ),
    #[error("failed to setup signal handler")]
    Signal(#[source] io::Error),
    #[error(transparent)]
    Subscriber(#[from] nats_subscriber::SubscriberError),
    #[error(transparent)]
    Validation(#[from] deadpool_cyclone::ExecutionError<ValidationResultSuccess>),
    #[error("wrong cyclone spec type for {0} spec: {1:?}")]
    WrongCycloneSpec(&'static str, Box<CycloneSpec>),
}

type ServerResult<T> = Result<T, ServerError>;

pub struct Server {
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    shutdown_broadcast_tx: broadcast::Sender<()>,
    shutdown_tx: mpsc::Sender<ShutdownSource>,
    shutdown_rx: oneshot::Receiver<()>,
    metadata: Arc<ServerMetadata>,
}

impl Server {
    #[instrument(name = "veritech.init.cyclone.http", level = "info", skip_all)]
    pub async fn for_cyclone_http(config: Config) -> ServerResult<Server> {
        match config.cyclone_spec() {
            CycloneSpec::LocalHttp(_spec) => {
                // TODO(fnichol): Hi there! Ultimately, the Veritech server should be able to work
                // with a LocalUds Cyclone backend and a LocalHttp version. But since this involves
                // threading through some generic types which themselves have trait
                // constraints--well we can add this back in the near future... Note that the
                // immediately prior state to this line change is roughly the starting point for
                // adding the types back. Good luck to us all.
                //
                // let nats = connect_to_nats(&config).await?;
                // let manager = Manager::new(spec.clone());
                // let cyclone_pool = Pool::builder(manager)
                //     .build()
                //     .map_err(|err| ServerError::CycloneSpec(Box::new(err)))?;

                // Ok(Server { nats, cyclone_pool })
                unimplemented!("get ready for a surprise!!")
            }
            wrong @ CycloneSpec::LocalUds(_) => Err(ServerError::WrongCycloneSpec(
                "LocalHttp",
                Box::new(wrong.clone()),
            )),
        }
    }

    #[instrument(name = "veritech.init.cyclone.uds", level = "info", skip_all)]
    pub async fn for_cyclone_uds(config: Config) -> ServerResult<Server> {
        match config.cyclone_spec() {
            CycloneSpec::LocalUds(spec) => {
                let (shutdown_tx, shutdown_rx) = mpsc::channel(4);
                // Note the channel parameter corresponds to the number of channels that may be
                // maintained when the sender is guaranteeing delivery. While this number may end
                // of being related to the number of subscribers, it's not
                // necessarily the same number.
                let (shutdown_broadcast_tx, _) = broadcast::channel(16);

                let nats = connect_to_nats(&config).await?;

                let mut cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec> =
                    PoolNoodle::new(
                        spec.pool_size.into(),
                        spec.clone(),
                        shutdown_broadcast_tx.subscribe(),
                    );

                spec.clone()
                    .setup()
                    .await
                    .map_err(|e| ServerError::CycloneSetupError(Box::new(e)))?;
                cyclone_pool.start();

                let metadata = ServerMetadata {
                    job_instance: config.instance_id().into(),
                    job_invoked_provider: "si",
                };

                let graceful_shutdown_rx =
                    prepare_graceful_shutdown(shutdown_rx, shutdown_broadcast_tx.clone())?;

                Ok(Server {
                    nats,
                    subject_prefix: config.subject_prefix().map(|s| s.to_string()),
                    cyclone_pool,
                    shutdown_broadcast_tx,
                    shutdown_tx,
                    shutdown_rx: graceful_shutdown_rx,
                    metadata: Arc::new(metadata),
                })
            }
            wrong @ CycloneSpec::LocalHttp(_) => Err(ServerError::WrongCycloneSpec(
                "LocalUds",
                Box::new(wrong.clone()),
            )),
        }
    }

    /// Gets a shutdown handle that can trigger the server's graceful shutdown process.
    pub fn shutdown_handle(&self) -> VeritechShutdownHandle {
        VeritechShutdownHandle {
            shutdown_tx: self.shutdown_tx.clone(),
        }
    }
}

impl Server {
    pub async fn run(self) -> ServerResult<()> {
        let _ = join!(
            process_resolver_function_requests_task(
                self.metadata.clone(),
                self.nats.clone(),
                self.subject_prefix.clone(),
                self.cyclone_pool.clone(),
                self.shutdown_broadcast_tx.subscribe(),
            ),
            process_validation_requests_task(
                self.metadata.clone(),
                self.nats.clone(),
                self.subject_prefix.clone(),
                self.cyclone_pool.clone(),
                self.shutdown_broadcast_tx.subscribe(),
            ),
            process_action_run_requests_task(
                self.metadata.clone(),
                self.nats.clone(),
                self.subject_prefix.clone(),
                self.cyclone_pool.clone(),
                self.shutdown_broadcast_tx.subscribe(),
            ),
            process_reconciliation_requests_task(
                self.metadata.clone(),
                self.nats.clone(),
                self.subject_prefix.clone(),
                self.cyclone_pool.clone(),
                self.shutdown_broadcast_tx.subscribe(),
            ),
            process_schema_variant_definition_requests_task(
                self.metadata.clone(),
                self.nats.clone(),
                self.subject_prefix.clone(),
                self.cyclone_pool.clone(),
                self.shutdown_broadcast_tx.subscribe(),
            ),
        );

        let _ = self.shutdown_rx.await;
        info!("received graceful shutdown, terminating server instance");

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ServerMetadata {
    job_instance: String,
    job_invoked_provider: &'static str,
}

pub struct VeritechShutdownHandle {
    shutdown_tx: mpsc::Sender<ShutdownSource>,
}

impl VeritechShutdownHandle {
    pub async fn shutdown(self) {
        if let Err(err) = self.shutdown_tx.send(ShutdownSource::Handle).await {
            warn!(error = ?err, "shutdown tx returned error, receiver is likely already closed");
        }
    }
}

// NOTE(fnichol): resolver function, action are parallel and extremely similar, so there
// is a lurking "unifying" refactor here. It felt like waiting until the third time adding one of
// these would do the trick, and as a result the first 2 impls are here and not split apart into
// their own modules.

async fn process_resolver_function_requests_task(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    shutdown_broadcast_rx: broadcast::Receiver<()>,
) {
    if let Err(err) = process_resolver_function_requests(
        metadata,
        nats,
        subject_prefix,
        cyclone_pool,
        shutdown_broadcast_rx,
    )
    .await
    {
        warn!(error = ?err, "processing resolver function requests failed");
    }
}

async fn process_resolver_function_requests(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    mut shutdown_broadcast_rx: broadcast::Receiver<()>,
) -> ServerResult<()> {
    let mut requests =
        FunctionSubscriber::resolver_function(&nats, subject_prefix.as_deref()).await?;

    loop {
        tokio::select! {
            // Got a broadcasted shutdown message
            _ = shutdown_broadcast_rx.recv() => {
                trace!("process resolver function requests task received shutdown");
                break;
            }
            // Got the next message on from the subscriber
            request = requests.next() => {
                match request {
                    Some(Ok(request)) => {
                        // Spawn a task an process the request
                        tokio::spawn(resolver_function_request_task(
                            metadata.clone(),
                            nats.clone(),
                            cyclone_pool.clone(),
                            request,
                        ));
                    }
                    Some(Err(err)) => {
                        warn!(error = ?err, "next resolver function request had error");
                    }
                    None => {
                        trace!("resolver function requests subscriber stream has closed");
                        break;
                    }
                }
            }
            // All other arms are closed, nothing left to do but return
            else => {
                trace!("returning with all select arms closed");
                break
            }
        }
    }

    // Unsubscribe from subscriber without draining the channel
    requests.unsubscribe_after(0).await?;

    Ok(())
}

async fn resolver_function_request_task(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    request: Request<ResolverFunctionRequest>,
) {
    let cyclone_request = request.payload;

    let reply_mailbox = match request.reply {
        Some(reply) => reply,
        None => {
            error!("no reply mailbox found");
            return;
        }
    };
    let execution_id = cyclone_request.execution_id.clone();
    let publisher = Publisher::new(&nats, &reply_mailbox);

    let function_result = resolver_function_request(
        metadata,
        &publisher,
        cyclone_pool,
        cyclone_request,
        &request.process_span,
    )
    .await;

    if let Err(err) = publisher.finalize_output().await {
        error!(error = ?err, "failed to finalize output by sending final message");
        let result = deadpool_cyclone::FunctionResult::Failure::<ResolverFunctionResultSuccess>(
            FunctionResultFailure {
                execution_id,
                error: FunctionResultFailureError {
                    kind: "veritechServer".to_string(),
                    message: "failed to finalize output by sending final message".to_string(),
                },
                timestamp: timestamp(),
            },
        );
        if let Err(err) = publisher.publish_result(&result).await {
            error!(error = ?err, "failed to publish errored result");
        }
        return;
    }

    let function_result = match function_result {
        Ok(fr) => fr,
        Err(err) => {
            error!(error = ?err, "failure trying to run function to completion");
            deadpool_cyclone::FunctionResult::Failure::<ResolverFunctionResultSuccess>(
                FunctionResultFailure {
                    execution_id,
                    error: FunctionResultFailureError {
                        kind: "veritechServer".to_string(),
                        message: err.to_string(),
                    },
                    timestamp: timestamp(),
                },
            )
        }
    };

    if let Err(err) = publisher.publish_result(&function_result).await {
        error!(error = ?err, "failed to publish result");
    };
}

#[instrument(
    name = "veritech.resolver_function_request",
    parent = process_span,
    level = "info",
    skip_all,
    fields(
        job.id = &cyclone_request.execution_id,
        job.instance = metadata.job_instance,
        job.invoked_name = &cyclone_request.handler,
        job.invoked_provider = metadata.job_invoked_provider,
        otel.kind = SpanKind::Server.as_str(),
        otel.status_code = Empty,
        otel.status_message = Empty,
    )
)]
async fn resolver_function_request(
    metadata: Arc<ServerMetadata>,
    publisher: &Publisher<'_>,
    mut cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    cyclone_request: ResolverFunctionRequest,
    process_span: &Span,
) -> ServerResult<FunctionResult<ResolverFunctionResultSuccess>> {
    let span = Span::current();

    let mut client = cyclone_pool
        .get()
        .await
        .map_err(|err| span.record_err(ServerError::CyclonePool(Box::new(err))))?;

    let mut progress = client
        .execute_resolver(cyclone_request)
        .await
        .map_err(|err| span.record_err(err))?
        .start()
        .await
        .map_err(|err| span.record_err(err))?;

    while let Some(msg) = progress.next().await {
        match msg {
            Ok(ProgressMessage::OutputStream(output)) => {
                publisher
                    .publish_output(&output)
                    .await
                    .map_err(|err| span.record_err(err))?;
            }
            Ok(ProgressMessage::Heartbeat) => {
                trace!("received heartbeat message");
                publisher
                    .publish_keep_alive()
                    .await
                    .map_err(|err| span.record_err(err))?;
            }
            Err(err) => {
                warn!(error = ?err, "next progress message was an error, bailing out");
                break;
            }
        }
    }

    let function_result = progress
        .finish()
        .await
        .map_err(|err| span.record_err(err))?;

    span.record_ok();
    Ok(function_result)
}

async fn process_validation_requests_task(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    shutdown_broadcast_rx: broadcast::Receiver<()>,
) {
    if let Err(err) = process_validation_requests(
        metadata,
        nats,
        subject_prefix,
        cyclone_pool,
        shutdown_broadcast_rx,
    )
    .await
    {
        warn!(error = ?err, "processing validation requests failed");
    }
}

async fn process_validation_requests(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    mut shutdown_broadcast_rx: broadcast::Receiver<()>,
) -> ServerResult<()> {
    let mut requests = FunctionSubscriber::validation(&nats, subject_prefix.as_deref()).await?;

    loop {
        tokio::select! {
            // Got a broadcasted shutdown message
            _ = shutdown_broadcast_rx.recv() => {
                trace!("process validation requests task received shutdown");
                break;
            }
            // Got the next message on from the subscriber
            request = requests.next() => {
                match request {
                    Some(Ok(request)) => {
                        // Spawn a task an process the request
                        tokio::spawn(validation_request_task(
                            metadata.clone(),
                            nats.clone(),
                            cyclone_pool.clone(),
                            request,
                        ));
                    }
                    Some(Err(err)) => {
                        warn!(error = ?err, "next validation request had error");
                    }
                    None => {
                        trace!("validation requests subscriber stream has closed");
                        break;
                    }
                }
            }
            // All other arms are closed, nothing left to do but return
            else => {
                trace!("returning with all select arms closed");
                break
            }
        }
    }

    // Unsubscribe from subscriber without draining the channel
    requests.unsubscribe_after(0).await?;

    Ok(())
}

async fn validation_request_task(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    request: Request<ValidationRequest>,
) {
    let process_span = request.process_span.clone();
    if let Err(err) = validation_request(metadata, nats, cyclone_pool, request, &process_span).await
    {
        warn!(error = ?err, "validation execution failed");
    }
}

#[instrument(
    name = "veritech.validation_request",
    parent = process_span,
    level = "info",
    skip_all,
    fields(
        job.id = &request.payload.execution_id,
        job.instance = metadata.job_instance,
        job.invoked_name = &request.payload.handler,
        job.invoked_provider = metadata.job_invoked_provider,
        otel.kind = SpanKind::Server.as_str(),
        otel.status_code = Empty,
        otel.status_message = Empty,
    )
)]
async fn validation_request(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    mut cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    request: Request<ValidationRequest>,
    process_span: &Span,
) -> ServerResult<()> {
    let span = Span::current();

    let cyclone_request = request.payload;

    let reply_mailbox = request
        .reply
        .ok_or(ServerError::NoReplyMailboxFound)
        .map_err(|err| span.record_err(err))?;

    let publisher = Publisher::new(&nats, &reply_mailbox);
    let mut client = cyclone_pool
        .get()
        .await
        .map_err(|err| span.record_err(ServerError::CyclonePool(Box::new(err))))?;

    let mut progress = client
        .execute_validation(cyclone_request)
        .await
        .map_err(|err| span.record_err(err))?
        .start()
        .await
        .map_err(|err| span.record_err(err))?;

    while let Some(msg) = progress.next().await {
        match msg {
            Ok(ProgressMessage::OutputStream(output)) => {
                publisher
                    .publish_output(&output)
                    .await
                    .map_err(|err| span.record_err(err))?;
            }
            Ok(ProgressMessage::Heartbeat) => {
                trace!("received heartbeat message");
            }
            Err(err) => {
                warn!(error = ?err, "next progress message was an error, bailing out");
                break;
            }
        }
    }
    publisher
        .finalize_output()
        .await
        .map_err(|err| span.record_err(err))?;

    let function_result = progress
        .finish()
        .await
        .map_err(|err| span.record_err(err))?;
    publisher
        .publish_result(&function_result)
        .await
        .map_err(|err| span.record_err(err))?;

    span.record_ok();
    Ok(())
}

async fn process_schema_variant_definition_requests_task(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    shutdown_broadcast_rx: broadcast::Receiver<()>,
) {
    if let Err(err) = process_schema_variant_definition_requests(
        metadata,
        nats,
        subject_prefix,
        cyclone_pool,
        shutdown_broadcast_rx,
    )
    .await
    {
        warn!(error = ?err, "processing schema variant definition requests failed");
    }
}

async fn process_schema_variant_definition_requests(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    mut shutdown_broadcast_rx: broadcast::Receiver<()>,
) -> ServerResult<()> {
    let mut requests =
        FunctionSubscriber::schema_variant_definition(&nats, subject_prefix.as_deref()).await?;

    loop {
        tokio::select! {
            // Got a broadcasted shutdown message
            _ = shutdown_broadcast_rx.recv() => {
                trace!("process schema_variant_definition requests task received shutdown");
                break;
            }
            // Got the next message on from the subscriber
            request = requests.next() => {
                match request {
                    Some(Ok(request)) => {
                        // Spawn a task an process the request
                        tokio::spawn(schema_variant_definition_request_task(
                            metadata.clone(),
                            nats.clone(),
                            cyclone_pool.clone(),
                            request,
                        ));
                    }
                    Some(Err(err)) => {
                        warn!(error = ?err, "next schema variant definition request had error");
                    }
                    None => {
                        trace!("schema variant definition requests subscriber stream has closed");
                        break;
                    }
                }
            }
            // All other arms are closed, nothing left to do but return
            else => {
                trace!("returning with all select arms closed");
                break
            }
        }
    }

    // Unsubscribe from subscriber without draining the channel
    requests.unsubscribe_after(0).await?;

    Ok(())
}

async fn schema_variant_definition_request_task(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    request: Request<SchemaVariantDefinitionRequest>,
) {
    let process_span = request.process_span.clone();
    if let Err(err) =
        schema_variant_definition_request(metadata, nats, cyclone_pool, request, &process_span)
            .await
    {
        warn!(error = ?err, "schema variant definition execution failed");
    }
}

#[instrument(
    name = "veritech.schema_variant_definition_request",
    parent = process_span,
    level = "info",
    skip_all,
    fields(
        job.id = &request.payload.execution_id,
        job.instance = metadata.job_instance,
        job.invoked_name = &request.payload.handler,
        job.invoked_provider = metadata.job_invoked_provider,
        otel.kind = SpanKind::Server.as_str(),
        otel.status_code = Empty,
        otel.status_message = Empty,
    )
)]
async fn schema_variant_definition_request(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    mut cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    request: Request<SchemaVariantDefinitionRequest>,
    process_span: &Span,
) -> ServerResult<()> {
    let span = Span::current();

    let cyclone_request = request.payload;
    let reply_mailbox = request
        .reply
        .ok_or(ServerError::NoReplyMailboxFound)
        .map_err(|err| span.record_err(err))?;

    let publisher = Publisher::new(&nats, &reply_mailbox);
    let mut client = cyclone_pool
        .get()
        .await
        .map_err(|err| span.record_err(ServerError::CyclonePool(Box::new(err))))?;

    let mut progress = client
        .execute_schema_variant_definition(cyclone_request)
        .await
        .map_err(|err| span.record_err(err))?
        .start()
        .await
        .map_err(|err| span.record_err(err))?;

    while let Some(msg) = progress.next().await {
        match msg {
            Ok(ProgressMessage::OutputStream(output)) => {
                publisher
                    .publish_output(&output)
                    .await
                    .map_err(|err| span.record_err(err))?;
            }
            Ok(ProgressMessage::Heartbeat) => {
                trace!("received heartbeat message");
            }
            Err(err) => {
                warn!(error = ?err, "next progress message was an error, bailing out");
                break;
            }
        }
    }
    publisher
        .finalize_output()
        .await
        .map_err(|err| span.record_err(err))?;

    let function_result = progress
        .finish()
        .await
        .map_err(|err| span.record_err(err))?;
    publisher
        .publish_result(&function_result)
        .await
        .map_err(|err| span.record_err(err))?;

    span.record_ok();
    Ok(())
}

async fn process_action_run_requests_task(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    shutdown_broadcast_rx: broadcast::Receiver<()>,
) {
    if let Err(err) = process_action_run_requests(
        metadata,
        nats,
        subject_prefix,
        cyclone_pool,
        shutdown_broadcast_rx,
    )
    .await
    {
        warn!(error = ?err, "processing action run requests failed");
    }
}

async fn process_action_run_requests(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    mut shutdown_broadcast_rx: broadcast::Receiver<()>,
) -> ServerResult<()> {
    let mut requests = FunctionSubscriber::action_run(&nats, subject_prefix.as_deref()).await?;

    loop {
        tokio::select! {
            // Got a broadcasted shutdown message
            _ = shutdown_broadcast_rx.recv() => {
                trace!("process action_run requests task received shutdown");
                break;
            }
            // Got the next message on from the subscriber
            request = requests.next() => {
                match request {
                    Some(Ok(request)) => {
                        // Spawn a task an process the request
                        tokio::spawn(action_run_request_task(
                            metadata.clone(),
                            nats.clone(),
                            cyclone_pool.clone(),
                            request,
                        ));
                    }
                    Some(Err(err)) => {
                        warn!(error = ?err, "next action run request had error");
                    }
                    None => {
                        trace!("action run requests subscriber stream has closed");
                        break;
                    }
                }
            }
            // All other arms are closed, nothing left to do but return
            else => {
                trace!("returning with all select arms closed");
                break
            }
        }
    }

    // Unsubscribe from subscriber without draining the channel
    requests.unsubscribe_after(0).await?;

    Ok(())
}

async fn action_run_request_task(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    request: Request<ActionRunRequest>,
) {
    let process_span = request.process_span.clone();
    if let Err(err) = action_run_request(metadata, nats, cyclone_pool, request, &process_span).await
    {
        warn!(error = ?err, "action run execution failed");
    }
}

#[instrument(
    name = "veritech.action_run_request",
    parent = process_span,
    level = "info",
    skip_all,
    fields(
        job.id = &request.payload.execution_id,
        job.instance = metadata.job_instance,
        job.invoked_name = &request.payload.handler,
        job.invoked_provider = metadata.job_invoked_provider,
        otel.kind = SpanKind::Server.as_str(),
        otel.status_code = Empty,
        otel.status_message = Empty,
    )
)]
async fn action_run_request(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    mut cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    request: Request<ActionRunRequest>,
    process_span: &Span,
) -> ServerResult<()> {
    let span = Span::current();

    let cyclone_request = request.payload;
    let reply_mailbox = request
        .reply
        .ok_or(ServerError::NoReplyMailboxFound)
        .map_err(|err| span.record_err(err))?;

    let publisher = Publisher::new(&nats, &reply_mailbox);
    let mut client = cyclone_pool
        .get()
        .await
        .map_err(|err| span.record_err(ServerError::CyclonePool(Box::new(err))))?;

    let mut progress = client
        .execute_action_run(cyclone_request)
        .await
        .map_err(|err| span.record_err(err))?
        .start()
        .await
        .map_err(|err| span.record_err(err))?;

    while let Some(msg) = progress.next().await {
        match msg {
            Ok(ProgressMessage::OutputStream(output)) => {
                publisher
                    .publish_output(&output)
                    .await
                    .map_err(|err| span.record_err(err))?;
            }
            Ok(ProgressMessage::Heartbeat) => {
                trace!("received heartbeat message");
            }
            Err(err) => {
                warn!(error = ?err, "next progress message was an error, bailing out");
                break;
            }
        }
    }
    publisher
        .finalize_output()
        .await
        .map_err(|err| span.record_err(err))?;

    let function_result = progress
        .finish()
        .await
        .map_err(|err| span.record_err(err))?;
    publisher
        .publish_result(&function_result)
        .await
        .map_err(|err| span.record_err(err))?;

    span.record_ok();
    Ok(())
}

async fn process_reconciliation_requests_task(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    shutdown_broadcast_rx: broadcast::Receiver<()>,
) {
    if let Err(err) = process_reconciliation_requests(
        metadata,
        nats,
        subject_prefix,
        cyclone_pool,
        shutdown_broadcast_rx,
    )
    .await
    {
        warn!(error = ?err, "processing reconciliation requests failed");
    }
}

async fn process_reconciliation_requests(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    mut shutdown_broadcast_rx: broadcast::Receiver<()>,
) -> ServerResult<()> {
    let mut requests = FunctionSubscriber::reconciliation(&nats, subject_prefix.as_deref()).await?;

    loop {
        tokio::select! {
            // Got a broadcasted shutdown message
            _ = shutdown_broadcast_rx.recv() => {
                trace!("process reconciliation requests task received shutdown");
                break;
            }
            // Got the next message on from the subscriber
            request = requests.next() => {
                match request {
                    Some(Ok(request)) => {
                        // Spawn a task an process the request
                        tokio::spawn(reconciliation_request_task(
                            metadata.clone(),
                            nats.clone(),
                            cyclone_pool.clone(),
                            request,
                        ));
                    }
                    Some(Err(err)) => {
                        warn!(error = ?err, "next reconciliation request had error");
                    }
                    None => {
                        trace!("reconciliation requests subscriber stream has closed");
                        break;
                    }
                }
            }
            // All other arms are closed, nothing left to do but return
            else => {
                trace!("returning with all select arms closed");
                break
            }
        }
    }

    // Unsubscribe from subscriber without draining the channel
    requests.unsubscribe_after(0).await?;

    Ok(())
}

async fn reconciliation_request_task(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    request: Request<ReconciliationRequest>,
) {
    let process_span = request.process_span.clone();
    if let Err(err) =
        reconciliation_request(metadata, nats, cyclone_pool, request, &process_span).await
    {
        warn!(error = ?err, "reconciliation execution failed");
    }
}

#[instrument(
    name = "veritech.reconciliation_request",
    parent = process_span,
    level = "info",
    skip_all,
    fields(
        job.id = &request.payload.execution_id,
        job.instance = metadata.job_instance,
        job.invoked_name = &request.payload.handler,
        job.invoked_provider = metadata.job_invoked_provider,
        otel.kind = SpanKind::Server.as_str(),
        otel.status_code = Empty,
        otel.status_message = Empty,
    )
)]
async fn reconciliation_request(
    metadata: Arc<ServerMetadata>,
    nats: NatsClient,
    mut cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
    request: Request<ReconciliationRequest>,
    process_span: &Span,
) -> ServerResult<()> {
    let span = Span::current();

    let cyclone_request = request.payload;
    let reply_mailbox = request
        .reply
        .ok_or(ServerError::NoReplyMailboxFound)
        .map_err(|err| span.record_err(err))?;

    let publisher = Publisher::new(&nats, &reply_mailbox);

    let mut client = cyclone_pool
        .get()
        .await
        .map_err(|err| span.record_err(ServerError::CyclonePool(Box::new(err))))?;

    let mut progress = client
        .execute_reconciliation(cyclone_request)
        .await
        .map_err(|err| span.record_err(err))?
        .start()
        .await
        .map_err(|err| span.record_err(err))?;

    while let Some(msg) = progress.next().await {
        match msg {
            Ok(ProgressMessage::OutputStream(output)) => {
                publisher
                    .publish_output(&output)
                    .await
                    .map_err(|err| span.record_err(err))?;
            }
            Ok(ProgressMessage::Heartbeat) => {
                trace!("received heartbeat message");
            }
            Err(err) => {
                warn!(error = ?err, "next progress message was an error, bailing out");
                break;
            }
        }
    }
    publisher
        .finalize_output()
        .await
        .map_err(|err| span.record_err(err))?;

    let function_result = progress
        .finish()
        .await
        .map_err(|err| span.record_err(err))?;
    publisher
        .publish_result(&function_result)
        .await
        .map_err(|err| span.record_err(err))?;

    span.record_ok();
    Ok(())
}

async fn connect_to_nats(config: &Config) -> ServerResult<NatsClient> {
    info!("connecting to NATS; url={}", config.nats().url);

    let nats = NatsClient::new(config.nats())
        .await
        .map_err(ServerError::NatsConnect)?;

    Ok(nats)
}

fn prepare_graceful_shutdown(
    mut shutdown_rx: mpsc::Receiver<ShutdownSource>,
    shutdown_broadcast_tx: broadcast::Sender<()>,
) -> ServerResult<oneshot::Receiver<()>> {
    let (graceful_shutdown_tx, graceful_shutdown_rx) = oneshot::channel::<()>();
    let mut sigterm_stream =
        unix::signal(unix::SignalKind::terminate()).map_err(ServerError::Signal)?;

    tokio::spawn(async move {
        fn send_graceful_shutdown(
            tx: oneshot::Sender<()>,
            shutdown_broadcast_tx: broadcast::Sender<()>,
        ) {
            // Send shutdown to all long running subscribers, so they can cleanly terminate
            if shutdown_broadcast_tx.send(()).is_err() {
                error!("all broadcast shutdown receivers have already been dropped");
            }
            // Send graceful shutdown to main server thread which stops it from accepting requests.
            // We'll do this step last so as to let all subscribers have a chance to shutdown.
            if tx.send(()).is_err() {
                error!("the server graceful shutdown receiver has already dropped");
            }
        }

        tokio::select! {
            _ = sigterm_stream.recv() => {
                trace!("received SIGTERM signal, performing graceful shutdown");
                send_graceful_shutdown(graceful_shutdown_tx, shutdown_broadcast_tx);
            }
            source = shutdown_rx.recv() => {
                trace!(
                    "received internal shutdown, performing graceful shutdown; source={:?}",
                    source,
                );
                send_graceful_shutdown(graceful_shutdown_tx, shutdown_broadcast_tx);
            }
            else => {
                // All other arms are closed, nothing left to do but return
                trace!("returning from graceful shutdown with all select arms closed");
            }
        };
    });

    Ok(graceful_shutdown_rx)
}

pub fn timestamp() -> u64 {
    u64::try_from(std::cmp::max(Utc::now().timestamp(), 0)).expect("timestamp not be negative")
}

#[remain::sorted]
#[derive(Debug, Eq, PartialEq)]
pub enum ShutdownSource {
    Handle,
}

impl Default for ShutdownSource {
    fn default() -> Self {
        Self::Handle
    }
}
