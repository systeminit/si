use std::env;

use futures::TryStreamExt;
use si_data_nats::{async_nats::jetstream, HeaderMap, NatsClient, NatsConfig};
use si_layer_cache::chunking_nats::ChunkingNats;
use ulid::Ulid;

const ENV_VAR_NATS_URL: &str = "SI_TEST_NATS_URL";

#[tokio::test]
async fn poop() {
    let prefix = Ulid::new().to_string();

    let mut nats_config = NatsConfig {
        subject_prefix: Some(prefix.clone()),
        ..Default::default()
    };
    #[allow(clippy::disallowed_methods)] // Environment variables are used exclusively in test
    if let Ok(value) = env::var(ENV_VAR_NATS_URL) {
        nats_config.url = value;
    }

    let nats = NatsClient::new(&nats_config)
        .await
        .expect("failed to connect to nats");
    let context = jetstream::new(nats.as_inner().clone());

    let stream = context
        .get_or_create_stream(jetstream::stream::Config {
            name: nats_stream_name(Some(prefix.as_ref()), "SI_LAYER"),
            subjects: vec![format!("{prefix}.si.test.si_layer.>")],
            retention: jetstream::stream::RetentionPolicy::Interest,
            storage: jetstream::stream::StorageType::Memory,
            ..Default::default()
        })
        .await
        .expect("failed to create stream");

    let consumer = stream
        .get_or_create_consumer(
            "test",
            jetstream::consumer::pull::Config {
                name: Some("test".to_owned()),
                max_ack_pending: 200,
                ..Default::default()
            },
        )
        .await
        .expect("failed to create consumer");

    let chunking_nats = ChunkingNats::new(context.clone());

    let mut messages = ChunkingNats::chunking_messages(
        consumer
            .messages()
            .await
            .expect("failed to get messages stream"),
    );

    let mut headers = HeaderMap::new();
    headers.insert("MARCO", "polo");

    let payload_size = 1024 * 1024 * 10;
    let payload = vec![b'A'; payload_size];
    let checksum = blake3::hash(&payload);

    let pub_payload = payload.clone();
    tokio::spawn(async move {
        chunking_nats
            .publish_with_headers(
                format!("{prefix}.si.test.si_layer.blob"),
                headers,
                pub_payload.into(),
            )
            .await
            .expect("failed to publish message");
    });

    let msg = messages.try_next().await.expect("failed to get message");
    assert!(msg.is_some());

    let msg = msg.unwrap();
    assert_eq!(checksum, blake3::hash(&msg.payload));
    assert_eq!(payload_size, msg.payload.len());
    assert_eq!(payload, msg.payload);
    assert_eq!(
        "polo",
        msg.headers
            .as_ref()
            .expect("no headers")
            .get("MARCO")
            .expect("failed to get header")
            .to_string()
    );
}

fn nats_stream_name(prefix: Option<&str>, suffix: impl AsRef<str>) -> String {
    let suffix = suffix.as_ref();

    match prefix {
        Some(prefix) => format!("{prefix}_{suffix}"),
        None => suffix.to_owned(),
    }
}
