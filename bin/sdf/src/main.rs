use color_eyre::Result;
use sdf::{Config, IncomingStream, MigrationMode, Server};
use tracing::{debug, info, trace};

mod args;
mod telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    telemetry::init()?;
    let args = args::parse();

    run(args).await
}

async fn run(args: args::Args) -> Result<()> {
    debug!(arguments =?args, "parsed cli arguments");
    let config = Config::try_from(args)?;

    let pg_pool = Server::create_pg_pool(config.pg_pool()).await?;

    if let MigrationMode::Run | MigrationMode::RunAndQuit = config.migration_mode() {
        Server::migrate_database(&pg_pool).await?;
        if let MigrationMode::RunAndQuit = config.migration_mode() {
            info!("migration mode is runAndQuit, shutting down");
            return Ok(());
        }
    } else {
        trace!("migration mode is skip, not running migrations");
    }

    let nats = Server::connect_to_nats(config.nats()).await?;

    match config.incoming_stream() {
        IncomingStream::HTTPSocket(_) => Server::http(config, pg_pool, nats)?.run().await?,
        IncomingStream::UnixDomainSocket(_) => {
            Server::uds(config, pg_pool, nats).await?.run().await?
        }
    }

    Ok(())
}
