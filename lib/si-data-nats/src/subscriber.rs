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
///
/// ```no_run
/// # #[tokio::main]
/// # async fn main() ->  Result<(), async_nats::Error> {
/// let mut nc = si_data_nats::ConnectOptions::new().connect("demo.nats.io", None).await?;
/// # nc.publish("test", "data".into()).await?;
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
            messaging_destination_name: subject.to_string(),
        };

        Self {
            inner,
            metadata: Arc::new(metadata),
            sub_span,
        }
    }

    /// Unsubscribes from subscription, draining all remaining messages.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = si_data_nats::Client::connect_with_options(
    ///     "demo.nats.io",
    ///     None,
    ///     Default::default(),
    /// ).await?;
    ///
    /// let mut subscriber = client.subscribe("foo").await?;
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
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.destination.name = self.metadata.messaging_destination_name.as_str(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Client.as_str(), // similar to an RPC operation
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
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

    /// Unsubscribes from subscription after reaching given number of messages.
    ///
    /// This is the total number of messages received by this subscriber in it's whole lifespan. If
    /// it already reached or surpassed the passed value, it will immediately stop.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use futures::StreamExt;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let client = si_data_nats::Client::connect_with_options(
    ///     "demo.nats.io",
    ///     None,
    ///     Default::default(),
    /// ).await?;
    ///
    /// let mut subscriber = client.subscribe("test").await?;
    /// subscriber.unsubscribe_after(3).await?;
    ///
    /// for _ in 0..3 {
    ///     client.publish("test", "data".into()).await?;
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
        name = "subscriber.unsubscribe_after",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.destination.name = self.metadata.messaging_destination_name.as_str(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Client.as_str(), // similar to an RPC operation
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn unsubscribe_after(&mut self, unsub_after: u64) -> Result<()> {
        let span = Span::current();
        span.follows_from(&self.sub_span);

        self.inner
            .unsubscribe_after(unsub_after)
            .await
            .map_err(|err| span.record_err(Error::NatsUnsubscribe(err)))?;

        span.record_ok();
        Ok(())
    }
}

// API extensions
impl Subscriber {
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
    messaging_destination_name: String,
}

impl SubscriberMessageMetadata {
    /// Gets a reference to the subscriber metadata's messaging destination name.
    pub fn messaging_destination_name(&self) -> &str {
        self.messaging_destination_name.as_str()
    }

    /// Gets a reference to the subscriber metadata's messaging client id.
    pub fn messaging_client_id(&self) -> &str {
        self.connection_metadata.messaging_client_id.as_ref()
    }

    /// Gets a reference to the subscriber metadata's messaging nats server id.
    pub fn messaging_nats_server_id(&self) -> &str {
        self.connection_metadata.messaging_nats_server_id.as_ref()
    }

    /// Gets a reference to the subscriber metadata's messaging nats server name.
    pub fn messaging_nats_server_name(&self) -> &str {
        self.connection_metadata.messaging_nats_server_name.as_ref()
    }

    /// Gets a reference to the subscriber metadata's messaging nats server version.
    pub fn messaging_nats_server_version(&self) -> &str {
        self.connection_metadata
            .messaging_nats_server_version
            .as_ref()
    }

    /// Gets a reference to the subscriber metadata's messaging system.
    pub fn messaging_system(&self) -> &str {
        self.connection_metadata.messaging_system
    }

    /// Gets a reference to the subscriber metadata's messaging url.
    pub fn messaging_url(&self) -> &str {
        self.connection_metadata.messaging_url.as_ref()
    }

    /// Gets a reference to the subscriber metadata's network peer address.
    pub fn network_peer_address(&self) -> &str {
        self.connection_metadata.network_peer_address.as_ref()
    }

    /// Gets a reference to the subscriber metadata's network protocol name.
    pub fn network_protocol_name(&self) -> &str {
        self.connection_metadata.network_protocol_name
    }

    /// Gets a reference to the subscriber metadata's network protocol version.
    pub fn network_protocol_version(&self) -> &str {
        self.connection_metadata.network_protocol_version.as_ref()
    }

    /// Gets a reference to the subscriber metadata's network transport.
    pub fn network_transport(&self) -> &str {
        self.connection_metadata.network_transport
    }

    /// Gets a reference to the subscriber metadata's server address.
    pub fn server_address(&self) -> &str {
        self.connection_metadata.server_address.as_ref()
    }

    /// Gets a reference to the subscriber metadata's server port.
    pub fn server_port(&self) -> u16 {
        self.connection_metadata.server_port
    }

    /// Gets the common prefix for use on all subjects.
    pub fn subject_prefix(&self) -> Option<&str> {
        self.connection_metadata.subject_prefix.as_deref()
    }

    /// Get a reference to the subscriber message metadata's connection metadata.
    pub fn as_connection_metadata(&self) -> Arc<ConnectionMetadata> {
        self.connection_metadata.clone()
    }
}
