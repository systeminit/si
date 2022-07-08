use std::sync::Arc;

use color_eyre::Result;
use sdf::{Config, FaktoryProducer, Server};
use telemetry::{
    tracing::{debug, info},
    TelemetryClient,
};
use tokio::runtime::Runtime;

mod args;

const RT_DEFAULT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024 * 3;

fn main() {
    std::thread::Builder::new()
        .stack_size(RT_DEFAULT_THREAD_STACK_SIZE)
        .spawn(move || {
            let runtime = Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .thread_stack_size(RT_DEFAULT_THREAD_STACK_SIZE)
                    .enable_all()
                    .build()?,
            );
            runtime.block_on(async_main(runtime.clone()))
        })
        .expect("pinga thread failed")
        .join()
        .expect("pinga thread panicked")
        .expect("pinga thread join failed");
}

async fn async_main(runtime: Arc<Runtime>) -> Result<()> {
    color_eyre::install()?;
    let config = telemetry::Config::builder()
        .service_name("pinga")
        .service_namespace("si")
        .app_modules(vec!["pinga", "sdf"])
        .build()?;
    let telemetry = telemetry::init(config)?;
    let args = args::parse();

    run(args, telemetry, runtime).await
}

async fn run(
    args: args::Args,
    mut telemetry: telemetry::Client,
    runtime: Arc<Runtime>,
) -> Result<()> {
    if args.verbose > 0 {
        telemetry.set_verbosity(args.verbose.into()).await?;
    }
    debug!(arguments =?args, "parsed cli arguments");

    Server::init()?;

    // TODO(fnichol): we have a mutex poisoning panic that happens, but is avoided if opentelemetry
    // is not running when the migrations are. For the moment we'll disable otel until after the
    // migrations, which means we miss out on some good migration telemetry in honeycomb, but the
    // service boots??
    //
    // See: https://app.shortcut.com/systeminit/story/1934/sdf-mutex-poison-panic-on-launch-with-opentelemetry-exporter
    let _disable_opentelemetry = args.disable_opentelemetry;
    telemetry.disable_opentelemetry().await?;
    // if args.disable_opentelemetry {
    //     telemetry.disable_opentelemetry().await?;
    // }

    if let (Some(secret_key_path), Some(public_key_path)) = (
        &args.generate_cyclone_secret_key_path,
        &args.generate_cyclone_public_key_path,
    ) {
        info!(
            "Generating Cyclone key pair at: (secret = {}, public = {})",
            secret_key_path.display(),
            public_key_path.display()
        );
        Server::generate_cyclone_key_pair(secret_key_path, public_key_path).await?;
        return Ok(());
    }

    let config = Config::try_from(args)?;

    let encryption_key = Server::load_encryption_key(config.cyclone_encryption_key_path()).await?;

    let nats = Server::connect_to_nats(config.nats()).await?;

    let faktory_conn = FaktoryProducer::new(config.faktory())?;

    let pg_pool = Server::create_pg_pool(config.pg_pool()).await?;

    let veritech = Server::create_veritech_client(nats.clone());

    Server::start_faktory_job_executor(
        pg_pool.clone(),
        nats.clone(),
        faktory_conn.clone(),
        config.faktory().to_owned(),
        veritech.clone(),
        encryption_key,
        runtime,
    )
    .await;

    Ok(())
}
