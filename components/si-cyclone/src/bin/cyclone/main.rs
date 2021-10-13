use std::convert::TryInto;

use color_eyre::Result;
use si_cyclone::{telemetry, Server};

mod args;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    telemetry::init()?;
    let config = args::parse().try_into()?;

    Server::init(config).await?.run().await.map_err(From::from)
}
