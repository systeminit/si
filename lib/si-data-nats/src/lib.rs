#![warn(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn
)]
#![allow(clippy::missing_errors_doc)]

use std::{fmt::Debug, io, sync::Arc, time::Duration};

use async_nats::{subject::ToSubject, ToServerAddrs};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::Mutex;

mod connect_options;
mod message;
pub mod service;
mod subscriber;

pub use async_nats::{
    connection::State, header, header::HeaderMap, rustls, status, subject, Auth, AuthError,
    HeaderName, HeaderValue, ServerAddr, ServerInfo, Subject,
};
pub use connect_options::ConnectOptions;
pub use message::{InnerMessage, Message};
pub use subscriber::Subscriber;

pub type NatsError = Error;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("nats connect error: {0}")]
    NatsConnect(#[from] async_nats::ConnectError),
    #[error("nats flush error: {0}")]
    NatsFlush(#[from] async_nats::client::FlushError),
    #[error("nats publish error: {0}")]
    NatsPublish(#[from] async_nats::PublishError),
    #[error("nats request error: {0}")]
    NatsRequest(#[from] async_nats::RequestError),
    #[error("nats subscribe error: {0}")]
    NatsSubscribe(#[from] async_nats::SubscribeError),
    #[error("nats unsubscribe error: {0}")]
    NatsUnsubscribe(#[from] async_nats::UnsubscribeError),
    #[error("error serializing object: {0}")]
    Serialize(#[source] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NatsConfig {
    pub connection_name: Option<String>,
    pub creds: Option<String>,
    pub creds_file: Option<String>,
    pub subject_prefix: Option<String>,
    pub url: String,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            connection_name: None,
            creds: None,
            creds_file: None,
            subject_prefix: None,
            url: "localhost".to_string(),
        }
    }
}

// Ensure that we only grab the current span if we're at debug level or lower, otherwise use none.
//
// When recording a parent span for long running tasks such as a transaction we want the direct
// span parent. However, `Span::current()` returns a suitable parent span, according to the tracing
// `Subscriber`, meaning that instead of capturing the transaction starting span, we might capture
// a calling function up the stack that is at the info level or higher. In other words, then
// "transaction span" might be an ancestor span unless we're really careful.
macro_rules! current_span_for_debug {
    () => {
        Span::none()
    };
}

pub type NatsClient = Client;

#[derive(Clone, Debug)]
pub struct Client {
    inner: async_nats::Client,
    metadata: Arc<ConnectionMetadata>,
}

impl Client {
    #[instrument(name = "client.new", skip_all, level = "debug")]
    pub async fn new(config: &NatsConfig) -> Result<Self> {
        let mut options = ConnectOptions::default();

        if let Some(creds) = &config.creds {
            options = options.credentials(creds)?;
        }
        if let Some(creds_file) = &config.creds_file {
            options = options.credentials_file(creds_file).await?;
        }
        if let Some(connection_name) = &config.connection_name {
            options = options.name(connection_name);
        }
        Self::connect_with_options(&config.url, config.subject_prefix.clone(), options).await
    }

    /// Returns last received info from the server.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main () -> Result<(), async_nats::Error> {
    /// let client = si_data_nats::Client::connect_with_options("demo.nats.io", None, Default::default()).await?;
    /// println!("info: {:?}", client.server_info());
    /// # Ok(())
    /// # }
    /// ```
    pub fn server_info(&self) -> async_nats::ServerInfo {
        self.inner.server_info()
    }

    /// Returns true if the server version is compatible with the version components.
    ///
    /// # Examples
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = si_data_nats::Client::connect_with_options("demo.nats.io", None, Default::default()).await?;
    /// assert!(client.is_server_compatible(2, 8, 4));
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_server_compatible(&self, major: i64, minor: i64, patch: i64) -> bool {
        self.inner.is_server_compatible(major, minor, patch)
    }

    /// Publish a [Message] to a given subject.
    ///
    /// # Examples
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let client = si_data_nats::Client::connect_with_options("demo.nats.io", None, Default::default()).await?;
    /// client
    ///     .publish("events.data".into(), "payload".into())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "client.publish",
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
    pub async fn publish(&self, subject: impl ToSubject, payload: Bytes) -> Result<()> {
        let span = Span::current();

        let subject = subject.to_subject();
        span.record("messaging.destination.name", subject.as_str());
        span.record("messaging.message.body.size", payload.len());
        span.record(
            "otel.name",
            format!("{} {}", &subject, MessagingOperation::Publish.as_str()).as_str(),
        );
        self.inner
            .publish(subject, payload)
            .await
            .map_err(|err| span.record_err(Error::NatsPublish(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Publish a [Message] with headers to a given subject.
    ///
    /// # Examples
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// use std::str::FromStr;
    /// let client = si_data_nats::Client::connect_with_options("demo.nats.io", None, Default::default()).await?;
    /// let mut headers = async_nats::HeaderMap::new();
    /// headers.insert(
    ///     "X-Header",
    ///     async_nats::HeaderValue::from_str("Value").unwrap(),
    /// );
    /// client
    ///     .publish_with_headers("events.data".into(), headers, "payload".into())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "client.publish_with_headers",
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
    pub async fn publish_with_headers(
        &self,
        subject: impl ToSubject,
        headers: HeaderMap,
        payload: Bytes,
    ) -> Result<()> {
        let span = Span::current();

        let subject = subject.to_subject();
        span.record("messaging.destination.name", subject.as_str());
        span.record("messaging.message.body.size", payload.len());
        span.record(
            "otel.name",
            format!("{} {}", &subject, MessagingOperation::Publish.as_str()).as_str(),
        );
        self.inner
            .publish_with_headers(subject, headers, payload)
            .await
            .map_err(|err| span.record_err(Error::NatsPublish(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Publish a [Message] to a given subject, with specified response subject to which the
    /// subscriber can respond. This method does not await for the response.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let client = si_data_nats::Client::connect_with_options("demo.nats.io", None, Default::default()).await?;
    /// client
    ///     .publish_with_reply(
    ///         "events.data".into(),
    ///         "reply_subject".into(),
    ///         "payload".into(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "client.publish_with_reply",
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
    pub async fn publish_with_reply(
        &self,
        subject: impl ToSubject,
        reply: impl ToSubject,
        payload: Bytes,
    ) -> Result<()> {
        let span = Span::current();

        let subject = subject.to_subject();
        span.record("messaging.destination.name", subject.as_str());
        span.record("messaging.message.body.size", payload.len());
        span.record(
            "otel.name",
            format!("{} {}", &subject, MessagingOperation::Publish.as_str()).as_str(),
        );
        self.inner
            .publish_with_reply(subject, reply, payload)
            .await
            .map_err(|err| span.record_err(Error::NatsPublish(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Publish a [Message] to a given subject with headers and specified response subject to which
    /// the subscriber can respond. This method does not await for the response.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// use std::str::FromStr;
    /// let client = si_data_nats::Client::connect_with_options("demo.nats.io", None, Default::default()).await?;
    /// let mut headers = async_nats::HeaderMap::new();
    /// client
    ///     .publish_with_reply_and_headers(
    ///         "events.data",
    ///         "reply_subject",
    ///         headers,
    ///         "payload".into(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "client.publish_with_reply_and_headers",
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
    pub async fn publish_with_reply_and_headers(
        &self,
        subject: impl ToSubject,
        reply: impl ToSubject,
        headers: HeaderMap,
        payload: Bytes,
    ) -> Result<()> {
        let span = Span::current();

        let subject = subject.to_subject();
        span.record("messaging.destination.name", subject.as_str());
        span.record("messaging.message.body.size", payload.len());
        span.record(
            "otel.name",
            format!("{} {}", &subject, MessagingOperation::Publish.as_str()).as_str(),
        );
        self.inner
            .publish_with_reply_and_headers(subject, reply, headers, payload)
            .await
            .map_err(|err| span.record_err(Error::NatsPublish(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Sends the request with headers.
    ///
    /// # Examples
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let client = si_data_nats::Client::connect_with_options("demo.nats.io", None, Default::default()).await?;
    /// let response = client.request("service", "data".into()).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "client.request",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.destination.name = Empty,
            messaging.message.body.size = Empty,
            // TODO: maybe use this and inject in headers?
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
            otel.kind = SpanKind::Client.as_str(), // similar to an RPC operation
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn request(&self, subject: impl ToSubject, payload: Bytes) -> Result<Message> {
        let span = Span::current();

        let subject = subject.to_subject();
        span.record("messaging.destination.name", subject.as_str());
        span.record("messaging.message.body.size", payload.len());
        span.record(
            "otel.name",
            format!("{} {}", &subject, MessagingOperation::Publish.as_str()).as_str(),
        );
        let msg = self
            .inner
            .request(subject, payload)
            .await
            .map_err(|err| span.record_err(Error::NatsRequest(err)))?;

        span.record_ok();
        Ok(Message::new(msg, self.metadata.clone()))
    }

    /// Sends the request with headers.
    ///
    /// # Examples
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = si_data_nats::Client::connect_with_options("demo.nats.io", None, Default::default()).await?;
    /// let mut headers = async_nats::HeaderMap::new();
    /// headers.insert("Key", "Value");
    /// let response = client
    ///     .request_with_headers("service", headers, "data".into())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "client.request_with_headers",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.destination.name = Empty,
            messaging.message.body.size = Empty,
            // TODO: maybe use this and inject in headers?
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
            otel.kind = SpanKind::Client.as_str(), // similar to an RPC operation
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn request_with_headers(
        &self,
        subject: impl ToSubject,
        headers: HeaderMap,
        payload: Bytes,
    ) -> Result<Message> {
        let span = Span::current();

        let subject = subject.to_subject();
        span.record("messaging.destination.name", subject.as_str());
        span.record("messaging.message.body.size", payload.len());
        span.record(
            "otel.name",
            format!("{} {}", &subject, MessagingOperation::Publish.as_str()).as_str(),
        );
        let msg = self
            .inner
            .request_with_headers(subject, headers, payload)
            .await
            .map_err(|err| span.record_err(Error::NatsRequest(err)))?;

        span.record_ok();
        Ok(Message::new(msg, self.metadata.clone()))
    }

    /// Sends the request created by the [Request].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = si_data_nats::Client::connect_with_options("demo.nats.io", None, Default::default()).await?;
    /// let request = async_nats::Request::new().payload("data".into());
    /// let response = client.send_request("service", request).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "client.send_request",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.destination.name = Empty,
            messaging.message.body.size = Empty,
            // TODO: maybe use this and inject in headers?
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
            otel.kind = SpanKind::Client.as_str(), // similar to an RPC operation
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn send_request(&self, subject: impl ToSubject, request: Request) -> Result<Message> {
        let span = Span::current();

        let subject = subject.to_subject();
        span.record("messaging.destination.name", subject.as_str());
        if let Some(ref payload) = request.payload {
            span.record("messaging.message.body.size", payload.len());
        }
        span.record(
            "otel.name",
            format!("{} {}", &subject, MessagingOperation::Publish.as_str()).as_str(),
        );
        let msg = self
            .inner
            .send_request(subject, request.into())
            .await
            .map_err(|err| span.record_err(Error::NatsRequest(err)))?;

        span.record_ok();
        Ok(Message::new(msg, self.metadata.clone()))
    }

    /// Create a new globally unique inbox which can be used for replies.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let client = si_data_nats::Client::connect_with_options("demo.nats.io", None, Default::default()).await?;
    /// let reply = nc.new_inbox();
    /// let rsub = nc.subscribe(reply).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn new_inbox(&self) -> String {
        self.inner.new_inbox()
    }

    /// Subscribes to a subject to receive [messages][Message].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// use futures::StreamExt;
    /// let client = si_data_nats::Client::connect_with_options("demo.nats.io", None, Default::default()).await?;
    /// let mut subscription = client.subscribe("events.>".into()).await?;
    /// while let Some(message) = subscription.next().await {
    ///     println!("received message: {:?}", message);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "client.subscribe",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.destination.name = Empty,
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.operation = MessagingOperation::Receive.as_str(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Consumer.as_str(),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn subscribe(&self, subject: impl ToSubject) -> Result<Subscriber> {
        let span = Span::current();

        let subject = subject.to_subject();
        span.record("messaging.destination.name", subject.as_str());
        span.record(
            "otel.name",
            format!("{} {}", &subject, MessagingOperation::Receive.as_str()).as_str(),
        );
        let sub_subject = subject.clone();
        let sub = self
            .inner
            .subscribe(sub_subject)
            .await
            .map_err(|err| span.record_err(Error::NatsSubscribe(err)))?;

        Ok(Subscriber::new(
            sub,
            &subject,
            self.metadata.clone(),
            current_span_for_debug!(),
        ))
    }

    /// Subscribes to a subject with a queue group to receive [messages][Message].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// use futures::StreamExt;
    /// let client = si_data_nats::Client::connect_with_options("demo.nats.io", None, Default::default()).await?;
    /// let mut subscription = client
    ///     .queue_subscribe("events.>".into(), "queue".into())
    ///     .await?;
    /// while let Some(message) = subscription.next().await {
    ///     println!("received message: {:?}", message);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "client.queue_subscribe",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.destination.name = Empty,
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.nats.queue_group = Empty,
            messaging.operation = MessagingOperation::Receive.as_str(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.protocol.version = self.metadata.network_protocol_version(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Consumer.as_str(),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn queue_subscribe(
        &self,
        subject: impl ToSubject,
        queue_group: String,
    ) -> Result<Subscriber> {
        let span = Span::current();

        let subject = subject.to_subject();
        span.record("messaging.destination.name", subject.as_str());
        span.record(
            "otel.name",
            format!("{} {}", &subject, MessagingOperation::Receive.as_str()).as_str(),
        );
        span.record("messaging.nats.queue_group", queue_group.as_str());
        let sub_subject = subject.clone();
        let sub = self
            .inner
            .queue_subscribe(sub_subject, queue_group)
            .await
            .map_err(|err| span.record_err(Error::NatsSubscribe(err)))?;

        Ok(Subscriber::new(
            sub,
            &subject,
            self.metadata.clone(),
            current_span_for_debug!(),
        ))
    }

    /// Flushes the internal buffer ensuring that all messages are sent.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let client = si_data_nats::Client::connect_with_options("demo.nats.io", None, Default::default()).await?;
    /// client.flush().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "client.flush",
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
            otel.kind = SpanKind::Client.as_str(), // similar to an RPC operation
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        )
    )]
    pub async fn flush(&self) -> Result<()> {
        let span = Span::current();

        self.inner
            .flush()
            .await
            .map_err(|err| span.record_err(Error::NatsFlush(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Returns the current state of the connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), async_nats::Error> {
    /// let client = si_data_nats::Client::connect_with_options("demo.nats.io", None, Default::default()).await?;
    /// println!("connection state: {}", client.connection_state());
    /// # Ok(())
    /// # }
    /// ```
    pub fn connection_state(&self) -> State {
        self.inner.connection_state()
    }
}

// API extensions
impl Client {
    #[instrument(
        name = "client.transaction",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = self.metadata.messaging_client_id(),
            messaging.nats.server.id = self.metadata.messaging_nats_server_id(),
            messaging.nats.server.name = self.metadata.messaging_nats_server_name(),
            messaging.nats.server.version = self.metadata.messaging_nats_server_version(),
            messaging.system = self.metadata.messaging_system(),
            messaging.url = self.metadata.messaging_url(),
            messaging.x.transaction = Empty,
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
    pub fn transaction(&self) -> NatsTxn {
        NatsTxn::new(
            self.clone(),
            self.metadata.clone(),
            current_span_for_debug!(),
        )
    }

    /// Establish a `Connection` with a NATS server.
    ///
    /// Multiple servers may be specified by separating them with commas.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let nc = Client::connect_with_options(
    ///         "demo.nats.io",
    ///         None,
    ///         ConnectOptions::default(),
    ///     ).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// In the below case, the second server is configured to use TLS but the first one is not.
    /// Using the `tls_required` method can ensure that all servers are connected to with TLS, if
    /// that is your intention.
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, ConnectOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let nc = Client::connect_with_options(
    ///         "nats://demo.nats.io:4222,tls://demo.nats.io:4443",
    ///         None,
    ///         ConnectOptions::default(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "client::connect_with_options",
        skip_all,
        level = "debug",
        fields(
            messaging.client_id = Empty,
            messaging.nats.server.id = Empty,
            messaging.nats.server.name = Empty,
            messaging.nats.server.version = Empty,
            messaging.system = Empty,
            messaging.url = Empty,
            network.peer.address = Empty,
            network.protocol.name = Empty,
            network.protocol.version = Empty,
            network.transport = Empty,
            otel.kind = SpanKind::Client.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = Empty,
            server.port = Empty,
        )
    )]
    pub async fn connect_with_options(
        addrs: impl ToServerAddrs,
        subject_prefix: Option<String>,
        options: ConnectOptions,
    ) -> Result<Self> {
        let addrs = addrs.to_server_addrs()?.collect::<Vec<_>>();

        let messaging_system = "nats";
        let messaging_url = addrs
            .clone()
            .into_iter()
            .map(|a| a.into_inner().into())
            .collect::<Vec<String>>()
            .join(",");
        let network_protocol_name = "nats";
        let network_transport = "ip_tcp";

        let span = Span::current();
        span.record("messaging.system", messaging_system);
        span.record("messaging.url", messaging_url.as_str());
        span.record("network.protocol.name", network_protocol_name);
        span.record("network.transport", network_transport);

        let inner = options
            .inner
            .connect(addrs)
            .await
            .map_err(|err| span.record_err(Error::NatsConnect(err)))?;
        debug!("successfully connected to nats");

        let server_info = inner.server_info();

        let metadata = ConnectionMetadata {
            messaging_client_id: server_info.client_id.to_string(),
            messaging_system,
            messaging_url,
            messaging_nats_server_id: server_info.server_id,
            messaging_nats_server_name: server_info.server_name,
            messaging_nats_server_version: server_info.version,
            network_peer_address: server_info.client_ip,
            network_transport,
            network_protocol_name,
            network_protocol_version: server_info.proto.to_string(),
            server_address: server_info.host,
            server_port: server_info.port,
            subject_prefix,
        };

        span.record("messaging.client_id", metadata.messaging_client_id.as_str());
        span.record(
            "messaging.nats.server.id",
            metadata.messaging_nats_server_id.as_str(),
        );
        span.record(
            "messaging.nats.server.name",
            metadata.messaging_nats_server_name.as_str(),
        );
        span.record(
            "messaging.nats.server.version",
            metadata.messaging_nats_server_version.as_str(),
        );
        span.record(
            "network.peer.address",
            metadata.network_peer_address.as_str(),
        );
        span.record(
            "network.protocol.version",
            metadata.network_protocol_version.as_str(),
        );
        span.record("server.address", metadata.server_address.as_str());
        span.record("server.port", metadata.server_port);

        span.record_ok();
        Ok(Self {
            inner,
            metadata: Arc::new(metadata),
        })
    }

    /// Gets a reference to the client's metadata.
    pub fn metadata(&self) -> &ConnectionMetadata {
        self.metadata.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct ConnectionMetadata {
    messaging_client_id: String,
    messaging_nats_server_id: String,
    messaging_nats_server_name: String,
    messaging_nats_server_version: String,
    messaging_system: &'static str,
    messaging_url: String,
    network_peer_address: String,
    network_protocol_name: &'static str,
    network_protocol_version: String,
    network_transport: &'static str,
    server_address: String,
    server_port: u16,
    subject_prefix: Option<String>,
}

impl ConnectionMetadata {
    /// Gets a reference to the connection metadata's messaging client id.
    pub fn messaging_client_id(&self) -> &str {
        self.messaging_client_id.as_ref()
    }

    /// Gets a reference to the connection metadata's messaging nats server id.
    pub fn messaging_nats_server_id(&self) -> &str {
        self.messaging_nats_server_id.as_ref()
    }

    /// Gets a reference to the connection metadata's messaging nats server name.
    pub fn messaging_nats_server_name(&self) -> &str {
        self.messaging_nats_server_name.as_ref()
    }

    /// Gets a reference to the connection metadata's messaging nats server version.
    pub fn messaging_nats_server_version(&self) -> &str {
        self.messaging_nats_server_version.as_ref()
    }

    /// Gets a reference to the connection metadata's messaging system.
    pub fn messaging_system(&self) -> &str {
        self.messaging_system
    }

    /// Gets a reference to the connection metadata's messaging url.
    pub fn messaging_url(&self) -> &str {
        self.messaging_url.as_ref()
    }

    /// Gets a reference to the connection metadata's network peer address.
    pub fn network_peer_address(&self) -> &str {
        self.network_peer_address.as_ref()
    }

    /// Gets a reference to the connection metadata's network protocol name.
    pub fn network_protocol_name(&self) -> &str {
        self.network_protocol_name
    }

    /// Gets a reference to the connection metadata's network protocol version.
    pub fn network_protocol_version(&self) -> &str {
        self.network_protocol_version.as_ref()
    }

    /// Gets a reference to the connection metadata's network transport.
    pub fn network_transport(&self) -> &str {
        self.network_transport
    }

    /// Gets a reference to the connection metadata's server address.
    pub fn server_address(&self) -> &str {
        self.server_address.as_ref()
    }

    /// Gets a reference to the connection metadata's server port.
    pub fn server_port(&self) -> u16 {
        self.server_port
    }

    /// Gets the common prefix for use on all subjects.
    pub fn subject_prefix(&self) -> Option<&str> {
        self.subject_prefix.as_deref()
    }
}

#[derive(Clone, Debug)]
pub struct NatsTxn {
    client: Client,
    pending_publish: Arc<Mutex<Vec<(Subject, serde_json::Value)>>>,
    metadata: Arc<ConnectionMetadata>,
    tx_span: Span,
}

impl NatsTxn {
    fn new(client: Client, metadata: Arc<ConnectionMetadata>, tx_span: Span) -> Self {
        Self {
            client,
            pending_publish: Arc::new(Mutex::new(Vec::new())),
            metadata,
            tx_span,
        }
    }

    #[instrument(
        name = "transaction.publish",
        skip_all,
        level = "debug",
        fields(
            messaging.destination.name = Empty,
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn publish<T>(&self, subject: impl ToSubject, object: &T) -> Result<()>
    where
        T: Serialize + Debug,
    {
        let span = Span::current();
        span.follows_from(&self.tx_span);

        let subject = subject.to_subject();
        span.record("messaging.destination.name", subject.as_str());
        let json: serde_json::Value = serde_json::to_value(object)
            .map_err(|err| span.record_err(self.tx_span.record_err(Error::Serialize(err))))?;
        let mut pending_publish = self.pending_publish.lock().await;
        pending_publish.push((subject, json));

        Ok(())
    }

    #[instrument(
        name = "transaction.publish_immediately",
        skip_all,
        level = "debug",
        fields(
            messaging.destination.name = Empty,
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn publish_immediately<T>(&self, subject: impl ToSubject, object: &T) -> Result<()>
    where
        T: Serialize + Debug,
    {
        let span = Span::current();
        span.follows_from(&self.tx_span);

        let subject = subject.to_subject();
        span.record("messaging.destination.name", subject.as_str());
        let json: serde_json::Value = serde_json::to_value(object)
            .map_err(|err| span.record_err(self.tx_span.record_err(Error::Serialize(err))))?;
        let msg = serde_json::to_vec(&json)
            .map_err(|err| span.record_err(self.tx_span.record_err(Error::Serialize(err))))?;
        self.client
            .publish(subject, msg.into())
            .await
            .map_err(|err| span.record_err(self.tx_span.record_err(err)))?;

        Ok(())
    }

    #[instrument(
        name = "transaction.commit_into_conn",
        skip_all,
        level = "debug",
        fields(
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn commit_into_conn(self) -> Result<Client> {
        let span = Span::current();
        span.follows_from(&self.tx_span);

        let mut pending_publish = self.pending_publish.lock_owned().await;
        for (subject, object) in pending_publish.drain(0..) {
            let msg = serde_json::to_vec(&object)
                .map_err(|err| span.record_err(self.tx_span.record_err(Error::Serialize(err))))?;
            self.client
                .publish(subject, msg.into())
                .await
                .map_err(|err| span.record_err(self.tx_span.record_err(err)))?;
        }

        self.tx_span.record_ok();
        self.tx_span.record("messaging.x.transaction", "commit");
        span.record_ok();

        Ok(self.client)
    }

    #[instrument(
        name = "transaction.commit",
        skip_all,
        level = "debug",
        fields(
            otel.kind = SpanKind::Internal.as_str(),
        )
    )]
    pub async fn commit(self) -> Result<()> {
        let _ = self.commit_into_conn().await?;
        Ok(())
    }

    #[instrument(
        name = "transaction.rollback_into_conn",
        skip_all,
        level = "debug",
        fields(
            otel.kind = SpanKind::Internal.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn rollback_into_conn(self) -> Result<Client> {
        let span = Span::current();
        span.follows_from(&self.tx_span);

        // Nothing much to do, we want to drop the pending publishes which happens when this
        // function returns (i.e. it consumes `self`).

        self.tx_span.record_ok();
        self.tx_span.record("messaging.x.transaction", "rollback");
        span.record_ok();

        Ok(self.client)
    }

    #[instrument(
        name = "transaction.rollback",
        skip_all,
        level = "debug",
        fields(
            otel.kind = SpanKind::Internal.as_str(),
        )
    )]
    pub async fn rollback(self) -> Result<()> {
        let _ = self.rollback_into_conn().await?;
        Ok(())
    }

    /// Gets a reference to the nats txn's metadata.
    pub fn metadata(&self) -> &ConnectionMetadata {
        self.metadata.as_ref()
    }
}

/// Used for building customized requests.
///
/// Note: this type is wrapped in order to expose the inner fields this wrapping module.
#[derive(Default)]
pub struct Request {
    payload: Option<Bytes>,
    headers: Option<HeaderMap>,
    timeout: Option<Option<Duration>>,
    inbox: Option<String>,
}

impl Request {
    pub fn new() -> Request {
        Default::default()
    }

    /// Sets the payload of the request. If not used, empty payload will be sent.
    ///
    /// # Examples
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let client = si_data_nats::connect("demo.nats.io").await?;
    /// let request = si_data_nats::Request::new().payload("data".into());
    /// client.send_request("service", request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload(mut self, payload: Bytes) -> Request {
        self.payload = Some(payload);
        self
    }

    /// Sets the headers of the requests.
    ///
    /// # Examples
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// use std::str::FromStr;
    /// let client = si_data_nats::connect("demo.nats.io").await?;
    /// let mut headers = si_data_nats::HeaderMap::new();
    /// headers.insert(
    ///     "X-Example",
    ///     si_data_nats::HeaderValue::from_str("Value").unwrap(),
    /// );
    /// let request = si_data_nats::Request::new()
    ///     .headers(headers)
    ///     .payload("data".into());
    /// client.send_request("service", request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn headers(mut self, headers: HeaderMap) -> Request {
        self.headers = Some(headers);
        self
    }

    /// Sets the custom timeout of the request. Overrides default [Client] timeout.
    /// Setting it to [Option::None] disables the timeout entirely which might result in deadlock.
    /// To use default timeout, simply do not call this function.
    ///
    /// # Examples
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let client = si_data_nats::connect("demo.nats.io").await?;
    /// let request = si_data_nats::Request::new()
    ///     .timeout(Some(std::time::Duration::from_secs(15)))
    ///     .payload("data".into());
    /// client.send_request("service", request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn timeout(mut self, timeout: Option<Duration>) -> Request {
        self.timeout = Some(timeout);
        self
    }

    /// Sets custom inbox for this request. Overrides both customized and default [Client] Inbox.
    ///
    /// # Examples
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// use std::str::FromStr;
    /// let client = si_data_nats::connect("demo.nats.io").await?;
    /// let request = si_data_nats::Request::new()
    ///     .inbox("custom_inbox".into())
    ///     .payload("data".into());
    /// client.send_request("service", request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn inbox(mut self, inbox: String) -> Request {
        self.inbox = Some(inbox);
        self
    }
}

impl From<Request> for async_nats::Request {
    fn from(value: Request) -> Self {
        let mut r = Self::new();
        if let Some(payload) = value.payload {
            r = r.payload(payload);
        }
        if let Some(headers) = value.headers {
            r = r.headers(headers);
        }
        if let Some(timeout) = value.timeout {
            r = r.timeout(timeout);
        }
        if let Some(inbox) = value.inbox {
            r = r.inbox(inbox);
        }
        r
    }
}
