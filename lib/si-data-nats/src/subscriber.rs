use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_nats::Subject;
use futures::Stream;
use telemetry::prelude::*;

use super::{ConnectionMetadata, Error, Message, Result};

/// Retrieves messages from given `subscription` created by [Client::subscribe].
///
/// Implements [futures::stream::Stream] for ergonomic async message processing.
///
/// # Examples
/// ```
/// # #[tokio::main]
/// # async fn main() ->  Result<(), async_nats::Error> {
/// let mut nc = async_nats::connect("demo.nats.io").await?;
/// # nc.publish("test".into(), "data".into()).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Subscriber {
    inner: async_nats::Subscriber,
    metadata: Arc<SubscriberMessageMetadata>,
    sub_span: Span,
}

impl Subscriber {
    pub(crate) fn new(
        inner: async_nats::Subscriber,
        subject: &Subject,
        connection_metadata: Arc<ConnectionMetadata>,
        sub_span: Span,
    ) -> Self {
        let metadata = SubscriberMessageMetadata {
            connection_metadata,
            messaging_destination: subject.to_string(),
            messaging_destination_kind: "topic",
            messaging_operation: "process",
            messaging_subject: subject.to_string(),
            process_otel_kind: FormattedSpanKind(SpanKind::Consumer).to_string(),
            process_otel_name: format!("{subject} process"),
        };

        Self {
            inner,
            metadata: Arc::new(metadata),
            sub_span,
        }
    }

    /// Unsubscribes from subscription after reaching given number of messages.
    /// This is the total number of messages received by this subscriber in it's whole
    /// lifespan. If it already reached or surpassed the passed value, it will immediately stop.
    ///
    /// # Examples
    /// ```
    /// # use futures::StreamExt;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = si_data_nats::Client::connect_with_options("demo.nats.io", None, Default::default()).await?;
    ///
    /// let mut subscriber = client.subscribe("test".into()).await?;
    /// subscriber.unsubscribe_after(3).await?;
    /// client.flush().await?;
    ///
    /// for _ in 0..3 {
    ///     client.publish("test".into(), "data".into()).await?;
    /// }
    ///
    /// while let Some(message) = subscriber.next().await {
    ///     println!("message received: {:?}", message);
    /// }
    /// println!("no more messages, unsubscribed");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "subscriber.unsubscribe",
        skip_all,
        level = "debug",
        fields(
            subscriber.unsub_after = %unsub_after,
            messaging.protocol = %self.metadata.messaging_protocol(),
            messaging.system = %self.metadata.messaging_system(),
            messaging.url = %self.metadata.messaging_url(),
            net.transport = %self.metadata.net_transport(),
            otel.kind = %FormattedSpanKind(SpanKind::Client),
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn unsubscribe_after(mut self, unsub_after: u64) -> Result<()> {
        let span = Span::current();
        span.follows_from(&self.sub_span);

        self.inner
            .unsubscribe_after(unsub_after)
            .await
            .map_err(|err| span.record_err(Error::NatsUnsubscribe(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Unsubscribes from subscription, draining all remaining messages.
    ///
    /// # Examples
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = si_data_nats::Client::connect_with_options("demo.nats.io", None, Default::default()).await?;
    ///
    /// let mut subscriber = client.subscribe("foo".into()).await?;
    ///
    /// subscriber.unsubscribe().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "subscriber.unsubscribe",
        skip_all,
        level = "debug",
        fields(
            messaging.protocol = %self.metadata.messaging_protocol(),
            messaging.system = %self.metadata.messaging_system(),
            messaging.url = %self.metadata.messaging_url(),
            net.transport = %self.metadata.net_transport(),
            otel.kind = %FormattedSpanKind(SpanKind::Client),
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn unsubscribe(mut self) -> Result<()> {
        let span = Span::current();
        span.follows_from(&self.sub_span);

        self.inner
            .unsubscribe()
            .await
            .map_err(|err| span.record_err(Error::NatsUnsubscribe(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Gets a reference to the subscriber's span.
    pub fn span(&self) -> &Span {
        &self.sub_span
    }

    /// Gets a reference to the subscriber's metadata.
    pub fn metadata(&self) -> &SubscriberMessageMetadata {
        self.metadata.as_ref()
    }
}

impl Stream for Subscriber {
    type Item = Message;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(msg)) => Poll::Ready(Some(Message::new(
                msg,
                self.metadata.as_connection_metadata(),
            ))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SubscriberMessageMetadata {
    connection_metadata: Arc<ConnectionMetadata>,
    messaging_destination: String,
    messaging_destination_kind: &'static str,
    messaging_operation: &'static str,
    messaging_subject: String,

    process_otel_kind: String,
    process_otel_name: String,
}

impl SubscriberMessageMetadata {
    /// Gets a reference to the subscriber message metadata's messaging destination.
    pub fn messaging_destination(&self) -> &str {
        self.messaging_destination.as_ref()
    }

    /// Gets a reference to the subscriber message metadata's messaging destination kind.
    pub fn messaging_destination_kind(&self) -> &str {
        self.messaging_destination_kind
    }

    /// Gets a reference to the subscriber message metadata's messaging operation.
    pub fn messaging_operation(&self) -> &str {
        self.messaging_operation
    }

    /// Gets a reference to the subscriber message metadata's messaging protocol.
    pub fn messaging_protocol(&self) -> &str {
        self.connection_metadata.messaging_protocol()
    }

    /// Gets a reference to the subscriber message metadata's messaging system.
    pub fn messaging_system(&self) -> &str {
        self.connection_metadata.messaging_system()
    }

    /// Gets a reference to the subscriber message metadata's messaging url.
    pub fn messaging_url(&self) -> &str {
        self.connection_metadata.messaging_url()
    }

    /// Gets a reference to the subscriber message metadata's messaging subject.
    pub fn messaging_subject(&self) -> &str {
        self.messaging_subject.as_ref()
    }

    /// Get a reference to the subscriber message metadata's net transport.
    pub fn net_transport(&self) -> &str {
        self.connection_metadata.net_transport()
    }

    /// Gets a reference to the subscriber message metadata's process otel kind.
    pub fn process_otel_kind(&self) -> &str {
        &self.process_otel_kind
    }

    /// Gets a reference to the subscriber message metadata's process otel name.
    pub fn process_otel_name(&self) -> &str {
        self.process_otel_name.as_ref()
    }

    /// Get a reference to the subscriber message metadata's connection metadata.
    pub fn as_connection_metadata(&self) -> Arc<ConnectionMetadata> {
        self.connection_metadata.clone()
    }
}
