use color_eyre::Result;
use sdf::{Config, IncomingStream, MigrationMode, Server};
use telemetry::{
    start_tracing_level_signal_handler_task,
    tracing::{debug, info, trace},
    TelemetryClient,
};

mod args;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let config = telemetry::Config::builder()
        .service_name("sdf")
        .service_namespace("si")
        .app_modules(vec!["sdf_cli", "sdf"])
        .build()?;
    let telemetry = telemetry::init(config)?;
    let args = args::parse();

    run(args, telemetry).await
}

async fn run(args: args::Args, mut telemetry: telemetry::Client) -> Result<()> {
    if args.verbose > 0 {
        telemetry.set_verbosity(args.verbose.into()).await?;
    }
    debug!(arguments =?args, "parsed cli arguments");
    if args.disable_opentelemetry {
        telemetry.disable_opentelemetry().await?;
    }
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

    start_tracing_level_signal_handler_task(&telemetry)?;

    match config.incoming_stream() {
        IncomingStream::HTTPSocket(_) => {
            Server::http(config, telemetry, pg_pool, nats)?
                .run()
                .await?;
        }
        IncomingStream::UnixDomainSocket(_) => {
            Server::uds(config, telemetry, pg_pool, nats)
                .await?
                .run()
                .await?;
        }
    }

    Ok(())
}
