use color_eyre::Result;
use telemetry_application::{
    prelude::*, ApplicationTelemetryClient, TelemetryClient, TelemetryConfig,
};
use tokio::sync::watch;

mod args;

const RT_DEFAULT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024 * 3;

fn main() {
    std::thread::Builder::new()
        .stack_size(RT_DEFAULT_THREAD_STACK_SIZE)
        .name("bin/council-std::thread".to_owned())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .thread_stack_size(RT_DEFAULT_THREAD_STACK_SIZE)
                .thread_name("bin/council-tokio::runtime".to_owned())
                .enable_all()
                .build()?;
            runtime.block_on(async_main())
        })
        .expect("council thread failed")
        .join()
        .expect("council thread panicked")
        .expect("council thread join failed");
}

async fn async_main() -> Result<()> {
    color_eyre::install()?;
    let config = TelemetryConfig::builder()
        .service_name("council")
        .service_namespace("si")
        .log_env_var_prefix("SI")
        .app_modules(vec!["council", "council_server"])
        .build()?;
    let telemetry = telemetry_application::init(config)?;
    let args = args::parse();

    let (_shutdown_request_tx, shutdown_request_rx) = watch::channel(());
    tokio::task::spawn(run(args, telemetry, shutdown_request_rx)).await??;

    Ok(())
}

async fn run(
    args: args::Args,
    mut telemetry: ApplicationTelemetryClient,
    shutdown_request_rx: watch::Receiver<()>,
) -> Result<()> {
    if args.verbose > 0 {
        telemetry.set_verbosity(args.verbose.into()).await?;
    }
    debug!(arguments =?args, "parsed cli arguments");

    if args.disable_opentelemetry {
        telemetry.disable_opentelemetry().await?;
    }

    let config = council_server::server::Config::try_from(args)?;
    let server = council_server::Server::new_with_config(config).await?;
    let (subscriber_started_tx, _subscriber_started_rx) = watch::channel(());
    server
        .run(subscriber_started_tx, shutdown_request_rx.clone())
        .await?;
    Ok(())
}
