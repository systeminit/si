use std::time::Duration;

use bedrock_server::{
    Config,
    Server,
};
use color_eyre::Result;
use si_service::{
    color_eyre,
    prelude::*,
    rt,
    startup,
    telemetry_application::{
        self,
        TelemetryShutdownGuard,
    },
};

use crate::args::{
    NAME,
    VERSION,
};

mod args;

const LIB_NAME: &str = concat!(env!("CARGO_BIN_NAME"), "_server");

const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

fn main() -> Result<()> {
    rt::block_on(NAME, async_main())
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
            .log_format(if args.log_json {
                LogFormat::Json
            } else {
                Default::default()
            })
            .log_file_directory(args.log_file_directory.clone())
            .service_name(NAME)
            .service_version(VERSION)
            .service_namespace("si")
            .log_env_var_prefix("SI")
            .app_modules(vec![NAME, LIB_NAME])
            .interesting_modules(vec![])
            .build()?;

        telemetry_application::init(config, &telemetry_tracker, telemetry_token.clone())?
    };

    startup::startup(NAME).await?;

    if args.verbose > 0 {
        telemetry
            .set_verbosity_and_wait(args.verbose.into())
            .await?;
    }
    debug!(arguments =?args, "parsed cli arguments");

    let config = Config::try_from(args)?;
    debug!(?config, "computed configuration");
    run_server(
        config,
        main_tracker,
        main_token,
        telemetry_tracker,
        telemetry_token,
        telemetry_shutdown,
    )
    .await
}

#[inline]
#[allow(clippy::too_many_arguments)]
async fn run_server(
    config: Config,
    main_tracker: TaskTracker,
    main_token: CancellationToken,
    telemetry_tracker: TaskTracker,
    telemetry_token: CancellationToken,
    telemetry_shutdown: TelemetryShutdownGuard,
) -> Result<()> {
    let server = Server::http(config, main_token.clone()).await?;

    main_tracker.spawn(async move {
        info!("ready to receive requests");
        server.run().await
    });

    shutdown::graceful()
        .group(main_tracker, main_token)
        .group(telemetry_tracker, telemetry_token)
        .telemetry_guard(telemetry_shutdown.into_future())
        .timeout(GRACEFUL_SHUTDOWN_TIMEOUT)
        .wait()
        .await
        .map_err(Into::into)
}
