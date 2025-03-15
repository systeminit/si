use std::time::Duration;

use si_service::{color_eyre, prelude::*, rt, shutdown, startup, telemetry_application};
use veritech_server::{Config, Server};

mod args;

const BIN_NAME: &str = env!("CARGO_BIN_NAME");
const LIB_NAME: &str = concat!(env!("CARGO_BIN_NAME"), "_server");

const DEFAULT_GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(60 * 20);

fn main() -> Result<()> {
    rt::block_on(BIN_NAME, async_main())
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
            .tokio_console(args.tokio_console)
            .service_name(BIN_NAME)
            .service_namespace("si")
            .log_env_var_prefix("SI")
            .app_modules(vec![BIN_NAME, LIB_NAME])
            .interesting_modules(vec!["naxum", "si_data_nats", "si_service"])
            .build()?;

        telemetry_application::init(config, &telemetry_tracker, telemetry_token.clone())?
    };
    tokio_watchdog::spawn(BIN_NAME, main_token.clone())?;

    startup::startup(BIN_NAME).await?;

    if args.verbose > 0 {
        telemetry
            .set_verbosity_and_wait(args.verbose.into())
            .await?;
    }
    let graceful_shutdown_timeout = match args.graceful_shutdown_timeout_secs {
        Some(provided) => Duration::from_secs(provided),
        None => DEFAULT_GRACEFUL_SHUTDOWN_TIMEOUT,
    };
    debug!(arguments =?args, "parsed cli arguments");

    let config = Config::try_from(args)?;
    debug!(?config, "computed configuration");

    let (server, maybe_heartbeat_app) = Server::from_config(config, main_token.clone()).await?;

    if let Some(mut heartbeat_app) = maybe_heartbeat_app {
        main_tracker.spawn(async move { heartbeat_app.run().await });
    }
    main_tracker.spawn(async move {
        info!("ready to receive messages");
        server.run().await
    });

    shutdown::graceful()
        .group(main_tracker, main_token)
        .group(telemetry_tracker, telemetry_token)
        .telemetry_guard(telemetry_shutdown.into_future())
        .timeout(graceful_shutdown_timeout)
        .wait()
        .await
        .map_err(Into::into)
}
