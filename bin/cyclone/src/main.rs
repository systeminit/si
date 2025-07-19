#[cfg(target_os = "linux")]
use cyclone_server::process_gatherer;
use cyclone_server::{
    Config,
    Runnable as _,
    Server,
};
#[cfg(target_os = "linux")]
use si_firecracker::stream::TcpStreamForwarder;
use si_service::{
    color_eyre,
    prelude::*,
    startup,
    telemetry_application,
};

mod args;

const BIN_NAME: &str = env!("CARGO_BIN_NAME");
const LIB_NAME: &str = concat!(env!("CARGO_BIN_NAME"), "_server");

#[tokio::main]
async fn main() -> Result<()> {
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
            .service_name(BIN_NAME)
            .service_namespace("si")
            .log_env_var_prefix("SI")
            .app_modules(vec![BIN_NAME, LIB_NAME])
            .interesting_modules(vec!["cyclone_core"])
            .build()?;

        telemetry_application::init(config, &task_tracker, shutdown_token.clone())?
    };

    startup::startup(BIN_NAME).await?;

    if args.verbose > 0 {
        telemetry
            .set_verbosity_and_wait(args.verbose.into())
            .await?;
    }
    debug!(arguments =?args, "parsed cli arguments");

    let config = Config::try_from(args)?;

    let telemetry = Box::new(telemetry);

    if config.enable_forwarder() {
        #[cfg(target_os = "linux")]
        TcpStreamForwarder::new().await?.start().await?;
    }

    #[cfg(target_os = "linux")]
    let gatherer_shutdown = process_gatherer::init(
        config.enable_process_gatherer(),
        &task_tracker,
        shutdown_token.clone(),
    )?;

    task_tracker.close();

    Server::from_config(config, telemetry).await?.run().await?;

    // TODO(fnichol): this will eventually go into the signal handler code but at the moment in
    // cyclone's case, this is embedded in server library code which is incorrect. At this moment in
    // the program however, axum has shut down so it's an appropriate time to cancel other
    // remaining tasks and wait on their graceful shutdowns
    {
        shutdown_token.cancel();
        #[cfg(target_os = "linux")]
        gatherer_shutdown.wait().await?;
        task_tracker.wait().await;
        telemetry_shutdown.wait().await?;
    }

    Ok(())
}
