use color_eyre::Result;
use faktory_async::{BeatState, Client, FailConfig};
use futures::{future::FutureExt, Future};
use std::{any::TypeId, panic::AssertUnwindSafe, sync::Arc, time::Duration};
use telemetry::{prelude::*, TelemetryClient};
use tokio::{
    signal,
    sync::{mpsc, watch},
    task::JoinError,
};

use dal::{
    job::consumer::{JobConsumer, JobConsumerError},
    job::definition::{CodeGeneration, DependentValuesUpdate, Qualification, Qualifications},
    CycloneKeyPair, DalContext, DalContextBuilder, JobFailure, JobFailureError, ServicesContext,
    TransactionsError,
};
use dal::{FaktoryProcessor, JobQueueProcessor};
use si_data::{NatsClient, PgPool, PgPoolError};
use uuid::Uuid;

mod args;
mod config;

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
    let config = telemetry::Config::builder()
        .service_name("pinga")
        .service_namespace("si")
        .app_modules(vec!["pinga"])
        .build()?;
    let telemetry = telemetry::init(config)?;
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
    mut telemetry: telemetry::Client,
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
        veritech::EncryptionKey::load(config.cyclone_encryption_key_path()).await?;

    let nats = NatsClient::new(config.nats()).await?;

    let pg_pool = PgPool::new(config.pg_pool()).await?;

    let veritech = veritech::Client::new(nats.clone());

    let (alive_marker, mut job_processor_shutdown_rx) = mpsc::channel(1);

    info!("Creating faktory connection");
    let config = faktory_async::Config::from_uri(
        &config.faktory().url,
        Some("pinga".to_string()),
        Some(Uuid::new_v4().to_string()),
    );

    let client = Client::new(config.clone(), 256);
    info!("Spawned faktory connection.");

    const NUM_TASKS: usize = 10;
    let mut handles = Vec::with_capacity(NUM_TASKS);

    for _ in 0..NUM_TASKS {
        handles.push(tokio::task::spawn(start_job_executor(
            client.clone(),
            pg_pool.clone(),
            nats.clone(),
            veritech.clone(),
            encryption_key,
            shutdown_request_rx.clone(),
            alive_marker.clone(),
        )));
    }
    drop(alive_marker);

    futures::future::join_all(handles).await;

    // Blocks until all FaktoryProcessors are gone so we don't skip jobs that are still being sent to faktory_async
    info!("Waiting for all faktory processors to finish pushing jobs");
    let _ = job_processor_shutdown_rx.recv().await;

    info!("Closing faktory-async client connection");
    if let Err(err) = client.close().await {
        error!("Error closing fakory-async client connection: {err}");
    }

    // Receiver can never be dropped as our caller owns it
    shutdown_finished_tx.send(()).await?;
    Ok(())
}

/// Start the faktory job executor
async fn start_job_executor(
    client: Client,
    pg: PgPool,
    nats: NatsClient,
    veritech: veritech::Client,
    encryption_key: veritech::EncryptionKey,
    mut shutdown_request_rx: watch::Receiver<()>,
    alive_marker: mpsc::Sender<()>,
) {
    let job_processor = Box::new(FaktoryProcessor::new(client.clone(), alive_marker))
        as Box<dyn JobQueueProcessor + Send + Sync>;
    let services_context = ServicesContext::new(
        pg.clone(),
        nats.clone(),
        job_processor,
        veritech.clone(),
        Arc::new(encryption_key),
    );
    let ctx_builder = DalContext::builder(services_context);

    loop {
        tokio::time::sleep(Duration::from_millis(10)).await;

        match client.last_beat().await {
            Ok(BeatState::Ok) => {
                let job = tokio::select! {
                    job = client.fetch(vec!["default".into()]) => match job {
                        Ok(Some(job)) => job,
                        Ok(None) => continue,
                        Err(err) => {
                            error!("Unable to fetch from faktory: {err}");
                            continue;
                        }
                    },
                    _ = shutdown_request_rx.changed() => {
                        info!("Worker task received shutdown notification: stopping");
                        break;
                    }
                };

                let jid = job.id().to_owned();
                match execute_job_fallible(job, ctx_builder.clone()).await {
                    Ok(()) => match client.ack(jid).await {
                        Ok(()) => {}
                        Err(err) => {
                            error!("Ack failed: {err}");
                            continue;
                        }
                    },
                    Err(err) => {
                        error!("Job execution failed: {err}");
                        // TODO: pass backtrace here
                        match client
                            .fail(FailConfig::new(
                                jid,
                                format!("{err:?}"),
                                err.to_string(),
                                None,
                            ))
                            .await
                        {
                            Ok(()) => {}
                            Err(err) => {
                                error!("Fail failed: {err}");
                                continue;
                            }
                        }
                    }
                }
            }
            Ok(BeatState::Quiet) => {
                // Getting a "quiet" state from the faktory server means that
                // someone has gone to the faktory UI and requested that this
                // particular worker finish what it's doing, and gracefully
                // shut down.
                info!("Gracefully shutting down from Faktory request.");
                break;
            }
            Ok(BeatState::Terminate) => {
                warn!("Faktory asked us to terminate");
                break;
            }
            Err(err) => {
                error!("Internal error in faktory-async, bailing out: {err}");
                break;
            }
        }
    }
}

macro_rules! job_match {
    ($job:ident, $( $job_struct:ident ),* ) => {
        match $job.kind() {
            $(
                stringify!($job_struct) => Ok(Box::new($job_struct::try_from($job)?) as Box<dyn JobConsumer + Send + Sync>),
            )*
            kind => Err(JobError::UnknownJobKind(kind.to_owned())),
        }
    };
}

async fn execute_job_fallible(
    job: faktory_async::Job,
    ctx_builder: DalContextBuilder,
) -> Result<(), JobError> {
    info!("Processing {job:?}");

    let job = match job_match!(
        job,
        Qualification,
        Qualifications,
        CodeGeneration,
        DependentValuesUpdate
    ) {
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
enum JobError {
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
    Pg(#[from] PgPoolError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    Faktory(#[from] faktory_async::Error),
    #[error(transparent)]
    JobConsumer(#[from] JobConsumerError),
    #[error(transparent)]
    JoinError(#[from] JoinError),
}
