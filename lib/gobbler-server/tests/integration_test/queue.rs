use dal::DalContext;
use pretty_assertions_sorted::assert_eq;
use si_rabbitmq::{Consumer, Environment, Producer};
use si_test_macros::gobbler_test as test;

#[test]
async fn produce(_ctx: &DalContext) {
    let stream = "test-produce";
    let environment = Environment::new().await.expect("could not connect");

    // FIXME(nick): add stream setup to test macro.
    environment
        .delete_stream(stream)
        .await
        .expect("could not delete stream");
    environment
        .create_stream(stream)
        .await
        .expect("could not create stream");

    let mut producer = Producer::new(&environment, "producer", stream)
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
    producer.close().await.expect("could not close producer");
}

#[test]
async fn consume(_ctx: &DalContext) {
    let stream = "test-consume";
    let environment = Environment::new().await.expect("could not connect");

    // FIXME(nick): add stream setup to test macro.
    environment
        .delete_stream(stream)
        .await
        .expect("could not delete stream");
    environment
        .create_stream(stream)
        .await
        .expect("could not create stream");

    let mut producer = Producer::new(&environment, "producer", stream)
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
    producer.close().await.expect("could not close producer");

    let mut consumer = Consumer::new(&environment, stream)
        .await
        .expect("could not create consumer");
    let handle = consumer.handle();

    // Grab the three deliveries that we expect.
    let delivery = consumer
        .next()
        .await
        .expect("could not consume next delivery")
        .expect("no delivery to consume")
        .expect("consumer delivery error");
    let data = consumer
        .process_delivery(&delivery)
        .expect("could not process delivery")
        .expect("no data in message");
    assert_eq!("foo", &data);
    let delivery = consumer
        .next()
        .await
        .expect("could not consume next delivery")
        .expect("no delivery to consume")
        .expect("consumer delivery error");
    let data = consumer
        .process_delivery(&delivery)
        .expect("could not process delivery")
        .expect("no data in message");
    assert_eq!("bar", &data);
    let delivery = consumer
        .next()
        .await
        .expect("could not consume next delivery")
        .expect("no delivery to consume")
        .expect("consumer delivery error");
    let data = consumer
        .process_delivery(&delivery)
        .expect("could not process delivery")
        .expect("no data in message");
    assert_eq!("baz", &data);

    handle
        .close()
        .await
        .expect("could not close the consumer associated to this hangler");
}
