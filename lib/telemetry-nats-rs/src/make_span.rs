use si_data_nats::{header, Message, Subject};
use telemetry::prelude::*;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::{headers::CORRELATION_ID, propagation::extract_opentelemetry_context};

/// Generates [`Span`]s from incoming NATS messages on a subscription.
#[derive(Clone, Debug)]
pub struct NatsMakeSpan {
    level: Level,
}

impl Default for NatsMakeSpan {
    #[inline]
    fn default() -> Self {
        Self { level: Level::INFO }
    }
}

impl NatsMakeSpan {
    /// Creates a new `NatsMakeSpan`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`Level`] used for the tracing [`Span`].
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Generate a [`Span`] from a message.
    pub fn make_span(&mut self, message: &Message, sub_subject: &Subject) -> Span {
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

        let metadata = message.metadata();

        // This ugly macro is needed, unfortunately, because `tracing::span!` required the level
        // argument to be static. Meaning we can't just pass `self.level` and a dynamic name.
        macro_rules! inner {
            ($level:expr, $name:expr) => {
                ::telemetry::tracing::span!(
                    $level,
                    $name,

                    // Messaging attributes
                    //
                    // See: https://opentelemetry.io/docs/specs/semconv/messaging/messaging-spans/#messaging-attributes

                    messaging.client_id = metadata.messaging_client_id(),
                    messaging.destination.name = sub_subject.as_str(),
                    messaging.message.body.size = message.payload().len(),
                    messaging.message.conversation_id = Empty,
                    messaging.message.envelope.size = message.length(),
                    messaging.nats.server.id = metadata.messaging_nats_server_id(),
                    messaging.nats.server.name = metadata.messaging_nats_server_name(),
                    messaging.nats.server.version = metadata.messaging_nats_server_version(),
                    messaging.operation = MessagingOperation::Receive.as_str(),
                    messaging.system = metadata.messaging_system(),
                    messaging.url = metadata.messaging_url(),
                    network.peer.address = metadata.network_peer_address(),
                    network.protocol.name = metadata.network_protocol_name(),
                    network.protocol.version = metadata.network_protocol_version(),
                    network.transport = metadata.network_transport(),
                    server.address = metadata.server_address(),
                    server.port = metadata.server_port(),

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
                )
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
            format!("{} {}", sub_subject, MessagingOperation::Receive.as_str()),
        );
        if let Some(headers) = message.headers() {
            if let Some(message_id) = headers.get(header::NATS_MESSAGE_ID) {
                span.record("messaging.message.id", message_id.as_str());
            }

            if let Some(correlation_id) = headers.get(CORRELATION_ID.as_ref()) {
                span.record("messaging.message.correlation.id", correlation_id.as_str());
            }

            // Extract OpenTelemetry parent span metadata from the request headers (if it exists) and
            // associate it with this request span
            span.set_parent(extract_opentelemetry_context(headers));
        }

        span
    }
}
