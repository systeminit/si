use deadpool_cyclone::{
    instance::cyclone::LocalUdsInstanceSpec, CodeGenerationRequest, CodeGenerationResultSuccess,
    CycloneClient, Manager, Pool, ProgressMessage, QualificationCheckRequest,
    QualificationCheckResultSuccess, ResolverFunctionRequest, ResolverFunctionResultSuccess,
    ResourceSyncRequest, ResourceSyncResultSuccess,
};
use futures::{channel::oneshot, join, StreamExt};
use si_data::NatsClient;
use std::io;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    signal::unix,
    sync::{broadcast, mpsc},
};

use super::{
    config::CycloneSpec, Config, Publisher, PublisherError, Request, Subscriber, SubscriberError,
};

#[derive(Error, Debug)]
pub enum ServerError {
    #[error(transparent)]
    Cyclone(#[from] deadpool_cyclone::client::ClientError),
    #[error("cyclone pool error")]
    CyclonePool(#[source] Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error("cyclone progress error")]
    CycloneProgress(#[source] Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error("cyclone spec builder error")]
    CycloneSpec(#[source] Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error("error connecting to nats")]
    NatsConnect(#[source] si_data::NatsError),
    #[error(transparent)]
    Publisher(#[from] PublisherError),
    #[error(transparent)]
    QualificationCheck(
        #[from] deadpool_cyclone::client::ExecutionError<QualificationCheckResultSuccess>,
    ),
    #[error(transparent)]
    ResolverFunction(
        #[from] deadpool_cyclone::client::ExecutionError<ResolverFunctionResultSuccess>,
    ),
    #[error(transparent)]
    ResourceSync(#[from] deadpool_cyclone::client::ExecutionError<ResourceSyncResultSuccess>),
    #[error(transparent)]
    CodeGeneration(#[from] deadpool_cyclone::client::ExecutionError<CodeGenerationResultSuccess>),
    #[error("failed to setup signal handler")]
    Signal(#[source] io::Error),
    #[error(transparent)]
    Subscriber(#[from] SubscriberError),
    #[error("wrong cyclone spec type for {0} spec: {1:?}")]
    WrongCycloneSpec(&'static str, CycloneSpec),
}

type Result<T> = std::result::Result<T, ServerError>;

pub struct Server {
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
    shutdown_broadcast_tx: broadcast::Sender<()>,
    shutdown_tx: mpsc::Sender<ShutdownSource>,
    shutdown_rx: oneshot::Receiver<()>,
}

impl Server {
    #[instrument(name = "veritech.init.cyclone.http", skip(config))]
    pub async fn for_cyclone_http(config: Config) -> Result<Server> {
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
            wrong @ CycloneSpec::LocalUds(_) => {
                Err(ServerError::WrongCycloneSpec("LocalHttp", wrong.clone()))
            }
        }
    }

    #[instrument(name = "veritech.init.cyclone.uds", skip(config))]
    pub async fn for_cyclone_uds(config: Config) -> Result<Server> {
        match config.cyclone_spec() {
            CycloneSpec::LocalUds(spec) => {
                let (shutdown_tx, shutdown_rx) = mpsc::channel(4);
                // Note the channel parameter corresponds to the number of channels that may be
                // maintained when the sender is guaranteeing delivery. While this number may end
                // of being related to the number of subscriptions, it's not
                // necessarily the same number.
                let (shutdown_broadcast_tx, _) = broadcast::channel(16);

                let nats = connect_to_nats(&config).await?;
                let manager = Manager::new(spec.clone());
                let cyclone_pool = Pool::builder(manager)
                    .build()
                    .map_err(|err| ServerError::CycloneSpec(Box::new(err)))?;

                let graceful_shutdown_rx =
                    prepare_graceful_shutdown(shutdown_rx, shutdown_broadcast_tx.clone())?;

                Ok(Server {
                    nats,
                    subject_prefix: config.subject_prefix().map(|s| s.to_string()),
                    cyclone_pool,
                    shutdown_broadcast_tx,
                    shutdown_tx,
                    shutdown_rx: graceful_shutdown_rx,
                })
            }
            wrong @ CycloneSpec::LocalHttp(_) => {
                Err(ServerError::WrongCycloneSpec("LocalUds", wrong.clone()))
            }
        }
    }

    /// Gets a shutdown handle that can trigger the server's graceful shutdown process.
    pub fn shutdown_handle(&self) -> ShutdownHandle {
        ShutdownHandle {
            shutdown_tx: self.shutdown_tx.clone(),
        }
    }
}

impl Server {
    pub async fn run(self) -> Result<()> {
        let _ = join!(
            process_qualification_check_requests_task(
                self.nats.clone(),
                self.subject_prefix.clone(),
                self.cyclone_pool.clone(),
                self.shutdown_broadcast_tx.subscribe(),
            ),
            process_resolver_function_requests_task(
                self.nats.clone(),
                self.subject_prefix.clone(),
                self.cyclone_pool.clone(),
                self.shutdown_broadcast_tx.subscribe(),
            ),
            process_resource_sync_requests_task(
                self.nats.clone(),
                self.subject_prefix.clone(),
                self.cyclone_pool.clone(),
                self.shutdown_broadcast_tx.subscribe(),
            ),
            process_code_generation_requests_task(
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

pub struct ShutdownHandle {
    shutdown_tx: mpsc::Sender<ShutdownSource>,
}

impl ShutdownHandle {
    pub async fn shutdown(self) {
        if let Err(err) = self.shutdown_tx.send(ShutdownSource::Handle).await {
            warn!(error = ?err, "shutdown tx returned error, receiver is likely already closed");
        }
    }
}

// NOTE(fnichol): the resolver_function, qualification_check, code_generation and resource_sync paths are parallel and extremely
// similar, so there is a lurking "unifying" refactor here. It felt like waiting until the third
// time adding one of these would do the trick, and as a result the first 2 impls are here and not
// split apart into their own modules.

async fn process_resolver_function_requests_task(
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
    shutdown_broadcast_rx: broadcast::Receiver<()>,
) {
    if let Err(err) = process_resolver_function_requests(
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
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
    mut shutdown_broadcast_rx: broadcast::Receiver<()>,
) -> Result<()> {
    let mut requests = Subscriber::resolver_function(&nats, subject_prefix.as_deref()).await?;

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

    // Unsubscribe from subscription
    requests.unsubscribe().await?;

    Ok(())
}

async fn resolver_function_request_task(
    nats: NatsClient,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
    request: Request<ResolverFunctionRequest>,
) {
    if let Err(err) = resolver_function_request(nats, cyclone_pool, request).await {
        warn!(error = ?err, "resolver function execution failed");
    }
}

async fn resolver_function_request(
    nats: NatsClient,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
    request: Request<ResolverFunctionRequest>,
) -> Result<()> {
    let (reply_mailbox, cyclone_request) = request.into_parts();

    let publisher = Publisher::new(&nats, &reply_mailbox);
    let mut client = cyclone_pool
        .get()
        .await
        .map_err(|err| ServerError::CyclonePool(Box::new(err)))?;
    let mut progress = client
        .execute_resolver(cyclone_request)
        .await?
        .start()
        .await?;

    while let Some(msg) = progress.next().await {
        match msg {
            Ok(ProgressMessage::OutputStream(output)) => {
                publisher.publish_output(&output).await?;
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
    publisher.finalize_output().await?;

    let function_result = progress.finish().await?;
    publisher.publish_result(&function_result).await?;

    Ok(())
}

async fn process_qualification_check_requests_task(
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
    shutdown_broadcast_rx: broadcast::Receiver<()>,
) {
    if let Err(err) = process_qualification_check_requests(
        nats,
        subject_prefix,
        cyclone_pool,
        shutdown_broadcast_rx,
    )
    .await
    {
        warn!(error = ?err, "processing qualification check requests failed");
    }
}

async fn process_qualification_check_requests(
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
    mut shutdown_broadcast_rx: broadcast::Receiver<()>,
) -> Result<()> {
    let mut requests = Subscriber::qualification_check(&nats, subject_prefix.as_deref()).await?;

    loop {
        tokio::select! {
            // Got a broadcasted shutdown message
            _ = shutdown_broadcast_rx.recv() => {
                trace!("process qualification check requests task received shutdown");
                break;
            }
            // Got the next message on from the subscriber
            request = requests.next() => {
                match request {
                    Some(Ok(request)) => {
                        // Spawn a task an process the request
                        tokio::spawn(qualification_check_request_task(
                            nats.clone(),
                            cyclone_pool.clone(),
                            request,
                        ));
                    }
                    Some(Err(err)) => {
                        warn!(error = ?err, "next qualification check request had error");
                    }
                    None => {
                        trace!("qualification check requests subscriber stream has closed");
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

    // Unsubscribe from subscription
    requests.unsubscribe().await?;

    Ok(())
}

async fn qualification_check_request_task(
    nats: NatsClient,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
    request: Request<QualificationCheckRequest>,
) {
    if let Err(err) = qualification_check_request(nats, cyclone_pool, request).await {
        warn!(error = ?err, "qualification check execution failed");
    }
}

async fn qualification_check_request(
    nats: NatsClient,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
    request: Request<QualificationCheckRequest>,
) -> Result<()> {
    let (reply_mailbox, cyclone_request) = request.into_parts();

    let publisher = Publisher::new(&nats, &reply_mailbox);
    let mut client = cyclone_pool
        .get()
        .await
        .map_err(|err| ServerError::CyclonePool(Box::new(err)))?;
    let mut progress = client
        .execute_qualification(cyclone_request)
        .await?
        .start()
        .await?;

    while let Some(msg) = progress.next().await {
        match msg {
            Ok(ProgressMessage::OutputStream(output)) => {
                publisher.publish_output(&output).await?;
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
    publisher.finalize_output().await?;

    let function_result = progress.finish().await?;
    publisher.publish_result(&function_result).await?;

    Ok(())
}

async fn process_resource_sync_requests_task(
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
    shutdown_broadcast_rx: broadcast::Receiver<()>,
) {
    if let Err(err) =
        process_resource_sync_requests(nats, subject_prefix, cyclone_pool, shutdown_broadcast_rx)
            .await
    {
        warn!(error = ?err, "processing resource sync requests failed");
    }
}

async fn process_resource_sync_requests(
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
    mut shutdown_broadcast_rx: broadcast::Receiver<()>,
) -> Result<()> {
    let mut requests = Subscriber::resource_sync(&nats, subject_prefix.as_deref()).await?;

    loop {
        tokio::select! {
            // Got a broadcasted shutdown message
            _ = shutdown_broadcast_rx.recv() => {
                trace!("process resource sync requests task received shutdown");
                break;
            }
            // Got the next message on from the subscriber
            request = requests.next() => {
                match request {
                    Some(Ok(request)) => {
                        // Spawn a task an process the request
                        tokio::spawn(resource_sync_request_task(
                            nats.clone(),
                            cyclone_pool.clone(),
                            request,
                        ));
                    }
                    Some(Err(err)) => {
                        warn!(error = ?err, "next resource sync request had error");
                    }
                    None => {
                        trace!("resource sync requests subscriber stream has closed");
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

    // Unsubscribe from subscription
    requests.unsubscribe().await?;

    Ok(())
}

async fn resource_sync_request_task(
    nats: NatsClient,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
    request: Request<ResourceSyncRequest>,
) {
    if let Err(err) = resource_sync_request(nats, cyclone_pool, request).await {
        warn!(error = ?err, "resource sync execution failed");
    }
}

async fn resource_sync_request(
    nats: NatsClient,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
    request: Request<ResourceSyncRequest>,
) -> Result<()> {
    let (reply_mailbox, cyclone_request) = request.into_parts();

    let publisher = Publisher::new(&nats, &reply_mailbox);
    let mut client = cyclone_pool
        .get()
        .await
        .map_err(|err| ServerError::CyclonePool(Box::new(err)))?;
    let mut progress = client.execute_sync(cyclone_request).await?.start().await?;

    while let Some(msg) = progress.next().await {
        match msg {
            Ok(ProgressMessage::OutputStream(output)) => {
                publisher.publish_output(&output).await?;
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
    publisher.finalize_output().await?;

    let function_result = progress.finish().await?;
    publisher.publish_result(&function_result).await?;

    Ok(())
}

async fn process_code_generation_requests_task(
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
    shutdown_broadcast_rx: broadcast::Receiver<()>,
) {
    if let Err(err) =
        process_code_generation_requests(nats, subject_prefix, cyclone_pool, shutdown_broadcast_rx)
            .await
    {
        warn!(error = ?err, "processing resource sync requests failed");
    }
}

async fn process_code_generation_requests(
    nats: NatsClient,
    subject_prefix: Option<String>,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
    mut shutdown_broadcast_rx: broadcast::Receiver<()>,
) -> Result<()> {
    let mut requests = Subscriber::code_generation(&nats, subject_prefix.as_deref()).await?;

    loop {
        tokio::select! {
            // Got a broadcasted shutdown message
            _ = shutdown_broadcast_rx.recv() => {
                trace!("process resource sync requests task received shutdown");
                break;
            }
            // Got the next message on from the subscriber
            request = requests.next() => {
                match request {
                    Some(Ok(request)) => {
                        // Spawn a task an process the request
                        tokio::spawn(code_generation_request_task(
                            nats.clone(),
                            cyclone_pool.clone(),
                            request,
                        ));
                    }
                    Some(Err(err)) => {
                        warn!(error = ?err, "next resource sync request had error");
                    }
                    None => {
                        trace!("resource sync requests subscriber stream has closed");
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

    // Unsubscribe from subscription
    requests.unsubscribe().await?;

    Ok(())
}

async fn code_generation_request_task(
    nats: NatsClient,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
    request: Request<CodeGenerationRequest>,
) {
    if let Err(err) = code_generation_request(nats, cyclone_pool, request).await {
        warn!(error = ?err, "resource sync execution failed");
    }
}

async fn code_generation_request(
    nats: NatsClient,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
    request: Request<CodeGenerationRequest>,
) -> Result<()> {
    let (reply_mailbox, cyclone_request) = request.into_parts();

    let publisher = Publisher::new(&nats, &reply_mailbox);
    let mut client = cyclone_pool
        .get()
        .await
        .map_err(|err| ServerError::CyclonePool(Box::new(err)))?;
    let mut progress = client
        .execute_code_generation(cyclone_request)
        .await?
        .start()
        .await?;

    while let Some(msg) = progress.next().await {
        match msg {
            Ok(ProgressMessage::OutputStream(output)) => {
                publisher.publish_output(&output).await?;
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
    publisher.finalize_output().await?;

    let function_result = progress.finish().await?;
    publisher.publish_result(&function_result).await?;

    Ok(())
}

async fn connect_to_nats(config: &Config) -> Result<NatsClient> {
    info!("connecting to NATS; url={}", config.nats().url);

    let nats = NatsClient::new(config.nats())
        .await
        .map_err(ServerError::NatsConnect)?;

    Ok(nats)
}

fn prepare_graceful_shutdown(
    mut shutdown_rx: mpsc::Receiver<ShutdownSource>,
    shutdown_broadcast_tx: broadcast::Sender<()>,
) -> Result<oneshot::Receiver<()>> {
    let (graceful_shutdown_tx, graceful_shutdown_rx) = oneshot::channel::<()>();
    let mut sigterm_stream =
        unix::signal(unix::SignalKind::terminate()).map_err(ServerError::Signal)?;

    tokio::spawn(async move {
        fn send_graceful_shutdown(
            tx: oneshot::Sender<()>,
            shutdown_broadcast_tx: broadcast::Sender<()>,
        ) {
            // Send shutdown to all long running subscriptions, so they can cleanly terminate
            if shutdown_broadcast_tx.send(()).is_err() {
                error!("all broadcast shutdown receivers have already been dropped");
            }
            // Send graceful shutdown to main server thread which stops it from accepting requests.
            // We'll do this step last so as to let all subscriptions have a chance to shutdown.
            if tx.send(()).is_err() {
                error!("the server graceful shutdown receiver has already dropped");
            }
        }

        tokio::select! {
            _ = sigterm_stream.recv() => {
                info!("received SIGTERM signal, performing graceful shutdown");
                send_graceful_shutdown(graceful_shutdown_tx, shutdown_broadcast_tx);
            }
            source = shutdown_rx.recv() => {
                info!(
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

#[derive(Debug, Eq, PartialEq)]
pub enum ShutdownSource {
    Handle,
}

impl Default for ShutdownSource {
    fn default() -> Self {
        Self::Handle
    }
}
