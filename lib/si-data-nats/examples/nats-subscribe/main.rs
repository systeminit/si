use std::env;

use futures::TryStreamExt;
use si_data_nats::{Message, NatsClient, NatsConfig, Subscription};
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

    let mut subscription = nats.subscribe(&subject).await?;

    let mut count = 0;
    while let Some(message) = subscription.try_next().await? {
        count += 1;

        process_message(message, count, &subscription);

        if let Some(max) = max_read {
            if count >= max {
                debug!("hit max read, closing subscription and ending");
                subscription.close().await?;
                break;
            }
        }
    }
    info!("subscription stream completed");

    Ok(())
}

#[instrument(
    skip_all,
    level = "debug",
    fields(
        messaging.destination = sub.metadata().messaging_destination(),
        messaging.destination_kind = sub.metadata().messaging_destination_kind(),
        messaging.operation = sub.metadata().messaging_operation(),
        messaging.protocol = sub.metadata().messaging_protocol(),
        messaging.system = sub.metadata().messaging_system(),
        messaging.url = sub.metadata().messaging_url(),
        messaging.subject = sub.metadata().messaging_subject(),
        net.transport = sub.metadata().net_transport(),
        otel.kind = sub.metadata().process_otel_kind(),
        otel.name = sub.metadata().process_otel_name(),
        otel.status_code = Empty,
        otel.status_message = Empty,
    )
)]
fn process_message(message: Message, count: u32, sub: &Subscription) {
    Span::current().follows_from(sub.span());

    let data = String::from_utf8_lossy(message.data());
    info!(message = ?message, data = data.as_ref(), count);
}
