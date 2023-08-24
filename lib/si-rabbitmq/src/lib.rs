//! This library provides the ability to [connect](Environment) to [RabbitMQ](https://rabbitmq.com)
//! nodes, [produce](Producer) stream messages, and [consume](Consumer) stream messages.

#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::missing_panics_doc
)]

mod consumer;
mod delivery;
mod environment;
mod error;
mod producer;

pub use consumer::Consumer;
pub use consumer::ConsumerHandle;
pub use consumer::ConsumerOffsetSpecification;
pub use delivery::Delivery;
pub use environment::Environment;
pub use error::RabbitError;
pub use error::RabbitResult;
pub use producer::Producer;

#[cfg(test)]
mod tests {
    use super::*;
    use rabbitmq_stream_client::types::OffsetSpecification;
    use tokio::test;

    #[test]
    async fn round_trip() {
        let environment = Environment::new()
            .await
            .expect("could not create environment");

        let stream = "test-stream";

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

        let mut consumer = Consumer::new(&environment, stream, OffsetSpecification::Next)
            .await
            .expect("could not create consumer");

        let message = "starfield";
        producer
            .send_single(message, None)
            .await
            .expect("could not send message");

        let delivery = consumer
            .next()
            .await
            .expect("could not consume")
            .expect("empty delivery");
        let found_contents: String =
            serde_json::from_value(delivery.message_contents.expect("message contents empty"))
                .expect("could not deserialize");

        assert_eq!(message, &found_contents);

        producer.close().await.expect("could not close producer");
        let handle = consumer.handle();
        handle.close().await.expect("could not close consumer");
        environment
            .delete_stream(stream)
            .await
            .expect("could not delete stream");
    }
}
