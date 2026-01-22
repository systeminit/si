use std::time::Duration;

use naxum::fair::{
    FairSchedulingConfig,
    spawn_task_listener,
};
use si_data_nats::jetstream;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::helpers::{
    SERVICE_NAME,
    create_test_streams,
    nats,
    nats_prefix,
};

#[tokio::test]
async fn stress_test_high_volume_single_workspace() {
    // Stress test: 100 messages in rapid succession for a single workspace
    let prefix = nats_prefix();
    let nats_client = nats(prefix.clone()).await;
    let (tasks_stream, requests_stream) = create_test_streams(&prefix, &nats_client).await;

    let config = FairSchedulingConfig::for_workspace_partitioning(
        SERVICE_NAME,
        tasks_stream,
        requests_stream.clone(),
        Some(prefix.clone()),
    );

    let (consumer_tx, mut consumer_rx) = mpsc::channel(10);
    let shutdown = CancellationToken::new();

    let _listener_handle = spawn_task_listener(config, consumer_tx, shutdown.clone());

    let js_context = jetstream::new(nats_client.clone());

    // Publish task notification
    js_context
        .publish(
            format!("{prefix}.{SERVICE_NAME}.tasks.workspace-1"),
            "notification".into(),
        )
        .await
        .expect("failed to publish");

    // Wait for consumer creation
    let ready = tokio::time::timeout(Duration::from_secs(5), consumer_rx.recv())
        .await
        .expect("timeout")
        .expect("consumer channel closed");

    let mut stream = ready.messages;

    // Publish 100 messages as fast as possible
    for i in 0..100 {
        let subject = format!("{prefix}.{SERVICE_NAME}.requests.workspace-1.test");
        js_context
            .publish(subject, format!("message-{i}").into())
            .await
            .expect("failed to publish")
            .await
            .expect("failed to ack publish");
    }

    // Consume all 100 messages
    use futures::StreamExt;
    for i in 0..100 {
        let msg = tokio::time::timeout(Duration::from_secs(10), stream.next())
            .await
            .unwrap_or_else(|_| panic!("timeout waiting for message {i}"))
            .expect("stream ended")
            .expect("stream error");

        msg.ack().await.expect("failed to ack");
    }

    shutdown.cancel();
}

#[tokio::test]
async fn stress_test_many_workspaces_concurrent() {
    // Stress test: 20 workspaces, 10 messages each, sent concurrently
    let prefix = nats_prefix();
    let nats_client = nats(prefix.clone()).await;
    let (tasks_stream, requests_stream) = create_test_streams(&prefix, &nats_client).await;

    let config = FairSchedulingConfig::for_workspace_partitioning(
        SERVICE_NAME,
        tasks_stream,
        requests_stream.clone(),
        Some(prefix.clone()),
    );

    let (consumer_tx, mut consumer_rx) = mpsc::channel(50);
    let shutdown = CancellationToken::new();

    let _listener_handle = spawn_task_listener(config, consumer_tx, shutdown.clone());

    let js_context = jetstream::new(nats_client.clone());

    // Publish task notifications for 20 workspaces
    let workspace_count = 20;
    for i in 0..workspace_count {
        js_context
            .publish(
                format!("{prefix}.{SERVICE_NAME}.tasks.workspace-{i}"),
                "notification".into(),
            )
            .await
            .expect("failed to publish");
    }

    // Collect all consumers
    let mut consumers = Vec::new();
    for _ in 0..workspace_count {
        let ready = tokio::time::timeout(Duration::from_secs(10), consumer_rx.recv())
            .await
            .expect("timeout")
            .expect("consumer channel closed");
        consumers.push(ready);
    }

    // Publish 10 messages to each workspace concurrently
    let mut publish_handles = vec![];
    for i in 0..workspace_count {
        let js = js_context.clone();
        let p = prefix.clone();
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                let subject = format!("{p}.{SERVICE_NAME}.requests.workspace-{i}.test");
                js.publish(subject, format!("ws-{i}-msg-{j}").into())
                    .await
                    .expect("failed to publish")
                    .await
                    .expect("failed to ack publish");
            }
        });
        publish_handles.push(handle);
    }

    // Wait for all publishes to complete
    for handle in publish_handles {
        handle.await.expect("publish task failed");
    }

    // Consume all messages from all workspaces concurrently
    use futures::StreamExt;
    let mut consume_handles = vec![];
    for ready in consumers {
        let mut stream = ready.messages;
        let handle = tokio::spawn(async move {
            for _ in 0..10 {
                let msg = tokio::time::timeout(Duration::from_secs(10), stream.next())
                    .await
                    .expect("timeout")
                    .expect("stream ended")
                    .expect("stream error");
                msg.ack().await.expect("failed to ack");
            }
        });
        consume_handles.push(handle);
    }

    // Wait for all consumers to finish
    for handle in consume_handles {
        handle.await.expect("consume task failed");
    }

    shutdown.cancel();
}
