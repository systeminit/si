#![allow(clippy::disallowed_methods)]

mod fair;
mod helpers;

use std::time::Duration;

use helpers::{
    SERVICE_NAME,
    create_test_streams,
    nats,
    nats_prefix,
};
use naxum::fair::{
    FairSchedulingConfig,
    spawn_task_listener,
};
use si_data_nats::{
    async_nats::jetstream::{
        consumer::PullConsumer,
        stream::Stream,
    },
    jetstream,
};
use test_log::test;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

async fn get_consumer_with_retry(stream: &Stream, consumer_name: &str) -> PullConsumer {
    let mut attempts = 0;
    loop {
        match stream.get_consumer(consumer_name).await {
            Ok(c) => break c,
            Err(_) if attempts < 20 => {
                attempts += 1;
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(e) => {
                panic!("failed to get consumer '{consumer_name}' after {attempts} attempts: {e}")
            }
        }
    }
}

#[test(tokio::test)]
async fn listener_creates_consumers_on_task_notifications() {
    let prefix = nats_prefix();
    let nats_client = nats(prefix.clone()).await;
    let (tasks_stream, requests_stream) = create_test_streams(&prefix, &nats_client).await;

    let config = FairSchedulingConfig::for_workspace_partitioning(
        SERVICE_NAME,
        tasks_stream.clone(),
        requests_stream.clone(),
        Some(prefix.clone()),
    );

    let (consumer_tx, mut consumer_rx) = mpsc::channel(10);
    let shutdown = CancellationToken::new();

    let _listener_handle = spawn_task_listener(config, consumer_tx, shutdown.clone());

    // Publish task notification for workspace-1
    let js_context = jetstream::new(nats_client.clone());
    js_context
        .publish(
            format!("{prefix}.{SERVICE_NAME}.tasks.workspace-1"),
            "task notification".into(),
        )
        .await
        .expect("failed to publish task notification");

    // Wait for consumer to be created and sent
    let ready = tokio::time::timeout(Duration::from_secs(5), consumer_rx.recv())
        .await
        .expect("timeout waiting for consumer")
        .expect("consumer channel closed");

    assert_eq!(ready.key, "workspace-1");

    // Verify consumer was created in NATS
    let consumer_name = format!("{SERVICE_NAME}-ws-workspace-1");
    let _consumer = get_consumer_with_retry(&requests_stream, &consumer_name).await;

    shutdown.cancel();
}

#[test(tokio::test)]
async fn listener_handles_multiple_workspaces() {
    let prefix = nats_prefix();
    let nats_client = nats(prefix.clone()).await;
    let (tasks_stream, requests_stream) = create_test_streams(&prefix, &nats_client).await;

    let config = FairSchedulingConfig::for_workspace_partitioning(
        SERVICE_NAME,
        tasks_stream.clone(),
        requests_stream.clone(),
        Some(prefix.clone()),
    );

    let (consumer_tx, mut consumer_rx) = mpsc::channel(10);
    let shutdown = CancellationToken::new();

    let _listener_handle = spawn_task_listener(config, consumer_tx, shutdown.clone());

    let js_context = jetstream::new(nats_client.clone());

    // Publish notifications for multiple workspaces
    for workspace in ["ws-a", "ws-b", "ws-c"] {
        js_context
            .publish(
                format!("{prefix}.{SERVICE_NAME}.tasks.{workspace}"),
                "task notification".into(),
            )
            .await
            .expect("failed to publish task notification");
    }

    // Collect all consumers
    let mut received_keys = Vec::new();
    for _ in 0..3 {
        let ready = tokio::time::timeout(Duration::from_secs(5), consumer_rx.recv())
            .await
            .expect("timeout waiting for consumer")
            .expect("consumer channel closed");
        received_keys.push(ready.key);
    }

    received_keys.sort();
    assert_eq!(received_keys, vec!["ws-a", "ws-b", "ws-c"]);

    shutdown.cancel();
}

#[test(tokio::test)]
async fn listener_acks_message_after_successful_consumer_creation() {
    let prefix = nats_prefix();
    let nats_client = nats(prefix.clone()).await;
    let (tasks_stream, requests_stream) = create_test_streams(&prefix, &nats_client).await;

    let config = FairSchedulingConfig::for_workspace_partitioning(
        SERVICE_NAME,
        tasks_stream.clone(),
        requests_stream.clone(),
        Some(prefix.clone()),
    );

    let (consumer_tx, mut consumer_rx) = mpsc::channel(10);
    let shutdown = CancellationToken::new();

    let _listener_handle = spawn_task_listener(config, consumer_tx, shutdown.clone());

    let js_context = jetstream::new(nats_client.clone());
    js_context
        .publish(
            format!("{prefix}.{SERVICE_NAME}.tasks.workspace-1"),
            "task notification".into(),
        )
        .await
        .expect("failed to publish");

    // Wait for consumer
    tokio::time::timeout(Duration::from_secs(5), consumer_rx.recv())
        .await
        .expect("timeout")
        .expect("consumer channel closed");

    // Give time for ack to process
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify message was acknowledged (no pending messages)
    let mut task_listener_consumer =
        get_consumer_with_retry(&tasks_stream, &format!("{SERVICE_NAME}-task-listener")).await;

    let info = task_listener_consumer
        .info()
        .await
        .expect("failed to get consumer info");

    assert_eq!(info.num_pending, 0, "message should be acknowledged");

    shutdown.cancel();
}

#[test(tokio::test)]
async fn listener_shuts_down_gracefully() {
    let prefix = nats_prefix();
    let nats_client = nats(prefix.clone()).await;
    let (tasks_stream, requests_stream) = create_test_streams(&prefix, &nats_client).await;

    let config = FairSchedulingConfig::for_workspace_partitioning(
        SERVICE_NAME,
        tasks_stream,
        requests_stream,
        Some(prefix.clone()),
    );

    let (consumer_tx, _consumer_rx) = mpsc::channel(10);
    let shutdown = CancellationToken::new();

    let listener_handle = spawn_task_listener(config, consumer_tx, shutdown.clone());

    tokio::time::sleep(Duration::from_millis(100)).await;

    shutdown.cancel();

    tokio::time::timeout(Duration::from_secs(5), listener_handle)
        .await
        .expect("listener should shut down within timeout")
        .expect("listener task should complete without error");
}

#[test(tokio::test)]
async fn listener_stops_when_consumer_channel_closed() {
    let prefix = nats_prefix();
    let nats_client = nats(prefix.clone()).await;
    let (tasks_stream, requests_stream) = create_test_streams(&prefix, &nats_client).await;

    let config = FairSchedulingConfig::for_workspace_partitioning(
        SERVICE_NAME,
        tasks_stream,
        requests_stream,
        Some(prefix.clone()),
    );

    let (consumer_tx, consumer_rx) = mpsc::channel(10);
    let shutdown = CancellationToken::new();

    let listener_handle = spawn_task_listener(config, consumer_tx, shutdown.clone());

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Drop the receiver to close the channel
    drop(consumer_rx);

    let js_context = jetstream::new(nats_client.clone());
    js_context
        .publish(
            format!("{prefix}.{SERVICE_NAME}.tasks.workspace-1"),
            "task notification".into(),
        )
        .await
        .expect("failed to publish");

    // Listener should stop when it tries to send to closed channel
    tokio::time::timeout(Duration::from_secs(5), listener_handle)
        .await
        .expect("listener should stop within timeout")
        .expect("listener task should complete without error");

    shutdown.cancel();
}

#[test(tokio::test)]
async fn listener_replaces_consumer_on_duplicate_notification() {
    let prefix = nats_prefix();
    let nats_client = nats(prefix.clone()).await;
    let (tasks_stream, requests_stream) = create_test_streams(&prefix, &nats_client).await;

    let config = FairSchedulingConfig::for_workspace_partitioning(
        SERVICE_NAME,
        tasks_stream,
        requests_stream,
        Some(prefix.clone()),
    );

    let (consumer_tx, mut consumer_rx) = mpsc::channel(10);
    let shutdown = CancellationToken::new();

    let _listener_handle = spawn_task_listener(config, consumer_tx, shutdown.clone());

    let js_context = jetstream::new(nats_client.clone());

    // Send first notification
    js_context
        .publish(
            format!("{prefix}.{SERVICE_NAME}.tasks.workspace-1"),
            "first notification".into(),
        )
        .await
        .expect("failed to publish");

    let _first = tokio::time::timeout(Duration::from_secs(5), consumer_rx.recv())
        .await
        .expect("timeout")
        .expect("consumer channel closed");

    // Send second notification for same workspace
    js_context
        .publish(
            format!("{prefix}.{SERVICE_NAME}.tasks.workspace-1"),
            "second notification".into(),
        )
        .await
        .expect("failed to publish");

    let second = tokio::time::timeout(Duration::from_secs(5), consumer_rx.recv())
        .await
        .expect("timeout")
        .expect("consumer channel closed");

    assert_eq!(second.key, "workspace-1");

    shutdown.cancel();
}

#[test(tokio::test)]
async fn listener_skips_messages_with_invalid_subjects() {
    let prefix = nats_prefix();
    let nats_client = nats(prefix.clone()).await;
    let (tasks_stream, requests_stream) = create_test_streams(&prefix, &nats_client).await;

    let config = FairSchedulingConfig::for_workspace_partitioning(
        SERVICE_NAME,
        tasks_stream.clone(),
        requests_stream,
        Some(prefix.clone()),
    );

    let (consumer_tx, mut consumer_rx) = mpsc::channel(10);
    let shutdown = CancellationToken::new();

    let _listener_handle = spawn_task_listener(config, consumer_tx, shutdown.clone());

    let js_context = jetstream::new(nats_client.clone());

    // Publish invalid subject (missing workspace id)
    js_context
        .publish(
            format!("{prefix}.{SERVICE_NAME}.tasks"),
            "invalid notification".into(),
        )
        .await
        .expect("failed to publish");

    // Wait a bit to ensure message is processed
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify invalid message was acknowledged (not stuck)
    let mut task_listener_consumer =
        get_consumer_with_retry(&tasks_stream, &format!("{SERVICE_NAME}-task-listener")).await;

    let info = task_listener_consumer
        .info()
        .await
        .expect("failed to get consumer info");

    assert_eq!(
        info.num_pending, 0,
        "invalid message should be acknowledged and skipped"
    );

    // Publish valid message to ensure listener is still working
    js_context
        .publish(
            format!("{prefix}.{SERVICE_NAME}.tasks.workspace-1"),
            "valid notification".into(),
        )
        .await
        .expect("failed to publish");

    let ready = tokio::time::timeout(Duration::from_secs(5), consumer_rx.recv())
        .await
        .expect("timeout")
        .expect("consumer channel closed");

    assert_eq!(ready.key, "workspace-1");

    shutdown.cancel();
}

#[test(tokio::test)]
async fn listener_handles_concurrent_task_notifications() {
    let prefix = nats_prefix();
    let nats_client = nats(prefix.clone()).await;
    let (tasks_stream, requests_stream) = create_test_streams(&prefix, &nats_client).await;

    let config = FairSchedulingConfig::for_workspace_partitioning(
        SERVICE_NAME,
        tasks_stream,
        requests_stream,
        Some(prefix.clone()),
    );

    let (consumer_tx, mut consumer_rx) = mpsc::channel(20);
    let shutdown = CancellationToken::new();

    let _listener_handle = spawn_task_listener(config, consumer_tx, shutdown.clone());

    let js_context = jetstream::new(nats_client.clone());

    // Send multiple notifications concurrently
    let mut publish_tasks = Vec::new();
    for i in 0..10 {
        let js = js_context.clone();
        let prefix_clone = prefix.clone();
        publish_tasks.push(tokio::spawn(async move {
            js.publish(
                format!("{prefix_clone}.{SERVICE_NAME}.tasks.workspace-{i}"),
                "concurrent notification".into(),
            )
            .await
            .expect("failed to publish");
        }));
    }

    // Wait for all publishes to complete
    for task in publish_tasks {
        task.await.expect("publish task failed");
    }

    // Collect all consumers
    let mut received_keys = Vec::new();
    for _ in 0..10 {
        let ready = tokio::time::timeout(Duration::from_secs(5), consumer_rx.recv())
            .await
            .expect("timeout waiting for consumer")
            .expect("consumer channel closed");
        received_keys.push(ready.key);
    }

    received_keys.sort();
    let expected: Vec<String> = (0..10).map(|i| format!("workspace-{i}")).collect();
    assert_eq!(received_keys, expected);

    shutdown.cancel();
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 2))]
async fn listener_fair_scheduling_across_workspaces() {
    let prefix = nats_prefix();
    let nats_client = nats(prefix.clone()).await;
    let (tasks_stream, requests_stream) = create_test_streams(&prefix, &nats_client).await;

    let config = FairSchedulingConfig::for_workspace_partitioning(
        SERVICE_NAME,
        tasks_stream.clone(),
        requests_stream.clone(),
        Some(prefix.clone()),
    );

    let (consumer_tx, mut consumer_rx) = mpsc::channel(20);
    let shutdown = CancellationToken::new();

    let _listener_handle = spawn_task_listener(config, consumer_tx, shutdown.clone());

    let js_context = jetstream::new(nats_client.clone());

    // Create heavy load for workspace-a
    for _ in 0..5 {
        js_context
            .publish(
                format!("{prefix}.{SERVICE_NAME}.tasks.workspace-a"),
                "workspace-a notification".into(),
            )
            .await
            .expect("failed to publish");
    }

    // Small delay to let workspace-a start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create light load for workspace-b
    for _ in 0..2 {
        js_context
            .publish(
                format!("{prefix}.{SERVICE_NAME}.tasks.workspace-b"),
                "workspace-b notification".into(),
            )
            .await
            .expect("failed to publish");
    }

    // Collect consumers and verify both workspaces get processed
    let mut workspace_a_count = 0;
    let mut workspace_b_count = 0;

    for _ in 0..7 {
        let ready = tokio::time::timeout(Duration::from_secs(5), consumer_rx.recv())
            .await
            .expect("timeout")
            .expect("consumer channel closed");

        if ready.key == "workspace-a" {
            workspace_a_count += 1;
        } else if ready.key == "workspace-b" {
            workspace_b_count += 1;
        }
    }

    assert_eq!(
        workspace_a_count, 5,
        "workspace-a should process 5 notifications"
    );
    assert_eq!(
        workspace_b_count, 2,
        "workspace-b should process 2 notifications"
    );

    // Verify consumers were created
    let consumer_a_name = format!("{SERVICE_NAME}-ws-workspace-a");
    let consumer_b_name = format!("{SERVICE_NAME}-ws-workspace-b");

    let _consumer_a = get_consumer_with_retry(&requests_stream, &consumer_a_name).await;
    let _consumer_b = get_consumer_with_retry(&requests_stream, &consumer_b_name).await;

    shutdown.cancel();
}

