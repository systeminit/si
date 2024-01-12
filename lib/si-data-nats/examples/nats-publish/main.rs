use std::env;

use si_data_nats::{NatsClient, NatsConfig};
use telemetry::prelude::*;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter, Registry,
};

const TRACING_LOG_ENV_VAR: &str = "SI_LOG";
const DEFAULT_TRACING_DIRECTIVES: &str = "nats_publish=trace,si_data=trace,info";

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
        .expect("usage: nats-publish SUBJECT BODY");
    let msg = env::args()
        .nth(2)
        .expect("usage: nats-publish SUBJECT BODY");
    let config = NatsConfig::default();
    let nats = NatsClient::new(&config).await?;

    nats.publish(subject.clone(), msg.clone().into()).await?;
    info!(
        msg = msg.as_str(),
        subject = subject.as_str(),
        "published message on subject"
    );

    Ok(())
}
