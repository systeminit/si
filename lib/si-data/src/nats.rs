use std::{fmt::Debug, io, sync::Arc, time::Duration};

use nats_client as nats;
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
pub use nats::{rustls, Headers};
pub use options::Options;
pub use subscription::Subscription;

#[derive(Debug, Error)]
pub enum Error {
    #[error("async runtime error")]
    Async(#[from] task::JoinError),
    #[error("nats client error")]
    Nats(#[from] io::Error),
    #[error("error serializing object")]
    Serialize(#[source] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NatsConfig {
    pub url: String,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            url: "localhost".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Client {
    inner: nats::Connection,
    metadata: Arc<ConnectionMetadata>,
}

impl Client {
    #[instrument(name = "client::new", skip_all)]
    pub async fn new(config: &NatsConfig) -> Result<Self> {
        Self::connect_with_options(&config.url, Options::default()).await
    }

    #[instrument(
        name = "client.transaction",
        skip_all,
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.transaction = Empty,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub fn transaction(&self) -> NatsTxn {
        NatsTxn::new(self.clone(), self.metadata.clone(), Span::current())
    }

    /// Establish a `Connection` with a NATS server.
    ///
    /// Multiple servers may be specified by separating them with commas.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data::nats; tokio_test::block_on(async {
    /// let nc = nats::Client::connect_with_options(
    ///         "demo.nats.io",
    ///         nats::Options::default(),
    ///     ).await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    ///
    /// In the below case, the second server is configured to use TLS but the first one is not.
    /// Using the `tls_required` method can ensure that all servers are connected to with TLS, if
    /// that is your intention.
    ///
    /// ```no_run
    /// # use si_data::nats; tokio_test::block_on(async {
    /// let nc = nats::Client::connect_with_options(
    ///         "nats://demo.nats.io:4222,tls://demo.nats.io:4443",
    ///         nats::Options::default(),
    ///     )
    ///     .await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client::connect_with_options",
        skip_all,
        fields(
            messaging.protocol = Empty,
            messaging.system = Empty,
            messaging.url = Empty,
            net.transport = Empty,
            otel.kind = %SpanKind::Client,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn connect_with_options(
        nats_url: impl Into<String>,
        options: Options,
    ) -> Result<Self> {
        let nats_url = nats_url.into();

        let mut metadata = ConnectionMetadata {
            messaging_consumer_id: String::with_capacity(0),
            messaging_protocol: "nats",
            messaging_system: "nats",
            messaging_url: nats_url.clone(),
            net_transport: "ip_tcp",
        };

        let span = Span::current();
        span.record("messaging.protocol", &metadata.messaging_protocol);
        span.record("messaging.system", &metadata.messaging_system);
        span.record("messaging.url", &metadata.messaging_url.as_str());
        span.record("net.transport", &metadata.net_transport);

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
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
    /// let sub = nc.subscribe("foo").await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.subscribe",
        skip_all,
        fields(
            messaging.consumer_id = %self.metadata.messaging_consumer_id,
            messaging.destination = Empty,
            messaging.destination_kind = "topic",
            messaging.operation = "receive",
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %SpanKind::Consumer,
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn subscribe(&self, subject: impl Into<String>) -> Result<Subscription> {
        let span = Span::current();

        let subject = subject.into();
        span.record("messaging.destination", &subject.as_str());
        span.record("otel.name", &format!("{} process", &subject).as_str());
        let inner = self.inner.clone();
        let sub = spawn_blocking(move || inner.subscribe(&subject))
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        Ok(Subscription::new(sub, self.metadata.clone(), span))
    }

    /// Create a queue subscription for the given NATS connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
    /// let sub = nc.queue_subscribe("foo", "production").await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.queue_subscribe",
        skip_all,
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
            otel.kind = %SpanKind::Consumer,
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
        span.record("messaging.destination", &subject.as_str());
        span.record("messaging.subscription.queue", &queue.as_str());
        span.record("otel.name", &format!("{} process", &subject).as_str());
        let inner = self.inner.clone();
        let sub = spawn_blocking(move || inner.queue_subscribe(&subject, &queue))
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        Ok(Subscription::new(sub, self.metadata.clone(), span))
    }

    /// Publish a message on the given subject.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
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
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
    /// let reply = nc.new_inbox();
    /// let rsub = nc.subscribe(&reply).await?;
    /// nc.publish_request("foo", &reply, "Help me!").await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.publish_request",
        skip_all,
        fields(
            messaging.destination = Empty,
            messaging.destination_kind = "topic",
            messaging.operation = "send",
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %SpanKind::Producer,
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
        span.record("messaging.destination", &subject.as_str());
        span.record("otel.name", &format!("{} send", &subject).as_str());
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
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
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
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
    /// # nc.subscribe("foo").await?.try_for_each(|m| async move { m.respond("ans=42").await });
    /// # nc.subscribe("foo").await?;
    /// let resp = nc.request("foo", "Help me?").await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.request",
        skip_all,
        fields(
            messaging.destination = Empty,
            messaging.destination_kind = "topic",
            messaging.operation = "send",
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %SpanKind::Client,
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
        span.record("messaging.destination", &subject.as_str());
        span.record("otel.name", &format!("{} send", &subject).as_str());
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
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
    /// # nc.subscribe("foo").await?.try_for_each(|m| async move { m.respond("ans=42").await });
    /// # nc.subscribe("foo").await?;
    /// let resp = nc.request_timeout("foo", "Help me?", std::time::Duration::from_secs(2)).await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.request_timeout",
        skip_all,
        fields(
            messaging.destination = Empty,
            messaging.destination_kind = "topic",
            messaging.operation = "send",
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %SpanKind::Client,
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
        span.record("messaging.destination", &subject.as_str());
        span.record("otel.name", &format!("{} send", &subject).as_str());
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
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
    /// # nc.subscribe("foo").await?.try_for_each(|m| async move { m.respond("ans=42").await });
    /// for msg in nc.request_multi("foo", "Help").await?.take(1).next().await {}
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.request_multi",
        skip_all,
        fields(
            messaging.destination = Empty,
            messaging.destination_kind = "topic",
            messaging.operation = "send",
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %SpanKind::Client,
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
        span.record("messaging.destination", &subject.as_str());
        span.record("otel.name", &format!("{} send", &subject).as_str());
        let inner = self.inner.clone();
        let sub = spawn_blocking(move || inner.request_multi(&subject, &msg))
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
            otel.kind = %SpanKind::Consumer,
            otel.name = %format!("{} receive", &sub_span_subject),
            otel.status_code = Empty,
            otel.status_message = Empty,
        );
        sub_span.follows_from(&span);

        span.record_ok();
        Ok(Subscription::new(sub, self.metadata.clone(), sub_span))
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
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
    /// nc.flush().await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.flush",
        skip_all,
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %SpanKind::Client,
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
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
    /// nc.flush_timeout(std::time::Duration::from_secs(8)).await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.flush_timeout",
        skip_all,
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %SpanKind::Client,
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
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
    /// nc.close().await?;
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.close",
        skip_all,
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %SpanKind::Client,
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
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
    /// println!("server rtt: {:?}", nc.rtt().await);
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[instrument(
        name = "client.rtt",
        skip_all,
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %SpanKind::Client,
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
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
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
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
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
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
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
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %SpanKind::Client,
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
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
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
        fields(
            messaging.destination = Empty,
            messaging.destination_kind = "topic",
            messaging.operation = "send",
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
            otel.kind = %SpanKind::Producer,
            otel.name = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn publish_with_reply_or_headers(
        &self,
        subject: impl Into<String>,
        reply: Option<impl Into<String>>,
        headers: Option<&Headers>,
        msg: impl Into<Vec<u8>>,
    ) -> Result<()> {
        let span = Span::current();

        let subject = subject.into();
        let headers = headers.map(Headers::clone);
        let reply = reply.map(Into::into);
        let msg = msg.into();
        span.record("messaging.destination", &subject.as_str());
        span.record("otel.name", &format!("{} send", &subject).as_str());
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
    /// # use si_data::nats; tokio_test::block_on(async {
    /// # let nc = nats::Options::default().connect("demo.nats.io").await?;
    /// println!("max payload: {:?}", nc.max_payload());
    /// # Ok::<(), Box<dyn std::error::Error + 'static>>(()) });
    /// ```
    #[must_use]
    pub fn max_payload(&self) -> usize {
        self.inner.max_payload()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ConnectionMetadata {
    messaging_consumer_id: String,
    messaging_protocol: &'static str,
    messaging_system: &'static str,
    messaging_url: String,
    net_transport: &'static str,
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
        name = "transaction.commit",
        skip_all,
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn commit(self) -> Result<()> {
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
        self.tx_span.record("messaging.transaction", &"commit");
        span.record_ok();
        Ok(())
    }

    #[instrument(
        name = "transaction.rollback",
        skip_all,
        fields(
            messaging.protocol = %self.metadata.messaging_protocol,
            messaging.system = %self.metadata.messaging_system,
            messaging.url = %self.metadata.messaging_url,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn rollback(self) -> Result<()> {
        let span = Span::current();
        span.follows_from(&self.tx_span);

        // Nothing much to do, we want to drop the pending publishes which happens when this
        // function returns (i.e. it consumes `self`).

        self.tx_span.record_ok();
        self.tx_span.record("messaging.transaction", &"rollback");
        span.record_ok();
        Ok(())
    }
}
