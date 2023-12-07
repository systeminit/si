use dal::{DalContext, JobQueueProcessor, ServicesContext};
use futures::{FutureExt, StreamExt};
use rebaser_core::{ManagementMessage, ManagementMessageAction, StreamNameGenerator};
use si_crypto::SymmetricCryptoService;
use si_data_nats::NatsClient;
use si_data_pg::PgPool;
use si_rabbitmq::{
    Config as SiRabbitMqConfig, Consumer, ConsumerHandle, ConsumerOffsetSpecification, Environment,
    Producer,
};
use si_rabbitmq::{Delivery, RabbitError};
use std::collections::HashMap;
use std::sync::Arc;
use stream_cancel::StreamExt as StreamCancelStreamExt;
use telemetry::prelude::*;
use tokio::sync::watch;
use ulid::Ulid;

use crate::server::{change_set_loop, ServerError, ServerResult};

#[allow(clippy::too_many_arguments)]
pub(crate) async fn management_loop_infallible_wrapper(
    recreate_management_stream: bool,
    pg_pool: PgPool,
    nats: NatsClient,
    veritech: veritech_client::Client,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    symmetric_crypto_service: SymmetricCryptoService,
    encryption_key: Arc<veritech_client::EncryptionKey>,
    shutdown_watch_rx: watch::Receiver<()>,
    rabbitmq_config: SiRabbitMqConfig,
) {
    if let Err(err) = management_loop(
        recreate_management_stream,
        pg_pool,
        nats,
        veritech,
        job_processor,
        symmetric_crypto_service,
        encryption_key,
        rabbitmq_config,
        shutdown_watch_rx,
    )
    .await
    {
        error!(error = ?err, "consuming stream failed");
    }
}

#[allow(clippy::too_many_arguments)]
async fn management_loop(
    recreate_management_stream: bool,
    pg_pool: PgPool,
    nats: NatsClient,
    veritech: veritech_client::Client,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    symmetric_crypto_service: SymmetricCryptoService,
    encryption_key: Arc<veritech_client::EncryptionKey>,
    rabbitmq_config: SiRabbitMqConfig,
    mut shutdown_watch_rx: watch::Receiver<()>,
) -> ServerResult<()> {
    let services_context = ServicesContext::new(
        pg_pool,
        nats.clone(),
        job_processor,
        veritech.clone(),
        encryption_key,
        None,
        None,
        symmetric_crypto_service,
        rabbitmq_config.clone(),
    );
    // let ctx_builder = DalContext::builder(services_context, false);

    // Meta: we can only have one rebaser instance right now due to https://github.com/rabbitmq/rabbitmq-stream-rust-client/issues/130
    //
    // 1) subscribe to "next" for changeset close/create events --> stream for ChangeSetClose or ChangeSetOpen
    //    --> "rebaser-management"
    // 2) query db for all named, open changesets
    // 3) start a subscription for each result for step 2
    //    --> "rebaser-<change-set-id>"
    //    1:N --> "rebaser-<change-set-id>-reply-<requester>-<ulid>"
    //      (e.g. "rebaser-<change-set-id>-reply-sdf-<ulid>")
    //             note: requester deletes stream upon reply
    //
    // NOTE: QUERY DB FOR OFFSET NUMBER OR GO TO FIRST SPECIFICATION

    // Prepare the environment and management stream.
    let management_stream = StreamNameGenerator::management(rabbitmq_config.stream_prefix());
    let environment = Environment::new(&rabbitmq_config).await?;
    if recreate_management_stream {
        environment.delete_stream(&management_stream).await?;
    }
    environment.create_stream(&management_stream).await?;

    let management_consumer = Consumer::new(
        &environment,
        &management_stream,
        ConsumerOffsetSpecification::Next,
    )
    .await?;
    let management_handle = management_consumer.handle();
    let mut rebaser_handles: HashMap<Ulid, (String, ConsumerHandle)> = HashMap::new();

    let mut inbound_management_stream = management_consumer
        .into_stream()
        .await?
        .take_until_if(Box::pin(shutdown_watch_rx.changed().map(|_| true)));

    while let Some(unprocessed_management_delivery) = inbound_management_stream.next().await {
        let management_delivery = Delivery::try_from(
            unprocessed_management_delivery.map_err(RabbitError::ConsumerDelivery)?,
        )?;

        let contents = management_delivery
            .message_contents
            .ok_or(ServerError::MissingManagementMessageContents)?;
        let reply_to = management_delivery
            .reply_to
            .ok_or(ServerError::MissingManagementMessageReplyTo)?;
        let mm: ManagementMessage = serde_json::from_value(contents)?;

        match mm.action {
            ManagementMessageAction::CloseChangeSet => {
                match rebaser_handles.remove(&mm.change_set_id) {
                    Some((stream, handle)) => {
                        if let Err(err) = handle.close().await {
                            warn!(error = ?err, "closing change set consumer failed");
                        }
                        if let Err(err) = environment.delete_stream(stream).await {
                            warn!(error = ?err, "deleting change set stream failed");
                        }
                    }
                    None => debug!(
                        "did not find handle for change set id ({}) (it have already been closed)",
                        mm.change_set_id
                    ),
                }
            }
            ManagementMessageAction::OpenChangeSet => {
                info!(
                    "finding or creating stream for change set: {}",
                    mm.change_set_id
                );
                let new_stream = StreamNameGenerator::change_set(
                    mm.change_set_id,
                    rabbitmq_config.stream_prefix(),
                );
                let stream_already_exists = environment.create_stream(&new_stream).await?;

                // Only create the new stream and loop if the stream does not already exist.
                if !stream_already_exists {
                    let consumer =
                        Consumer::new(&environment, &new_stream, ConsumerOffsetSpecification::Next)
                            .await?;
                    let handle = consumer.handle();
                    rebaser_handles.insert(mm.change_set_id, (new_stream.clone(), handle));

                    let ctx_builder = DalContext::builder(services_context.clone(), false);
                    let rabbitmq_config = rabbitmq_config.clone();
                    tokio::spawn(change_set_loop::change_set_loop_infallible_wrapper(
                        ctx_builder,
                        consumer,
                        rabbitmq_config,
                    ));
                }

                // Return the requested stream and then close the producer.
                let mut producer = Producer::new(&environment, reply_to).await?;
                producer.send_single(new_stream, None).await?;
                producer.close().await?;
            }
        }
    }

    // Once the loop is done, perform cleanup.
    for (_, (_change_set_stream, change_set_consumer_handle)) in rebaser_handles.drain() {
        if let Err(err) = change_set_consumer_handle.close().await {
            warn!(error = ?err, "closing change set consumer failed during cleanup");
        }
    }
    if let Err(err) = management_handle.close().await {
        warn!(error = ?err, "closing management consumer failed during cleanup");
    }
    Ok(())
}
