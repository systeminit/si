use std::time::Duration;

use si_service::{color_eyre, prelude::*, rt, shutdown, startup, telemetry_application};
use veritech_server::{Config, Server};

mod args;

const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(60 * 60 * 6);

fn main() -> Result<()> {
    rt::block_on("bin/veritch-tokio::runtime", async_main())
}

async fn async_main() -> Result<()> {
    let main_tracker = TaskTracker::new();
    let main_token = CancellationToken::new();
    let telemetry_tracker = TaskTracker::new();
    let telemetry_token = CancellationToken::new();

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
            .service_name("veritech")
            .service_namespace("si")
            .log_env_var_prefix("SI")
            .app_modules(vec!["veritech", "veritech_server"])
            .interesting_modules(vec!["naxum", "si_data_nats"])
            .build()?;

        telemetry_application::init(config, &telemetry_tracker, telemetry_token.clone())?
    };

    startup::startup("veritech").await?;

    if args.verbose > 0 {
        telemetry
            .set_verbosity_and_wait(args.verbose.into())
            .await?;
    }
    debug!(arguments =?args, "parsed cli arguments");

    let config = Config::try_from(args)?;

    let server = Server::from_config(config, main_token.clone()).await?;

    main_tracker.spawn(async move {
        info!("ready to receive messages");
        server.run().await
    });

    shutdown::graceful(
        [
            (main_tracker, main_token),
            (telemetry_tracker, telemetry_token),
        ],
        Some(telemetry_shutdown.into_future()),
        Some(GRACEFUL_SHUTDOWN_TIMEOUT),
    )
    .await
    .map_err(Into::into)
}
