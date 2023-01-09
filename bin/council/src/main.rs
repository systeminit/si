use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

use color_eyre::Result;
use futures::StreamExt;
use si_data_nats::NatsClient;
use telemetry_application::{
    prelude::*, ApplicationTelemetryClient, TelemetryClient, TelemetryConfig,
};
use tokio::{
    signal,
    sync::{mpsc, watch},
};
use ulid::Ulid;

mod args;
mod config;

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
        .app_modules(vec!["council"])
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

    let config = config::Config::try_from(args)?;

    let nats = NatsClient::new(config.nats()).await?;

    // TODO: create a task per workspace
    let handles = vec![tokio::task::spawn(start_graph_processor(
        nats.clone(),
        shutdown_request_rx.clone(),
    ))];

    futures::future::join_all(handles).await;

    // Receiver can never be dropped as our caller owns it
    shutdown_finished_tx.send(()).await?;
    Ok(())
}

async fn start_graph_processor(nats: NatsClient, mut shutdown_request_rx: watch::Receiver<()>) {
    let mut subscription = loop {
        // TODO: have a queue per workspace
        match nats.subscribe("council").await {
            Ok(sub) => break sub,
            Err(err) => {
                error!("Unable to subscribe to the council request channel on nats: {err}");
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    };

    let mut complete_graph = Graph::default();
    loop {
        tokio::time::sleep(Duration::from_millis(10)).await;

        let request = tokio::select! {
            req = subscription.next() => match req {
                None => continue,
                Some(result) => match result {
                    Ok(msg) => match serde_json::from_slice::<Request>(msg.data()) {
                        Ok(req) => req,
                        Err(err) => {
                            error!("Unable to deserialize request: {err}");
                            continue;
                        }
                    }
                    Err(err) => {
                        error!("Internal error in nats, bailing out: {err}");
                        break;
                    }
                }
            },
            _ = shutdown_request_rx.changed() => {
                info!("Worker task received shutdown notification: stopping");
                break;
            }
        };

        match request {
            Request::CreateValues { request_id, graph } => todo!(),
            Request::Process { request_id, graph } => {
                // Check if graph has intersection with global_graph defined above the loop
                if let Err(err) = nats.publish(format!("council-{request_id}"), "no go").await {
                    panic!("{err}: what do?");
                }
            }
        };
    }
}

pub type Id = Ulid;
pub type Graph = HashMap<Id, Vec<Id>>;

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Request {
    CreateValues { request_id: Id, graph: Graph },
    Process { request_id: Id, graph: Graph },
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Nats(#[from] si_data_nats::Error),
}
