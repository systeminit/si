//! This module provides [`Client`], which is used for communicating with a running
//! rebaser [`Server`](rebaser_server::Server).

use rebaser_core::{
    ChangeSetMessage, ChangeSetReplyMessage, ManagementMessage, ManagementMessageAction,
    REBASER_MANAGEMENT_STREAM,
};
use si_rabbitmq::{Consumer, ConsumerOffsetSpecification, Environment, Producer};
use std::collections::HashMap;
use std::time::Duration;

use telemetry::prelude::*;
use ulid::Ulid;

use crate::{ClientError, ClientResult};

const REBASER_REPLY_STREAM_PREFIX: &str = "rebaser-reply";
const REPLY_TIMEOUT_SECONDS: u64 = 10;

/// A client for communicating with a running rebaser [`Server`](rebaser_server::Server).
#[allow(missing_debug_implementations)]
pub struct Client {
    management_stream: Stream,
    streams: HashMap<Ulid, Stream>,
    reply_timeout: Duration,
}

#[allow(missing_debug_implementations)]
struct Stream {
    producer: Producer,
    reply_stream: String,
    reply_consumer: Consumer,
}

impl Client {
    /// Creates a new [`Client`] to communicate with a running rebaser
    /// [`Server`](rebaser_server::Server).
    pub async fn new() -> ClientResult<Self> {
        let environment = Environment::new().await?;

        // First, create the reply stream. We do not check if it already exists since the reply
        // stream name is ULID-based. It's unlikely that there will be a collision.
        let unique_identifier = Ulid::new().to_string();
        let management_reply_stream = format!("rebaser-management-reply-{unique_identifier}");
        environment.create_stream(&management_reply_stream).await?;
        let management_reply_consumer = Consumer::new(
            &environment,
            &management_reply_stream,
            ConsumerOffsetSpecification::Next,
        )
        .await?;

        // Name the producer using the reply stream, but produce to the primary rebaser stream. This
        // may... will... uh... potentially?... be useful for tracing.
        let management_producer =
            Producer::new(&environment, unique_identifier, REBASER_MANAGEMENT_STREAM).await?;

        Ok(Self {
            management_stream: Stream {
                producer: management_producer,
                reply_stream: management_reply_stream,
                reply_consumer: management_reply_consumer,
            },
            streams: HashMap::new(),
            reply_timeout: Duration::from_secs(REPLY_TIMEOUT_SECONDS),
        })
    }

    /// Send a message to a rebaser stream for a change set and block for a reply.
    pub async fn send_with_reply(
        &mut self,
        change_set_to_update: Ulid,
        workspace_snapshot_to_rebase_on_top_of_current_snapshot_being_pointed_at: Ulid,
        change_set_that_dictates_changes: Ulid,
    ) -> ClientResult<ChangeSetReplyMessage> {
        let stream = self
            .streams
            .get_mut(&change_set_to_update)
            .ok_or(ClientError::RebaserStreamForChangeSetNotFound)?;
        stream
            .producer
            .send_single(
                ChangeSetMessage {
                    change_set_to_update,
                    workspace_snapshot_to_rebase_on_top_of_current_snapshot_being_pointed_at,
                    change_set_that_dictates_changes,
                },
                Some(stream.reply_stream.clone()),
            )
            .await?;
        let maybe_delivery =
            match tokio::time::timeout(self.reply_timeout, stream.reply_consumer.next()).await {
                Ok(result) => result?,
                Err(e) => return Err(ClientError::ReplyTimeout(e)),
            };

        let delivery = maybe_delivery.ok_or(ClientError::EmptyDelivery(
            stream.reply_consumer.stream().to_string(),
        ))?;
        let contents = delivery
            .clone()
            .message_contents
            .ok_or(ClientError::EmptyMessageContentsForDelivery(delivery))?;

        Ok(serde_json::from_value(contents)?)
    }

    /// Send a message to the management stream to open a rebaser loop and block for a reply.
    pub async fn send_management_open_change_set(
        &mut self,
        change_set_id: Ulid,
    ) -> ClientResult<String> {
        self.management_stream
            .producer
            .send_single(
                ManagementMessage {
                    change_set_id,
                    action: ManagementMessageAction::OpenChangeSet,
                },
                Some(self.management_stream.reply_stream.clone()),
            )
            .await?;

        let maybe_delivery = match tokio::time::timeout(
            self.reply_timeout,
            self.management_stream.reply_consumer.next(),
        )
        .await
        {
            Ok(result) => result?,
            Err(e) => return Err(ClientError::ReplyTimeout(e)),
        };

        let delivery = maybe_delivery.ok_or(ClientError::EmptyDelivery(
            self.management_stream.reply_consumer.stream().to_string(),
        ))?;
        let contents = delivery
            .clone()
            .message_contents
            .ok_or(ClientError::EmptyMessageContentsForDelivery(delivery))?;

        let change_set_stream: String = serde_json::from_value(contents)?;

        let environment = Environment::new().await?;
        let reply_stream = format!("{REBASER_REPLY_STREAM_PREFIX}-{change_set_id}");
        environment.create_stream(&reply_stream).await?;

        // FIXME(nick): name the producer properly.
        let producer = Producer::new(&environment, "producer", &change_set_stream).await?;
        let reply_consumer = Consumer::new(
            &environment,
            &reply_stream,
            ConsumerOffsetSpecification::First,
        )
        .await?;

        self.streams.insert(
            change_set_id,
            Stream {
                producer,
                reply_stream,
                reply_consumer,
            },
        );
        Ok(change_set_stream)
    }

    /// Send a message to the management stream to close a rebaser loop and do not wait for a reply.
    pub async fn send_management_close_change_set(
        &mut self,
        change_set_id: Ulid,
    ) -> ClientResult<()> {
        self.management_stream
            .producer
            .send_single(
                ManagementMessage {
                    change_set_id,
                    action: ManagementMessageAction::CloseChangeSet,
                },
                Some(self.management_stream.reply_stream.clone()),
            )
            .await?;

        match self.streams.remove(&change_set_id) {
            Some(stream) => {
                if let Err(e) = stream.producer.close().await {
                    error!("{e}");
                }
                let handle = stream.reply_consumer.handle();
                if let Err(e) = handle.close().await {
                    error!("{e}");
                }
                let environment = Environment::new().await?;
                environment.delete_stream(stream.reply_stream).await?;
            }
            None => {
                debug!("producer and reply consumer not found for change set id: {change_set_id}")
            }
        }
        Ok(())
    }

    /// This method performs an infallible close of all producers and consumers created by the
    /// client.
    pub async fn close(mut self) {
        // First, close all producers and consumers for the streams.
        for (_, stream) in self.streams.drain() {
            if let Err(e) = stream.producer.close().await {
                error!("{e}");
            }
            let handle = stream.reply_consumer.handle();
            if let Err(e) = handle.close().await {
                error!("{e}");
            }
        }

        // Then, close the management producer and consumer.
        if let Err(e) = self.management_stream.producer.close().await {
            error!("{e}");
        }
        let handle = self.management_stream.reply_consumer.handle();
        if let Err(e) = handle.close().await {
            error!("{e}");
        }

        // Finally, delete the reply stream.
        match Environment::new().await {
            Ok(environment) => {
                if let Err(e) = environment
                    .delete_stream(self.management_stream.reply_stream)
                    .await
                {
                    error!("{e}");
                }
            }
            Err(e) => error!("{e}"),
        }
    }
}
