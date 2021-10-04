use color_eyre::Result;
use hyper::server::conn::AddrIncoming;
use si_cyclone::{app, telemetry};
use std::net::SocketAddr;

mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    telemetry::init()?;
    let config = cli::parse().into();

    Server::init(config).await?.run().await
}

struct Server;

impl Server {
    async fn init(_config: Config) -> Result<Self> {
        Ok(Self)
    }

    async fn run(self) -> Result<()> {
        // TODO(fnichol): different incoming depending on UDS, TLS/TCP, etc
        let incoming = AddrIncoming::bind(&SocketAddr::from(([127, 0, 0, 1], 3000)))?;

        axum::Server::builder(incoming)
            .serve(app().into_make_service())
            .await?;

        Ok(())
    }
}

struct Config;

impl From<cli::Args> for Config {
    fn from(_val: cli::Args) -> Self {
        Self
    }
}
