use std::convert::TryInto;

use color_eyre::Result;
use veritech::{Config, CycloneStream, Server};

mod args;
mod telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    telemetry::init()?;
    let config = args::parse().try_into()?;

    run(config).await
}

async fn run(config: Config) -> Result<()> {
    match config.cyclone_stream() {
        CycloneStream::HttpSocket(_) => Server::for_cyclone_http(config).await?.run().await?,
        CycloneStream::UnixDomainSocket(_) => Server::for_cyclone_uds(config).await?.run().await?,
    }

    Ok(())
}
