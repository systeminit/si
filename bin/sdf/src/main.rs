#![recursion_limit = "256"]

use std::{
    path::PathBuf,
    time::Duration,
};

use innit_client::InnitClient;
use sdf_server::{
    Config,
    Migrator,
    Server,
    SnapshotGarbageCollector,
    key_generation,
};
use si_service::{
    color_eyre,
    prelude::*,
    rt,
    shutdown,
    startup,
    telemetry_application::{
        self,
        TelemetryShutdownGuard,
    },
};

use crate::args::{
    NAME,
    load_config_with_provider,
};

mod args;

const BIN_NAME: &str = env!("CARGO_BIN_NAME");
const LIB_NAME: &str = concat!(env!("CARGO_BIN_NAME"), "_server");

const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(60 * 10);

fn main() -> Result<()> {
    rt::block_on(BIN_NAME, async_main())
}

async fn async_main() -> Result<()> {
    let main_tracker = TaskTracker::new();
    let main_token = CancellationToken::new();
    let helping_tasks_tracker = TaskTracker::new();
    let helping_tasks_token = CancellationToken::new();
    let telemetry_tracker = TaskTracker::new();
    let telemetry_token = CancellationToken::new();

    color_eyre::install()?;
    let args = args::parse();
    let (mut telemetry, telemetry_shutdown) = {
        let config = TelemetryConfig::builder()
            .force_color(args.force_color.then_some(true))
            .no_color(args.no_color.then_some(true))
            .log_format(args.log_json.then_some(LogFormat::Json).unwrap_or_default())
            .log_file_directory(args.log_file_directory.clone())
            .tokio_console(args.tokio_console)
            .service_name(BIN_NAME)
            .service_namespace("si")
            .log_env_var_prefix("SI")
            .app_modules(vec![BIN_NAME, LIB_NAME])
            .interesting_modules(vec![
                "dal",
                "si_data_nats",
                "si_data_pg",
                "si_layer_cache",
                "si_service",
                "foyer_storage",
            ])
            .build()?;

        telemetry_application::init(config, &telemetry_tracker, telemetry_token.clone())?
    };

    startup::startup(BIN_NAME).await?;

    if args.verbose > 0 {
        telemetry
            .set_verbosity_and_wait(args.verbose.into())
            .await?;
    }
    debug!(arguments =?args, "parsed cli arguments");

    if let Some((secret_key_path, public_key_path)) = args.generating_veritech_key_pair() {
        generate_veritech_key_pair(
            secret_key_path,
            public_key_path,
            main_tracker,
            main_token,
            telemetry_tracker,
            telemetry_token,
            telemetry_shutdown,
        )
        .await
    } else if let Some(symmetric_key_path) = args.generating_symmetric_key() {
        generate_symmetric_key(
            symmetric_key_path,
            main_tracker,
            main_token,
            telemetry_tracker,
            telemetry_token,
            telemetry_shutdown,
        )
        .await
    } else {
        debug!("creating innit-client...");
        let provider = Some(InnitClient::new_from_environment(NAME.to_string()).await?);
        let config = load_config_with_provider(args, provider).await?;

        debug!(?config, "computed configuration");

        if config.migration_mode().is_run_and_quit() {
            migrate_and_quit(
                config,
                main_tracker,
                main_token,
                helping_tasks_tracker,
                helping_tasks_token,
                telemetry_tracker,
                telemetry_token,
                telemetry_shutdown,
            )
            .await
        } else if config.migration_mode().is_garbage_collect_snapshots() {
            garbage_collect_snapshots(
                config,
                main_tracker,
                main_token,
                helping_tasks_tracker,
                helping_tasks_token,
                telemetry_tracker,
                telemetry_token,
                telemetry_shutdown,
            )
            .await
        } else {
            run_server(
                config,
                main_tracker,
                main_token,
                helping_tasks_tracker,
                helping_tasks_token,
                telemetry_tracker,
                telemetry_token,
                telemetry_shutdown,
            )
            .await
        }
    }
}

