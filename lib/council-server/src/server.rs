use futures::StreamExt;
use graph::ChangeSetGraph;
use si_data_nats::{NatsClient, Subject, Subscriber};
use std::time::Duration;
use telemetry::prelude::*;
use telemetry_nats::propagation;
use tokio::{signal, sync::watch};

use crate::subject_generator::{ManagementChannel, ManagementReplyChannel};
use crate::{Graph, Id, Request, Response};
use crate::{RequestDiscriminants, SubjectGenerator};

pub use config::Config;

pub mod config;
mod graph;

#[derive(Debug, Clone)]
pub struct Server {
    nats: NatsClient,
}

impl Server {
    pub async fn new_with_config(config: config::Config) -> Result<Self> {
        Ok(Self {
            nats: NatsClient::new(config.nats()).await?,
        })
    }

    pub async fn run(
        self,
        subscriber_started_tx: watch::Sender<()>,
        mut shutdown_request_rx: watch::Receiver<()>,
    ) -> Result<()> {
        let (subscriber_channel, management_channel, management_reply_channel) =
            SubjectGenerator::for_server(
                self.nats.metadata().subject_prefix().map(|p| p.to_string()),
            );
        let mut subscriber = loop {
            match self.nats.subscribe(subscriber_channel.clone()).await {
                Ok(sub) => break sub,
                Err(err) => {
                    error!("Unable to subscribe to the council request channel on nats: {err}");
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                }
            }
        };
        let _ = subscriber_started_tx.send(());

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

        // Before entering the main loop, tell everyone subscribing to the management channel (i.e. all pinga instances)
        // to restart their jobs.
        self.nats
            .publish_with_reply(
                management_channel.clone(),
                management_reply_channel.clone(),
                serde_json::to_vec(&Response::Restart)?.into(),
            )
            .await?;
        info!(
            %management_channel,
            %management_reply_channel,
            "published message for all active pinga instances to restart jobs in progress",
        );

        // Begin the main loop. Everything after this point should be infallible.
        self.core_loop_infallible_wrapper(
            &mut subscriber,
            &mut shutdown_request_rx,
            &mut our_shutdown_request_rx,
            &management_channel,
            &management_reply_channel,
        )
        .await;

        Ok(())
    }

    async fn core_loop_infallible_wrapper(
        &self,
        subscriber: &mut Subscriber,
        shutdown_request_rx: &mut watch::Receiver<()>,
        our_shutdown_request_rx: &mut watch::Receiver<()>,
        management_channel: &ManagementChannel,
        management_reply_channel: &ManagementReplyChannel,
    ) {
        let mut complete_graph = ChangeSetGraph::default();
        loop {
            match self
                .core_loop(
                    subscriber,
                    shutdown_request_rx,
                    our_shutdown_request_rx,
                    management_channel,
                    management_reply_channel,
                    &mut complete_graph,
                )
                .await
            {
                Ok(_) => break,
                Err(err) => {
                    error!(error = ?err, "core loop encountered an error; restarting loop");
                }
            }
        }
    }

