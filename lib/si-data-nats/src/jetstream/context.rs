use std::{borrow::Borrow, sync::Arc, time::Duration};

use async_nats::{
    header::{self, IntoHeaderName, IntoHeaderValue},
    jetstream::{
        account::Account,
        consumer::{FromConsumer, IntoConsumerConfig},
        context::{
            AccountError, CreateKeyValueError, CreateObjectStoreError, CreateStreamError,
            DeleteObjectStore, DeleteStreamError, GetStreamError, KeyValueError, ObjectStoreError,
            PublishAckFuture, PublishError, RequestError, UpdateStreamError,
        },
        stream::{Config, ConsumerError, DeleteStatus},
    },
    subject::ToSubject,
    HeaderMap, HeaderValue,
};
use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};
use telemetry::prelude::*;

use crate::{Client, ConnectionMetadata};

/// A context which can perform jetstream scoped requests.
#[derive(Debug, Clone)]
pub struct Context {
    inner: async_nats::jetstream::Context,
    metadata: Arc<ConnectionMetadata>,
}

impl Context {
    // TODO(fnichol): refactor
    pub fn as_inner(&self) -> &async_nats::jetstream::Context {
        &self.inner
    }

    pub(crate) fn new(client: Client) -> Self {
        let (inner_client, metadata) = client.into_parts();
        let inner = async_nats::jetstream::new(inner_client);

        Self { inner, metadata }
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.inner.set_timeout(timeout)
    }

    pub(crate) fn with_prefix(client: Client, prefix: &str) -> Self {
        let (inner_client, metadata) = client.into_parts();
        let inner = async_nats::jetstream::with_prefix(inner_client, prefix);

        Self { inner, metadata }
    }

    pub(crate) fn with_domain<T: AsRef<str>>(client: Client, domain: T) -> Self {
        let (inner_client, metadata) = client.into_parts();
        let inner = async_nats::jetstream::with_domain(inner_client, domain);

        Self { inner, metadata }
    }

