use color_eyre::Result;
use telemetry_application::prelude::*;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use veritech_server::{Config, CycloneSpec, Server};

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
            .service_name("veritech")
            .service_namespace("si")
            .log_env_var_prefix("SI")
            .app_modules(vec!["veritech", "veritech_server"])
            .build()?;

        telemetry_application::init(config, &task_tracker, shutdown_token.clone())?
    };

    if args.verbose > 0 {
        telemetry.set_verbosity(args.verbose.into()).await?;
    }
    trace!(arguments =?args, "parsed cli arguments");

    let config = Config::try_from(args)?;

    task_tracker.close();

    match config.cyclone_spec() {
        CycloneSpec::LocalHttp(_) => {
            Server::for_cyclone_http(config).await?.run().await?;
        }
        CycloneSpec::LocalUds(_) => {
            Server::for_cyclone_uds(config).await?.run().await?;
        }
    }

    // TODO(fnichol): this will eventually go into the signal handler code but at the moment in
    // veritech's case, this is embedded in server library code which is incorrect. At this moment
    // in the program however, the server has shut down so it's an appropriate time to cancel other
    // remaining tasks and wait on their graceful shutdowns
    {
        shutdown_token.cancel();
        task_tracker.wait().await;
        telemetry_shutdown.wait().await?;
    }

    info!("graceful shutdown complete.");
    Ok(())
}
