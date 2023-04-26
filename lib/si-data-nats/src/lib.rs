#![warn(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn
)]
#![allow(clippy::missing_errors_doc)]

use std::{fmt::Debug, io, sync::Arc, time::Duration};

use crossbeam_channel::RecvError;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    sync::Mutex,
    task::{self, spawn_blocking},
};

pub mod jetstream;
mod message;
mod options;
mod subscription;

pub use message::Message;
pub use nats::{header::HeaderMap, rustls};
pub use options::Options;
pub use subscription::Subscription;

pub type NatsError = Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("async runtime error: {0}")]
    Async(#[from] task::JoinError),
    #[error("nats client error: {0}")]
    Nats(#[from] io::Error),
    #[error("crossbeam select error: {0}")]
    CrossBeamChannel(#[from] RecvError),
    #[error("error serializing object: {0}")]
    Serialize(#[source] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NatsConfig {
    pub url: String,
    pub subject_prefix: Option<String>,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            url: "localhost".to_string(),
            subject_prefix: None,
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
    inner: nats::Connection,
    metadata: Arc<ConnectionMetadata>,
}

impl Client {
    #[instrument(name = "client::new", skip_all, level = "debug")]
    pub async fn new(config: &NatsConfig) -> Result<Self> {
        Self::connect_with_options(
            &config.url,
            config.subject_prefix.clone(),
            Options::default(),
        )
        .await
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
    /// # use si_data_nats::{Client, Options}; tokio_test::block_on(async {
    /// let nc = Client::connect_with_options(
    ///         "demo.nats.io",
    ///         None,
    ///         Options::default(),
    ///     ).await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    ///
    /// In the below case, the second server is configured to use TLS but the first one is not.
    /// Using the `tls_required` method can ensure that all servers are connected to with TLS, if
    /// that is your intention.
    ///
    /// ```no_run
    /// # use si_data_nats::{Client, Options}; tokio_test::block_on(async {
    /// let nc = Client::connect_with_options(
    ///         "nats://demo.nats.io:4222,tls://demo.nats.io:4443",
    ///         None,
    ///         Options::default(),
    ///     )
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
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
            otel.kind = %FormattedSpanKind(SpanKind::Client),
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn connect_with_options(
        nats_url: impl Into<String>,
        subject_prefix: Option<String>,
        options: Options,
    ) -> Result<Self> {
        let nats_url = nats_url.into();

        let mut metadata = ConnectionMetadata {
            messaging_consumer_id: String::with_capacity(0),
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

        let inner = spawn_blocking(move || options.inner.connect(&nats_url))
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;
        debug!("successfully connected to nats");

        metadata.messaging_consumer_id = inner.client_id().to_string();

        span.record_ok();
        Ok(Self {
            inner,
            metadata: Arc::new(metadata),
        })
    }

    /// Create a subscription for the given NATS connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// let sub = nc.subscribe("foo").await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.subscribe",
        skip_all,
        level = "debug",
        fields(
            messaging.consumer_id = %self.metadata.messaging_consumer_id,
            messaging.destination = Empty,
            messaging.destination_kind = "topic",
            messaging.operation = "receive",
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            messaging.subject = Empty,
            net.transport = %self.metadata.net_transport,
            otel.kind = %FormattedSpanKind(SpanKind::Client),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn subscribe(&self, subject: impl Into<String>) -> Result<Subscription> {
        let span = Span::current();

        let subject = subject.into();
        span.record("messaging.destination", subject.as_str());
        span.record("otel.name", format!("{} receive", &subject).as_str());
        let inner = self.inner.clone();
        let sub_subject = subject.clone();
        let sub = spawn_blocking(move || inner.subscribe(&sub_subject))
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        Ok(Subscription::new(
            sub,
            subject,
            self.metadata.clone(),
            current_span_for_debug!(),
        ))
    }

    /// Create a queue subscription for the given NATS connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// let sub = nc.queue_subscribe("foo", "production").await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.queue_subscribe",
        skip_all,
        level = "debug",
        fields(
            messaging.consumer_id = %self.metadata.messaging_consumer_id,
            messaging.destination = Empty,
            messaging.destination_kind = "topic",
            messaging.operation = "receive",
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.subscription.queue = Empty,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %FormattedSpanKind(SpanKind::Client),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn queue_subscribe(
        &self,
        subject: impl Into<String>,
        queue: impl Into<String>,
    ) -> Result<Subscription> {
        let span = Span::current();

        let subject = subject.into();
        let queue = queue.into();
        span.record("messaging.destination", subject.as_str());
        span.record("messaging.subscription.queue", queue.as_str());
        span.record("otel.name", format!("{} receive", &subject).as_str());
        let inner = self.inner.clone();
        let sub_subject = subject.clone();
        let sub = spawn_blocking(move || inner.queue_subscribe(&sub_subject, &queue))
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        Ok(Subscription::new(
            sub,
            subject,
            self.metadata.clone(),
            current_span_for_debug!(),
        ))
    }

    /// Publish a message on the given subject.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// nc.publish("foo", "Hello World!").await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub async fn publish(&self, subject: impl Into<String>, msg: impl Into<Vec<u8>>) -> Result<()> {
        self.publish_with_reply_or_headers(subject, None::<String>, None, msg)
            .await
    }

    /// Publish a message on the given subject with a reply subject for responses.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// let reply = nc.new_inbox();
    /// let rsub = nc.subscribe(&reply).await?;
    /// nc.publish_request("foo", &reply, "Help me!").await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.publish_request",
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
            otel.kind = %FormattedSpanKind(SpanKind::Producer),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn publish_request(
        &self,
        subject: impl Into<String>,
        reply: impl Into<String>,
        msg: impl Into<Vec<u8>>,
    ) -> Result<()> {
        let span = Span::current();

        let subject = subject.into();
        let reply = reply.into();
        let msg = msg.into();
        span.record("messaging.destination", subject.as_str());
        span.record("otel.name", format!("{} send", &subject).as_str());
        let inner = self.inner.clone();
        spawn_blocking(move || inner.publish_request(&subject, &reply, &msg))
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Create a new globally unique inbox which can be used for replies.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// let reply = nc.new_inbox();
    /// let rsub = nc.subscribe(&reply).await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[must_use]
    pub fn new_inbox(&self) -> String {
        self.inner.new_inbox()
    }

    /// Publish a message on the given subject as a request and receive the response.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use futures::{TryStreamExt};
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// # nc.subscribe("foo").await?.try_for_each(|m| async move { m.respond("ans=42").await });
    /// # nc.subscribe("foo").await?;
    /// let resp = nc.request("foo", "Help me?").await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
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
            otel.kind = %FormattedSpanKind(SpanKind::Client),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn request(
        &self,
        subject: impl Into<String>,
        msg: impl Into<Vec<u8>>,
    ) -> Result<Message> {
        let span = Span::current();

        let subject = subject.into();
        let msg = msg.into();
        span.record("messaging.destination", subject.as_str());
        span.record("otel.name", format!("{} send", &subject).as_str());
        let inner = self.inner.clone();
        let msg = spawn_blocking(move || inner.request(&subject, &msg))
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        span.record_ok();
        Ok(Message::new(msg, self.metadata.clone()))
    }

    /// Publish a message on the given subject as a request and receive the response.
    ///
    /// This call will return after the timeout duration if no response is received.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use futures::{TryStreamExt};
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// # nc.subscribe("foo").await?.try_for_each(|m| async move { m.respond("ans=42").await });
    /// # nc.subscribe("foo").await?;
    /// let resp = nc.request_timeout("foo", "Help me?", std::time::Duration::from_secs(2)).await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.request_timeout",
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
            otel.kind = %FormattedSpanKind(SpanKind::Client),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn request_timeout(
        &self,
        subject: impl Into<String>,
        msg: impl Into<Vec<u8>>,
        timeout: Duration,
    ) -> Result<Message> {
        let span = Span::current();

        let subject = subject.into();
        let msg = msg.into();
        span.record("messaging.destination", subject.as_str());
        span.record("otel.name", format!("{} send", &subject).as_str());
        let inner = self.inner.clone();
        let msg = spawn_blocking(move || inner.request_timeout(&subject, &msg, timeout))
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        span.record_ok();
        Ok(Message::new(msg, self.metadata.clone()))
    }

    /// Publish a message on the given subject as a request and allow multiple responses.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use futures::{TryStreamExt, StreamExt};
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// # nc.subscribe("foo").await?.try_for_each(|m| async move { m.respond("ans=42").await });
    /// for msg in nc.request_multi("foo", "Help").await?.take(1).next().await {}
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.request_multi",
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
            otel.kind = %FormattedSpanKind(SpanKind::Client),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn request_multi(
        &self,
        subject: impl Into<String>,
        msg: impl Into<Vec<u8>>,
    ) -> Result<Subscription> {
        let span = Span::current();

        let subject = subject.into();
        let sub_span_subject = subject.clone();
        let msg = msg.into();
        span.record("messaging.destination", subject.as_str());
        span.record("otel.name", format!("{} send", &subject).as_str());
        let inner = self.inner.clone();
        let sub_subject = subject.clone();
        let sub = spawn_blocking(move || inner.request_multi(&sub_subject, &msg))
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        let sub_span = span!(
            Level::INFO,
            "client.request_multi.subscribe",
            messaging.consumer_id = %self.metadata.messaging_consumer_id,
            messaging.destination = %sub_span_subject,
            messaging.destination_kind = "topic",
            messaging.operation = "receive",
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %FormattedSpanKind(SpanKind::Client),
            otel.name = %format!("{} receive", &sub_span_subject),
            otel.status_code = Empty,
            otel.status_message = Empty,
        );
        sub_span.follows_from(&span);

        span.record_ok();
        Ok(Subscription::new(
            sub,
            subject,
            self.metadata.clone(),
            current_span_for_debug!(),
        ))
    }

    /// Flush a NATS connection by sending a `PING` protocol and waiting for the responding `PONG`.
    ///
    /// Will fail with `TimedOut` if the server does not respond with in 10 seconds. Will fail with
    /// `NotConnected` if the server is not currently connected. Will fail with `BrokenPipe` if the
    /// connection to the server is lost.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// nc.flush().await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
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
            otel.kind = %FormattedSpanKind(SpanKind::Client),
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn flush(&self) -> Result<()> {
        let span = Span::current();

        let inner = self.inner.clone();
        spawn_blocking(move || inner.flush())
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Flush a NATS connection by sending a `PING` protocol and waiting for the responding `PONG`.
    ///
    /// Will fail with `TimedOut` if the server takes longer than this duration to respond. Will
    /// fail with `NotConnected` if the server is not currently connected. Will fail with
    /// `BrokenPipe` if the connection to the server is lost.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// nc.flush_timeout(std::time::Duration::from_secs(8)).await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.flush_timeout",
        skip_all,
        level = "debug",
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %FormattedSpanKind(SpanKind::Client),
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn flush_timeout(&self, duration: Duration) -> Result<()> {
        let span = Span::current();

        let inner = self.inner.clone();
        spawn_blocking(move || inner.flush_timeout(duration))
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Close a NATS connection. All clones of this `Connection` will also be closed, as the
    /// backing IO threads are shared.
    ///
    /// If the client is currently connected to a server, the outbound write buffer will be flushed
    /// in the process of shutting down.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// nc.close().await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.close",
        skip_all,
        level = "debug",
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %FormattedSpanKind(SpanKind::Client),
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn close(self) -> Result<()> {
        let span = Span::current();

        let inner = self.inner.clone();
        spawn_blocking(move || inner.close())
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Calculates the round trip time between this client and the server, if the server is
    /// currently connected.
    ///
    /// Fails with `TimedOut` if the server takes more than 10 seconds to respond.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// println!("server rtt: {:?}", nc.rtt().await);
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.rtt",
        skip_all,
        level = "debug",
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %FormattedSpanKind(SpanKind::Client),
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn rtt(&self) -> Result<Duration> {
        let span = Span::current();

        let inner = self.inner.clone();
        let duration = spawn_blocking(move || inner.rtt())
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        span.record_ok();
        Ok(duration)
    }

    /// Returns the client IP as known by the server. Supported as of server version 2.1.6.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// println!("ip: {:?}", nc.client_ip());
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    pub fn client_ip(&self) -> Result<std::net::IpAddr> {
        self.inner.client_ip().map_err(Into::into)
    }

    /// Returns the client ID as known by the most recently connected server.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// println!("ip: {:?}", nc.client_id());
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[must_use]
    pub fn client_id(&self) -> u64 {
        self.inner.client_id()
    }

    /// Send an unsubscription for all subs then flush the connection, allowing any unprocessed
    /// messages to be handled by a handler function if one is configured.
    ///
    /// After the flush returns, we know that a round-trip to the server has happened after it
    /// received our unsubscription, so we shut down the subscriber afterwards.
    ///
    /// A similar method exists for the `Subscription` struct which will drain a single
    /// `Subscription` without shutting down the entire connection afterward.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::{Arc, atomic::{AtomicBool, Ordering::SeqCst}};
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// let received = Arc::new(AtomicBool::new(false));
    /// let received_2 = received.clone();
    ///
    /// nc.subscribe("test.drain").await?;
    ///
    /// nc.publish("test.drain", "message").await?;
    /// nc.drain().await?;
    ///
    /// # std::thread::sleep(std::time::Duration::from_secs(1));
    ///
    /// assert!(received.load(SeqCst));
    ///
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.drain",
        skip_all,
        level = "debug",
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %FormattedSpanKind(SpanKind::Client),
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn drain(&self) -> Result<()> {
        let span = Span::current();

        let inner = self.inner.clone();
        spawn_blocking(move || inner.drain())
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Publish a message which may have a reply subject or headers set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use futures::StreamExt;
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// let mut sub = nc.subscribe("foo.headers").await?;
    /// let headers = [("header1", "value1"),
    ///                ("header2", "value2")].iter().collect();
    /// let reply_to = None::<String>;
    /// nc.publish_with_reply_or_headers(
    ///     "foo.headers",
    ///     reply_to,
    ///     Some(&headers),
    ///     "Hello World!"
    /// ).await?;
    /// nc.flush().await?;
    /// let message = sub.next().await.unwrap()?;
    /// assert_eq!(message.headers().unwrap().len(), 2);
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.publish_with_reply_or_headers",
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
            otel.kind = %FormattedSpanKind(SpanKind::Producer),
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn publish_with_reply_or_headers(
        &self,
        subject: impl Into<String>,
        reply: Option<impl Into<String>>,
        headers: Option<&HeaderMap>,
        msg: impl Into<Vec<u8>>,
    ) -> Result<()> {
        let span = Span::current();

        let subject = subject.into();
        let headers = headers.map(HeaderMap::clone);
        let reply = reply.map(Into::into);
        let msg = msg.into();
        span.record("messaging.destination", subject.as_str());
        span.record("otel.name", format!("{} send", &subject).as_str());
        let inner = self.inner.clone();
        spawn_blocking(move || {
            inner.publish_with_reply_or_headers(&subject, reply.as_deref(), headers.as_ref(), &msg)
        })
        .await
        .map_err(|err| span.record_err(Error::Async(err)))?
        .map_err(|err| span.record_err(Error::Nats(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Returns the maximum payload size the most recently connected server will accept.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data_nats::Options; tokio_test::block_on(async {
    /// # let nc = Options::default().connect("demo.nats.io", None).await?;
    /// println!("max payload: {:?}", nc.max_payload());
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[must_use]
    pub fn max_payload(&self) -> usize {
        self.inner.max_payload()
    }

    /// Gets a reference to the client's metadata.
    pub fn metadata(&self) -> &ConnectionMetadata {
        self.metadata.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct ConnectionMetadata {
    messaging_consumer_id: String,
    messaging_protocol: &'static str,
    messaging_system: &'static str,
    messaging_url: String,
    subject_prefix: Option<String>,
    net_transport: &'static str,
}

impl ConnectionMetadata {
    /// Gets a reference to the connection metadata's messaging consumer id.
    pub fn messaging_consumer_id(&self) -> &str {
        self.messaging_consumer_id.as_ref()
    }

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
    pending_publish: Arc<Mutex<Vec<(String, serde_json::Value)>>>,
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
    pub async fn publish<T>(&self, subject: impl Into<String>, object: &T) -> Result<()>
    where
        T: Serialize + Debug,
    {
        let span = Span::current();
        span.follows_from(&self.tx_span);

        let subject = subject.into();
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