    /// Publishes [jetstream::Message][super::message::Message] to the [Stream] without waiting for
    /// acknowledgment from the server that the message has been successfully delivered.
    ///
    /// Acknowledgment future that can be polled is returned instead.
    ///
    /// If the stream does not exist, `no responders` error will be returned.
    ///
    /// # Examples
    ///
    /// Publish, and after each publish, await for acknowledgment.
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    ///
    /// let ack = jetstream.publish("events", "data".into()).await?;
    /// ack.await?;
    /// jetstream.publish("events", "data".into()).await?.await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Publish and do not wait for the acknowledgment. Await can be deferred to when needed or
    /// ignored entirely.
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    ///
    /// let first_ack = jetstream.publish("events", "data".into()).await?;
    /// let second_ack = jetstream.publish("events", "data".into()).await?;
    /// first_ack.await?;
    /// second_ack.await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.publish",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.destination.name = Empty,
            messaging.message.body.size = Empty,
            // messaging.message.conversation_id = Empty,
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.operation = MessagingOperation::Publish.as_str(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Producer.as_str(),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn publish<S: ToSubject>(
        &self,
        subject: S,
        payload: Bytes,
    ) -> Result<PublishAckFuture, PublishError> {
        let span = Span::current();

        let subject = subject.to_subject();
        span.record("messaging.destination.name", subject.as_str());
        span.record("messaging.message.body.size", payload.len());
        span.record(
            "otel.name",
            format!("{} {}", &subject, MessagingOperation::Publish.as_str()).as_str(),
        );
        let fut = self
            .inner
            .publish(subject, payload)
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(fut)
    }

    /// Publish a message with headers to a given subject associated with a stream and returns an
    /// acknowledgment from the server that the message has been successfully delivered.
    ///
    /// If the stream does not exist, `no responders` error will be returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    ///
    /// let mut headers = async_nats::HeaderMap::new();
    /// headers.append("X-key", "Value");
    /// let ack = jetstream
    ///     .publish_with_headers("events", headers, "data".into())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.publish_with_headers",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.destination.name = Empty,
            messaging.message.body.size = Empty,
            // messaging.message.conversation_id = Empty,
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.operation = MessagingOperation::Publish.as_str(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Producer.as_str(),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn publish_with_headers<S: ToSubject>(
        &self,
        subject: S,
        headers: async_nats::header::HeaderMap,
        payload: Bytes,
    ) -> Result<PublishAckFuture, PublishError> {
        let span = Span::current();

        let subject = subject.to_subject();
        span.record("messaging.destination.name", subject.as_str());
        span.record("messaging.message.body.size", payload.len());
        span.record(
            "otel.name",
            format!("{} {}", &subject, MessagingOperation::Publish.as_str()).as_str(),
        );
        let fut = self
            .inner
            .publish_with_headers(subject, headers, payload)
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(fut)
    }

    /// Publish a message built by [Publish] and returns an acknowledgment future.
    ///
    /// If the stream does not exist, `no responders` error will be returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_nats::jetstream::context::Publish;
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    ///
    /// let ack = jetstream
    ///     .send_publish(
    ///         "events",
    ///         Publish::build().payload("data".into()).message_id("uuid"),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.send_publish",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.destination.name = Empty,
            messaging.message.body.size = Empty,
            // messaging.message.conversation_id = Empty,
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.operation = MessagingOperation::Publish.as_str(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Producer.as_str(),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn send_publish<S: ToSubject>(
        &self,
        subject: S,
        publish: Publish,
    ) -> Result<PublishAckFuture, PublishError> {
        let span = Span::current();

        let subject = subject.to_subject();
        span.record("messaging.destination.name", subject.as_str());
        span.record("messaging.message.body.size", publish.payload.len());
        span.record(
            "otel.name",
            format!("{} {}", &subject, MessagingOperation::Publish.as_str()).as_str(),
        );
        let fut = self
            .inner
            .send_publish(subject, publish.into())
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(fut)
    }

    /// Query the server for account information.
    #[instrument(
        name = "context.query_account",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn query_account(&self) -> Result<Account, AccountError> {
        let span = Span::current();

        let account = self
            .inner
            .query_account()
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(account)
    }

    /// Create a JetStream [Stream] with given config and return a handle to it.
    ///
    /// That handle can be used to manage and use [Consumer].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// use async_nats::jetstream::stream::Config;
    /// use async_nats::jetstream::stream::DiscardPolicy;
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    ///
    /// let stream = jetstream
    ///     .create_stream(Config {
    ///         name: "events".to_string(),
    ///         max_messages: 100_000,
    ///         discard: DiscardPolicy::Old,
    ///         ..Default::default()
    ///     })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.create_stream",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn create_stream<S>(
        &self,
        stream_config: S,
    ) -> Result<async_nats::jetstream::stream::Stream, CreateStreamError>
    where
        Config: From<S>,
    {
        let span = Span::current();

        let stream = self
            .inner
            .create_stream(stream_config)
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(stream)
    }

    /// Checks for [Stream] existence on the server and returns handle to it.
    /// That handle can be used to manage and use [Consumer].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    ///
    /// let stream = jetstream.get_stream("events").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.get_stream",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn get_stream<T: AsRef<str>>(
        &self,
        stream: T,
    ) -> Result<async_nats::jetstream::stream::Stream, GetStreamError> {
        let span = Span::current();

        let stream = self
            .inner
            .get_stream(stream)
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(stream)
    }

    /// Create a stream with the given configuration on the server if it is not present.
    ///
    /// Returns a handle to the stream on the server.
    ///
    /// Note: This does not validate if the Stream on the server is compatible with the
    /// configuration passed in.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// use async_nats::jetstream::stream::Config;
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    ///
    /// let stream = jetstream
    ///     .get_or_create_stream(Config {
    ///         name: "events".to_string(),
    ///         max_messages: 10_000,
    ///         ..Default::default()
    ///     })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.get_or_create_stream",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn get_or_create_stream<S>(
        &self,
        stream_config: S,
    ) -> Result<async_nats::jetstream::stream::Stream, CreateStreamError>
    where
        S: Into<Config>,
    {
        let span = Span::current();

        let stream_config = stream_config.into();
        trace!(?stream_config);

        let stream = self
            .inner
            .get_or_create_stream(stream_config)
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(stream)
    }

