use faktory_async::{BeatState, Client, FailConfig};
use futures::{future::FutureExt, Future};
use std::{any::TypeId, panic::AssertUnwindSafe, sync::Arc, time::Duration};
use telemetry::{prelude::*, TelemetryClient};
use tokio::{
    signal,
    sync::{broadcast, mpsc},
    task::JoinError,
};

use dal::{
    job::consumer::{JobConsumer, JobConsumerError},
    job::definition::component_post_processing::ComponentPostProcessing,
    job::definition::dependent_values_update::DependentValuesUpdate,
    CycloneKeyPair, DalContext, DalContextBuilder, JobFailure, JobFailureError, MigrationMode,
    ServicesContext, TransactionsError,
};
use dal::{FaktoryProcessor, JobQueueProcessor};
use si_data::{NatsClient, PgPool, PgPoolError};
use uuid::Uuid;

mod args;
mod config;

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
        .app_modules(vec!["pinga"])
        .build()?;
    let telemetry = telemetry::init(config)?;
    let args = args::parse();

    let (shutdown_send, shutdown_receive) = broadcast::channel(1);

    let run_result = tokio::task::spawn(run(args, telemetry, shutdown_receive));

    let mut sigterm_watcher = signal::unix::signal(signal::unix::SignalKind::terminate())?;

    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("SIGINT received; initiating graceful shutdown");
            shutdown_send.send(())?;
        }
        _ = sigterm_watcher.recv() => {
            info!("SIGTERM received; initiating graceful shutdown");
            shutdown_send.send(())?;
        }
    }

    // We should return any errors that happened in our run method's task.
    run_result.await?
}

async fn run(
    args: args::Args,
    mut telemetry: telemetry::Client,
    mut shutdown_channel: broadcast::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    if let MigrationMode::Run | MigrationMode::RunAndQuit = config.migration_mode() {
        info!("Running migrations");

        let faktory_config = faktory_async::Config::from_uri(
            &config.faktory().url,
            Some("pinga-migratoins".to_string()),
            None,
        );
        let client = Client::new(faktory_config, 128);
        let job_processor =
            Box::new(FaktoryProcessor::new(client)) as Box<dyn JobQueueProcessor + Send + Sync>;

        dal::migrate_all(
            &pg_pool,
            &nats,
            job_processor,
            veritech.clone(),
            &encryption_key,
        )
        .await?;
        if let MigrationMode::RunAndQuit = config.migration_mode() {
            info!(
                "migration mode is {}, shutting down",
                config.migration_mode()
            );
            return Ok(());
        }
    } else {
        trace!("migration mode is skip, not running migrations");
    }

    loop {
        info!("Creating faktory connection");
        let config = faktory_async::Config::from_uri(
            &config.faktory().url,
            Some("pinga".to_string()),
            Some(Uuid::new_v4().to_string()),
        );

        let client = Client::new(config.clone(), 256);
        info!("Spawned faktory connection.");

        const NUM_TASKS: usize = 5;
        let (task_wait_send, mut task_wait_recv) = mpsc::channel(1);
        let mut handles = Vec::with_capacity(NUM_TASKS + 1);
        let beat_client = client.clone();
        // The heartbeat gets its own shutdown channel, because we don't want it to shut down until
        // _after_ all the worker tasks have already shutdown. If we shut down the heartbeat before
        // then, the faktory server is likely to kill our TCP connection, and that is A Bad Thing™.
        let (beat_shutdown_send, mut beat_shutdown_channel) = broadcast::channel(1);
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
                // We use the "old" receive end in the async block,
                // instead of the newly created one, so that we don't
                // risk missing a shutdown message. We will then use
                // the "new" one next time around in the loop.
                let new_shutdown_channel = beat_shutdown_channel.resubscribe();
                let shutdown_future = async move {
                    let _ = beat_shutdown_channel.recv().await;
                };
                beat_shutdown_channel = new_shutdown_channel;
                tokio::select! {
                    _ = shutdown_future => {
                        info!("Heartbeat task received shutdown notification; stopping.");
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_secs(15)) => {}
                }
            }
        }));

        for _ in 0..NUM_TASKS {
            handles.push(tokio::task::spawn(start_job_executor(
                client.clone(),
                pg_pool.clone(),
                nats.clone(),
                veritech.clone(),
                encryption_key,
                shutdown_channel.resubscribe(),
                task_wait_send.clone(),
            )));
        }

        let new_shutdown_channel = shutdown_channel.resubscribe();
        let shutdown_future = async move {
            let _ = shutdown_channel.recv().await;
        };
        shutdown_channel = new_shutdown_channel;

        // Watch for either all job tasks exiting on their own, or us
        // receiving a shutdown signal. If they all exited on their
        // own, we should restart them, but if we received a shutdown
        // signal, then we should wait for them to exit before exiting
        // the loop that would respawn them (and the heartbeat).
        tokio::select! {
            _ = shutdown_future => {
                info!("Main executor loop received shutdown notification; stopping.");
                drop(task_wait_send);
                task_wait_recv.recv().await;
                // Now that all the worker tasks have exited, we can
                // safely stop sending heartbeats to the faktory
                // server.
                let _ = beat_shutdown_send.send(());
                info!("Closing faktory-async client connection");
                if let Err(err) = client.close() {
                    error!("Error closing faktory-async client connection: {err}");
                }
                info!("All executor jobs finished; exiting main executor loop.");
                break Ok(());
            }
            _ = futures::future::join_all(handles) => {}
        }

        info!("Closing faktory-async client connection");
        if let Err(err) = client.close() {
            error!("Erro closing fakory-async client connection: {err}");
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
    mut shutdown_channel: broadcast::Receiver<()>,
    _sender: mpsc::Sender<()>,
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
        match shutdown_channel.try_recv() {
            Err(broadcast::error::TryRecvError::Empty) => {
                // No shutdown signal yet; nothing to do.
            }
            _ => {
                // The sender has shut down, we're lagged (on a channel that can only hold 1 thing),
                // or we got something over the channel. In any of these scenarios, we should be
                // shutting down.
                info!("Worker task received shutdown notification: stopping");
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;

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
