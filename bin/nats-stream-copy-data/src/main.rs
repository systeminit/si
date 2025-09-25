use std::error;

use futures::TryStreamExt;
use si_data_nats::{
    NatsClient,
    NatsConfig,
    Subject,
    async_nats::jetstream::consumer::push,
    jetstream,
};
use tracing::{
    Level,
    debug,
    field,
    info,
    span,
    trace,
};
use tracing_subscriber::{
    EnvFilter,
    Registry,
    fmt::{
        self,
        format::FmtSpan,
    },
    layer::SubscriberExt as _,
    util::SubscriberInitExt as _,
};

mod args;

const BIN_NAME: &str = env!("CARGO_BIN_NAME");

const TRACING_LOG_ENV_VAR: &str = "SI_LOG";
const DEFAULT_TRACING_DIRECTIVES: &str = "nats_stream_copy_data=debug,si_data_nats=warn,info";

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error + Send + Sync>> {
    Registry::default()
        .with(
            EnvFilter::try_from_env(TRACING_LOG_ENV_VAR)
                .unwrap_or_else(|_| EnvFilter::new(DEFAULT_TRACING_DIRECTIVES)),
        )
        .with(
            fmt::layer()
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_span_events(FmtSpan::CLOSE)
                .pretty(),
        )
        .try_init()?;

    let args = args::parse();
    debug!(arguments =?args, "parsed cli arguments");

    let source_nats_config = NatsConfig {
        connection_name: Some(format!("{BIN_NAME}-source")),
        url: args.source_nats_url(),
        creds_file: args.source_nats_creds(),
        ..Default::default()
    };

    let destination_nats_config = NatsConfig {
        connection_name: Some(format!("{BIN_NAME}-destination")),
        url: args.destination_nats_url(),
        creds_file: args.destination_nats_creds(),
        ..Default::default()
    };

    stream_copy_data(
        &source_nats_config,
        args.source_stream.as_str(),
        args.subject,
        &destination_nats_config,
        args.destination_stream.as_str(),
    )
    .await
}

async fn stream_copy_data(
    source_config: &NatsConfig,
    source_stream_name: &str,
    filter_subjects: Vec<Subject>,
    destination_config: &NatsConfig,
    destination_stream_name: &str,
) -> Result<(), Box<dyn error::Error + Send + Sync>> {
    let source_client = NatsClient::new(source_config).await?;
    let source_ctx = jetstream::new(source_client.clone());
    let source_stream = source_ctx.get_stream(source_stream_name).await?;

    let mut messages = source_stream
        .create_consumer(push::OrderedConfig {
            deliver_subject: source_client.new_inbox(),
            filter_subjects: filter_subjects.into_iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        })
        .await?
        .messages()
        .await?;

    let destination_client = NatsClient::new(destination_config).await?;
    let destination_ctx = jetstream::new(destination_client);
    let _destination_stream = destination_ctx.get_stream(destination_stream_name).await?;

    let mut num_copied = 0;

    let span = span!(Level::INFO, "copy_data", num_copied = field::Empty);
    let _enter = span.enter();

    while let Some(message) = messages.try_next().await? {
        let info = message.info()?;
        let pending = info.pending;
        trace!(pending = pending);

        let (message, _) = message.split();

        destination_ctx
            .publish(message.subject, message.payload)
            .await?;

        num_copied += 1;

        if pending == 0 {
            info!("no more pending messages; copy complete");
            break;
        }
    }

    span.record("num_copied", num_copied);

    Ok(())
}
