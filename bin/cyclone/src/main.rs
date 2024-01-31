use color_eyre::Result;
use cyclone_server::{Config, Runnable as _, Server};
use telemetry_application::prelude::*;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

mod args;

// Override the default tracing level of `info` to warn.
//
// Note: Cyclone servers are spawned as child processes (or managed processes) of a Veritech server
// instance so in many cases the logging output of a Cyclone server is written to the same output
// stream (i.e. terminal, console) as the Veritech server's logging output. This higher threshold
// is an attempt to reduce the amount of "normal" logging that is emited for Cyclone instances.
const CUSTOM_DEFAULT_TRACING_LEVEL: &str = "warn";

#[tokio::main]
async fn main() -> Result<()> {
    let shutdown_token = CancellationToken::new();
    let task_tracker = TaskTracker::new();

    color_eyre::install()?;
    let config = TelemetryConfig::builder()
        .service_name("cyclone")
        .service_namespace("si")
        .log_env_var_prefix("SI")
        .app_modules(vec!["cyclone", "cyclone_server"])
        .custom_default_tracing_level(CUSTOM_DEFAULT_TRACING_LEVEL)
        .build()?;
    let mut telemetry = telemetry_application::init(config, &task_tracker, shutdown_token.clone())?;
    let args = args::parse();

    if args.verbose > 0 {
        telemetry.set_verbosity(args.verbose.into()).await?;
    }
    debug!(arguments =?args, "parsed cli arguments");

    let decryption_key = Server::load_decryption_key(&args.decryption_key).await?;

    let config = Config::try_from(args)?;

    let telemetry = Box::new(telemetry);

    task_tracker.close();

    Server::from_config(config, telemetry, decryption_key)
        .await?
        .run()
        .await?;

    // TODO(fnichol): this will eventually go into the signal handler code but at the moment in
    // cyclone's case, this is embedded in server library code which is incorrect. At this moment in
    // the program however, axum has shut down so it's an appropriate time to cancel other
    // remaining tasks and wait on their graceful shutdowns
    {
        shutdown_token.cancel();
        task_tracker.wait().await;
    }

    Ok(())
}
