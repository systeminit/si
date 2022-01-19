use deadpool_cyclone::{
    instance::cyclone::LocalUdsInstanceSpec, CycloneClient, Manager, Pool, ProgressMessage,
    ResolverFunctionRequest,
};
use futures::StreamExt;
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
    #[error("cyclone spec builder error")]
    CycloneSpec(#[source] Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error("error connecting to nats")]
    NatsConnect(#[source] si_data::NatsError),
    #[error(transparent)]
    Publisher(#[from] PublisherError),
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
        let mut resolver_function_requests = Subscriber::resolver_function(&self.nats).await?;

        while let Some(request) = resolver_function_requests.next().await {
            match request {
                Ok(request) => {
                    // Spawn a task an process the request
                    tokio::spawn(resolver_function_request_task(
                        self.nats.clone(),
                        self.cyclone_pool.clone(),
                        request,
                    ));
                }
                Err(err) => {
                    warn!(error = ?err, "next resolver function request had error");
                }
            }
        }
        resolver_function_requests.unsubscribe().await?;

        Ok(())
    }
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
    let mut client = cyclone_pool.get().await.expect("DODO");
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
            Err(e) => todo!("deal with this: {:?}", e),
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
