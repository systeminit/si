use color_eyre::Result;
use cyclone_server::{Config, Runnable as _, Server};
use telemetry_application::prelude::*;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

mod args;

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
            .console_log_format(
                args.log_json
                    .then_some(ConsoleLogFormat::Json)
                    .unwrap_or_default(),
            )
            .service_name("cyclone")
            .service_namespace("si")
            .log_env_var_prefix("SI")
            .app_modules(vec!["cyclone", "cyclone_server"])
            .interesting_modules(vec!["cyclone_core"])
            .build()?;

        telemetry_application::init(config, &task_tracker, shutdown_token.clone())?
    };

    if args.verbose > 0 {
        telemetry
            .set_verbosity_and_wait(args.verbose.into())
            .await?;
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
        telemetry_shutdown.wait().await?;
    }

    Ok(())
}
