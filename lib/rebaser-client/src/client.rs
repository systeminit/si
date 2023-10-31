//! This module provides [`Client`], which is used for communicating with a running
//! rebaser [`Server`](rebaser_server::Server).

use rebaser_core::{
    ChangeSetMessage, ChangeSetReplyMessage, ManagementMessage, ManagementMessageAction,
    StreamNameGenerator,
};
use si_rabbitmq::{Config, Consumer, ConsumerOffsetSpecification, Environment, Producer};
use std::collections::HashMap;
use std::time::Duration;
use telemetry::prelude::*;
use ulid::Ulid;

use crate::{ClientError, ClientResult};

const REPLY_TIMEOUT_SECONDS: u64 = 10;

/// A client for communicating with a running rebaser [`Server`](rebaser_server::Server).
#[allow(missing_debug_implementations)]
pub struct Client {
    id: Ulid,
    management_stream: Stream,
    streams: HashMap<Ulid, Stream>,
    reply_timeout: Duration,
    config: Config,
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
    pub async fn new(config: Config) -> ClientResult<Self> {
        let environment = Environment::new(&config).await?;

        let id = Ulid::new();
        let management_stream = StreamNameGenerator::management();
        let management_reply_stream = StreamNameGenerator::management_reply(id);

        environment.create_stream(&management_reply_stream).await?;
        let management_reply_consumer = Consumer::new(
            &environment,
            &management_reply_stream,
            ConsumerOffsetSpecification::Next,
        )
        .await?;

        // Name the producer using the reply stream, but produce to the primary rebaser stream. This
        // may... will... uh... potentially?... be useful for tracing.
        let management_producer = Producer::new(&environment, management_stream).await?;

        Ok(Self {
            id,
            management_stream: Stream {
                producer: management_producer,
                reply_stream: management_reply_stream,
                reply_consumer: management_reply_consumer,
            },
            streams: HashMap::new(),
            reply_timeout: Duration::from_secs(REPLY_TIMEOUT_SECONDS),
            config,
        })
    }

    /// Send a message to a rebaser stream for a change set and block for a reply.
    pub async fn request_rebase(
        &mut self,
        to_rebase_change_set_id: Ulid,
        onto_workspace_snapshot_id: Ulid,
        onto_vector_clock_id: Ulid,
    ) -> ClientResult<ChangeSetReplyMessage> {
        let stream = self
            .streams
            .get_mut(&to_rebase_change_set_id)
            .ok_or(ClientError::RebaserStreamForChangeSetNotFound)?;
        stream
            .producer
            .send_single(
                ChangeSetMessage {
                    to_rebase_change_set_id,
                    onto_workspace_snapshot_id,
                    onto_vector_clock_id,
                },
                Some(stream.reply_stream.clone()),
            )
            .await?;
        let maybe_delivery = match tokio::time::timeout(
            self.reply_timeout,
            stream.reply_consumer.next(),
        )
        .await
        {
            Ok(result) => result?,
            Err(_elapsed) => {
                debug!(
                    "hit timeout for consuming on the reply stream (\"{}\") from the rebaser server",
                    stream.reply_consumer.stream()
                );
                return Err(ClientError::ReplyTimeout);
            }
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
    pub async fn open_stream_for_change_set(
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

        // FIXME(nick): we should probably not await a reply and assume that it is working OR we
        // should await a reply, but only to see if it was successful. This is because we should
        // know the name already and not have to get it from the route.
        let maybe_delivery = match tokio::time::timeout(
            self.reply_timeout,
            self.management_stream.reply_consumer.next(),
        )
        .await
        {
            Ok(result) => result?,
            Err(_elapsed) => return Err(ClientError::ReplyTimeout),
        };

        let delivery = maybe_delivery.ok_or(ClientError::EmptyDelivery(
            self.management_stream.reply_consumer.stream().to_string(),
        ))?;
        let contents = delivery
            .clone()
            .message_contents
            .ok_or(ClientError::EmptyMessageContentsForDelivery(delivery))?;

        let change_set_stream: String = serde_json::from_value(contents)?;

        // TODO(nick): move stream generation to a common crate.
        let environment = Environment::new(&self.config).await?;
        let reply_stream = StreamNameGenerator::change_set_reply(change_set_id, self.id);
        environment.create_stream(&reply_stream).await?;

        // FIXME(nick): name the producer properly.
        let producer = Producer::new(&environment, &change_set_stream).await?;
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
    pub async fn close_stream_for_change_set(&mut self, change_set_id: Ulid) -> ClientResult<()> {
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
                let environment = Environment::new(&self.config).await?;
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
        match Environment::new(&self.config).await {
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
