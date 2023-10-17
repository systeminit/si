#![recursion_limit = "256"]

use std::path::PathBuf;
use std::sync::Arc;

use color_eyre::Result;
use sdf_server::{
    Config, IncomingStream, JobProcessorClientCloser, JobProcessorConnector, MigrationMode, Server,
    ServicesContext,
};
use telemetry_application::{
    prelude::*, start_tracing_level_signal_handler_task, ApplicationTelemetryClient,
    TelemetryClient, TelemetryConfig,
};

mod args;

type JobProcessor = sdf_server::NatsProcessor;

const RT_DEFAULT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024 * 10;

fn main() -> Result<()> {
    let thread_builder = ::std::thread::Builder::new().stack_size(RT_DEFAULT_THREAD_STACK_SIZE);
    let thread_handler = thread_builder.spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .thread_stack_size(RT_DEFAULT_THREAD_STACK_SIZE)
            .thread_name("bin/sdf-tokio::runtime")
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
        .log_env_var_prefix("SI")
        .app_modules(vec!["sdf", "sdf_server"])
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

    if args.disable_opentelemetry {
        telemetry.disable_opentelemetry().await?;
    }

    Server::init()?;

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

    let encryption_key = Server::load_encryption_key(config.cyclone_encryption_key_path()).await?;
    let jwt_public_signing_key =
        Server::load_jwt_public_signing_key(config.jwt_signing_public_key_path()).await?;

    let nats_conn = Server::connect_to_nats(config.nats()).await?;

    let (job_client, job_processor) = JobProcessor::connect(&config).await?;

    let pg_pool = Server::create_pg_pool(config.pg_pool()).await?;

    let veritech = Server::create_veritech_client(nats_conn.clone());

    let symmetric_crypto_service =
        Server::create_symmetric_crypto_service(config.symmetric_crypto_service()).await?;

    let pkgs_path: PathBuf = config.pkgs_path().try_into()?;

    let module_index_url = config.module_index_url().to_string();

    let services_context = ServicesContext::new(
        pg_pool,
        nats_conn,
        job_processor,
        veritech,
        Arc::from(encryption_key),
        Some(pkgs_path),
        Some(module_index_url),
        symmetric_crypto_service,
    );

    if let MigrationMode::Run | MigrationMode::RunAndQuit = config.migration_mode() {
        Server::migrate_database(&services_context).await?;
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

    start_tracing_level_signal_handler_task(&telemetry)?;

    let posthog_client = Server::start_posthog(config.posthog()).await?;

    match config.incoming_stream() {
        IncomingStream::HTTPSocket(_) => {
            let (server, initial_shutdown_broadcast_rx) = Server::http(
                config,
                services_context.clone(),
                jwt_public_signing_key,
                posthog_client,
            )?;
            let second_shutdown_broadcast_rx = initial_shutdown_broadcast_rx.resubscribe();

            // Server::start_resource_refresh_scheduler(
            //     services_context.clone(),
            //     initial_shutdown_broadcast_rx,
            // )
            // .await;

            // Server::start_status_updater(services_context, second_shutdown_broadcast_rx).await?;

            server.run().await?;
        }
        IncomingStream::UnixDomainSocket(_) => {
            let (server, initial_shutdown_broadcast_rx) = Server::uds(
                config,
                services_context.clone(),
                jwt_public_signing_key,
                posthog_client,
            )
            .await?;
            let second_shutdown_broadcast_rx = initial_shutdown_broadcast_rx.resubscribe();

            // Server::start_resource_refresh_scheduler(
            //     services_context.clone(),
            //     initial_shutdown_broadcast_rx,
            // )
            // .await;

            // Server::start_status_updater(services_context, second_shutdown_broadcast_rx).await?;

            server.run().await?;
        }
    }

    info!("Shutting down the job processor client");
    if let Err(err) = (&job_client as &dyn JobProcessorClientCloser).close().await {
        error!("Failed to close job client: {err}");
    }

    Ok(())
}