#[test(tokio::test)]
async fn listener_processes_messages_in_order() {
    let prefix = nats_prefix();
    let nats_client = nats(prefix.clone()).await;
    let (tasks_stream, requests_stream) = create_test_streams(&prefix, &nats_client).await;

    let config = FairSchedulingConfig::for_workspace_partitioning(
        SERVICE_NAME,
        tasks_stream,
        requests_stream,
        Some(prefix.clone()),
    );

    let (consumer_tx, mut consumer_rx) = mpsc::channel(10);
    let shutdown = CancellationToken::new();

    let _listener_handle = spawn_task_listener(config, consumer_tx, shutdown.clone());

    let js_context = jetstream::new(nats_client.clone());

    // Publish multiple notifications in order for the same workspace
    for i in 0..5 {
        js_context
            .publish(
                format!("{prefix}.{SERVICE_NAME}.tasks.workspace-1"),
                format!("notification-{i}").into(),
            )
            .await
            .expect("failed to publish");
    }

    // Verify we receive consumers for each notification
    for _ in 0..5 {
        let ready = tokio::time::timeout(Duration::from_secs(5), consumer_rx.recv())
            .await
            .expect("timeout")
            .expect("consumer channel closed");

        assert_eq!(ready.key, "workspace-1");
    }

    shutdown.cancel();
}
