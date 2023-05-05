use std::{fmt, sync::Arc};

use telemetry::prelude::*;
use tokio::task::spawn_blocking;

use super::{
    jetstream::{AckKind, JetStreamMessageInfo},
    ConnectionMetadata, Error, HeaderMap, Result,
};

#[derive(Clone)]
pub struct Message {
    inner: nats::Message,
    metadata: Arc<ConnectionMetadata>,
}

impl Message {
    pub(crate) fn new(inner: nats::Message, metadata: Arc<ConnectionMetadata>) -> Self {
        Self { inner, metadata }
    }

    /// Gets a reference to the subject of this message.
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.inner.subject
    }

    /// Gets a reference to the reply of this message.
    #[must_use]
    pub fn reply(&self) -> Option<&str> {
        self.inner.reply.as_deref()
    }

    /// Gets a reference to the message contents.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.inner.data
    }

    /// Gets a reference to the headers of this message.
    #[must_use]
    pub fn headers(&self) -> Option<&HeaderMap> {
        self.inner.headers.as_ref()
    }

    /// Consumes the message and returns the inner data.
    #[must_use]
    pub fn into_data(self) -> Vec<u8> {
        self.inner.data
    }

    /// Consumes the message and returns the inner data and reply subject.
    #[must_use]
    pub fn into_parts(self) -> (Vec<u8>, Option<String>) {
        (self.inner.data, self.inner.reply)
    }

    /// Respond to a request message.
    #[instrument(
        name = "message.respond",
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
    pub async fn respond(&self, msg: impl Into<Vec<u8>>) -> Result<()> {
        let span = Span::current();

        let msg = msg.into();
        if let Some(reply) = self.reply() {
            span.record("messaging.destination", reply);
            span.record("otel.name", format!("{} send", &reply).as_str());
        }
        let inner = self.inner.clone();
        spawn_blocking(move || inner.respond(&msg))
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Acknowledge a `JetStream` message with a default acknowledgement.
    ///
    /// See `AckKind` documentation for details of what other types of acks are available. If you
    /// need to send a non-default ack, use the `ack_kind` method below. If you need to block until
    /// the server acks your ack, use the `double_ack` method instead.
    ///
    /// Returns immediately if this message has already been double-acked.
    #[instrument(
        name = "message.ack",
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
    pub async fn ack(&self) -> Result<()> {
        let span = Span::current();

        if let Some(reply) = self.reply() {
            span.record("messaging.destination", reply);
            span.record("otel.name", format!("{} send", &reply).as_str());
        }
        let inner = self.inner.clone();
        spawn_blocking(move || inner.ack())
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Acknowledge a `JetStream` message.
    ///
    /// See `AckKind` documentation for details of what each variant means. If you need to block
    /// until the server acks your ack, use the `double_ack` method instead.
    ///
    /// Does not check whether this message has already been double-acked.
    #[instrument(
        name = "message.ack_kind",
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
    pub async fn ack_kind(&self, ack_kind: AckKind) -> Result<()> {
        let span = Span::current();

        if let Some(reply) = self.reply() {
            span.record("messaging.destination", reply);
            span.record("otel.name", format!("{} send", &reply).as_str());
        }
        let inner = self.inner.clone();
        spawn_blocking(move || inner.ack_kind(ack_kind))
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Acknowledge a `JetStream` message and wait for acknowledgement from the server that it has
    /// received our ack.
    ///
    /// Retry acknowledgement until we receive a response. See `AckKind` documentation for details
    /// of what each variant means.
    ///
    /// Returns immediately if this message has already been double-acked.
    #[instrument(
        name = "message.double_ack",
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
    pub async fn double_ack(&self, ack_kind: AckKind) -> Result<()> {
        let span = Span::current();

        if let Some(reply) = self.reply() {
            span.record("messaging.destination", reply);
            span.record("otel.name", format!("{} send", &reply).as_str());
        }
        let inner = self.inner.clone();
        spawn_blocking(move || inner.double_ack(ack_kind))
            .await
            .map_err(|err| span.record_err(Error::Async(err)))?
            .map_err(|err| span.record_err(Error::Nats(err)))?;

        span.record_ok();
        Ok(())
    }

    /// Returns the `JetStream` message ID if this is a `JetStream` message.
    ///
    /// Returns `None` if this is not a `JetStream` message with headers set.
    #[must_use]
    pub fn jetstream_message_info(&self) -> Option<JetStreamMessageInfo<'_>> {
        self.inner.jetstream_message_info()
    }
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
