use std::env;

use si_data_nats::{
    NatsClient,
    NatsConfig,
    async_nats::jetstream::stream::{
        Config as StreamConfig,
        Stream,
    },
    jetstream,
};
use uuid::Uuid;

pub const SERVICE_NAME: &str = "test-service";

pub fn nats_config(subject_prefix: String) -> NatsConfig {
    let mut config = NatsConfig::default();
    #[allow(clippy::disallowed_methods)]
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
    #[allow(clippy::disallowed_methods)]
    let test_name = env::var("NEXTEST_TEST_NAME").unwrap_or_else(|_| {
        std::thread::current()
            .name()
            .unwrap_or("unknown")
            .to_string()
    });
    format!("test-{}-{}", Uuid::new_v4(), test_name)
}

pub async fn create_test_streams(prefix: &str, nats_client: &NatsClient) -> (Stream, Stream) {
    let js_context = jetstream::new(nats_client.clone());

    let tasks_stream_name = format!("{prefix}-tasks-test");
    let requests_stream_name = format!("{prefix}-requests-test");

    let tasks_stream = js_context
        .get_or_create_stream(StreamConfig {
            name: tasks_stream_name.clone(),
            subjects: vec![format!("{}.{}.tasks.*", prefix, SERVICE_NAME)],
            ..Default::default()
        })
        .await
        .expect("failed to create tasks stream");

    let requests_stream = js_context
        .get_or_create_stream(StreamConfig {
            name: requests_stream_name.clone(),
            subjects: vec![format!("{}.{}.requests.>", prefix, SERVICE_NAME)],
            ..Default::default()
        })
        .await
        .expect("failed to create requests stream");

    (tasks_stream, requests_stream)
}
