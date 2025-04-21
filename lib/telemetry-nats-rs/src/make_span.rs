use std::sync::Arc;

use naxum::{
    Extensions, HeadRef, Message, MessageHead, extract::MatchedSubject, middleware::trace::MakeSpan,
};
use si_data_nats::{ConnectionMetadata, header};
use telemetry::prelude::*;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::{headers::CORRELATION_ID, propagation::extract_opentelemetry_context};

/// Marker type which informs [`NatsMakeSpan`] to skip OpenTelemetry propagation header extraction.
#[derive(Clone, Debug)]
pub struct ParentSpan(Span);

impl ParentSpan {
    /// Creates a new ParentSpan with the given [`Span`].
    pub fn new(span: Span) -> Self {
        Self(span)
    }

    /// Consumes into the inner [`Span`].
    pub fn into_inner(self) -> Span {
        self.0
    }

    /// Returns a reference to the [`Span`].
    pub fn as_span(&self) -> &Span {
        &self.0
    }
}

/// Generates [`Span`]s from incoming NATS messages on a subscription.
#[derive(Clone, Debug)]
pub struct NatsMakeSpan {
    level: Level,
    metadata: Arc<ConnectionMetadata>,
}

pub struct NatsMakeSpanBuilder {
    level: Level,
    connection_metadata: Arc<ConnectionMetadata>,
}

impl NatsMakeSpanBuilder {
    /// Sets the [`Level`] used for the tracing [`Span`].
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Builds and returns a new [`NatsMakeSpan`].
    pub fn build(self) -> NatsMakeSpan {
        NatsMakeSpan {
            level: self.level,
            metadata: self.connection_metadata,
        }
    }
}

impl NatsMakeSpan {
    /// Creates a new `NatsMakeSpan` builder.
    pub fn builder(connection_metadata: Arc<ConnectionMetadata>) -> NatsMakeSpanBuilder {
        NatsMakeSpanBuilder {
            level: Level::INFO,
            connection_metadata,
        }
    }

    /// Generate a [`Span`] from a message.
    //
    // TODO(fnichol): Method is `pub` exclusively for `lib/nats-subscriber`. If and when that crate
    // can be retired, this should be deleted
    pub fn span_from_core_message(&mut self, message: &si_data_nats::Message) -> Span {
        let extensions = Extensions::new();
        let head_ref = HeadRef {
            subject: message.subject(),
            reply: message.reply(),
            headers: message.headers(),
            status: message.status(),
            description: message.description(),
            length: message.length(),
            payload_length: message.payload().len(),
            extensions: &extensions,
        };

        self.span_from_head(head_ref)
    }

