use dal::change_set_pointer::{ChangeSetPointer, ChangeSetPointerError, ChangeSetPointerId};
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
use telemetry::prelude::*;
use thiserror::Error;
use tokio::time::Instant;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
enum ChangeSetLoopError {
    #[error("workspace snapshot error: {0}")]
    ChangeSetPointer(#[from] ChangeSetPointerError),
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
                match Producer::new(environment, reply_to).await {
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
            to_rebase_change_set.vector_clock_id(),
            &mut onto_workspace_snapshot,
            onto_vector_clock_id,
        )
        .await?;
    info!(
        "count: conflicts ({}) and updates ({}), {:?}",
        conflicts.len(),
        updates.len(),
        start.elapsed()
    );

    // If there are conflicts, immediately assemble a reply message that conflicts were found.
    // Otherwise, we can perform updates and assemble a "success" reply message.
    let message: ChangeSetReplyMessage = if conflicts.is_empty() {
        // TODO(nick): store the offset with the change set.
        to_rebase_workspace_snapshot.perform_updates(
            &to_rebase_change_set,
            &mut onto_workspace_snapshot,
            updates.as_slice(),
        )?;

        // Once all updates have been performed, we can write out, mark everything as recently seen
        // and update the pointer.
        to_rebase_workspace_snapshot
            .write(ctx, to_rebase_change_set.vector_clock_id())
            .await?;
        to_rebase_change_set
            .update_pointer(ctx, to_rebase_workspace_snapshot.id())
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
    let mut producer = Producer::new(environment, reply_to_stream).await?;
    producer
        .send_single(serde_json::to_value(message)?, None)
        .await?;

    // Close the producer _after_ logging, but do not make it an infallible close. We do that
    // because the function managing the change set loop is infallible and will log the error.
    info!("sent reply to \"{reply_to_stream}\", {:?}", start.elapsed());
    producer.close().await?;

    Ok(())
}
