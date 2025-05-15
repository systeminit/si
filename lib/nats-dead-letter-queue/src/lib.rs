use si_data_nats::{
    Bytes,
    HeaderMap,
    async_nats::jetstream::{
        context::{
            CreateStreamError,
            PublishAckFuture,
            PublishError,
        },
        stream::{
            Config,
            RetentionPolicy,
        },
    },
    jetstream::Context,
    subject::ToSubject,
};
use thiserror::Error;

const STREAM_NAME: &str = "DEAD_LETTER_QUEUES";
const STREAM_DESCRIPTION: &str = "Dead Letter Queues";
const STREAM_SUBJECTS: &[&str] = &[
    // Subscribe to *all* stream and consumer max deliveries events. This subject is of the form:
    // `$JS.EVENT.ADVISORY.CONSUMER.MAX_DELIVERIES.<STREAM>.<CONSUMER>`
    //
    // See: https://docs.nats.io/running-a-nats-service/nats_admin/monitoring/monitoring_jetstream
    "$JS.EVENT.ADVISORY.CONSUMER.MAX_DELIVERIES.*.*",
    // Create a service-namespaced subject space for services to place full message copies if they
    // need to clear their own streams while retaining these messages for later forensics.
    //
    // The expected subject prefix is of the form: `dlq.<SERVICE_NAME>...`, for example:
    // `dlq.rebaser.requests.$wk_id.$cs_id`
    "dlq.*.>",
];
const SUBJECT_PREFIX_DLQ: &str = "dlq";

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum Error {
    #[error("create stream error: {0}")]
    CreateStream(#[from] CreateStreamError),
    #[error("jetstream publish error: {0}")]
    Publish(#[from] PublishError),
}

pub type NatsDeadLetterQueueError = Error;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug)]
pub struct DeadLetterQueue {
    context: Context,
}

impl DeadLetterQueue {
    /// Ensures that the "dead letter queues" stream is created.
    pub async fn create_stream(context: Context) -> Result<Self> {
        let prefix = context.metadata().subject_prefix();

        let subjects: Vec<_> = STREAM_SUBJECTS
            .iter()
            .map(|subject| prefixed_subject(prefix, subject))
            .collect();

        context
            .get_or_create_stream(Config {
                name: prefixed_stream_name(prefix, STREAM_NAME),
                description: Some(STREAM_DESCRIPTION.to_string()),
                retention: RetentionPolicy::Limits,
                subjects,
                ..Default::default()
            })
            .await?;

        Ok(Self { context })
    }

    // Publishes a message with headers to a given subject suffix which will be appropriately
    // prefixed for a dead letter queue subject.
    pub async fn publish_with_headers<S: ToSubject>(
        &self,
        subject_suffix: S,
        headers: HeaderMap,
        payload: Bytes,
    ) -> Result<PublishAckFuture> {
        let subject_prefix = self.context.metadata().subject_prefix();

        let subject_suffix = subject_suffix.to_subject();
        let subject_suffix_str = subject_suffix.as_str();

        // Strip any subject prefix off the subject as we're going to prepend a DLQ subject part
        // and then re-prefix the subject is required
        let stripped_subject_suffix_str = match subject_prefix {
            Some(subject_prefix) => match subject_suffix_str.strip_prefix(subject_prefix) {
                Some(stripped_subject) => stripped_subject,
                None => subject_suffix_str,
            },
            None => subject_suffix_str,
        };

        let dlq_subject = prefixed_subject(
            subject_prefix,
            &format!("{SUBJECT_PREFIX_DLQ}.{stripped_subject_suffix_str}"),
        );

        self.context
            .publish_with_headers(dlq_subject, headers, payload)
            .await
            .map_err(Into::into)
    }
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
