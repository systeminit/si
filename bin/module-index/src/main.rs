use color_eyre::Result;
use module_index_server::{Config, Server};
use telemetry_application::{
    prelude::*, start_tracing_level_signal_handler_task, ApplicationTelemetryClient,
    TelemetryClient, TelemetryConfig,
};

mod args;

fn main() -> Result<()> {
    let thread_builder = ::std::thread::Builder::new();
    let thread_handler = thread_builder.spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .thread_name("bin/module-index-tokio::runtime")
            .enable_all()
            .build()?
            .block_on(async_main())
    })?;
    thread_handler.join().unwrap()
}

async fn async_main() -> Result<()> {
    color_eyre::install()?;
    let config = TelemetryConfig::builder()
        .service_name("module-index")
        .service_namespace("si")
        .log_env_var_prefix("SI")
        .app_modules(vec!["module_index", "module_index_server"])
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

    // Server::init()?;

    let config = Config::try_from(args)?;

    let jwt_public_signing_key =
        Server::load_jwt_public_signing_key(config.jwt_signing_public_key_path()).await?;

    // our pg pool works for migrations (refinery) but doesnt work for SeaORM :(
    // so we set up both connections for now... Would like to clean this up
    let si_pg_pool = Server::create_pg_pool(config.pg_pool()).await?;
    Server::run_migrations(&si_pg_pool).await?;
    drop(si_pg_pool); // close connection since we no longer need it

    // this is SeaOrm's managed Pg Pool
    let pg_pool = Server::create_db_connection(config.pg_pool()).await?;

    start_tracing_level_signal_handler_task(&telemetry)?;

    let posthog_client = Server::start_posthog(config.posthog()).await?;

    let (server, initial_shutdown_broadcast_rx) =
        Server::http(config, pg_pool, jwt_public_signing_key, posthog_client)?;
    let _second_shutdown_broadcast_rx = initial_shutdown_broadcast_rx.resubscribe();

    server.run().await?;

    Ok(())
}
