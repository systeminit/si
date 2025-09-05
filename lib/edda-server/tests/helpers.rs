use std::{
    collections::HashMap,
    env,
};

use edda_core::api_types::{
    Container,
    RequestId,
    rebuild_request::{
        RebuildRequest,
        RebuildRequestVCurrent,
    },
};
use futures::TryStreamExt as _;
use nats_std::subject;
use si_data_nats::{
    NatsClient,
    NatsConfig,
    Subject,
    async_nats::jetstream::{
        self,
        consumer::push,
    },
    jetstream::Context,
};
use uuid::Uuid;

const STREAM_NAME: &str = "TEST_COMPRESSING_STREAM";
const SUBJECT_PREFIX: &str = "test.compressing_stream";

fn nats_config(subject_prefix: String) -> NatsConfig {
    let mut config = NatsConfig::default();
    #[allow(clippy::disallowed_methods)] // Used only in tests & so prefixed with `SI_TEST_`
    if let Ok(value) = env::var("SI_TEST_NATS_URL") {
        config.url = value;
    }
    config.subject_prefix = Some(subject_prefix);
    config
}

pub async fn nats(subject_prefix: String) -> NatsClient {
    NatsClient::new(&nats_config(subject_prefix))
        .await
        .expect("failed to connect to NATS")
}

pub fn nats_prefix() -> String {
    Uuid::new_v4().as_simple().to_string()
}

pub fn context(nats: NatsClient) -> Context {
    si_data_nats::jetstream::new(nats)
}

pub async fn stream(context: &Context) -> jetstream::stream::Stream {
    context
        .get_or_create_stream(jetstream::stream::Config {
            name: STREAM_NAME.to_string(),
            description: Some("CompressingStream integration tests".to_string()),
            subjects: vec![
                subject::prefixed(Some(&format!("{SUBJECT_PREFIX}.*")), ">").to_string(),
            ],
            allow_direct: true,
            retention: jetstream::stream::RetentionPolicy::Limits,
            storage: jetstream::stream::StorageType::Memory,
            ..Default::default()
        })
        .await
        .expect("failed to get or create jetstream stream")
}

pub fn pub_sub_subject<'a>(prefix: impl Into<Option<&'a str>>, subject_suffix: &'a str) -> Subject {
    let prefix = prefix.into().map(|p| format!("{SUBJECT_PREFIX}.{p}"));
    subject::prefixed(prefix.as_deref(), subject_suffix)
}

pub async fn message_count_on_subject(
    stream: &jetstream::stream::Stream,
    subject: &Subject,
) -> usize {
    let info: HashMap<_, _> = stream
        .info_with_subjects(subject.as_str())
        .await
        .expect("failed to get stream info")
        .try_collect()
        .await
        .expect("failed to collect stream info");

    info.get(subject.as_str()).copied().unwrap_or(0)
}

pub async fn incoming_messages(
    nats: &NatsClient,
    stream: &jetstream::stream::Stream,
    test_name: &str,
) -> push::Ordered {
    stream
        .create_consumer(push::OrderedConfig {
            deliver_subject: nats.new_inbox(),
            filter_subject: pub_sub_subject(nats.metadata().subject_prefix(), test_name)
                .to_string(),
            ..Default::default()
        })
        .await
        .expect("failed to create consumer")
        .messages()
        .await
        .expect("failed to subscribe")
}

pub fn rebuild_request() -> RebuildRequest {
    RebuildRequest::new(RebuildRequestVCurrent {
        id: RequestId::new(),
    })
}
