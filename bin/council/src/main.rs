use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    time::Duration,
};

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

// Note: All messages from Pinga include the change set ID.
//
// | Pinga                                                                      | Council                                                                                                                                                                                                                        |
// | -------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
// | Pull job from queue                                                        | N/A                                                                                 |
// |                                                                            |                                                                                     |
// | Inform Council it would like to attribute_value_create_dependent_values_v1 | Add Pinga job ID to create dependent values queue                                   |
// |                                                                            |                                                                                     |
// | Wait for "proceed to create values" message from Council                   | Pop job IDs off queue as "finished creating values" messages are received to inform |
// |                                                                            | the popped ID it can proceed to create                                              |
// |                                                                            |                                                                                     |
// | Generate dependent values graph & send to council                          | Merge graph data into "global" state for change set ID                              |
// |                                                                            |                                                                                     |
// | Wait                                                                       | Check graph data for AttributeValueIds that have an empty "depends_on" list and no  |
// |                                                                            | "processing by" job id, inform first "wanted_by" job id it can process the          |
// |                                                                            | AttributeValueId, store job id in "processing by" for that node.                    |
// |                                                                            |                                                                                     |
// | Process any AttributeValueIds that Council informs us we should process,   | Always unset "processing by", pop job id from "wanted by" list. Remove              |
// | and notify Council when we're done.                                        | AttributeValueId from graph data if "depends on" is empty (both as a key in hash    |
// |                                                                            | map, and as value in "depends on" for all entries in the hash map).                 |
// |                                                                            |                                                                                     |
// | Goto: Wait                                                                 | Goto: Check graph data.                                                             |

async fn start_graph_processor(nats: NatsClient, mut shutdown_request_rx: watch::Receiver<()>) {
    let mut subscription = loop {
        // TODO: have a queue per workspace
        match nats.subscribe("council.*").await {
            Ok(sub) => break sub,
            Err(err) => {
                error!("Unable to subscribe to the council request channel on nats: {err}");
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    };

    let mut value_create_queue = ValueCreationQueue::default();
    let mut complete_graph = WorkspaceGraph::default();
    loop {
        tokio::time::sleep(Duration::from_millis(10)).await;

        let (subject, request) = tokio::select! {
            req = subscription.next() => match req {
                None => continue,
                Some(result) => match result {
                    Ok(msg) => match serde_json::from_slice::<Request>(msg.data()) {
                        Ok(req) => (msg.subject().to_owned(), req),
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

        let job_id = Ulid::from_string(subject.split('.').last().unwrap()).unwrap();

        match request {
            Request::CreateValues => {
                // Move to a falible wrapper
                let _ = job_would_like_to_create_attribute_values(
                    &nats,
                    &mut value_create_queue,
                    job_id,
                )
                .await;
            }
            Request::ValueCreationDone => {
                let _ = job_finished_value_creation(&nats, &mut value_create_queue, job_id).await;
            }
            Request::ValueDependencyGraph {
                change_set_id,
                dependency_graph: Graph,
            } => {
                let _ = register_graph_from_job(&nats, &mut complete_graph, job_id, change_set_id)
                    .await;
            }
            Request::ProcessedValue {
                change_set_id,
                node_id,
            } => {
                let _ = job_processed_a_value(
                    &nats,
                    &mut complete_graph,
                    job_id,
                    change_set_id,
                    node_id,
                )
                .await;
            }
            Request::Bye { change_set_id } => {
                let _ = job_is_going_away(
                    &nats,
                    &mut complete_graph,
                    &mut value_create_queue,
                    job_id,
                    change_set_id,
                )
                .await;
            }
        };
    }
}

pub type Id = Ulid;
pub type Graph = HashMap<Id, Vec<Id>>;

#[derive(Default, Debug)]
pub struct ValueCreationQueue {
    processing: Option<Id>,
    queue: VecDeque<Id>,
}

impl ValueCreationQueue {
    pub fn push(&mut self, new_job_id: Id) {
        self.queue.push_back(new_job_id);
    }

    pub fn is_busy(&self) -> bool {
        self.processing.is_some()
    }

    pub fn fetch_next(&mut self) -> Option<Id> {
        if self.is_busy() {
            return None;
        }
        let next_id = self.queue.pop_front();
        self.processing = next_id;

        next_id
    }
}

#[derive(Default, Debug)]
pub struct WorkspaceGraph {
    workspace_data: HashMap<Id, ChangeSetGraph>,
}

impl WorkspaceGraph {
    pub fn new() -> Self {
        WorkspaceGraph::default()
    }
}

#[derive(Default, Debug)]
pub struct ChangeSetGraph {
    dependency_data: HashMap<Id, HashMap<Id, NodeMetadata>>,
}

#[derive(Default, Debug)]
pub struct NodeMetadata {
    wanted_by: Vec<Id>,
    processing: Option<Id>,
    depends_on: HashSet<Id>,
}

impl NodeMetadata {
    pub fn merge_metadata(&mut self, job_id: Id, dependencies: &Vec<Id>) {
        self.wanted_by.push(job_id);
        self.depends_on.extend(dependencies);
    }
}

impl ChangeSetGraph {
    pub fn merge_dependency_graph(
        &mut self,
        job_id: Id,
        new_dependency_data: Graph,
        change_set_id: Id,
    ) -> Result<(), Error> {
        let change_set_graph_data = self.dependency_data.get_mut(&change_set_id).unwrap();

        for (attribute_value_id, dependencies) in new_dependency_data {
            change_set_graph_data
                .entry(attribute_value_id)
                .and_modify(|node| {
                    node.merge_metadata(job_id, &dependencies);
                })
                .or_insert_with(|| {
                    let mut new_metadata = NodeMetadata::default();
                    new_metadata.merge_metadata(job_id, &dependencies);

                    new_metadata
                });
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Request {
    CreateValues,
    ValueCreationDone,
    ValueDependencyGraph {
        change_set_id: Id,
        dependency_graph: Graph,
    },
    ProcessedValue {
        change_set_id: Id,
        node_id: Id,
    },
    Bye {
        change_set_id: Id,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Response {
    OkToCreate,
    OkToProcess { node_ids: Vec<Id> },
    BeenProcessed { node_id: Id },
    Shutdown,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Nats(#[from] si_data_nats::Error),
}

pub async fn job_would_like_to_create_attribute_values(
    nats: &NatsClient,
    value_create_queue: &mut ValueCreationQueue,
    job_id: Id,
) -> Result<(), Error> {
    value_create_queue.push(job_id);

    Ok(())
}

pub async fn job_finished_value_creation(
    nats: &NatsClient,
    value_create_queue: &mut ValueCreationQueue,
    job_id: Id,
) -> Result<(), Error> {
    todo!()
}

pub async fn register_graph_from_job(
    nats: &NatsClient,
    complete_graph: &mut WorkspaceGraph,
    job_id: Id,
    change_set_id: Id,
) -> Result<(), Error> {
    todo!()
}

pub async fn job_processed_a_value(
    nats: &NatsClient,
    complete_graph: &mut WorkspaceGraph,
    job_id: Id,
    change_set_id: Id,
    node_id: Id,
) -> Result<(), Error> {
    todo!()
}

pub async fn job_is_going_away(
    nats: &NatsClient,
    complete_graph: &mut WorkspaceGraph,
    value_create_queue: &mut ValueCreationQueue,
    job_id: Id,
    change_set_id: Id,
) -> Result<(), Error> {
    todo!()
}
