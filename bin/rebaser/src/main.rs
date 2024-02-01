use color_eyre::Result;
use rebaser_server::{Config, Server};
use telemetry_application::prelude::*;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

mod args;

const RT_DEFAULT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024 * 3;

fn main() -> Result<()> {
    let thread_builder = ::std::thread::Builder::new().stack_size(RT_DEFAULT_THREAD_STACK_SIZE);
    let thread_handler = thread_builder.spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .thread_stack_size(RT_DEFAULT_THREAD_STACK_SIZE)
            .thread_name("bin/rebaser-tokio::runtime")
            .enable_all()
            .build()?
            .block_on(async_main())
    })?;
    thread_handler.join().unwrap()
}

async fn async_main() -> Result<()> {
    color_eyre::install()?;

    let shutdown_token = CancellationToken::new();
    let task_tracker = TaskTracker::new();

    let config = TelemetryConfig::builder()
        .service_name("rebaser")
        .service_namespace("si")
        .log_env_var_prefix("SI")
        .app_modules(vec!["rebaser", "rebaser_server"])
        .build()?;
    let mut telemetry = telemetry_application::init(config, &task_tracker, shutdown_token.clone())?;
    let args = args::parse();

    if args.verbose > 0 {
        telemetry.set_verbosity(args.verbose.into()).await?;
    }
    debug!(arguments =?args, "parsed cli arguments");

    let config = Config::try_from(args)?;

    Server::from_config(config).await?.run().await?;

    {
        shutdown_token.cancel();
        task_tracker.wait().await;
    }

    Ok(())
}
