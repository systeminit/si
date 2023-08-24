use rabbitmq_stream_client::types::Message;
use rabbitmq_stream_client::{Dedup, Producer as UpstreamProducer};
use serde::Serialize;

use crate::environment::Environment;
use crate::{RabbitError, RabbitResult};

/// An interface for producing and sending RabbitMQ stream messages.
#[allow(missing_debug_implementations)]
pub struct Producer {
    inner: UpstreamProducer<Dedup>,
    closed: bool,
}

impl Producer {
    /// Creates a new [`Producer`] for producing and sending RabbitMQ stream messages.
    pub async fn new(
        environment: &Environment,
        name: impl AsRef<str>,
        stream: impl AsRef<str>,
    ) -> RabbitResult<Self> {
        let inner = environment
            .inner()
            .producer()
            .name(name.as_ref())
            .build(stream.as_ref())
            .await?;
        Ok(Self {
            inner,
            closed: false,
        })
    }

    /// Creates a new [`Producer`] for replying to the sender from an inbound stream.
    pub async fn for_reply(
        environment: &Environment,
        inbound_stream: impl AsRef<str>,
        reply_to_stream: impl AsRef<str>,
    ) -> RabbitResult<Self> {
        let inbound_stream = inbound_stream.as_ref();
        let reply_to_stream = reply_to_stream.as_ref();
        Self::new(
            &environment,
            format!("{inbound_stream}-reply-{reply_to_stream}"),
            reply_to_stream,
        )
        .await
    }

    /// Sends a single message to a stream.
    pub async fn send_single<T: Serialize>(
        &mut self,
        input: T,
        reply_to: Option<String>,
    ) -> RabbitResult<()> {
        if self.closed {
            return Err(RabbitError::ProducerClosed);
        }
        let value = serde_json::to_value(input)?;
        let mut message_builder = Message::builder().body(serde_json::to_vec(&value)?);
        if let Some(reply_to) = reply_to {
            message_builder = message_builder
                .properties()
                .reply_to(reply_to)
                .message_builder();
        }
        let message = message_builder.build();

        self.inner.send_with_confirm(message).await?;
        Ok(())
    }

    /// Closes the producer connection and renders the producer unusable.
    pub async fn close(mut self) -> RabbitResult<()> {
        self.inner.close().await?;
        self.closed = true;
        Ok(())
    }
}