#[inline]
#[allow(clippy::too_many_arguments)]
async fn run_server(
    config: Config,
    main_tracker: TaskTracker,
    main_token: CancellationToken,
    helping_tasks_tracker: TaskTracker,
    helping_tasks_token: CancellationToken,
    telemetry_tracker: TaskTracker,
    telemetry_token: CancellationToken,
    telemetry_shutdown: TelemetryShutdownGuard,
) -> Result<()> {
    let migration_mode_is_run = config.migration_mode().is_run();
    let is_dev_mode = config.dev_mode();

    let server = Server::from_config(
        config,
        main_token.clone(),
        &helping_tasks_tracker,
        helping_tasks_token.clone(),
    )
    .await?;

    if migration_mode_is_run {
        // If migrations fail, process will exit with an error.
        //
        // Note that signals are not yet listened for, so a `SIGTERM`/`SIGINT` will cancel this
        // operation and simply exit.
        server.migrator().run_migrations(is_dev_mode, false).await?;
    }

    main_tracker.spawn(async move {
        info!("ready to receive requests");
        server.run().await
    });

    shutdown::graceful()
        .group(main_tracker, main_token)
        .group(helping_tasks_tracker, helping_tasks_token)
        .group(telemetry_tracker, telemetry_token)
        .telemetry_guard(telemetry_shutdown.into_future())
        .timeout(GRACEFUL_SHUTDOWN_TIMEOUT)
        .wait()
        .await
        .map_err(Into::into)
}

#[inline]
#[allow(clippy::too_many_arguments)]
async fn migrate_and_quit(
    config: Config,
    main_tracker: TaskTracker,
    main_token: CancellationToken,
    helping_tasks_tracker: TaskTracker,
    helping_tasks_token: CancellationToken,
    telemetry_tracker: TaskTracker,
    telemetry_token: CancellationToken,
    telemetry_shutdown: TelemetryShutdownGuard,
) -> Result<()> {
    let migrator =
        Migrator::from_config(config, &helping_tasks_tracker, helping_tasks_token.clone()).await?;

    let handle = main_tracker.spawn(migrator.run_migrations(false, false));

    shutdown::graceful_with_handle(handle)
        .group(main_tracker, main_token)
        .group(helping_tasks_tracker, helping_tasks_token)
        .group(telemetry_tracker, telemetry_token)
        .telemetry_guard(telemetry_shutdown.into_future())
        .timeout(GRACEFUL_SHUTDOWN_TIMEOUT)
        .wait()
        .await
        .map_err(Into::into)
}

#[inline]
#[allow(clippy::too_many_arguments)]
async fn garbage_collect_snapshots(
    config: Config,
    main_tracker: TaskTracker,
    main_token: CancellationToken,
    helping_tasks_tracker: TaskTracker,
    helping_tasks_token: CancellationToken,
    telemetry_tracker: TaskTracker,
    telemetry_token: CancellationToken,
    telemetry_shutdown: TelemetryShutdownGuard,
) -> Result<()> {
    let garbage_collector =
        SnapshotGarbageCollector::new(config, &helping_tasks_tracker, helping_tasks_token.clone())
            .await?;

    let handle = main_tracker.spawn(garbage_collector.garbage_collect_snapshots());

    shutdown::graceful_with_handle(handle)
        .group(main_tracker, main_token)
        .group(helping_tasks_tracker, helping_tasks_token)
        .group(telemetry_tracker, telemetry_token)
        .telemetry_guard(telemetry_shutdown.into_future())
        .timeout(GRACEFUL_SHUTDOWN_TIMEOUT)
        .wait()
        .await
        .map_err(Into::into)
}

#[inline]
async fn generate_veritech_key_pair(
    secret_key_path: PathBuf,
    public_key_path: PathBuf,
    main_tracker: TaskTracker,
    main_token: CancellationToken,
    telemetry_tracker: TaskTracker,
    telemetry_token: CancellationToken,
    telemetry_shutdown: TelemetryShutdownGuard,
) -> Result<()> {
    info!(
        secret = %secret_key_path.display(),
        public = %public_key_path.display(),
        "generating veritech key pair",
    );

    let handle = main_tracker.spawn(key_generation::generate_veritech_key_pair(
        secret_key_path,
        public_key_path,
    ));

    shutdown::graceful_with_handle(handle)
        .group(main_tracker, main_token)
        .group(telemetry_tracker, telemetry_token)
        .telemetry_guard(telemetry_shutdown.into_future())
        .timeout(GRACEFUL_SHUTDOWN_TIMEOUT)
        .wait()
        .await
        .map_err(Into::into)
}

#[inline]
async fn generate_symmetric_key(
    symmetric_key_path: PathBuf,
    main_tracker: TaskTracker,
    main_token: CancellationToken,
    telemetry_tracker: TaskTracker,
    telemetry_token: CancellationToken,
    telemetry_shutdown: TelemetryShutdownGuard,
) -> Result<()> {
    info!(path = %symmetric_key_path.display(), "enerating symmetric key");

    let handle = main_tracker.spawn(key_generation::generate_symmetric_key(symmetric_key_path));

    shutdown::graceful_with_handle(handle)
        .group(main_tracker, main_token)
        .group(telemetry_tracker, telemetry_token)
        .telemetry_guard(telemetry_shutdown.into_future())
        .timeout(GRACEFUL_SHUTDOWN_TIMEOUT)
        .wait()
        .await
        .map_err(Into::into)
}
