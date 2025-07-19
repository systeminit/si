use color_eyre::Result;
use module_index_server::{
    Config,
    Server,
};
use telemetry_application::prelude::*;
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};

mod args;

const RT_DEFAULT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024 * 3;

fn main() -> Result<()> {
    let thread_builder = ::std::thread::Builder::new().stack_size(RT_DEFAULT_THREAD_STACK_SIZE);
    let thread_handler = thread_builder.spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .thread_stack_size(RT_DEFAULT_THREAD_STACK_SIZE)
            .thread_name("bin/module-index-tokio::runtime")
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
            .log_format(if args.log_json {
                LogFormat::Json
            } else {
                Default::default()
            })
            .log_file_directory(args.log_file_directory.clone())
            .tokio_console(args.tokio_console)
            .service_name("module-index")
            .service_namespace("si")
            .log_env_var_prefix("SI")
            .app_modules(vec!["module_index", "module_index_server"])
            .interesting_modules(vec!["si_data_pg"])
            .build()?;

        telemetry_application::init(config, &task_tracker, shutdown_token.clone())?
    };

    if args.verbose > 0 {
        telemetry
            .set_verbosity_and_wait(args.verbose.into())
            .await?;
    }
    debug!(arguments =?args, "parsed cli arguments");

    let config = Config::try_from(args)?;

    let jwt_public_signing_key = Server::load_jwt_public_signing_key(&config).await?;

    // our pg pool works for migrations (refinery) but doesnt work for SeaORM :(
    // so we set up both connections for now... Would like to clean this up
    let si_pg_pool = Server::create_pg_pool(config.pg_pool()).await?;
    Server::run_migrations(&si_pg_pool).await?;
    drop(si_pg_pool); // close connection since we no longer need it

    // this is the SeaOrm-managed Pg Pool
    let pg_pool = Server::create_db_connection(config.pg_pool()).await?;

    let posthog_client = Server::start_posthog(config.posthog()).await?;

    task_tracker.close();

    let (server, initial_shutdown_broadcast_rx) =
        Server::http(config, pg_pool, jwt_public_signing_key, posthog_client)?;
    let _second_shutdown_broadcast_rx = initial_shutdown_broadcast_rx.resubscribe();

    server.run().await?;

    // TODO(fnichol): this will eventually go into the signal handler code but at the moment in
    // module-index's case, this is embedded in server library code which is incorrect. At this
    // moment in the program however, axum has shut down so it's an appropriate time to cancel
    // other remaining tasks and wait on their graceful shutdowns
    {
        shutdown_token.cancel();
        task_tracker.wait().await;
        telemetry_shutdown.wait().await?;
    }

    info!("graceful shutdown complete.");
    Ok(())
}
