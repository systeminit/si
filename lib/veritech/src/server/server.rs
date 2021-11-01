use cyclone::{
    client::Connection,
    resolver_function::{ResolverFunctionExecutingMessage, ResolverFunctionRequest},
    Client, CycloneClient, HttpClient, UdsClient,
};
use futures::{StreamExt, TryStreamExt};
use si_data::NatsConn;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{error, info, instrument, trace, warn};

use super::{Config, Publisher, PublisherError, Request, Subscriber, SubscriberError};
use crate::server::config::CycloneStream;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error(transparent)]
    Cyclone(#[from] cyclone::ClientError),
    #[error("failed to connecto to nats")]
    NatsConnection(#[source] si_data::NatsTxnError),
    #[error(transparent)]
    Publisher(#[from] PublisherError),
    #[error(transparent)]
    ResolverFunction(#[from] cyclone::client::ResolverFunctionExecutionError),
    #[error(transparent)]
    Subscriber(#[from] SubscriberError),
    #[error("wrong cyclone stream for {0} server: {1:?}")]
    WrongCycloneStream(&'static str, CycloneStream),
}

type Result<T> = std::result::Result<T, ServerError>;

pub struct Server<Cycl> {
    nats_conn: NatsConn,
    cyclone_client: Cycl,
}

impl Server<()> {
    #[instrument(name = "veritech.init.cyclone.http", skip(config))]
    pub async fn for_cyclone_http(config: Config) -> Result<Server<HttpClient>> {
        match config.cyclone_stream() {
            CycloneStream::HttpSocket(socket_addr) => {
                let nats_conn = connect_to_nats(&config).await?;
                // TODO(fnichol): determine client type and connection details
                let cyclone_client = Client::http(socket_addr)?;

                Ok(Server {
                    nats_conn,
                    cyclone_client,
                })
            }
            wrong @ CycloneStream::UnixDomainSocket(_) => {
                Err(ServerError::WrongCycloneStream("http", wrong.clone()))
            }
        }
    }

    #[instrument(name = "veritech.init.cyclone.uds", skip(config))]
    pub async fn for_cyclone_uds(config: Config) -> Result<Server<UdsClient>> {
        match config.cyclone_stream() {
            CycloneStream::UnixDomainSocket(path) => {
                let nats_conn = connect_to_nats(&config).await?;
                // TODO(fnichol): determine client type and connection details
                let cyclone_client = Client::uds(path)?;

                Ok(Server {
                    nats_conn,
                    cyclone_client,
                })
            }
            wrong @ CycloneStream::HttpSocket(_) => {
                Err(ServerError::WrongCycloneStream("http", wrong.clone()))
            }
        }
    }
}

impl<Cycl> Server<Cycl> {
    pub async fn run<Strm>(self) -> Result<()>
    where
        Cycl: CycloneClient<Strm> + Send + Clone + 'static,
        Strm: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
    {
        let mut resolver_function_requests = Subscriber::resolver_function(&self.nats_conn).await?;

        // NOTE: for the moment, this stream will terminate on first error, being a NATS IO error,
        // a serde deserialize, etc. This may not be what we want, or we want to have a
        // retry/re-subscribe loop
        while let Some(request) = resolver_function_requests.try_next().await? {
            // Spawn a task an process the request
            tokio::spawn(resolver_function_request_task(
                self.nats_conn.clone(),
                self.cyclone_client.clone(),
                request,
            ));
        }

        resolver_function_requests.unsubscribe().await?;

        Ok(())
    }
}

async fn resolver_function_request_task<Cycl, Strm>(
    nats_conn: NatsConn,
    cyclone_client: Cycl,
    request: Request<ResolverFunctionRequest>,
) where
    Cycl: CycloneClient<Strm>,
    Strm: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    if let Err(err) = resolver_function_request(nats_conn, cyclone_client, request).await {
        warn!(error = ?err, "resolver function execution failed");
    }
}

async fn resolver_function_request<Cycl, Strm>(
    nats_conn: NatsConn,
    mut cyclone_client: Cycl,
    request: Request<ResolverFunctionRequest>,
) -> Result<()>
where
    Cycl: CycloneClient<Strm>,
    Strm: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    let (reply_mailbox, cyclone_request) = request.into_parts();

    let publisher = Publisher::new(&nats_conn, &reply_mailbox);
    let mut progress = cyclone_client
        .execute_resolver(cyclone_request)
        .await?
        .start()
        .await?;

    while let Some(msg) = progress.next().await {
        match msg {
            Ok(ResolverFunctionExecutingMessage::OutputStream(output)) => {
                publisher.publish(&output).await?;
            }
            Ok(ResolverFunctionExecutingMessage::Heartbeat) => {
                trace!("received heartbeat message");
            }
            Err(e) => todo!("deal with this: {:?}", e),
        }
    }

    let function_result = progress.finish().await?;
    publisher.publish(&function_result).await?;

    Ok(())
}

async fn connect_to_nats(config: &Config) -> Result<NatsConn> {
    info!("connecting to NATS; url={}", config.nats().url);

    let nats_conn = NatsConn::new(config.nats())
        .await
        .map_err(ServerError::NatsConnection)?;

    Ok(nats_conn)
}