    /// Deletes a [Stream] with a given name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// use async_nats::jetstream::stream::Config;
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    ///
    /// let stream = jetstream.delete_stream("events").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.delete_stream",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn delete_stream<T: AsRef<str>>(
        &self,
        stream: T,
    ) -> Result<async_nats::jetstream::stream::DeleteStatus, DeleteStreamError> {
        let span = Span::current();

        let status = self
            .inner
            .delete_stream(stream)
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(status)
    }

    /// Updates a [Stream] with a given config. If specific field cannot be updated, error is
    /// returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// use async_nats::jetstream::stream::Config;
    /// use async_nats::jetstream::stream::DiscardPolicy;
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    ///
    /// let stream = jetstream
    ///     .update_stream(&Config {
    ///         name: "events".to_string(),
    ///         discard: DiscardPolicy::New,
    ///         max_messages: 50_000,
    ///         ..Default::default()
    ///     })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.update_stream",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn update_stream<S>(
        &self,
        config: S,
    ) -> Result<async_nats::jetstream::stream::Info, UpdateStreamError>
    where
        S: Borrow<Config>,
    {
        let span = Span::current();

        let info = self
            .inner
            .update_stream(config)
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(info)
    }

    /// Lists names of all streams for current context.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// use futures::TryStreamExt;
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    /// let mut names = jetstream.stream_names();
    /// while let Some(stream) = names.try_next().await? {
    ///     println!("stream: {}", stream);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn stream_names(&self) -> async_nats::jetstream::context::StreamNames {
        self.inner.stream_names()
    }

    /// Lists all streams info for current context.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// use futures::TryStreamExt;
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    /// let mut streams = jetstream.streams();
    /// while let Some(stream) = streams.try_next().await? {
    ///     println!("stream: {:?}", stream);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn streams(&self) -> async_nats::jetstream::context::Streams {
        self.inner.streams()
    }

    /// Returns an existing key-value bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    /// let kv = jetstream.get_key_value("bucket").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.get_key_value",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn get_key_value<T: Into<String>>(
        &self,
        bucket: T,
    ) -> Result<async_nats::jetstream::kv::Store, KeyValueError> {
        let span = Span::current();

        let store = self
            .inner
            .get_key_value(bucket)
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(store)
    }

    /// Creates a new key-value bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    /// let kv = jetstream
    ///     .create_key_value(async_nats::jetstream::kv::Config {
    ///         bucket: "kv".to_string(),
    ///         history: 10,
    ///         ..Default::default()
    ///     })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.create_key_value",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn create_key_value(
        &self,
        config: async_nats::jetstream::kv::Config,
    ) -> Result<async_nats::jetstream::kv::Store, CreateKeyValueError> {
        let span = Span::current();

        let store = self
            .inner
            .create_key_value(config)
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(store)
    }

    /// Deletes given key-value bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    /// let kv = jetstream
    ///     .create_key_value(async_nats::jetstream::kv::Config {
    ///         bucket: "kv".to_string(),
    ///         history: 10,
    ///         ..Default::default()
    ///     })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.delete_key_value",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn delete_key_value<T: AsRef<str>>(
        &self,
        bucket: T,
    ) -> Result<async_nats::jetstream::stream::DeleteStatus, KeyValueError> {
        let span = Span::current();

        let status = self
            .inner
            .delete_key_value(bucket)
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(status)
    }

    /// Get a [crate::jetstream::consumer::Consumer] straight from [Context], without binding to a
    /// [Stream] first.
    ///
    /// It has one less interaction with the server when binding to only one
    /// [crate::jetstream::consumer::Consumer].
    ///
    /// # Examples:
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// use async_nats::jetstream::consumer::PullConsumer;
    ///
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    ///
    /// let consumer: PullConsumer = jetstream
    ///     .get_consumer_from_stream("consumer", "stream")
    ///     .await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.get_consumer_from_stream",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn get_consumer_from_stream<T, C, S>(
        &self,
        consumer: C,
        stream: S,
    ) -> Result<async_nats::jetstream::consumer::Consumer<T>, ConsumerError>
    where
        T: FromConsumer + IntoConsumerConfig,
        S: AsRef<str>,
        C: AsRef<str>,
    {
        let span = Span::current();

        let consumer = self
            .inner
            .get_consumer_from_stream(consumer, stream)
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(consumer)
    }

    /// Delete a [crate::jetstream::consumer::Consumer] straight from [Context], without binding to
    /// a [Stream] first.
    ///
    /// It has one less interaction with the server when binding to only one
    /// [crate::jetstream::consumer::Consumer].
    ///
    /// # Examples:
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// use async_nats::jetstream::consumer::PullConsumer;
    ///
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    ///
    /// jetstream
    ///     .delete_consumer_from_stream("consumer", "stream")
    ///     .await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.delete_consumer_from_stream",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn delete_consumer_from_stream<C: AsRef<str>, S: AsRef<str>>(
        &self,
        consumer: C,
        stream: S,
    ) -> Result<DeleteStatus, ConsumerError> {
        let span = Span::current();

        let status = self
            .inner
            .delete_consumer_from_stream(consumer, stream)
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(status)
    }

    /// Create a new `Durable` or `Ephemeral` Consumer (if `durable_name` was not provided) and
    /// returns the info from the server about created [Consumer] without binding to a [Stream]
    /// first.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// use async_nats::jetstream::consumer;
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    ///
    /// let consumer: consumer::PullConsumer = jetstream
    ///     .create_consumer_on_stream(
    ///         consumer::pull::Config {
    ///             durable_name: Some("pull".to_string()),
    ///             ..Default::default()
    ///         },
    ///         "stream",
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.create_consumer_from_stream",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn create_consumer_on_stream<C: IntoConsumerConfig + FromConsumer, S: AsRef<str>>(
        &self,
        config: C,
        stream: S,
    ) -> Result<async_nats::jetstream::consumer::Consumer<C>, ConsumerError> {
        let span = Span::current();

        let consumer = self
            .inner
            .create_consumer_on_stream(config, stream)
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(consumer)
    }

    /// Send a request to the jetstream JSON API.
    ///
    /// This is a low level API used mostly internally, that should be used only in specific cases
    /// when this crate API on [Consumer] or [Stream] does not provide needed functionality.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # use async_nats::jetstream::stream::Info;
    /// # use async_nats::jetstream::response::Response;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    ///
    /// let response: Response<Info> = jetstream.request("STREAM.INFO.events", &()).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.request",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.destination.name = Empty,
            // messaging.message.conversation_id = Empty,
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.operation = MessagingOperation::Publish.as_str(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Producer.as_str(),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn request<S, T, V>(&self, subject: S, payload: &T) -> Result<V, RequestError>
    where
        S: ToSubject,
        T: ?Sized + Serialize,
        V: DeserializeOwned,
    {
        let span = Span::current();

        let subject = subject.to_subject();
        span.record("messaging.destination.name", subject.as_str());
        span.record(
            "otel.name",
            format!("{} {}", &subject, MessagingOperation::Publish.as_str()).as_str(),
        );
        let v = self
            .inner
            .request(subject, payload)
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(v)
    }

    /// Creates a new object store bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    /// let bucket = jetstream
    ///     .create_object_store(async_nats::jetstream::object_store::Config {
    ///         bucket: "bucket".to_string(),
    ///         ..Default::default()
    ///     })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.create_object_store",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn create_object_store(
        &self,
        config: async_nats::jetstream::object_store::Config,
    ) -> Result<async_nats::jetstream::object_store::ObjectStore, CreateObjectStoreError> {
        let span = Span::current();

        let store = self
            .inner
            .create_object_store(config)
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(store)
    }

    /// Get an existing object store bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    /// let bucket = jetstream.get_object_store("bucket").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.get_object_store",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn get_object_store<T: AsRef<str>>(
        &self,
        bucket_name: T,
    ) -> Result<async_nats::jetstream::object_store::ObjectStore, ObjectStoreError> {
        let span = Span::current();

        let store = self
            .inner
            .get_object_store(bucket_name)
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(store)
    }

    /// Delete a object store bucket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = Client::connect_with_options(
    ///     "localhost:4222",
    ///     None,
    ///     ConnectOptions::default(),
    /// ).await?;
    /// let jetstream = si_data_nats::jetstream::new(client);
    /// let bucket = jetstream.delete_object_store("bucket").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "context.delete_object_store",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn delete_object_store<T: AsRef<str>>(
        &self,
        bucket_name: T,
    ) -> Result<(), DeleteObjectStore> {
        let span = Span::current();

        self.inner
            .delete_object_store(bucket_name)
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(())
    }
}

