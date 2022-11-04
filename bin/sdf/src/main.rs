#![recursion_limit = "256"]

use color_eyre::Result;
use sdf::{
    Config, FaktoryProcessor, IncomingStream, JobQueueProcessor, MigrationMode, Server,
    SyncProcessor,
};
use telemetry_application::{
    prelude::*, start_tracing_level_signal_handler_task, ApplicationTelemetryClient,
    TelemetryClient, TelemetryConfig,
};
use tokio::sync::mpsc;

mod args;

const RT_DEFAULT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024 * 3;

fn main() -> Result<()> {
    let thread_builder = ::std::thread::Builder::new().stack_size(RT_DEFAULT_THREAD_STACK_SIZE);
    let thread_handler = thread_builder.spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .thread_stack_size(RT_DEFAULT_THREAD_STACK_SIZE)
            .thread_name("bin/sdf-tokio::runtime".to_owned())
            .enable_all()
            .build()?
            .block_on(async_main())
    })?;
    thread_handler.join().unwrap()
}

async fn async_main() -> Result<()> {
    color_eyre::install()?;
    let config = TelemetryConfig::builder()
        .service_name("sdf")
        .service_namespace("si")
        .app_modules(vec!["sdf_cli", "sdf"])
        .build()?;
    let telemetry = telemetry_application::init(config)?;
    let args = args::parse();

    run(args, telemetry).await
}

async fn run(args: args::Args, mut telemetry: ApplicationTelemetryClient) -> Result<()> {
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

    let faktory = Server::create_faktory_client(config.faktory());

    let (alive_marker, mut job_processor_shutdown_rx) = mpsc::channel(1);
    let job_processor = Box::new(FaktoryProcessor::new(faktory.clone(), alive_marker))
        as Box<dyn JobQueueProcessor + Send + Sync>;

    let pg_pool = Server::create_pg_pool(config.pg_pool()).await?;

    let veritech = Server::create_veritech_client(nats.clone());

    if let MigrationMode::Run | MigrationMode::RunAndQuit = config.migration_mode() {
        Server::migrate_database(
            &pg_pool,
            &nats,
            job_processor.clone(),
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

    //Server::start_faktory_job_executor(
    //    pg_pool.clone(),
    //    nats.clone(),
    //    faktory_conn.clone(),
    //    veritech.clone(),
    //    encryption_key,
    //)
    //.await;

    match config.incoming_stream() {
        IncomingStream::HTTPSocket(_) => {
            let (server, shutdown_broadcast_rx) = Server::http(
                config,
                telemetry,
                pg_pool.clone(),
                nats.clone(),
                job_processor,
                veritech.clone(),
                encryption_key,
                jwt_secret_key,
            )?;

            Server::start_resource_refresh_scheduler(
                pg_pool,
                nats,
                Box::new(SyncProcessor::new()),
                veritech,
                encryption_key,
                shutdown_broadcast_rx,
            )
            .await;

            server.run().await?;
        }
        IncomingStream::UnixDomainSocket(_) => {
            let (server, shutdown_broadcast_rx) = Server::uds(
                config,
                telemetry,
                pg_pool.clone(),
                nats.clone(),
                job_processor,
                veritech.clone(),
                encryption_key,
                jwt_secret_key,
            )
            .await?;

            Server::start_resource_refresh_scheduler(
                pg_pool,
                nats,
                Box::new(SyncProcessor::new()),
                veritech,
                encryption_key,
                shutdown_broadcast_rx,
            )
            .await;

            server.run().await?;
        }
    }

    // Blocks until all FaktoryProcessors are gone so we don't skip jobs that are still being sent to faktory_async
    info!("Waiting for all faktory processors to finish pushing jobs");
    let _ = job_processor_shutdown_rx.recv().await;

    info!("Shutting down the faktory client");
    if let Err(err) = faktory.close().await {
        error!("Failed to close faktory client: {err}");
    }

    Ok(())
}
