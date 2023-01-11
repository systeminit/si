use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    time::Duration,
};
use council::{Id, Graph, Request, Response};

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
        match nats.subscribe("council.*").await {
            Ok(sub) => break sub,
            Err(err) => {
                error!("Unable to subscribe to the council request channel on nats: {err}");
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    };

    let mut value_create_queue = ValueCreationQueue::default();
    let mut complete_graph = ChangeSetGraph::default();
    loop {
        tokio::time::sleep(Duration::from_millis(10)).await;

        if let Some(reply_channel) = value_create_queue.fetch_next() {
            nats.publish(reply_channel, serde_json::to_vec(&council::Response::OkToCreate).unwrap()).await.unwrap();
        }

        // FIXME: This will block here until a new value is provided, so nothing will evolve without a new message
        let (job_id, reply_channel, request) = tokio::select! {
            req = subscription.next() => match req {
                None => continue,
                Some(result) => match result {
                    Ok(msg) => match serde_json::from_slice::<Request>(msg.data()) {
                        Ok(req) => match (msg.subject().split('.').last().map(Ulid::from_string), msg.reply()) {
                            (Some(Ok(job_id)), Some(reply)) => (job_id, reply.to_owned(), req),
                            (Some(Err(err)), _) => {
                                error!("Unable to convert job id to ulid: {msg:?}");
                                continue;
                            }
                            _ => {
                                error!("No reply channel provided or subject didn't contain job_id: {msg:?}");
                                continue;
                            }
                        }
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
            Request::CreateValues => {
                // Move to a falible wrapper
                job_would_like_to_create_attribute_values(
                    &nats,
                    &mut value_create_queue,
                    reply_channel,
                )
                .await.unwrap();
            }
            Request::ValueCreationDone => {
                job_finished_value_creation(&nats, &mut value_create_queue, reply_channel).await.unwrap();
            }
            Request::ValueDependencyGraph {
                change_set_id,
                dependency_graph,
            } => {
                let _ = register_graph_from_job(
                    &nats,
                    &mut complete_graph,
                    job_id,
                    change_set_id,
                    dependency_graph,
                )
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

#[derive(Default, Debug)]
pub struct ValueCreationQueue {
    processing: Option<String>,
    queue: VecDeque<String>,
}

impl ValueCreationQueue {
    pub fn push(&mut self, reply_channel: String) {
        self.queue.push_back(reply_channel);
    }

    pub fn is_busy(&self) -> bool {
        self.processing.is_some()
    }

    pub fn fetch_next(&mut self) -> Option<String> {
        if self.is_busy() {
            return None;
        }
        let next_channel = self.queue.pop_front();
        self.processing = next_channel.clone();

        next_channel
    }

    pub fn finished_processing(&mut self, reply_channel: String) -> Result<(), Error> {
        if self.processing.as_ref() != Some(&reply_channel) {
            return Err(Error::UnexpectedJobId);
        }

        self.processing = None;

        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct ChangeSetGraph {
    dependency_data: HashMap<Id, HashMap<Id, NodeMetadata>>,
}

#[derive(Default, Debug)]
pub struct NodeMetadata {
    // This should really be an ordered set, to remove duplicates, but we'll deal with
    // that later.
    wanted_by_job_ids: Vec<Id>,
    processing: Option<Id>,
    depends_on_node_ids: HashSet<Id>,
}

impl NodeMetadata {
    pub fn merge_metadata(&mut self, job_id: Id, dependencies: &Vec<Id>) {
        self.wanted_by_job_ids.push(job_id);
        self.depends_on_node_ids.extend(dependencies);
    }

    pub fn remove_dependency(&mut self, node_id: Id) {
        self.depends_on_node_ids.remove(&node_id);
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
                    let mut new_node = NodeMetadata::default();
                    new_node.merge_metadata(job_id, &dependencies);

                    new_node
                });

            for dependency in dependencies {
                change_set_graph_data
                    .entry(dependency)
                    .and_modify(|node| {
                        node.merge_metadata(job_id, &Vec::new());
                    })
                    .or_insert_with(|| {
                        let mut new_node = NodeMetadata::default();
                        new_node.merge_metadata(job_id, &Vec::new());

                        new_node
                    });
            }
        }

        Ok(())
    }

    pub fn mark_node_as_processed(
        &mut self,
        job_id: Id,
        change_set_id: Id,
        node_id: Id,
    ) -> Result<(), Error> {
        let change_set_graph_data = self.dependency_data.get_mut(&change_set_id).unwrap();

        let node_is_complete;
        if let Some(node_metadata) = change_set_graph_data.get_mut(&node_id) {
            if node_metadata.processing != Some(job_id) {
                return Err(Error::ShouldNotBeProcessingByJob);
            }
            node_metadata.processing = None;

            let wanted_by_job_ids = &mut node_metadata.wanted_by_job_ids;
            wanted_by_job_ids.retain(|x| *x != job_id);

            node_is_complete = node_metadata.depends_on_node_ids.is_empty();
        } else {
            return Err(Error::UnknownNodeId);
        }

        if node_is_complete {
            let _node_metadata = change_set_graph_data.remove(&node_id);
            // for every entry entry in the wanted_by_job_ids, tell them that it has already been processed.

            for node_metadata in change_set_graph_data.values_mut() {
                node_metadata.remove_dependency(node_id);
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Nats(#[from] si_data_nats::Error),

    #[error("Unexpected JobId")]
    UnexpectedJobId,
    #[error("Unknown NodeId")]
    UnknownNodeId,
    #[error("Job reported finishing processing, but we expected a different job to be processing")]
    ShouldNotBeProcessingByJob,
}

pub async fn job_would_like_to_create_attribute_values(
    nats: &NatsClient,
    value_create_queue: &mut ValueCreationQueue,
    reply_channel: String,
) -> Result<(), Error> {
    value_create_queue.push(reply_channel);

    Ok(())
}

pub async fn job_finished_value_creation(
    nats: &NatsClient,
    value_create_queue: &mut ValueCreationQueue,
    reply_channel: String,
) -> Result<(), Error> {
    value_create_queue.finished_processing(reply_channel)
}

pub async fn register_graph_from_job(
    nats: &NatsClient,
    complete_graph: &mut ChangeSetGraph,
    job_id: Id,
    change_set_id: Id,
    new_dependency_data: Graph,
) -> Result<(), Error> {
    complete_graph.merge_dependency_graph(job_id, new_dependency_data, change_set_id)
}

pub async fn job_processed_a_value(
    nats: &NatsClient,
    complete_graph: &mut ChangeSetGraph,
    job_id: Id,
    change_set_id: Id,
    node_id: Id,
) -> Result<(), Error> {
    complete_graph.mark_node_as_processed(job_id, change_set_id, node_id)
}

pub async fn job_is_going_away(
    nats: &NatsClient,
    complete_graph: &mut ChangeSetGraph,
    value_create_queue: &mut ValueCreationQueue,
    job_id: Id,
    change_set_id: Id,
) -> Result<(), Error> {
    todo!()
}
