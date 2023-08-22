use dal::DalContext;
use dal_test::random_identifier_string;
use si_rabbitmq::{Producer, StreamManager};
use si_test_macros::gobbler_test as test;

/// Recommended to run with the following environment variable:
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=none
/// ```
#[test]
async fn produce(_ctx: &DalContext) {
    let stream = &random_identifier_string();
    let manager = StreamManager::new().await.expect("could not connect");
    manager
        .create_stream(stream)
        .await
        .expect("could not create stream");

    let mut producer = Producer::new(&manager, "producer", stream)
        .await
        .expect("could not create producer");
    producer
        .send_single("foo")
        .await
        .expect("could not singe message");
    producer
        .send_batch(vec!["bar".as_bytes(), "baz".as_bytes()])
        .await
        .expect("could not send message batch");
}