    fn span_from_head(&mut self, head: HeadRef<'_>) -> Span {
        enum InnerLevel {
            Error,
            Warn,
            Info,
            Debug,
            Trace,
        }
        impl From<Level> for InnerLevel {
            fn from(value: Level) -> Self {
                match value {
                    Level::ERROR => InnerLevel::Error,
                    Level::WARN => InnerLevel::Warn,
                    Level::INFO => InnerLevel::Info,
                    Level::DEBUG => InnerLevel::Debug,
                    _ => InnerLevel::Trace,
                }
            }
        }

        let parent_span = head.extensions.get::<ParentSpan>().map(|s| s.as_span());
        let matched_subject = head
            .extensions
            .get::<MatchedSubject>()
            .map(|ms| ms.as_str());

        // This ugly macro is needed, unfortunately, because `tracing::span!` required the level
        // argument to be static. Meaning we can't just pass `self.level` and a dynamic name.
        macro_rules! inner {
            ($level:expr_2021, $name:expr_2021) => {
                match parent_span {
                    Some(parent_span) => {
                        ::telemetry::tracing::span!(
                            parent: parent_span,
                            $level,
                            $name,

                            // Messaging attributes
                            //
                            // See: https://opentelemetry.io/docs/specs/semconv/messaging/messaging-spans/#messaging-attributes

                            messaging.client_id = self.metadata.messaging_client_id(),
                            messaging.destination.name = head.subject.as_str(),
                            messaging.message.body.size = head.payload_length,
                            messaging.message.conversation_id = Empty,
                            messaging.message.correlation.id = Empty,
                            messaging.message.envelope.size = head.length,
                            messaging.message.id = Empty,
                            messaging.nats.message.status = Empty,
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
                            server.address = self.metadata.server_address(),
                            server.port = self.metadata.server_port(),

                            // Set special `otel.*` fields which tracing-opentelemetry will use when
                            // transmitting traces via OpenTelemetry protocol
                            //
                            // See:
                            // https://docs.rs/tracing-opentelemetry/0.22.0/tracing_opentelemetry/#special-fields
                            //

                            otel.kind = SpanKind::Consumer.as_str(),
                            otel.name = Empty,
                            // Default for OpenTelemetry status is `Unset` which should map to an empty/unset
                            // tracing value.
                            //
                            // See: https://docs.rs/opentelemetry/0.21.0/opentelemetry/trace/enum.Status.html
                            otel.status_code = Empty,
                            // Only set if status_code == Error
                            otel.status_message = Empty,

                            // System Initiative attributes

                            si.change_set.id = Empty,
                            si.workspace.id = Empty,
                        )
                    }
                    None => {
                        ::telemetry::tracing::span!(
                            $level,
                            $name,

                            // Messaging attributes
                            //
                            // See: https://opentelemetry.io/docs/specs/semconv/messaging/messaging-spans/#messaging-attributes

                            messaging.client_id = self.metadata.messaging_client_id(),
                            messaging.destination.name = head.subject.as_str(),
                            messaging.message.body.size = head.payload_length,
                            messaging.message.conversation_id = Empty,
                            messaging.message.correlation.id = Empty,
                            messaging.message.envelope.size = head.length,
                            messaging.message.id = Empty,
                            messaging.nats.message.status = Empty,
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
                            server.address = self.metadata.server_address(),
                            server.port = self.metadata.server_port(),

                            // Set special `otel.*` fields which tracing-opentelemetry will use when
                            // transmitting traces via OpenTelemetry protocol
                            //
                            // See:
                            // https://docs.rs/tracing-opentelemetry/0.22.0/tracing_opentelemetry/#special-fields
                            //

                            otel.kind = SpanKind::Consumer.as_str(),
                            otel.name = Empty,
                            // Default for OpenTelemetry status is `Unset` which should map to an empty/unset
                            // tracing value.
                            //
                            // See: https://docs.rs/opentelemetry/0.21.0/opentelemetry/trace/enum.Status.html
                            otel.status_code = Empty,
                            // Only set if status_code == Error
                            otel.status_message = Empty,

                            // System Initiative attributes

                            si.change_set.id = Empty,
                            si.workspace.id = Empty,
                        )
                    }
                }
            };
        }

        let span = match InnerLevel::from(self.level) {
            InnerLevel::Error => inner!(Level::ERROR, MessagingOperation::RECEIVE_STR),
            InnerLevel::Warn => inner!(Level::ERROR, MessagingOperation::RECEIVE_STR),
            InnerLevel::Info => inner!(Level::ERROR, MessagingOperation::RECEIVE_STR),
            InnerLevel::Debug => inner!(Level::ERROR, MessagingOperation::RECEIVE_STR),
            InnerLevel::Trace => inner!(Level::ERROR, MessagingOperation::RECEIVE_STR),
        };

        span.record(
            "otel.name",
            match matched_subject {
                Some(matched_subject) => {
                    format!("{matched_subject} {}", MessagingOperation::Receive.as_str())
                }
                None => format!(
                    "{} {}",
                    head.subject.as_str(),
                    MessagingOperation::Receive.as_str()
                ),
            },
        );

        if let Some(headers) = head.headers {
            if let Some(message_id) = headers.get(header::NATS_MESSAGE_ID) {
                span.record("messaging.message.id", message_id.as_str());
            }

            if let Some(correlation_id) = headers.get(CORRELATION_ID.as_ref()) {
                span.record("messaging.message.correlation.id", correlation_id.as_str());
            }

            if parent_span.is_none() {
                // Extract OpenTelemetry parent span metadata from the message headers (if it
                // exists) and associate it with this request span
                span.set_parent(extract_opentelemetry_context(headers));
            }
        }

        span
    }
}

impl<R> MakeSpan<R> for NatsMakeSpan
where
    R: MessageHead,
{
    fn make_span(&mut self, req: &Message<R>) -> Span {
        self.span_from_head(req.head())
    }
}
