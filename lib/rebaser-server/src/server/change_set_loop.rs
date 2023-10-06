use dal::change_set_pointer::{ChangeSetPointer, ChangeSetPointerError, ChangeSetPointerId};
use dal::workspace_snapshot::WorkspaceSnapshotError;
use dal::{
    DalContext, DalContextBuilder, Tenancy, TransactionsError, Visibility, WorkspacePk,
    WorkspaceSnapshot,
};
use rebaser_core::{ChangeSetMessage, ChangeSetReplyMessage};
use si_rabbitmq::{Consumer, Delivery, Environment, Producer, RabbitError};
use telemetry::prelude::*;
use thiserror::Error;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
enum ChangeSetLoopError {
    #[error("workspace snapshot error: {0}")]
    ChangeSetPointer(#[from] ChangeSetPointerError),
    #[error("missing change set message \"reply_to\" field")]
    MissingChangeSetMessageReplyTo,
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
) {
    if let Err(err) = change_set_loop(ctx_builder, consumer).await {
        error!(error = ?err, "change set loop failed");
    }
}

async fn change_set_loop(
    ctx_builder: DalContextBuilder,
    mut consumer: Consumer,
) -> ChangeSetLoopResult<Option<(String, String)>> {
    let mut ctx = ctx_builder.build_default().await?;
    ctx.update_visibility(Visibility::new_head(false));
    ctx.update_tenancy(Tenancy::new(WorkspacePk::NONE));

    // Create an environment for reply streams.
    let environment = Environment::new().await?;
    while let Some(delivery) = consumer.next().await? {
        // TODO(nick): first detect conflicts and updates, second perform the updates.
        // If conflicts appears, do not perform updates if they exist, and report conflicts back.
        // In other words...
        //   1) succeed everywhere
        //   2) store offset with changeset
        //   3) update requester stream w/out waiting for reply
        process_delivery_infallible_wrapper(&mut ctx, &environment, consumer.stream(), &delivery)
            .await;
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
                match Producer::for_reply(&environment, inbound_stream, reply_to).await {
                    Ok(mut producer) => {
                        if let Err(err) = producer
                            .send_single(ChangeSetReplyMessage::Error(err.to_string()), None)
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
    reply_to: impl AsRef<str>,
) -> ChangeSetLoopResult<()> {
    let raw_message = match &delivery.message_contents {
        Some(found_raw_message) => found_raw_message,
        None => return Err(ChangeSetLoopError::MissingChangeSetMessageReplyTo),
    };
    let message: ChangeSetMessage = serde_json::from_value(raw_message.clone())?;

    // Gather everything we need to detect conflicts and updates from the inbound message.
    let mut to_rebase_workspace_snapshot: WorkspaceSnapshot =
        WorkspaceSnapshot::find(ctx, message.to_rebase_workspace_snapshot_id.into()).await?;
    let onto_change_set = ChangeSetPointer::find(ctx, message.onto_change_set_id.into()).await?;
    let onto_workspace_snapshot_id = onto_change_set.workspace_snapshot_id.ok_or(
        ChangeSetLoopError::MissingWorkspaceSnapshotForChangeSet(onto_change_set.id),
    )?;
    let mut onto_workspace_snapshot =
        WorkspaceSnapshot::find(ctx, onto_workspace_snapshot_id).await?;

    // Perform the conflicts and updates detection.
    let (conflicts, updates) = to_rebase_workspace_snapshot
        .detect_conflicts_and_updates(
            message.to_rebase_vector_clock_id.into(),
            &mut onto_workspace_snapshot,
            onto_change_set.vector_clock_id(),
        )
        .await?;

    // If there are conflicts, immediately assemble a reply message that conflicts were found.
    // Otherwise, we can perform updates and assemble a "success" reply message.
    let message: ChangeSetReplyMessage = if conflicts.is_empty() {
        ChangeSetReplyMessage::ConflictsFound {
            conflicts_found: serde_json::to_value(conflicts)?,
            updates_found_and_skipped: serde_json::to_value(updates)?,
        }
    } else {
        // TODO(nick): actually perform updates.
        ChangeSetReplyMessage::Success(serde_json::to_value(updates)?)
    };

    let mut producer = Producer::for_reply(&environment, inbound_stream, reply_to).await?;
    producer
        .send_single(serde_json::to_value(message)?, None)
        .await?;
    producer.close().await?;

    Ok(())
}
