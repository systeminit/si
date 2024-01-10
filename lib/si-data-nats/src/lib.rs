#![warn(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn
)]
#![allow(clippy::missing_errors_doc)]

use std::{fmt::Debug, io, sync::Arc};

use async_nats::subject::ToSubject;
use crossbeam_channel::RecvError;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::Mutex;

mod connect_options;
mod message;
mod subscriber;

pub use async_nats::{header::HeaderMap, rustls, Subject};
pub use connect_options::ConnectOptions;
pub use message::Message;
pub use subscriber::Subscriber;

pub type NatsError = Error;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum Error {
    #[error("crossbeam select error: {0}")]
    CrossBeamChannel(#[from] RecvError),
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
        if span_enabled!(target: "si_data_nats", Level::DEBUG) {
            Span::current()
        } else {
            Span::none()
        }
    }
}

pub type NatsClient = Client;

#[derive(Clone, Debug)]
pub struct Client {
    inner: async_nats::Client,
    metadata: Arc<ConnectionMetadata>,
}

impl Client {
    #[instrument(name = "client::new", skip_all, level = "debug")]
    pub async fn new(config: &NatsConfig) -> Result<Self> {
        let mut options = ConnectOptions::default();

        if let Some(creds) = &config.creds {
            options = options.credentials(creds)?;
        }
        if let Some(creds_file) = &config.creds_file {
            options = options.credentials_file(creds_file.into()).await?;
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

    #[instrument(
        name = "client.transaction",
        skip_all,
        level = "debug",
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.transaction = Empty,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
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
            messaging.protocol = Empty,
            messaging.system = Empty,
            messaging.url = Empty,
            net.transport = Empty,
            otel.kind = SpanKind::Client.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn connect_with_options(
        nats_url: impl Into<String>,
        subject_prefix: Option<String>,
        options: ConnectOptions,
    ) -> Result<Self> {
        let nats_url = nats_url.into();

        let metadata = ConnectionMetadata {
            messaging_protocol: "nats",
            messaging_system: "nats",
            messaging_url: nats_url.clone(),
            net_transport: "ip_tcp",
            subject_prefix,
        };

        let span = Span::current();
        span.record("messaging.protocol", metadata.messaging_protocol);
        span.record("messaging.system", metadata.messaging_system);
        span.record("messaging.url", metadata.messaging_url.as_str());
        span.record("net.transport", metadata.net_transport);

        let inner = options
            .inner
            .connect(&nats_url)
            .await
            .map_err(|err| span.record_err(Error::NatsConnect(err)))?;
        debug!("successfully connected to nats");

        span.record_ok();
        Ok(Self {
            inner,
            metadata: Arc::new(metadata),
        })
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
            messaging.destination = Empty,
            messaging.destination_kind = "topic",
            messaging.operation = "receive",
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            messaging.subject = Empty,
            net.transport = %self.metadata.net_transport,
            otel.kind = SpanKind::Client.as_str(),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn subscribe(&self, subject: impl ToSubject) -> Result<Subscriber> {
        let span = Span::current();

        let subject = subject.to_subject();
        span.record("messaging.destination", subject.as_str());
        span.record("otel.name", format!("{} receive", &subject).as_str());
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
            messaging.destination = Empty,
            messaging.destination_kind = "topic",
            messaging.operation = "receive",
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.subscriber.queue = Empty,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = SpanKind::Client.as_str(),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn queue_subscribe(
        &self,
        subject: impl ToSubject,
        queue: impl Into<String>,
    ) -> Result<Subscriber> {
        let span = Span::current();

        let subject = subject.to_subject();
        let queue = queue.into();
        span.record("messaging.destination", subject.as_str());
        span.record("messaging.subscriber.queue", queue.as_str());
        span.record("otel.name", format!("{} receive", &subject).as_str());
        let sub_subject = subject.clone();
        let sub = self
            .inner
            .queue_subscribe(sub_subject, queue)
            .await
            .map_err(|err| span.record_err(Error::NatsSubscribe(err)))?;

        Ok(Subscriber::new(
            sub,
            &subject,
            self.metadata.clone(),
            current_span_for_debug!(),
        ))
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
    pub async fn publish(&self, subject: impl ToSubject, msg: impl Into<Vec<u8>>) -> Result<()> {
        let span = Span::current();

        let subject = subject.to_subject();
        let msg = msg.into();
        span.record("messaging.destination", subject.as_str());
        span.record("otel.name", format!("{} send", &subject).as_str());
        self.inner
            .publish(subject, msg.into())
            .await
            .map_err(|err| span.record_err(Error::NatsPublish(err)))?;

        span.record_ok();
        Ok(())
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

    /// Sends the request with headers.
    ///
    /// # Examples
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), si_data_nats::Error> {
    /// let client = si_data_nats::Client::connect_with_options("demo.nats.io", None, Default::default()).await?;
    /// let response = client.request("service".into(), "data".into()).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "client.request",
        skip_all,
        level = "debug",
        fields(
            messaging.destination = Empty,
            messaging.destination_kind = "topic",
            messaging.operation = "send",
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = SpanKind::Client.as_str(),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn request(
        &self,
        subject: impl ToSubject,
        msg: impl Into<Vec<u8>>,
    ) -> Result<Message> {
        let span = Span::current();

        let subject = subject.to_subject();
        let msg = msg.into();
        span.record("messaging.destination", subject.as_str());
        span.record("otel.name", format!("{} send", &subject).as_str());
        let msg = self
            .inner
            .request(subject, msg.into())
            .await
            .map_err(|err| span.record_err(Error::NatsRequest(err)))?;

        span.record_ok();
        Ok(Message::new(msg, self.metadata.clone()))
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
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = SpanKind::Client.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
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

    /// Publish a [Message] to a given subject, with specified response subject
    /// to which the subscriber can respond.
    /// This method does not await for the response.
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
            messaging.destination = Empty,
            messaging.destination_kind = "topic",
            messaging.operation = "send",
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = SpanKind::Producer.as_str(),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn publish_with_reply(
        &self,
        subject: impl ToSubject,
        reply: impl ToSubject,
        msg: impl Into<Vec<u8>>,
    ) -> Result<()> {
        let span = Span::current();

        let subject = subject.to_subject();
        let msg = msg.into();
        span.record("messaging.destination", subject.as_str());
        span.record("otel.name", format!("{} send", &subject).as_str());
        self.inner
            .publish_with_reply(subject, reply, msg.into())
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
            messaging.destination = Empty,
            messaging.destination_kind = "topic",
            messaging.operation = "send",
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = SpanKind::Producer.as_str(),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn publish_with_headers(
        &self,
        subject: impl ToSubject,
        headers: HeaderMap,
        msg: impl Into<Vec<u8>>,
    ) -> Result<()> {
        let span = Span::current();

        let subject = subject.to_subject();
        let msg = msg.into();
        span.record("messaging.destination", subject.as_str());
        span.record("otel.name", format!("{} send", &subject).as_str());
        self.inner
            .publish_with_headers(subject, headers, msg.into())
            .await
            .map_err(|err| span.record_err(Error::NatsPublish(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Gets a reference to the client's metadata.
    pub fn metadata(&self) -> &ConnectionMetadata {
        self.metadata.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct ConnectionMetadata {
    messaging_protocol: &'static str,
    messaging_system: &'static str,
    messaging_url: String,
    subject_prefix: Option<String>,
    net_transport: &'static str,
}

impl ConnectionMetadata {
    /// Gets a reference to the connection metadata's messaging protocol.
    pub fn messaging_protocol(&self) -> &str {
        self.messaging_protocol
    }

    /// Gets a reference to the connection metadata's messaging system.
    pub fn messaging_system(&self) -> &str {
        self.messaging_system
    }

    /// Gets a reference to the connection metadata's messaging url.
    pub fn messaging_url(&self) -> &str {
        self.messaging_url.as_ref()
    }

    /// Gets a reference to the connection metadata's net transport.
    pub fn net_transport(&self) -> &str {
        self.net_transport
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
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn publish<T>(&self, subject: impl ToSubject, object: &T) -> Result<()>
    where
        T: Serialize + Debug,
    {
        let span = Span::current();
        span.follows_from(&self.tx_span);

        let subject = subject.to_subject();
        let json: serde_json::Value = serde_json::to_value(object)
            .map_err(|err| span.record_err(self.tx_span.record_err(Error::Serialize(err))))?;
        let mut pending_publish = self.pending_publish.lock().await;
        pending_publish.push((subject, json));

        Ok(())
    }

    #[instrument(
        name = "transaction.commit_into_conn",
        skip_all,
        level = "debug",
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
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
                .publish(subject, msg)
                .await
                .map_err(|err| span.record_err(self.tx_span.record_err(err)))?;
        }

        self.tx_span.record_ok();
        self.tx_span.record("messaging.transaction", "commit");
        span.record_ok();

        Ok(self.client)
    }

    #[instrument(
        name = "transaction.commit",
        skip_all,
        level = "debug",
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn commit(self) -> Result<()> {
        let _ = self.commit_into_conn().await?;
        Ok(())
    }

    #[instrument(
        name = "transaction.rollback",
        skip_all,
        level = "debug",
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn rollback_into_conn(self) -> Result<Client> {
        let span = Span::current();
        span.follows_from(&self.tx_span);

        // Nothing much to do, we want to drop the pending publishes which happens when this
        // function returns (i.e. it consumes `self`).

        self.tx_span.record_ok();
        self.tx_span.record("messaging.transaction", "rollback");
        span.record_ok();

        Ok(self.client)
    }

    #[instrument(
        name = "transaction.rollback",
        skip_all,
        level = "debug",
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
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
