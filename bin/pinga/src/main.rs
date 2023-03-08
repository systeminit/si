use std::{any::TypeId, panic::AssertUnwindSafe, sync::Arc, time::Duration};

use color_eyre::Result;
use dal::{
    job::consumer::{JobConsumer, JobConsumerError, JobInfo},
    job::definition::{DependentValuesUpdate, FixesJob, WorkflowRun},
    CycloneKeyPair, DalContext, DalContextBuilder, JobFailure, JobFailureError, JobQueueProcessor,
    ServicesContext, TransactionsError,
};
use futures::{future::FutureExt, Future};
use si_data_nats::NatsClient;
use si_data_pg::{PgPool, PgPoolError};
use telemetry_application::{
    prelude::*, ApplicationTelemetryClient, TelemetryClient, TelemetryConfig,
};
use tokio::{
    signal,
    sync::{mpsc, watch},
    task::JoinError,
};

mod args;
mod config;
mod transport;

use transport::{Consumer, ExecutionState};

type Transport = si_data_nats::Subscription;

const RT_DEFAULT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024 * 3;

fn main() {
    std::thread::Builder::new()
        .stack_size(RT_DEFAULT_THREAD_STACK_SIZE)
        .name("bin/pinga-std::thread".to_owned())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .thread_stack_size(RT_DEFAULT_THREAD_STACK_SIZE)
                .thread_name("bin/pinga-tokio::runtime".to_owned())
                .enable_all()
                .build()?;
            runtime.block_on(async_main())
        })
        .expect("pinga thread failed")
        .join()
        .expect("pinga thread panicked")
        .expect("pinga thread join failed");
}

async fn async_main() -> Result<()> {
    color_eyre::install()?;
    let config = TelemetryConfig::builder()
        .service_name("pinga")
        .service_namespace("si")
        .log_env_var_prefix("SI")
        .app_modules(vec!["pinga"])
        .build()?;
    let telemetry = telemetry_application::init(config)?;
    let args = args::parse();

    let (shutdown_request_tx, shutdown_request_rx) = watch::channel(());
    let (shutdown_finished_tx, mut shutdown_finished_rx) = mpsc::channel(1);

    let run_result = tokio::task::spawn(run(
        args,
        telemetry,
        shutdown_request_rx,
        shutdown_finished_tx,
    ));

    let mut sigterm_watcher = signal::unix::signal(signal::unix::SignalKind::terminate())?;

    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("SIGINT received; initiating graceful shutdown");
            // fails if shutdown_request_rx has been dropped, which means shutdown has already happened
            let _ = shutdown_request_tx.send(());
        }
        _ = sigterm_watcher.recv() => {
            info!("SIGTERM received; initiating graceful shutdown");
            // fails if shutdown_request_rx has been dropped, which means shutdown has already happened
            let _ = shutdown_request_tx.send(());
        }
        // fails if shutdown_finished_tx has been dropped, which means shutdown has already happened
        _ = shutdown_finished_rx.recv() => {},
    }

    // Joins run(...) spawned task
    run_result.await?
}

async fn run(
    args: args::Args,
    mut telemetry: ApplicationTelemetryClient,
    shutdown_request_rx: watch::Receiver<()>,
    shutdown_finished_tx: mpsc::Sender<()>,
) -> Result<()> {
    if args.verbose > 0 {
        telemetry.set_verbosity(args.verbose.into()).await?;
    }
    debug!(arguments =?args, "parsed cli arguments");

    dal::init()?;

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
        CycloneKeyPair::create(secret_key_path, public_key_path).await?;
        return Ok(());
    }

    let config = config::Config::try_from(args)?;

    let encryption_key =
        veritech_client::EncryptionKey::load(config.cyclone_encryption_key_path()).await?;

    let nats = NatsClient::new(config.nats()).await?;

    let pg_pool = PgPool::new(config.pg_pool()).await?;

    let veritech = veritech_client::Client::new(nats.clone());

    let (alive_marker, mut job_processor_shutdown_rx) = mpsc::channel(1);

    info!("Creating transport connection");

    let client = Transport::connect(&config).await?;

    info!("Spawned transport connection.");

    let job_processor = Transport::new_processor(client.clone(), alive_marker);

    const NUM_TASKS: usize = 10;
    let mut handles = Vec::with_capacity(NUM_TASKS);
    for _ in 0..NUM_TASKS {
        handles.push(tokio::task::spawn(start_job_executor(
            Transport::subscribe(&client).await?,
            pg_pool.clone(),
            nats.clone(),
            veritech.clone(),
            job_processor.clone(),
            encryption_key,
            shutdown_request_rx.clone(),
        )));
    }
    drop(job_processor);

    futures::future::join_all(handles).await;

    // Blocks until all JobProcessors are gone so we don't skip jobs that are still being sent to transport
    info!("Waiting for all job processors to finish pushing jobs");
    let _ = job_processor_shutdown_rx.recv().await;

    info!("Closing transport client connection");
    if let Err(err) = Transport::end(client).await {
        error!("Error closing transport client connection: {err}");
    }

    // Receiver can never be dropped as our caller owns it
    shutdown_finished_tx.send(()).await?;
    Ok(())
}

