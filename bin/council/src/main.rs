use council::{Graph, Id, Request, Response};
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
            nats.publish(
                reply_channel,
                serde_json::to_vec(&Response::OkToCreate).unwrap(),
            )
            .await
            .unwrap();
        }

        // TODO: handle nodes that dont depend on anything

        // FIXME: handle timeouts
        let (reply_channel, request) = tokio::select! {
            req = subscription.next() => match req {
                Some(Ok(msg)) => match (serde_json::from_slice::<Request>(msg.data()), msg.reply()) {
                    (Ok(req), Some(reply)) => (reply.to_owned(), req),
                    (Err(err), _) => {
                        error!("Unable to deserialize request: {err}");
                        continue;
                    }
                    _ => {
                        error!("No reply channel provided: {msg:?}");
                        continue;
                    }
                }
                Some(Err(err)) => {
                    error!("Internal error in nats, bailing out: {err}");
                    break;
                }
                // FIXME: reconnect
                None => break, // Happens if subscription has been unsubscribed or if connection is closed
            },
            _ = shutdown_request_rx.changed() => {
                info!("Worker task received shutdown notification: stopping");
                break;
            }
        };

        match request {
            Request::CreateValues => {
                // Move to a falible wrapper
                job_would_like_to_create_attribute_values(&mut value_create_queue, reply_channel)
                    .await
                    .unwrap();
            }
            Request::ValueCreationDone => {
                job_finished_value_creation(&mut value_create_queue, reply_channel)
                    .await
                    .unwrap();
            }
            Request::ValueDependencyGraph {
                change_set_id,
                dependency_graph,
            } => {
                register_graph_from_job(
                    &mut complete_graph,
                    reply_channel,
                    change_set_id,
                    dependency_graph,
                )
                .await
                .unwrap();
            }
            Request::ProcessedValue {
                change_set_id,
                node_id,
            } => {
                job_processed_a_value(
                    &nats,
                    &mut complete_graph,
                    reply_channel,
                    change_set_id,
                    node_id,
                )
                .await
                .unwrap();
            }
            Request::Bye { change_set_id } => {
                job_is_going_away(
                    &mut complete_graph,
                    &mut value_create_queue,
                    reply_channel,
                    change_set_id,
                )
                .await
                .unwrap();
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
    wanted_by_reply_channels: Vec<String>,
    processing_reply_channel: Option<String>, // reply channel
    depends_on_node_ids: HashSet<Id>,
}

impl NodeMetadata {
    pub fn merge_metadata(&mut self, reply_channel: String, dependencies: &Vec<Id>) {
        self.wanted_by_reply_channels.push(reply_channel);
        self.depends_on_node_ids.extend(dependencies);
    }

    pub fn remove_dependency(&mut self, node_id: Id) {
        self.depends_on_node_ids.remove(&node_id);
    }
}

impl ChangeSetGraph {
    pub fn merge_dependency_graph(
        &mut self,
        reply_channel: String,
        new_dependency_data: Graph,
        change_set_id: Id,
    ) -> Result<(), Error> {
        let change_set_graph_data = self.dependency_data.get_mut(&change_set_id).unwrap();

        for (attribute_value_id, dependencies) in new_dependency_data {
            change_set_graph_data
                .entry(attribute_value_id)
                .and_modify(|node| {
                    node.merge_metadata(reply_channel.clone(), &dependencies);
                })
                .or_insert_with(|| {
                    let mut new_node = NodeMetadata::default();
                    new_node.merge_metadata(reply_channel.clone(), &dependencies);

                    new_node
                });

            for dependency in dependencies {
                change_set_graph_data
                    .entry(dependency)
                    .and_modify(|node| {
                        node.merge_metadata(reply_channel.clone(), &Vec::new());
                    })
                    .or_insert_with(|| {
                        let mut new_node = NodeMetadata::default();
                        new_node.merge_metadata(reply_channel.clone(), &Vec::new());

                        new_node
                    });
            }
        }

        Ok(())
    }

    pub fn mark_node_as_processed(
        &mut self,
        reply_channel: String,
        change_set_id: Id,
        node_id: Id,
    ) -> Result<Vec<String>, Error> {
        let change_set_graph_data = self.dependency_data.get_mut(&change_set_id).unwrap();

        let node_is_complete;
        if let Some(node_metadata) = change_set_graph_data.get_mut(&node_id) {
            if node_metadata.processing_reply_channel.as_ref() != Some(&reply_channel) {
                return Err(Error::ShouldNotBeProcessingByJob);
            }
            node_metadata.processing_reply_channel = None;

            node_metadata
                .wanted_by_reply_channels
                .retain(|x| x != &reply_channel);

            node_is_complete = node_metadata.depends_on_node_ids.is_empty();
        } else {
            return Err(Error::UnknownNodeId);
        }

        if node_is_complete {
            // Timeout could race here
            let node_metadata = change_set_graph_data.remove(&node_id).unwrap();

            for node_metadata in change_set_graph_data.values_mut() {
                node_metadata.remove_dependency(node_id);
            }

            return Ok(node_metadata.wanted_by_reply_channels);
        }

        Ok(Vec::new())
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
    value_create_queue: &mut ValueCreationQueue,
    reply_channel: String,
) -> Result<(), Error> {
    value_create_queue.push(reply_channel);

    Ok(())
}

pub async fn job_finished_value_creation(
    value_create_queue: &mut ValueCreationQueue,
    reply_channel: String,
) -> Result<(), Error> {
    value_create_queue.finished_processing(reply_channel)
}

pub async fn register_graph_from_job(
    complete_graph: &mut ChangeSetGraph,
    reply_channel: String,
    change_set_id: Id,
    new_dependency_data: Graph,
) -> Result<(), Error> {
    complete_graph.merge_dependency_graph(reply_channel, new_dependency_data, change_set_id)
}

pub async fn job_processed_a_value(
    nats: &NatsClient,
    complete_graph: &mut ChangeSetGraph,
    reply_channel: String,
    change_set_id: Id,
    node_id: Id,
) -> Result<(), Error> {
    for reply_channel in
        complete_graph.mark_node_as_processed(reply_channel, change_set_id, node_id)?
    {
        nats.publish(
            reply_channel,
            serde_json::to_vec(&Response::BeenProcessed { node_id }).unwrap(),
        )
        .await
        .unwrap();
    }
    Ok(())
}

pub async fn job_is_going_away(
    _complete_graph: &mut ChangeSetGraph,
    _value_create_queue: &mut ValueCreationQueue,
    _reply_channel: String,
    _change_set_id: Id,
) -> Result<(), Error> {
    todo!()
}
