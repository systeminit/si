use dal::change_set_pointer::{ChangeSetPointer, ChangeSetPointerError, ChangeSetPointerId};
use dal::workspace_snapshot::graph::NodeIndex;
use dal::workspace_snapshot::update::Update;
use dal::workspace_snapshot::vector_clock::VectorClockId;
use dal::workspace_snapshot::WorkspaceSnapshotError;
use dal::{
    DalContext, DalContextBuilder, Tenancy, TransactionsError, Visibility, WorkspacePk,
    WorkspaceSnapshot,
};
use rebaser_core::{ChangeSetMessage, ChangeSetReplyMessage};
use si_rabbitmq::{
    Config as SiRabbitMqConfig, Consumer, Delivery, Environment, Producer, RabbitError,
};
use std::collections::HashMap;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::time::Instant;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
enum ChangeSetLoopError {
    #[error("workspace snapshot error: {0}")]
    ChangeSetPointer(#[from] ChangeSetPointerError),
    #[error("when performing updates, could not find the newly imported subgraph (may god have mercy on your soul)")]
    DestinationNotUpdatedWhenImportingSubgraph,
    #[error("missing change set message \"reply_to\" field")]
    MissingChangeSetMessageReplyTo,
    #[error("missing change set pointer")]
    MissingChangeSetPointer(ChangeSetPointerId),
    #[error("missing workspace snapshot for change set ({0}) (the change set likely isn't pointing at a workspace snapshot)")]
    MissingWorkspaceSnapshotForChangeSet(ChangeSetPointerId),
    #[error("rabbit error: {0}")]
    Rabbit(#[from] RabbitError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

type ChangeSetLoopResult<T> = Result<T, ChangeSetLoopError>;

pub(crate) async fn change_set_loop_infallible_wrapper(
    ctx_builder: DalContextBuilder,
    consumer: Consumer,
    rabbitmq_config: SiRabbitMqConfig,
) {
    if let Err(err) = change_set_loop(ctx_builder, consumer, &rabbitmq_config).await {
        error!(error = ?err, "change set loop failed");
    }
}

async fn change_set_loop(
    ctx_builder: DalContextBuilder,
    mut consumer: Consumer,
    rabbitmq_config: &SiRabbitMqConfig,
) -> ChangeSetLoopResult<Option<(String, String)>> {
    // Create an environment for reply streams.
    let environment = Environment::new(rabbitmq_config).await?;
    while let Some(delivery) = consumer.next().await? {
        let mut ctx = ctx_builder.build_default().await?;
        ctx.update_visibility(Visibility::new_head(false));
        ctx.update_tenancy(Tenancy::new(WorkspacePk::NONE));

        let start = Instant::now();
        process_delivery_infallible_wrapper(&mut ctx, &environment, consumer.stream(), &delivery)
            .await;
        info!("process delivery total: {:?}", start.elapsed());
    }
    Ok(None)
}

// NOTE(nick): reply to whoever sent the message if a failure happens.
async fn process_delivery_infallible_wrapper(
    ctx: &mut DalContext,
    environment: &Environment,
    inbound_stream: impl AsRef<str>,
    delivery: &Delivery,
) {
    let inbound_stream = inbound_stream.as_ref();
    match &delivery.reply_to {
        Some(reply_to) => {
            if let Err(err) =
                process_delivery(ctx, environment, inbound_stream, delivery, reply_to).await
            {
                error!(error = ?err, "processing delivery failed, attempting to reply");
                match Producer::new(&environment, reply_to).await {
                    Ok(mut producer) => {
                        if let Err(err) = producer
                            .send_single(
                                ChangeSetReplyMessage::Error {
                                    message: err.to_string(),
                                },
                                None,
                            )
                            .await
                        {
                            error!(error = ?err, "sending reply failed");
                        }
                        if let Err(err) = producer.close().await {
                            error!(error = ?err, "closing reply producer failed");
                        }
                    }
                    Err(err) => error!(error = ?err, "creating reply producer failed"),
                }
            }
        }
        None => error!(
            "cannot reply: empty reply field found for delivery: {:?}",
            delivery
        ),
    }
}

async fn process_delivery(
    ctx: &mut DalContext,
    environment: &Environment,
    inbound_stream: impl AsRef<str>,
    delivery: &Delivery,
    reply_to_stream: impl AsRef<str>,
) -> ChangeSetLoopResult<()> {
    let start = Instant::now();
    let raw_message = match &delivery.message_contents {
        Some(found_raw_message) => found_raw_message,
        None => return Err(ChangeSetLoopError::MissingChangeSetMessageReplyTo),
    };
    let message: ChangeSetMessage = serde_json::from_value(raw_message.clone())?;

    // Gather everything we need to detect conflicts and updates from the inbound message.
    let mut to_rebase_change_set =
        ChangeSetPointer::find(ctx, message.to_rebase_change_set_id.into())
            .await?
            .ok_or(ChangeSetLoopError::MissingChangeSetPointer(
                message.to_rebase_change_set_id.into(),
            ))?;
    let to_rebase_workspace_snapshot_id = to_rebase_change_set.workspace_snapshot_id.ok_or(
        ChangeSetLoopError::MissingWorkspaceSnapshotForChangeSet(to_rebase_change_set.id),
    )?;
    info!("before snapshot fetch and parse: {:?}", start.elapsed());
    let mut to_rebase_workspace_snapshot =
        WorkspaceSnapshot::find(ctx, to_rebase_workspace_snapshot_id).await?;
    let mut onto_workspace_snapshot: WorkspaceSnapshot =
        WorkspaceSnapshot::find(ctx, message.onto_workspace_snapshot_id.into()).await?;
    info!(
        "to_rebase_id: {}, onto_id: {}",
        to_rebase_workspace_snapshot_id,
        onto_workspace_snapshot.id()
    );
    info!("after snapshot fetch and parse: {:?}", start.elapsed());

    // Perform the conflicts and updates detection.
    let onto_vector_clock_id: VectorClockId = message.onto_vector_clock_id.into();
    let (conflicts, updates) = to_rebase_workspace_snapshot
        .detect_conflicts_and_updates(
            dbg!(to_rebase_change_set.vector_clock_id()),
            &mut onto_workspace_snapshot,
            dbg!(onto_vector_clock_id),
        )
        .await?;
    info!(
        "conflicts and updates detected: {conflicts:?} {updates:?}, {:?}",
        start.elapsed()
    );

    // If there are conflicts, immediately assemble a reply message that conflicts were found.
    // Otherwise, we can perform updates and assemble a "success" reply message.
    let message: ChangeSetReplyMessage = if conflicts.is_empty() {
        // TODO(nick): store the offset with the change set.
        perform_updates_and_write_out_and_update_pointer(
            ctx,
            &mut to_rebase_workspace_snapshot,
            &mut to_rebase_change_set,
            &mut onto_workspace_snapshot,
            &updates,
        )
        .await?;
        ChangeSetReplyMessage::Success {
            updates_performed: serde_json::to_value(updates)?,
        }
    } else {
        ChangeSetReplyMessage::ConflictsFound {
            conflicts_found: serde_json::to_value(conflicts)?,
            updates_found_and_skipped: serde_json::to_value(updates)?,
        }
    };

    info!("updates performed: {:?}", start.elapsed());

    // Before replying to the requester, we must commit.
    ctx.commit_no_rebase().await?;

    // Send reply to the "reply to stream" for the specific client.
    let inbound_stream = inbound_stream.as_ref();
    let reply_to_stream = reply_to_stream.as_ref();
    info!(
        "processed delivery from \"{inbound_stream}\", committed transaction and sending reply to \"{reply_to_stream}\"",
    );
    let mut producer = Producer::new(&environment, reply_to_stream).await?;
    producer
        .send_single(serde_json::to_value(message)?, None)
        .await?;

    // Close the producer _after_ logging, but do not make it an infallible close. We do that
    // because the function managing the change set loop is infallible and will log the error.
    info!("sent reply to \"{reply_to_stream}\", {:?}", start.elapsed());
    producer.close().await?;

    Ok(())
}

async fn perform_updates_and_write_out_and_update_pointer(
    ctx: &DalContext,
    to_rebase_workspace_snapshot: &mut WorkspaceSnapshot,
    to_rebase_change_set: &mut ChangeSetPointer,
    onto_workspace_snapshot: &mut WorkspaceSnapshot,
    updates: &Vec<Update>,
) -> ChangeSetLoopResult<()> {
    let mut updated = HashMap::new();
    for update in updates {
        match update {
            Update::NewEdge {
                source,
                destination,
                edge_weight,
            } => {
                let updated_source = *updated.get(source).unwrap_or(source);
                let destination = find_in_to_rebase_or_create_using_onto(
                    *destination,
                    &mut updated,
                    onto_workspace_snapshot,
                    to_rebase_workspace_snapshot,
                )?;
                let new_edge_index = to_rebase_workspace_snapshot.add_edge(
                    updated_source,
                    edge_weight.clone(),
                    destination,
                )?;
                let (new_source, _) =
                    to_rebase_workspace_snapshot.edge_endpoints(new_edge_index)?;
                updated.insert(*source, new_source);
            }
            Update::RemoveEdge {
                source,
                destination,
                edge_kind,
            } => {
                let updated_source = *updated.get(source).unwrap_or(source);
                let destination = *updated.get(destination).unwrap_or(destination);
                updated.extend(to_rebase_workspace_snapshot.remove_edge(
                    to_rebase_change_set,
                    updated_source,
                    destination,
                    *edge_kind,
                )?);
            }
            Update::ReplaceSubgraph { onto, to_rebase } => {
                let updated_to_rebase = *updated.get(to_rebase).unwrap_or(to_rebase);
                let new_subgraph_root = find_in_to_rebase_or_create_using_onto(
                    *onto,
                    &mut updated,
                    onto_workspace_snapshot,
                    to_rebase_workspace_snapshot,
                )?;
                updated.extend(
                    to_rebase_workspace_snapshot
                        .replace_references(updated_to_rebase, new_subgraph_root)?,
                );
            }
        }
    }

    // Once all updates have been performed, we can write out, mark everything as recently seen
    // and update the pointer.

    //dbg!("onto_workspace_snapshot");
    //onto_workspace_snapshot.dot();
    //dbg!("to_rebase_workspace_snapshot");
    //to_rebase_workspace_snapshot.dot();

    to_rebase_workspace_snapshot
        .write(ctx, to_rebase_change_set.vector_clock_id())
        .await?;
    to_rebase_change_set
        .update_pointer(ctx, to_rebase_workspace_snapshot.id())
        .await?;
    //   dbg!(to_rebase_workspace_snapshot.id());

    Ok(())
}

fn find_in_to_rebase_or_create_using_onto(
    unchecked: NodeIndex,
    updated: &mut HashMap<NodeIndex, NodeIndex>,
    onto_workspace_snapshot: &mut WorkspaceSnapshot,
    to_rebase_workspace_snapshot: &mut WorkspaceSnapshot,
) -> ChangeSetLoopResult<NodeIndex> {
    let found_or_created = match updated.get(&unchecked) {
        Some(found) => *found,
        None => {
            let unchecked_node_weight = onto_workspace_snapshot.get_node_weight(unchecked)?;
            match to_rebase_workspace_snapshot.find_equivalent_node(
                unchecked_node_weight.id(),
                unchecked_node_weight.lineage_id(),
            )? {
                Some(found_equivalent_node) => {
                    updated.extend(
                        to_rebase_workspace_snapshot
                            .import_subgraph(onto_workspace_snapshot, unchecked)?,
                    );
                    updated.insert(found_equivalent_node, unchecked);

                    unchecked
                }
                None => {
                    updated.extend(
                        to_rebase_workspace_snapshot
                            .import_subgraph(onto_workspace_snapshot, unchecked)?,
                    );
                    *updated
                        .get(&unchecked)
                        .ok_or(ChangeSetLoopError::DestinationNotUpdatedWhenImportingSubgraph)?
                }
            }
        }
    };
    Ok(found_or_created)
}
