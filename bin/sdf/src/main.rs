#![recursion_limit = "256"]

use std::path::PathBuf;

use color_eyre::Result;
use nats_multiplexer::Multiplexer;
use sdf_server::server::{CRDT_MULTIPLEXER_SUBJECT, WS_MULTIPLEXER_SUBJECT};
use sdf_server::{
    Config, IncomingStream, JobProcessorClientCloser, JobProcessorConnector, MigrationMode, Server,
    ServicesContext,
};
use telemetry_application::prelude::*;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

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
    let shutdown_token = CancellationToken::new();
    let task_tracker = TaskTracker::new();

    color_eyre::install()?;
    let args = args::parse();
    let (mut telemetry, telemetry_shutdown) = {
        let config = TelemetryConfig::builder()
            .force_color(args.force_color.then_some(true))
            .no_color(args.no_color.then_some(true))
            .service_name("sdf")
            .service_namespace("si")
            .log_env_var_prefix("SI")
            .app_modules(vec!["sdf", "sdf_server"])
            .build()?;

        telemetry_application::init(config, &task_tracker, shutdown_token.clone())?
    };

    if args.verbose > 0 {
        telemetry.set_verbosity(args.verbose.into()).await?;
    }
    trace!(arguments =?args, "parsed cli arguments");

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

    if let Some(symmetric_key_path) = &args.generate_symmetric_key_path {
        info!(
            "Generating Symmetric key at: {}",
            symmetric_key_path.display()
        );
        Server::generate_symmetric_key(symmetric_key_path).await?;
        return Ok(());
    }

    let config = Config::try_from(args)?;

    let encryption_key = Server::load_encryption_key(config.crypto().clone()).await?;
    let jwt_public_signing_key =
        Server::load_jwt_public_signing_key(config.jwt_signing_public_key().clone()).await?;

    let nats_conn = Server::connect_to_nats(config.nats()).await?;

    let (job_client, job_processor) = JobProcessor::connect(&config).await?;

    let pg_pool = Server::create_pg_pool(config.pg_pool()).await?;

    let veritech = Server::create_veritech_client(nats_conn.clone());

    let symmetric_crypto_service =
        Server::create_symmetric_crypto_service(config.symmetric_crypto_service()).await?;

    let pkgs_path: PathBuf = config.pkgs_path().try_into()?;

    let module_index_url = config.module_index_url().to_string();

    let (ws_multiplexer, ws_multiplexer_client) =
        Multiplexer::new(&nats_conn, WS_MULTIPLEXER_SUBJECT).await?;
    let (crdt_multiplexer, crdt_multiplexer_client) =
        Multiplexer::new(&nats_conn, CRDT_MULTIPLEXER_SUBJECT).await?;

    let services_context = ServicesContext::new(
        pg_pool,
        nats_conn,
        job_processor,
        veritech,
        encryption_key,
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

    let posthog_client = Server::start_posthog(config.posthog()).await?;

    task_tracker.close();

    match config.incoming_stream() {
        IncomingStream::HTTPSocket(_) => {
            let (server, initial_shutdown_broadcast_rx) = Server::http(
                config,
                services_context.clone(),
                jwt_public_signing_key,
                posthog_client,
                ws_multiplexer,
                ws_multiplexer_client,
                crdt_multiplexer,
                crdt_multiplexer_client,
            )?;
            let second_shutdown_broadcast_rx = initial_shutdown_broadcast_rx.resubscribe();

            Server::start_resource_refresh_scheduler(
                services_context.clone(),
                initial_shutdown_broadcast_rx,
            )
            .await;

            Server::start_status_updater(services_context, second_shutdown_broadcast_rx).await?;

            server.run().await?;
        }
        IncomingStream::UnixDomainSocket(_) => {
            let (server, initial_shutdown_broadcast_rx) = Server::uds(
                config,
                services_context.clone(),
                jwt_public_signing_key,
                posthog_client,
                ws_multiplexer,
                ws_multiplexer_client,
                crdt_multiplexer,
                crdt_multiplexer_client,
            )
            .await?;
            let second_shutdown_broadcast_rx = initial_shutdown_broadcast_rx.resubscribe();

            Server::start_resource_refresh_scheduler(
                services_context.clone(),
                initial_shutdown_broadcast_rx,
            )
            .await;

            Server::start_status_updater(services_context, second_shutdown_broadcast_rx).await?;

            server.run().await?;
        }
    }

    // TODO(fnichol): this will eventually go into the signal handler code but at the moment in
    // sdf's case, this is embedded in server library code which is incorrect. At this moment in
    // the program however, axum has shut down so it's an appropriate time to cancel other
    // remaining tasks and wait on their graceful shutdowns
    {
        shutdown_token.cancel();
        task_tracker.wait().await;
        telemetry_shutdown.wait().await?;
    }

    if let Err(err) = (&job_client as &dyn JobProcessorClientCloser).close().await {
        error!("Failed to close job client: {err}");
    }

    info!("graceful shutdown complete.");
    Ok(())
}
