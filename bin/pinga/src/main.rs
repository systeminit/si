use faktory_async::{BeatState, Client, FailConfig};
use futures::{future::FutureExt, Future};
use std::{any::TypeId, panic::AssertUnwindSafe, sync::Arc, time::Duration};
use telemetry::{prelude::*, TelemetryClient};
use tokio::task::JoinError;

use dal::{
    job::consumer::{JobConsumer, JobConsumerError},
    job::definition::component_post_processing::ComponentPostProcessing,
    job::definition::dependent_values_update::DependentValuesUpdate,
    DalContext, DalContextBuilder, JobFailure, JobFailureError, ServicesContext, TransactionsError,
};
use sdf::{Config, FaktoryProcessor, JobQueueProcessor, Server};
use si_data::{NatsClient, PgPool, PgPoolError};

mod args;

type Result<T, E = JobError> = std::result::Result<T, E>;

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

async fn async_main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    color_eyre::install()?;
    let config = telemetry::Config::builder()
        .service_name("pinga")
        .service_namespace("si")
        .app_modules(vec!["pinga", "sdf"])
        .build()?;
    let telemetry = telemetry::init(config)?;
    let args = args::parse();

    run(args, telemetry).await
}

async fn run(
    args: args::Args,
    mut telemetry: telemetry::Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    let pg_pool = Server::create_pg_pool(config.pg_pool()).await?;

    let veritech = Server::create_veritech_client(nats.clone());

    loop {
        let mut config = faktory_async::Config::from_uri(config.faktory().url.clone());
        config.does_consume();
        config.set_hostname("pinga");

        let client = match Client::new(&config).await {
            Ok(c) => {
                info!("Connected to faktory");
                c
            }
            Err(err) => {
                error!("Job execution failed: {err}");
                tokio::time::sleep(Duration::from_millis(5000)).await;
                continue;
            }
        };

        const NUM_TASKS: usize = 5;
        let mut handles = Vec::with_capacity(NUM_TASKS + 1);
        let beat_client = client.clone();
        handles.push(tokio::task::spawn(async move {
            loop {
                match beat_client.beat().await {
                    Ok(BeatState::Ok) => {}
                    // Both the Quiet and Terminate states from the
                    // faktory server mean that we should initiate a
                    // shutdown.
                    Ok(BeatState::Quiet) | Ok(BeatState::Terminate) => break,
                    Err(err) => {
                        error!("Beat failed: {err}");
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_secs(15)).await;
            }
        }));

        for _ in 0..NUM_TASKS {
            handles.push(tokio::task::spawn(start_job_executor(
                client.clone(),
                pg_pool.clone(),
                nats.clone(),
                veritech.clone(),
                encryption_key,
            )));
        }
        futures::future::join_all(handles).await;
        if let Err(err) = client.close().await {
            error!("Unable to close client connection: {err}");
        }
    }
}

/// Start the faktory job executor
async fn start_job_executor(
    client: Client,
    pg: PgPool,
    nats: NatsClient,
    veritech: veritech::Client,
    encryption_key: veritech::EncryptionKey,
) {
    let job_processor =
        Box::new(FaktoryProcessor::new(client.clone())) as Box<dyn JobQueueProcessor + Send + Sync>;
    let services_context = ServicesContext::new(
        pg.clone(),
        nats.clone(),
        job_processor,
        veritech.clone(),
        Arc::new(encryption_key),
    );
    let ctx_builder = Arc::new(DalContext::builder(services_context));

    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;

        match client.reconnect_if_needed().await {
            Ok(did_reconnect) => {
                if did_reconnect {
                    info!("Reconnected to faktory");
                }
            }
            Err(err) => error!("Could not reconnect to faktory: {err})"),
        };

        match client.last_beat().await {
            Ok(BeatState::Ok) => {
                let job = match client.fetch(&["default".to_owned()]).await {
                    Ok(Some(job)) => job,
                    Ok(None) => {
                        continue;
                    }
                    Err(err) => {
                        error!("Unable to fetch from faktory: {err}");
                        continue;
                    }
                };

                let jid = job.id().to_owned();
                match start_job_executor_fallible(job, ctx_builder.clone(), client.clone()).await {
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
                error!("Connection closed: {err}");
                continue;
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

async fn start_job_executor_fallible(
    job: faktory_async::Job,
    ctx_builder: Arc<DalContextBuilder>,
    client: Client,
) -> Result<()> {
    let job_task = tokio::task::spawn(execute_job_fallible(job, ctx_builder));

    loop {
        // The sleep in this loop means that any job will take _at minimum_ 1 second to complete,
        // and will really only complete in 1 second increments.
        //
        // Ideally, we'd do something like selecting on the job task with a timeout that let the
        // task keep running if the timeout is reached, or select on the job_task and the future for
        // the sleep. Unfortunately, tokio::select! both takes ownership of what's being selected,
        // and cancels everything that's not the first one to finish. Since we don't want to limit
        // all jobs to 1 second, and want to be able to check multiple times if the job task is
        // done, this was the least awkward way to do it for now.
        tokio::time::sleep(Duration::from_secs(1)).await;
        if !job_task.is_finished() {
            if let Ok(BeatState::Terminate) = client.last_beat().await {
                job_task.abort();
                break;
            }
        } else {
            // The job task has finished, so there's no point in
            // continuing to poll the client state watching to see if
            // we should terminate early.
            break;
        }
    }

    job_task.await?
}

async fn execute_job_fallible(
    job: faktory_async::Job,
    ctx_builder: Arc<DalContextBuilder>,
) -> Result<()> {
    info!("Processing {job:?}");

    let job = match job_match!(job, ComponentPostProcessing, DependentValuesUpdate) {
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
            let mut txns = ctx_builder.transactions_starter().await?;
            let txns = txns.start().await?;
            let ctx = ctx_builder.build(access_builder.build(visibility), &txns);

            JobFailure::new(&ctx, job_type_name.clone(), err_message).await?;

            txns.commit().await?;
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
