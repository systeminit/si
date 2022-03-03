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

    Server::init()?;

    // TODO(fnichol): we have a mutex poisoning panic that happens, but is avoided if opentelemetry
    // is not running when the migrations are. For the moment we'll disable otel until after the
    // migrations, which means we miss out on some good migration telemetry in honeycomb, but the
    // service boots??
    //
    // See: https://app.shortcut.com/systeminit/story/1934/sdf-mutex-poison-panic-on-launch-with-opentelemetry-exporter
    let _disable_opentelemetry = args.disable_opentelemetry;
    telemetry.disable_opentelemetry().await?;
    // if args.disable_opentelemetry {
    //     telemetry.disable_opentelemetry().await?;
    // }

    if let Some(path) = args.generate_jwt_secret_key {
        info!("Generating JWT secret at: {}", path.display());
        let _key = Server::generate_jwt_secret_key(path).await?;
        return Ok(());
    }

    if let (Some(secret_key_path), Some(public_key_path)) = (
        &args.generate_cyclone_secret_key_path,
        &args.generate_cyclone_public_key_path,
    ) {
        info!(
            "Generating Cyclone key pair at: (secret = {}, public = {})",
            secret_key_path.display(),
            public_key_path.display()
        );
        Server::generate_cyclone_key_pair(secret_key_path, public_key_path).await?;
        return Ok(());
    }

    let config = Config::try_from(args)?;

    let jwt_secret_key = Server::load_jwt_secret_key(config.jwt_secret_key_path()).await?;
    let encryption_key = Server::load_encryption_key(config.cyclone_encryption_key_path()).await?;

    let nats = Server::connect_to_nats(config.nats()).await?;

    let pg_pool = Server::create_pg_pool(config.pg_pool()).await?;

    let veritech = Server::create_veritech_client(nats.clone());

    if let MigrationMode::Run | MigrationMode::RunAndQuit = config.migration_mode() {
        Server::migrate_database(
            &pg_pool,
            &nats,
            &jwt_secret_key,
            veritech.clone(),
            &encryption_key,
        )
        .await?;
        if let MigrationMode::RunAndQuit = config.migration_mode() {
            info!(
                "migration mode is {}, shutting down",
                config.migration_mode()
            );
            return Ok(());
        }
    } else {
        trace!("migration mode is skip, not running migrations");
    }

    // TODO(fnichol): re-enable, which we shouldn't need in the long run
    //if !disable_opentelemetry {
    //    telemetry.enable_opentelemetry().await?;
    //}

    start_tracing_level_signal_handler_task(&telemetry)?;

    Server::start_resource_sync_scheduler(
        pg_pool.clone(),
        nats.clone(),
        veritech.clone(),
        encryption_key,
    )
    .await;

    match config.incoming_stream() {
        IncomingStream::HTTPSocket(_) => {
            Server::http(
                config,
                telemetry,
                pg_pool,
                nats,
                veritech,
                encryption_key,
                jwt_secret_key,
            )?
            .run()
            .await?;
        }
        IncomingStream::UnixDomainSocket(_) => {
            Server::uds(
                config,
                telemetry,
                pg_pool,
                nats,
                veritech,
                encryption_key,
                jwt_secret_key,
            )
            .await?
            .run()
            .await?;
        }
    }

    Ok(())
}