/// Start the job executor
async fn start_job_executor(
    mut subscription: Transport,
    pg: PgPool,
    nats: NatsClient,
    veritech: veritech_client::Client,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    encryption_key: veritech_client::EncryptionKey,
    mut shutdown_request_rx: watch::Receiver<()>,
) {
    let services_context = ServicesContext::new(
        pg.clone(),
        nats.clone(),
        job_processor,
        veritech.clone(),
        Arc::new(encryption_key),
        "council".to_owned(),
        None,
    );
    let ctx_builder = DalContext::builder(services_context);

    loop {
        tokio::time::sleep(Duration::from_millis(10)).await;

        match subscription.fetch_next(&mut shutdown_request_rx).await {
            ExecutionState::Stop => break,
            ExecutionState::Idle => {}
            ExecutionState::Process(job) => {
                let jid = job.id.to_owned();
                let result = execute_job_fallible(job, ctx_builder.clone()).await;
                subscription.post_process(jid, result).await;
            }
        }
    }
}

macro_rules! job_match {
    ($job:ident, $( $job_struct:ident ),* $(,)* ) => {
        match $job.kind() {
            $(
                stringify!($job_struct) => Ok(Box::new($job_struct::try_from($job)?) as Box<dyn JobConsumer + Send + Sync>),
            )*
            kind => Err(JobError::UnknownJobKind(kind.to_owned())),
        }
    };
}

async fn execute_job_fallible(
    job: JobInfo,
    ctx_builder: DalContextBuilder,
) -> Result<(), JobError> {
    info!("Processing {job:?}");

    let job = match job_match!(job, DependentValuesUpdate, WorkflowRun, FixesJob,) {
        Ok(job) => job,
        Err(err) => return Err(err),
    };

    let outer_ctx_builder = ctx_builder.clone();
    let access_builder = job.access_builder();
    let visibility = job.visibility();
    let job_type_name = job.type_name();

    let result = panic_wrapper(
        format!("Job execution: {job_type_name}"),
        job.run_job(outer_ctx_builder),
    )
    .await;

    if let Err(err) = result {
        error!("Job execution failed: {err}");

        let err_message = err.to_string();
        panic_wrapper(format!("Job failure reporting: {job_type_name}"), async {
            let ctx = ctx_builder.build(access_builder.build(visibility)).await?;

            JobFailure::new(&ctx, job_type_name.clone(), err_message).await?;

            ctx.commit().await?;
            Result::<(), JobFailureError>::Ok(())
        })
        .await?;

        return Err(err);
    }
    Ok(())
}

async fn panic_wrapper<
    E: std::error::Error + Send + Sync + 'static,
    F: Future<Output = Result<(), E>>,
>(
    label: String,
    future: F,
) -> Result<(), JobError> {
    match AssertUnwindSafe(future).catch_unwind().await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => Err(JobError::ExecutionFailed(label, Box::new(err))),
        Err(any) => {
            // Note: Technically panics can be of any form, but most should be &str or String
            match any.downcast::<String>() {
                Ok(msg) => Err(JobError::Panic(label, *msg)),
                Err(any) => match any.downcast::<&str>() {
                    Ok(msg) => Err(JobError::Panic(label, msg.to_string())),
                    Err(any) => {
                        let id = any.type_id();
                        error!("{label}: Panic message downcast failed of {id:?}",);
                        Err(JobError::UnknownPanic(label, id))
                    }
                },
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JobError {
    #[error("unknown job kind {0}")]
    UnknownJobKind(String),
    #[error("execution failed for {0}: {1}")]
    ExecutionFailed(String, Box<dyn std::error::Error + Send + Sync>),
    #[error("{0} panicked: {1}")]
    Panic(String, String),
    #[error("{0} panicked with an unknown payload of {1:?}")]
    UnknownPanic(String, TypeId),

    #[error(transparent)]
    FailureReporting(#[from] JobFailureError),
    #[error(transparent)]
    Nats(#[from] si_data_nats::Error),
    #[error(transparent)]
    Pg(#[from] PgPoolError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    JobConsumer(#[from] JobConsumerError),
    #[error(transparent)]
    JoinError(#[from] JoinError),
}