    async fn core_loop(
        &self,
        subscriber: &mut Subscriber,
        shutdown_request_rx: &mut watch::Receiver<()>,
        our_shutdown_request_rx: &mut watch::Receiver<()>,
        management_channel: &ManagementChannel,
        management_reply_channel: &ManagementReplyChannel,
        complete_graph: &mut ChangeSetGraph,
    ) -> Result<()> {
        loop {
            for (reply_channel, node_ids) in complete_graph.fetch_all_available() {
                info!(%reply_channel, ?node_ids, "Ok to process AttributeValue");
                self.nats
                    .publish_with_headers(
                        reply_channel,
                        propagation::empty_injected_headers(),
                        serde_json::to_vec(&Response::OkToProcess { node_ids })?.into(),
                    )
                    .await?;
            }

            let sleep = tokio::time::sleep(Duration::from_secs(60));
            tokio::pin!(sleep);
            // FIXME: handle timeouts
            let (reply_channel, request) = tokio::select! {
                _ = &mut sleep => {
                    if !complete_graph.is_empty() {
                        warn!(
                            ?complete_graph,
                            "has values in graph but has been waiting for messages for 60 seconds",
                        );
                    }
                    continue;
                }
                req = subscriber.next() => match req {
                    Some(msg) => {
                        propagation::associate_current_span_from_headers(msg.headers());
                        match (serde_json::from_slice::<Request>(msg.payload()), msg.reply()) {
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
                    }
                    None => {
                        // FIXME(nick): reconnect. Same "FIXME" as the one found in the original listener.
                        warn!("subscriber has been unsubscribed or the connection has been closed");
                        return Ok(());
                    }
                },
                Ok(()) = shutdown_request_rx.changed() => {
                    info!("Worker task received shutdown notification: stopping");
                    return Ok(());
                }
                _ = our_shutdown_request_rx.changed() => {
                    info!("Worker task received our shutdown notification: stopping");
                    return Ok(());
                }
                else => unreachable!(),
            };

            // Cache the reply channel in case we are missing dependency data and need to restart.
            let cached_reply_channel = reply_channel.clone();

            let (result, discrim) = match request {
                Request::ValueDependencyGraph {
                    change_set_id,
                    dependency_graph,
                } => (
                    register_graph_from_job(
                        complete_graph,
                        reply_channel,
                        change_set_id,
                        dependency_graph,
                    )
                    .await,
                    RequestDiscriminants::ValueDependencyGraph,
                ),
                Request::ProcessedValue {
                    change_set_id,
                    node_id,
                } => (
                    job_processed_a_value(
                        &self.nats,
                        complete_graph,
                        reply_channel,
                        change_set_id,
                        node_id,
                    )
                    .await,
                    RequestDiscriminants::ProcessedValue,
                ),
                Request::Bye { change_set_id } => (
                    job_is_going_away(complete_graph, reply_channel, change_set_id).await,
                    RequestDiscriminants::Bye,
                ),
                Request::ValueProcessingFailed {
                    change_set_id,
                    node_id,
                } => (
                    job_failed_processing_a_value(
                        &self.nats,
                        complete_graph,
                        reply_channel,
                        change_set_id,
                        node_id,
                    )
                    .await,
                    RequestDiscriminants::ValueProcessingFailed,
                ),
                Request::Restart => {
                    debug!(
                        %management_channel,
                        %management_reply_channel,
                        "found restart request sent to everyone subscribing to management channel: no-op",
                    );
                    (Ok(()), RequestDiscriminants::Restart)
                }
            };

            match result {
                Ok(()) => match discrim {
                    RequestDiscriminants::Restart => {
                        debug!("no-op successful for restart request")
                    }
                    discrim => debug!(?discrim, "processing request successful"),
                },
                Err(err) => match err {
                    Error::DependencyDataMissing => {
                        self.nats
                            .publish(
                                cached_reply_channel,
                                serde_json::to_vec(&Response::Restart)?.into(),
                            )
                            .await?;
                    }
                    err => return Err(err),
                },
            }
        }
    }
}

// Note: All messages from Pinga include the change set ID.
//
// | Pinga                                                                      | Council                                                                                                                                                                                                                        |
// | -------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
// | Pull job from queue                                                        | N/A                                                                                 |
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

#[remain::sorted]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error("missing dependency data")]
    DependencyDataMissing,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Nats(#[from] si_data_nats::Error),
    #[error("serde json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Job reported finishing processing, but we expected a different job to be processing")]
    ShouldNotBeProcessingByJob,
    #[error("Unexpected JobId")]
    UnexpectedJobId,
    #[error("Unknown NodeId")]
    UnknownNodeId,
}

#[instrument(level = "info")]
pub async fn register_graph_from_job(
    complete_graph: &mut ChangeSetGraph,
    reply_channel: Subject,
    change_set_id: Id,
    new_dependency_data: Graph,
) -> Result<(), Error> {
    debug!(%reply_channel, %change_set_id, ?new_dependency_data, ?complete_graph, "Job registered graph of work");
    complete_graph.merge_dependency_graph(reply_channel, new_dependency_data, change_set_id)
}

#[instrument(level = "info", skip(nats, complete_graph))]
pub async fn job_processed_a_value(
    nats: &NatsClient,
    complete_graph: &mut ChangeSetGraph,
    reply_channel: Subject,
    change_set_id: Id,
    node_id: Id,
) -> Result<(), Error> {
    info!(%reply_channel, %change_set_id, %node_id, "Job finished processing graph node");
    for reply_channel in
        complete_graph.mark_node_as_processed(&reply_channel, change_set_id, node_id)?
    {
        info!(%reply_channel, ?node_id, "AttributeValue has been processed by a job");
        nats.publish_with_headers(
            reply_channel,
            propagation::empty_injected_headers(),
            serde_json::to_vec(&Response::BeenProcessed { node_id })?.into(),
        )
        .await?;
    }
    debug!(?complete_graph);
    Ok(())
}

#[instrument(level = "info", skip(nats, complete_graph))]
pub async fn job_failed_processing_a_value(
    nats: &NatsClient,
    complete_graph: &mut ChangeSetGraph,
    reply_channel: Subject,
    change_set_id: Id,
    node_id: Id,
) -> Result<(), Error> {
    warn!(%reply_channel, %change_set_id, %node_id, ?complete_graph, "Job failed to process node");

    for (reply_channel, failed_node_id) in
        complete_graph.remove_node_and_dependents(reply_channel, change_set_id, node_id)?
    {
        nats.publish_with_headers(
            reply_channel,
            propagation::empty_injected_headers(),
            serde_json::to_vec(&Response::Failed {
                node_id: failed_node_id,
            })?
            .into(),
        )
        .await?;
    }

    Ok(())
}

#[instrument(level = "info")]
pub async fn job_is_going_away(
    complete_graph: &mut ChangeSetGraph,
    reply_channel: Subject,
    change_set_id: Id,
) -> Result<(), Error> {
    debug!(%reply_channel, %change_set_id, ?complete_graph, "Job is going away");
    complete_graph.remove_channel(change_set_id, &reply_channel);
    debug!(?complete_graph);

    Ok(())
}
