use crate::{Graph, Id, Request, Response};
use std::time::Duration;

use futures::StreamExt;
use si_data_nats::NatsClient;
use telemetry::prelude::*;
use tokio::{signal, sync::watch};

pub mod config;
mod graph;
pub use config::Config;

use graph::{ChangeSetGraph, ValueCreationQueue};

#[derive(Debug, Clone)]
pub struct Server {
    nats: NatsClient,
    subject_prefix: String,
}

impl Server {
    pub async fn new_with_config(config: config::Config) -> Result<Self> {
        Ok(Self {
            nats: NatsClient::new(config.nats()).await?,
            subject_prefix: config.subject_prefix().unwrap_or("council").to_owned(),
        })
    }

    pub async fn run(
        self,
        subscription_started_tx: watch::Sender<()>,
        mut shutdown_request_rx: watch::Receiver<()>,
    ) -> Result<()> {
        let mut subscription = loop {
            match self
                .nats
                .subscribe(format!("{}.*", self.subject_prefix))
                .await
            {
                Ok(sub) => break sub,
                Err(err) => {
                    error!("Unable to subscribe to the council request channel on nats: {err}");
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                }
            }
        };
        let _ = subscription_started_tx.send(());

        let mut sigterm_watcher = signal::unix::signal(signal::unix::SignalKind::terminate())?;
        let (our_shutdown_request_tx, mut our_shutdown_request_rx) =
            tokio::sync::watch::channel(());
        let mut outer_shutdown_request_rx = shutdown_request_rx.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    info!("SIGINT received; initiating graceful shutdown");
                    // fails if shutdown_request_rx has been dropped, which means shutdown has already happened
                    let _ = our_shutdown_request_tx.send(());
                }
                _ = sigterm_watcher.recv() => {
                    info!("SIGTERM received; initiating graceful shutdown");
                    // fails if shutdown_request_rx has been dropped, which means shutdown has already happened
                    let _ = our_shutdown_request_tx.send(());
                }
                // fails if shutdown_request_tx has been dropped, which means shutdown has already happened
                Ok(()) = outer_shutdown_request_rx.changed() => {}
                else => unreachable!(),
            }
        });

        let mut value_create_queue = ValueCreationQueue::default();
        let mut complete_graph = ChangeSetGraph::default();
        loop {
            if let Some(reply_channel) = value_create_queue.fetch_next() {
                self.nats
                    .publish(
                        reply_channel,
                        serde_json::to_vec(&Response::OkToCreate).unwrap(),
                    )
                    .await
                    .unwrap();
            }

            for (reply_channel, node_id) in complete_graph.fetch_all_available() {
                self.nats
                    .publish(
                        reply_channel,
                        serde_json::to_vec(&Response::OkToProcess {
                            node_ids: vec![node_id],
                        })
                        .unwrap(),
                    )
                    .await
                    .unwrap();
            }

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
                Ok(()) = shutdown_request_rx.changed() => {
                    info!("Worker task received shutdown notification: stopping");
                    break;
                }
                _ = our_shutdown_request_rx.changed() => {
                    info!("Worker task received our shutdown notification: stopping");
                    break;
                }
                else => unreachable!(),
            };

            match request {
                Request::CreateValues => {
                    // Move to a falible wrapper
                    job_would_like_to_create_attribute_values(
                        &mut value_create_queue,
                        reply_channel,
                    )
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
                        &self.nats,
                        &mut complete_graph,
                        reply_channel,
                        change_set_id,
                        node_id,
                    )
                    .await
                    .unwrap();
                }
                Request::Bye { change_set_id } => {
                    dbg!(&value_create_queue);
                    dbg!(&complete_graph);
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

        Ok(())
    }
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

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Nats(#[from] si_data_nats::Error),
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error(transparent)]
    Io(#[from] std::io::Error),

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
    Ok(())
    //todo!()
}
