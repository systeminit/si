use si_data_nats::{
    async_nats::jetstream::{
        context::CreateStreamError,
        stream::{Config, RetentionPolicy},
    },
    jetstream::Context,
};
use thiserror::Error;

const STREAM_NAME: &str = "DEAD_LETTER_QUEUES";
const STREAM_DESCRIPTION: &str = "Dead Letter Queues";
// Subscribe to *all* stream and consumer max deliveries events. This subject is of the form:
// `$JS.EVENT.ADVISORY.CONSUMER.MAX_DELIVERIES.<STREAM>.<CONSUMER>`
//
// See: https://docs.nats.io/running-a-nats-service/nats_admin/monitoring/monitoring_jetstream
const STREAM_SUBJECTS: &str = "$JS.EVENT.ADVISORY.CONSUMER.MAX_DELIVERIES.*.*";

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum Error {
    #[error("create stream error: {0}")]
    CreateStream(#[from] CreateStreamError),
}

pub type NatsDeadLetterQueueError = Error;

type Result<T, E = Error> = std::result::Result<T, E>;

/// Ensures that the "dead letter queue" stream is created
pub async fn create_stream(context: &Context) -> Result<()> {
    let prefix = context.metadata().subject_prefix();

    context
        .get_or_create_stream(Config {
            name: prefixed_stream_name(prefix, STREAM_NAME),
            description: Some(STREAM_DESCRIPTION.to_string()),
            retention: RetentionPolicy::Limits,
            subjects: vec![prefixed_subject(prefix, STREAM_SUBJECTS)],
            ..Default::default()
        })
        .await?;

    Ok(())
}

fn prefixed_stream_name(prefix: Option<&str>, stream_name: &str) -> String {
    match prefix {
        Some(prefix) => format!("{prefix}_{stream_name}"),
        None => stream_name.to_owned(),
    }
}

fn prefixed_subject(prefix: Option<&str>, subject: &str) -> String {
    match prefix {
        Some(prefix) => format!("{prefix}.{subject}"),
        None => subject.to_owned(),
    }
}
