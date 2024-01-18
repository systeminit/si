use std::env;

use futures::StreamExt;
use si_data_nats::{Message, NatsClient, NatsConfig, Subscriber};
use telemetry::prelude::*;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter, Registry,
};

const TRACING_LOG_ENV_VAR: &str = "SI_LOG";
const DEFAULT_TRACING_DIRECTIVES: &str = "nats_subscribe=trace,si_data=trace,info";

#[tokio::main]
async fn main() -> Result<(), Box<(dyn std::error::Error + 'static)>> {
    Registry::default()
        .with(
            EnvFilter::try_from_env(TRACING_LOG_ENV_VAR)
                .unwrap_or_else(|_| EnvFilter::new(DEFAULT_TRACING_DIRECTIVES)),
        )
        .with(
            fmt::layer()
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE),
        )
        .try_init()?;

    run().await
}

#[instrument(name = "main", skip_all)]
async fn run() -> Result<(), Box<(dyn std::error::Error + 'static)>> {
    let subject = env::args()
        .nth(1)
        .expect("usage: nats-subscribe SUBJECT [MAX_READ]");
    let max_read = env::args()
        .nth(2)
        .map(|i| i.parse::<u32>().expect("MAX_READ must be a positive int"));
    if let Some(max) = max_read {
        info!(
            "reading maximum of {} messages on subject '{}'",
            max, &subject
        );
    }
    let config = NatsConfig::default();
    let nats = NatsClient::new(&config).await?;

    let mut subscriber = nats.subscribe(subject).await?;

    let mut count = 0;
    while let Some(message) = subscriber.next().await {
        count += 1;

        process_message(message, count, &subscriber);

        if let Some(max) = max_read {
            if count >= max {
                debug!("hit max read, closing subscriber and ending");
                subscriber.unsubscribe_after(0).await?;
                break;
            }
        }
    }
    info!("subscriber stream completed");

    Ok(())
}

#[instrument(
    skip_all,
    level = "debug",
    fields(
        messaging.client_id = sub.metadata().messaging_client_id(),
        messaging.destination.name = sub.metadata().messaging_destination_name(),
        messaging.message.body.size = message.payload().len(),
        messaging.nats.server.id = sub.metadata().messaging_nats_server_id(),
        messaging.nats.server.name = sub.metadata().messaging_nats_server_name(),
        messaging.nats.server.version = sub.metadata().messaging_nats_server_version(),
        messaging.operation = MessagingOperation::Receive.as_str(),
        messaging.system = sub.metadata().messaging_system(),
        messaging.url = sub.metadata().messaging_url(),
        network.peer.address = sub.metadata().network_peer_address(),
        network.protocol.name = sub.metadata().network_protocol_name(),
        network.protocol.version = sub.metadata().network_protocol_version(),
        network.transport = sub.metadata().network_transport(),
        otel.kind = SpanKind::Consumer.as_str(), // similar to an RPC operation
        otel.name = Empty,
        otel.status_code = Empty,
        otel.status_message = Empty,
        server.address = sub.metadata().server_address(),
        server.port = sub.metadata().server_port(),
    )
)]
fn process_message(message: Message, count: u32, sub: &Subscriber) {
    let span = Span::current();
    span.follows_from(sub.span());

    span.record(
        "otel.name",
        format!(
            "{} {}",
            message.subject(),
            MessagingOperation::Receive.as_str()
        )
        .as_str(),
    );

    let data = String::from_utf8_lossy(message.payload());
    info!(message = ?message, data = data.as_ref(), count);
}
