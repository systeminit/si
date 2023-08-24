//! This module provides [`Client`], which is used for communicating with a running
//! gobbler [`Server`](gobbler_server::Server).

use gobbler_server::{ManagementMessage, ManagementMessageAction, GOBBLER_MANAGEMENT_STREAM};
use serde::Serialize;
use si_rabbitmq::{Consumer, ConsumerOffsetSpecification, Environment, Producer};
use std::collections::HashMap;
use telemetry::prelude::{debug, error};
use ulid::Ulid;

use crate::{ClientError, ClientResult};

const GOBBLER_REPLY_STREAM_PREFIX: &str = "gobbler-reply";

/// A client for communicating with a running gobbler [`Server`](gobbler_server::Server).
#[allow(missing_debug_implementations)]
pub struct Client {
    management_stream: Stream,
    streams: HashMap<Ulid, Stream>,
}

#[allow(missing_debug_implementations)]
struct Stream {
    producer: Producer,
    reply_stream: String,
    reply_consumer: Consumer,
}

impl Client {
    /// Creates a new [`Client`] to communicate with a running gobbler
    /// [`Server`](gobbler_server::Server).
    pub async fn new() -> ClientResult<Self> {
        let environment = Environment::new().await?;

        // First, create the reply stream. We do not check if it already exists since the reply
        // stream name is ULID-based. It's unlikely that there will be a collision.
        let unique_identifier = Ulid::new().to_string();
        let management_reply_stream = format!("gobbler-management-reply-{unique_identifier}");
        environment.create_stream(&management_reply_stream).await?;
        let management_reply_consumer = Consumer::new(
            &environment,
            &management_reply_stream,
            ConsumerOffsetSpecification::Next,
        )
        .await?;

        // Name the producer using the reply stream, but produce to the primary gobbler stream. This
        // may... will... uh... potentially?... be useful for tracing.
        let management_producer =
            Producer::new(&environment, unique_identifier, GOBBLER_MANAGEMENT_STREAM).await?;

        Ok(Self {
            management_stream: Stream {
                producer: management_producer,
                reply_stream: management_reply_stream,
                reply_consumer: management_reply_consumer,
            },
            streams: HashMap::new(),
        })
    }

    /// Send a message to a gobbler stream for a change set and block for a reply.
    pub async fn send_with_reply<T: Serialize>(
        &mut self,
        message: T,
        change_set_id: Ulid,
    ) -> ClientResult<Option<String>> {
        let stream = self
            .streams
            .get_mut(&change_set_id)
            .ok_or(ClientError::GobblerStreamForChangeSetNotFound)?;
        stream
            .producer
            .send_single(message, Some(stream.reply_stream.clone()))
            .await?;
        if let Some(delivery) = stream.reply_consumer.next().await? {
            if let Some(contents) = delivery.message_contents {
                return Ok(Some(serde_json::from_value(contents)?));
            }
        }
        Ok(None)
    }

    /// Send a message to the management stream to open a gobbler loop and block for a reply.
    pub async fn send_management_open(
        &mut self,
        change_set_id: Ulid,
    ) -> ClientResult<Option<String>> {
        self.management_stream
            .producer
            .send_single(
                ManagementMessage {
                    change_set_id,
                    action: ManagementMessageAction::Open,
                },
                Some(self.management_stream.reply_stream.clone()),
            )
            .await?;
        if let Some(delivery) = self.management_stream.reply_consumer.next().await? {
            if let Some(contents) = delivery.message_contents {
                let change_set_stream: String = serde_json::from_value(contents)?;

                let environment = Environment::new().await?;
                let reply_stream = format!("{GOBBLER_REPLY_STREAM_PREFIX}-{change_set_id}");
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
                return Ok(Some(change_set_stream));
            }
        }
        Ok(None)
    }

    /// Send a message to the management stream to close a gobbler loop and do not wait for a reply.
    pub async fn send_management_close(&mut self, change_set_id: Ulid) -> ClientResult<()> {
        self.management_stream
            .producer
            .send_single(
                ManagementMessage {
                    change_set_id,
                    action: ManagementMessageAction::Close,
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
