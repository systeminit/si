use std::{
    env,
    error,
    time::Duration,
};

use shuttle_server::{
    FINAL_MESSAGE_HEADER_KEY,
    Shuttle,
};
use si_data_nats::{
    NatsClient,
    NatsConfig,
    Subject,
    async_nats::jetstream::stream::Config,
    jetstream,
    jetstream::Context,
};
use si_events::ulid::Ulid;
use telemetry::prelude::*;
use telemetry_nats::propagation;
use tokio_util::task::TaskTracker;

const MESSAGE_COUNT: u64 = 100;

async fn setup_nats() -> std::result::Result<(NatsClient, Context), Box<dyn error::Error>> {
    let mut config = NatsConfig::default();

    #[allow(clippy::disallowed_methods)]
    if let Ok(url) = env::var("NATS_URL") {
        config.url = url;
    } else if let Ok(url) = env::var("SI_TEST_NATS_URL") {
        config.url = url;
    } else {
        config.url = "nats://localhost:4222".to_owned();
    }

    let client = NatsClient::new(&config).await?;
    let context = jetstream::new(client.clone());

    Ok((client, context))
}

#[tokio::test]
async fn integration() -> std::result::Result<(), Box<dyn error::Error>> {
    let (client, context) = setup_nats().await?;

    // Get a new set of streams for every test execution.
    let prefix = Ulid::new();

    // Create both streams.
    let (source_stream, destination_stream) = {
        let source_subject = Subject::from(format!("{prefix}.shuttle.test.source.>"));
        let source_stream_name = format!("SHUTTLE_TEST_SOURCE_{prefix}");
        let destination_subject = Subject::from(format!("{prefix}.shuttle.test.destination.>"));
        let destination_stream_name = format!("SHUTTLE_TEST_DESTINATION_{prefix}");

        let source_stream = context
            .get_or_create_stream(Config {
                name: source_stream_name.to_string(),
                subjects: vec![source_subject.to_string()],
                ..Default::default()
            })
            .await?;
        let destination_stream = context
            .get_or_create_stream(Config {
                name: destination_stream_name.to_string(),
                subjects: vec![destination_subject.to_string()],
                ..Default::default()
            })
            .await?;

        (source_stream, destination_stream)
    };

    // Spawn the shuttle instance using a tracker.
    let tracker = TaskTracker::new();
    let source_stream_clone = source_stream.clone();
    tracker.spawn(async move {
        match Shuttle::new(
            client,
            source_stream_clone,
            Subject::from(format!("{prefix}.shuttle.test.source.some.inner.*")),
            Subject::from(format!(
                "{prefix}.shuttle.test.destination.some.inner.messages"
            )),
        )
        .await
        {
            Ok(shuttle) => {
                if let Err(err) = shuttle.try_run().await {
                    error!(?err, "error running shuttle instance");
                }
            }
            Err(err) => {
                error!(?err, "error creating shuttle instance");
            }
        }
    });

    // Publish messages on the source stream to ensure that shuttle works.
    {
        let data_setup_subject =
            Subject::from(format!("{prefix}.shuttle.test.source.some.inner.messages"));

        // Publish many messages to be shuttled.
        for index in 0..MESSAGE_COUNT {
            let ack = context
                .publish_with_headers(
                    data_setup_subject.to_owned(),
                    propagation::empty_injected_headers(),
                    index.to_string().into(),
                )
                .await?;
            ack.await?;
        }

        // Publish the final message.
        let mut headers = propagation::empty_injected_headers();
        headers.insert(FINAL_MESSAGE_HEADER_KEY, "");
        let ack = context
            .publish_with_headers(data_setup_subject, headers, serde_json::to_vec("")?.into())
            .await?;
        ack.await?;
    }

    // Close the tracker and wait for all tasks to close.
    tracker.close();
    tokio::time::timeout(Duration::from_secs(5), tracker.wait()).await?;

    // Now that everything has shut down, confirm that shuttle did its job.
    assert_eq!(0, source_stream.get_info().await?.state.messages);
    assert_eq!(
        MESSAGE_COUNT,
        destination_stream.get_info().await?.state.messages
    );

    Ok(())
}
