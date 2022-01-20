use cyclone::QualificationCheckRequest;
use deadpool_cyclone::{
    instance::cyclone::LocalUdsInstanceSpec, CycloneClient, Manager, Pool, ProgressMessage,
    ResolverFunctionRequest,
};
use futures::{join, StreamExt};
use si_data::NatsClient;
use telemetry::prelude::*;
use thiserror::Error;

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
    QualificationCheck(#[from] deadpool_cyclone::client::QualificationCheckExecutionError),
    #[error(transparent)]
    ResolverFunction(#[from] deadpool_cyclone::client::ResolverFunctionExecutionError),
    #[error(transparent)]
    Subscriber(#[from] SubscriberError),
    #[error("wrong cyclone spec type for {0} spec: {1:?}")]
    WrongCycloneSpec(&'static str, CycloneSpec),
}

type Result<T> = std::result::Result<T, ServerError>;

pub struct Server {
    nats: NatsClient,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
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
                let nats = connect_to_nats(&config).await?;
                let manager = Manager::new(spec.clone());
                let cyclone_pool = Pool::builder(manager)
                    .build()
                    .map_err(|err| ServerError::CycloneSpec(Box::new(err)))?;

                Ok(Server { nats, cyclone_pool })
            }
            wrong @ CycloneSpec::LocalHttp(_) => {
                Err(ServerError::WrongCycloneSpec("LocalUds", wrong.clone()))
            }
        }
    }
}

impl Server {
    pub async fn run(self) -> Result<()> {
        // TODO(fnichol): eventually veritech will be made configurable as to what requests it will
        // service, so this will cease to be a `join!` macro, but likely a set of spawned tasks
        // with a shutdown oneshot channel handler for graceful draining and teardown. However
        // right this second we're going to service all known request types, so the join macro it
        // is!
        join!(
            process_resolver_function_requests_task(self.nats.clone(), self.cyclone_pool.clone()),
            process_qualification_check_requests_task(self.nats.clone(), self.cyclone_pool.clone()),
        );

        Ok(())
    }
}

// NOTE(fnichol): the resolver_function and qualification_check paths are parallel and extremely
// similar, so there is a lurking "unifying" refactor here. It felt like waiting until the third
// time adding one of these would do the trick, and as a result the first 2 impls are here and not
// split apart into their own modules.

async fn process_resolver_function_requests_task(
    nats: NatsClient,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
) {
    if let Err(err) = process_resolver_function_requests(nats, cyclone_pool).await {
        warn!(error = ?err, "processing resolver function requests failed");
    }
}

async fn process_resolver_function_requests(
    nats: NatsClient,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
) -> Result<()> {
    let mut requests = Subscriber::resolver_function(&nats).await?;

    while let Some(request) = requests.next().await {
        match request {
            Ok(request) => {
                // Spawn a task an process the request
                tokio::spawn(resolver_function_request_task(
                    nats.clone(),
                    cyclone_pool.clone(),
                    request,
                ));
            }
            Err(err) => {
                warn!(error = ?err, "next resolver function request had error");
            }
        }
    }
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
                warn!(error = ?err, "next progress message was an error, skipping to next message");
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
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
) {
    if let Err(err) = process_qualification_check_requests(nats, cyclone_pool).await {
        warn!(error = ?err, "processing qualification check requests failed");
    }
}

async fn process_qualification_check_requests(
    nats: NatsClient,
    cyclone_pool: Pool<LocalUdsInstanceSpec>,
) -> Result<()> {
    let mut requests = Subscriber::qualification_check(&nats).await?;

    while let Some(request) = requests.next().await {
        match request {
            Ok(request) => {
                // Spawn a task an process the request
                tokio::spawn(qualification_check_request_task(
                    nats.clone(),
                    cyclone_pool.clone(),
                    request,
                ));
            }
            Err(err) => {
                warn!(error = ?err, "next qualification check request had error");
            }
        }
    }
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
                warn!(error = ?err, "next progress message was an error, skipping to next message");
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
