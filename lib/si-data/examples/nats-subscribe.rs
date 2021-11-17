use futures::{StreamExt, TryStreamExt};
use si_data::{NatsClient, NatsConfig};
use std::env;
use telemetry::prelude::*;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter, Registry,
};

#[tokio::main]
async fn main() -> Result<(), Box<(dyn std::error::Error + 'static)>> {
    Registry::default()
        .with(
            EnvFilter::try_from_env("SI_LOG")
                .unwrap_or_else(|_| EnvFilter::new("debug,si_data=trace")),
        )
        .with(
            fmt::layer()
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_span_events(FmtSpan::FULL),
        )
        .try_init()?;

    let subject = env::args().skip(1).next().unwrap();
    let config = NatsConfig::default();
    let nats = NatsClient::new(&config).await?;

    let mut subscription = nats.subscribe(&subject).await?;

    while let Some(msg) = subscription.async_next().await? {
        info!(message = ?msg, "new message");
    }
    info!("ALL DONE");

    Ok(())
}