/// Used for building customized `publish` message.
///
/// Note: this type is wrapped in order to expose the inner fields this wrapping module.
#[derive(Default, Clone, Debug)]
pub struct Publish {
    payload: Bytes,
    headers: Option<async_nats::header::HeaderMap>,
}

impl Publish {
    /// Creates a new custom Publish struct to be used with.
    pub fn build() -> Self {
        Default::default()
    }

    /// Sets the payload for the message.
    pub fn payload(mut self, payload: Bytes) -> Self {
        self.payload = payload;
        self
    }

    /// Adds headers to the message.
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers = Some(headers);
        self
    }

    /// A shorthand to add a single header.
    pub fn header<N: IntoHeaderName, V: IntoHeaderValue>(mut self, name: N, value: V) -> Self {
        self.headers
            .get_or_insert(header::HeaderMap::new())
            .insert(name, value);
        self
    }

    /// Sets the `Nats-Msg-Id` header, that is used by stream deduplicate window.
    pub fn message_id<T: AsRef<str>>(self, id: T) -> Self {
        self.header(header::NATS_MESSAGE_ID, id.as_ref())
    }

    /// Sets expected last message ID.
    ///
    /// It sets the `Nats-Expected-Last-Msg-Id` header with provided value.
    pub fn expected_last_message_id<T: AsRef<str>>(self, last_message_id: T) -> Self {
        self.header(
            header::NATS_EXPECTED_LAST_MESSAGE_ID,
            last_message_id.as_ref(),
        )
    }

    /// Sets the last expected stream sequence.
    ///
    /// It sets the `Nats-Expected-Last-Sequence` header with provided value.
    pub fn expected_last_sequence(self, last_sequence: u64) -> Self {
        self.header(
            header::NATS_EXPECTED_LAST_SEQUENCE,
            HeaderValue::from(last_sequence),
        )
    }

    /// Sets the last expected stream sequence for a subject this message will be published to.
    ///
    /// It sets the `Nats-Expected-Last-Subject-Sequence` header with provided value.
    pub fn expected_last_subject_sequence(self, subject_sequence: u64) -> Self {
        self.header(
            header::NATS_EXPECTED_LAST_SUBJECT_SEQUENCE,
            HeaderValue::from(subject_sequence),
        )
    }

    /// Sets the expected stream name.
    ///
    /// It sets the `Nats-Expected-Stream` header with provided value.
    pub fn expected_stream<T: AsRef<str>>(self, stream: T) -> Self {
        self.header(
            header::NATS_EXPECTED_STREAM,
            HeaderValue::from(stream.as_ref()),
        )
    }
}

impl From<Publish> for async_nats::jetstream::context::Publish {
    fn from(value: Publish) -> Self {
        let mut p = Self::build();
        p = p.payload(value.payload);
        if let Some(headers) = value.headers {
            p = p.headers(headers);
        }
        p
    }
}
