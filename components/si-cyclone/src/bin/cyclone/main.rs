use color_eyre::Result;
use si_cyclone::{telemetry, Server};
use std::convert::TryInto;

mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    telemetry::init()?;
    let config = cli::parse().try_into()?;

    Server::init(config).await?.run().await.map_err(From::from)
}
